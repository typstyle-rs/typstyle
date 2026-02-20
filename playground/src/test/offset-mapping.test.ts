import { describe, expect, it } from "vitest";
import {
  buildOffsetMapping,
  queryMapping,
  queryReverseMapping,
} from "@/utils/offset-mapping";

describe("offset-mapping", () => {
  // T-U1: Empty documents
  it("returns empty array for empty documents", () => {
    expect(buildOffsetMapping("", "")).toEqual([]);
    expect(buildOffsetMapping("", "abc")).toEqual([]);
    expect(buildOffsetMapping("abc", "")).toEqual([]);
  });

  // T-U2: Identical text
  it("maps identical text with srcOffset === outOffset", () => {
    const text = "function add(a, b) { return a + b; }";
    const mapping = buildOffsetMapping(text, text);

    expect(mapping.length).toBeGreaterThan(0);
    for (const anchor of mapping) {
      expect(anchor.srcOffset).toBe(anchor.outOffset);
    }
  });

  // T-U3: Whitespace-only changes
  it("aligns all non-whitespace chars when only whitespace differs", () => {
    const source = "f(a,b)";
    const formatted = "f(a, b)";
    const mapping = buildOffsetMapping(source, formatted);

    // Count non-whitespace chars in source
    const srcNonWs = source.replace(/\s/g, "").length;
    expect(mapping.length).toBe(srcNonWs);

    // Verify char alignment
    expect(mapping[0]).toEqual({ srcOffset: 0, outOffset: 0 }); // f
    expect(mapping[1]).toEqual({ srcOffset: 1, outOffset: 1 }); // (
    expect(mapping[2]).toEqual({ srcOffset: 2, outOffset: 2 }); // a
    expect(mapping[3]).toEqual({ srcOffset: 3, outOffset: 3 }); // ,
    // After comma, formatted has a space
    expect(mapping[4]).toEqual({ srcOffset: 4, outOffset: 5 }); // b
    expect(mapping[5]).toEqual({ srcOffset: 5, outOffset: 6 }); // )
  });

  // T-U4: Character insertions/deletions (bracket changes)
  it("handles character additions/deletions via diff tolerance", () => {
    const source = "f((x))";
    const formatted = "f(x)";
    const mapping = buildOffsetMapping(source, formatted);

    // Should still have some matched chars
    expect(mapping.length).toBeGreaterThan(0);

    // f should match f
    expect(mapping[0].srcOffset).toBe(0);
    expect(mapping[0].outOffset).toBe(0);

    // x should be matched
    const xAnchor = mapping.find(
      (a) => source[a.srcOffset] === "x" && formatted[a.outOffset] === "x",
    );
    expect(xAnchor).toBeDefined();
  });

  // T-U5: queryMapping boundary cases
  it("clamps to boundaries for out-of-range offsets", () => {
    const mapping = buildOffsetMapping("abc", "abc");

    // Before first anchor
    const first = queryMapping(mapping, -100);
    expect(first).toBe(mapping[0].outOffset);

    // After last anchor
    const last = queryMapping(mapping, 10000);
    expect(last).toBe(mapping[mapping.length - 1].outOffset);

    // Empty mapping
    expect(queryMapping([], 5)).toBe(0);
  });

  // T-U6: queryMapping interpolation
  it("linearly interpolates between anchors", () => {
    const source = "a   b";
    const formatted = "a         b";
    const mapping = buildOffsetMapping(source, formatted);

    // mapping: [{srcOffset:0, outOffset:0}, {srcOffset:4, outOffset:10}]
    expect(mapping.length).toBe(2);
    expect(mapping[0]).toEqual({ srcOffset: 0, outOffset: 0 });
    expect(mapping[1]).toEqual({ srcOffset: 4, outOffset: 10 });

    // Midpoint: srcOffset=2 → should interpolate to outOffset=5
    const mid = queryMapping(mapping, 2);
    expect(mid).toBe(5);

    // Quarter: srcOffset=1 → outOffset ~2.5 → rounds to 3
    const quarter = queryMapping(mapping, 1);
    expect(quarter).toBe(3); // (1/4) * 10 = 2.5 → rounds to 3
  });

  // T-U7: queryReverseMapping consistency
  it("reverse mapping is consistent with forward mapping", () => {
    const source = "function test(x, y) {\n  return x + y;\n}";
    const formatted = "function test(x, y) {\n    return x + y;\n}";
    const mapping = buildOffsetMapping(source, formatted);

    // For each anchor, verify reverse mapping returns close to original
    for (const anchor of mapping) {
      const reverseSrc = queryReverseMapping(mapping, anchor.outOffset);
      expect(reverseSrc).toBe(anchor.srcOffset);
    }
  });

  // T-U8: Large text performance
  it("builds mapping for 1000+ line text in under 100ms", () => {
    // Generate large text
    const lines: string[] = [];
    for (let i = 0; i < 1200; i++) {
      lines.push(
        `#let var_${i} = ${i} + ${i * 2}  // this is line number ${i}`,
      );
    }
    const source = lines.join("\n");

    // Simulate formatting: add some spaces
    const formatted = source.replace(/,/g, ", ").replace(/\+/g, " + ");

    const start = performance.now();
    const mapping = buildOffsetMapping(source, formatted);
    const elapsed = performance.now() - start;

    expect(mapping.length).toBeGreaterThan(0);
    expect(elapsed).toBeLessThan(100);
  });
});
