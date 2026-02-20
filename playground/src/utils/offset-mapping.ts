import diff from "fast-diff";

/**
 * A single anchor point mapping a source offset to a formatted output offset.
 */
export interface Anchor {
  srcOffset: number;
  outOffset: number;
}

/**
 * Sorted array of anchor points from source to formatted output.
 */
export type OffsetMapping = Anchor[];

/**
 * A span mapping from WASM: maps a source byte range to an output text range.
 * Used for AST and IR tab sync where WASM provides node-level mappings.
 */
export interface SpanMapping {
  srcStart: number;
  srcEnd: number;
  outStart: number;
  outEnd: number;
}

/**
 * Extract non-whitespace characters with their original UTF-16 offsets.
 * Iterates by code point to correctly handle surrogate pairs (emoji, CJK extensions),
 * while tracking the UTF-16 offset (used by Monaco and JS strings).
 */
function extractNonWhitespace(
  text: string,
): { char: string; offset: number }[] {
  const result: { char: string; offset: number }[] = [];
  let utf16Offset = 0;
  for (const codePoint of text) {
    if (
      codePoint !== " " &&
      codePoint !== "\t" &&
      codePoint !== "\n" &&
      codePoint !== "\r"
    ) {
      result.push({ char: codePoint, offset: utf16Offset });
    }
    // Advance by the number of UTF-16 code units this code point occupies
    utf16Offset += codePoint.length;
  }
  return result;
}

/**
 * Build an offset mapping between source and formatted text.
 *
 * 1. Extract non-whitespace character sequences from both texts
 * 2. Diff the two character sequences using fast-diff
 * 3. Each matched character pair produces an anchor (srcOffset, outOffset)
 */
export function buildOffsetMapping(
  source: string,
  formatted: string,
): OffsetMapping {
  if (source.length === 0 || formatted.length === 0) return [];

  const srcChars = extractNonWhitespace(source);
  const fmtChars = extractNonWhitespace(formatted);

  if (srcChars.length === 0 || fmtChars.length === 0) return [];

  const srcStr = srcChars.map((c) => c.char).join("");
  const fmtStr = fmtChars.map((c) => c.char).join("");

  const changes = diff(srcStr, fmtStr);

  const anchors: OffsetMapping = [];
  let srcIdx = 0;
  let fmtIdx = 0;

  for (const [op, text] of changes) {
    switch (op) {
      case diff.EQUAL:
        for (let i = 0; i < text.length; i++) {
          anchors.push({
            srcOffset: srcChars[srcIdx].offset,
            outOffset: fmtChars[fmtIdx].offset,
          });
          srcIdx++;
          fmtIdx++;
        }
        break;
      case diff.DELETE:
        srcIdx += text.length;
        break;
      case diff.INSERT:
        fmtIdx += text.length;
        break;
    }
  }

  return anchors;
}

/**
 * Query the mapping: given a source offset, find the corresponding output offset.
 * Uses binary search + linear interpolation between anchors.
 */
export function queryMapping(
  mapping: OffsetMapping,
  srcOffset: number,
): number {
  if (mapping.length === 0) return 0;

  if (srcOffset <= mapping[0].srcOffset) return mapping[0].outOffset;
  if (srcOffset >= mapping[mapping.length - 1].srcOffset)
    return mapping[mapping.length - 1].outOffset;

  let lo = 0;
  let hi = mapping.length - 1;
  while (lo < hi - 1) {
    const mid = (lo + hi) >> 1;
    if (mapping[mid].srcOffset <= srcOffset) {
      lo = mid;
    } else {
      hi = mid;
    }
  }

  const a = mapping[lo];
  const b = mapping[hi];
  if (a.srcOffset === b.srcOffset) return a.outOffset;
  const t = (srcOffset - a.srcOffset) / (b.srcOffset - a.srcOffset);
  return Math.round(a.outOffset + t * (b.outOffset - a.outOffset));
}

/**
 * Reverse query: given an output offset, find the corresponding source offset.
 * Uses binary search + linear interpolation between anchors.
 */
export function queryReverseMapping(
  mapping: OffsetMapping,
  outOffset: number,
): number {
  if (mapping.length === 0) return 0;

  if (outOffset <= mapping[0].outOffset) return mapping[0].srcOffset;
  if (outOffset >= mapping[mapping.length - 1].outOffset)
    return mapping[mapping.length - 1].srcOffset;

  let lo = 0;
  let hi = mapping.length - 1;
  while (lo < hi - 1) {
    const mid = (lo + hi) >> 1;
    if (mapping[mid].outOffset <= outOffset) {
      lo = mid;
    } else {
      hi = mid;
    }
  }

  const a = mapping[lo];
  const b = mapping[hi];
  if (a.outOffset === b.outOffset) return a.srcOffset;
  const t = (outOffset - a.outOffset) / (b.outOffset - a.outOffset);
  return Math.round(a.srcOffset + t * (b.srcOffset - a.srcOffset));
}

/**
 * Query span mapping (from WASM AST/IR): given a source offset,
 * find the corresponding output offset using interval lookup.
 *
 * Finds the span whose source range contains the offset,
 * then interpolates within that span.
 */
export function querySpanMapping(
  mapping: SpanMapping[],
  srcOffset: number,
): number {
  if (mapping.length === 0) return 0;

  // Find the best matching span (binary search for containing span)
  let best: SpanMapping | null = null;
  let lo = 0;
  let hi = mapping.length - 1;

  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    const span = mapping[mid];
    if (srcOffset < span.srcStart) {
      hi = mid - 1;
    } else if (srcOffset >= span.srcEnd) {
      lo = mid + 1;
    } else {
      best = span;
      break;
    }
  }

  // If no containing span, find the nearest
  if (!best) {
    if (lo >= mapping.length) return mapping[mapping.length - 1].outEnd;
    if (hi < 0) return mapping[0].outStart;
    // Pick the closer boundary
    const prev = hi >= 0 ? mapping[hi] : null;
    const next = lo < mapping.length ? mapping[lo] : null;
    if (prev && next) {
      best =
        srcOffset - prev.srcEnd < next.srcStart - srcOffset ? prev : next;
    } else {
      best = (prev ?? next)!;
    }
  }

  // Interpolate within the span
  const spanLen = best.srcEnd - best.srcStart;
  if (spanLen === 0) return best.outStart;
  const t = Math.max(
    0,
    Math.min(1, (srcOffset - best.srcStart) / spanLen),
  );
  const outLen = best.outEnd - best.outStart;
  return Math.round(best.outStart + t * outLen);
}

/**
 * Reverse query span mapping: given an output offset,
 * find the corresponding source offset.
 *
 * Builds a sorted-by-outStart index for binary search (O(n log n) first call,
 * O(log n) lookup). The sorted index is cached on the array instance.
 */
export function querySpanMappingReverse(
  mapping: SpanMapping[],
  outOffset: number,
): number {
  if (mapping.length === 0) return 0;

  // Get or build an index sorted by outStart for binary search
  const sorted = getOutputSortedIndex(mapping);

  // Binary search for containing span
  let best: SpanMapping | null = null;
  let lo = 0;
  let hi = sorted.length - 1;

  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    const span = sorted[mid];
    if (outOffset < span.outStart) {
      hi = mid - 1;
    } else if (outOffset >= span.outEnd) {
      lo = mid + 1;
    } else {
      best = span;
      break;
    }
  }

  // If no containing span, find the nearest boundary
  if (!best) {
    if (lo >= sorted.length) return sorted[sorted.length - 1].srcEnd;
    if (hi < 0) return sorted[0].srcStart;
    const prev = hi >= 0 ? sorted[hi] : null;
    const next = lo < sorted.length ? sorted[lo] : null;
    if (prev && next) {
      best =
        outOffset - prev.outEnd < next.outStart - outOffset ? prev : next;
    } else {
      best = (prev ?? next)!;
    }
  }

  const outLen = best.outEnd - best.outStart;
  if (outLen === 0) return best.srcStart;
  const t = Math.max(
    0,
    Math.min(1, (outOffset - best.outStart) / outLen),
  );
  const srcLen = best.srcEnd - best.srcStart;
  return Math.round(best.srcStart + t * srcLen);
}

/**
 * Find the best matching span for a given source offset (for debug visualization).
 * Returns the matched span or null if the mapping is empty.
 */
export function findContainingSpan(
  mapping: SpanMapping[],
  srcOffset: number,
): SpanMapping | null {
  if (mapping.length === 0) return null;

  let lo = 0;
  let hi = mapping.length - 1;

  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    const span = mapping[mid];
    if (srcOffset < span.srcStart) {
      hi = mid - 1;
    } else if (srcOffset >= span.srcEnd) {
      lo = mid + 1;
    } else {
      return span;
    }
  }

  if (lo >= mapping.length) return mapping[mapping.length - 1];
  if (hi < 0) return mapping[0];
  const prev = hi >= 0 ? mapping[hi] : null;
  const next = lo < mapping.length ? mapping[lo] : null;
  if (prev && next) {
    return srcOffset - prev.srcEnd < next.srcStart - srcOffset ? prev : next;
  }
  return (prev ?? next)!;
}

/**
 * Find the best matching span for a given output offset (reverse, for debug visualization).
 * Returns the matched span or null if the mapping is empty.
 */
export function findContainingSpanReverse(
  mapping: SpanMapping[],
  outOffset: number,
): SpanMapping | null {
  if (mapping.length === 0) return null;

  const sorted = getOutputSortedIndex(mapping);
  let lo = 0;
  let hi = sorted.length - 1;

  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    const span = sorted[mid];
    if (outOffset < span.outStart) {
      hi = mid - 1;
    } else if (outOffset >= span.outEnd) {
      lo = mid + 1;
    } else {
      return span;
    }
  }

  if (lo >= sorted.length) return sorted[sorted.length - 1];
  if (hi < 0) return sorted[0];
  const prev = hi >= 0 ? sorted[hi] : null;
  const next = lo < sorted.length ? sorted[lo] : null;
  if (prev && next) {
    return outOffset - prev.outEnd < next.outStart - outOffset ? prev : next;
  }
  return (prev ?? next)!;
}

/** WeakMap cache for output-sorted span indices. */
const outputSortedCache = new WeakMap<SpanMapping[], SpanMapping[]>();

/** Get or create a copy of the span array sorted by outStart. */
function getOutputSortedIndex(mapping: SpanMapping[]): SpanMapping[] {
  let sorted = outputSortedCache.get(mapping);
  if (!sorted) {
    sorted = [...mapping].sort(
      (a, b) => a.outStart - b.outStart || a.outEnd - b.outEnd,
    );
    outputSortedCache.set(mapping, sorted);
  }
  return sorted;
}
