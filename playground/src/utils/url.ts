import * as lz from "lz-string";
import queryString from "query-string";
import { type FormatOptions } from "./formatter";
import { fetchFromPastebin, uploadToPastebin } from "./pastebin";

export interface PlaygroundState {
  sourceCode: string;
  formatOptions: Partial<FormatOptions>;
}

// Maximum URL length before using pastebin for source code storage
// We use a shorter (< 2048) length to avoid verbosity
const MAX_URL_LENGTH = 256;

/**
 * Updates the URL with current options and source code
 */
export function updateUrlWithState(
  sourceCode: string,
  options: Partial<FormatOptions>,
): void {
  const query = {
    ...options,
    code:
      sourceCode === "" ? null : lz.compressToEncodedURIComponent(sourceCode),
  };
  const qs = queryString.stringify(query, {
    skipNull: true,
    skipEmptyString: true,
    sort: false,
  });

  // Replace the current URL without reloading
  window.history.replaceState({}, "", `?${qs}`);
}

/**
 * Extracts playground state from the current URL
 */
export async function getStateFromUrl(): Promise<PlaygroundState | null> {
  const url = new URL(window.location.href);
  const params = url.searchParams;

  let sourceCode = "";

  // Check for code parameter first
  const codeParam = params.get("code");
  if (codeParam) {
    try {
      const decompressed = lz.decompressFromEncodedURIComponent(codeParam);
      if (decompressed !== null) {
        sourceCode = decompressed;
      }
    } catch (error) {
      console.warn("Failed to decompress code parameter:", error);
    }
  } else {
    // Check for pastebin parameter
    const pasteParam = params.get("paste");
    if (pasteParam) {
      try {
        const content = await fetchFromPastebin(pasteParam);
        if (content) {
          sourceCode = content;
        }
        params.delete("paste");
        window.history.replaceState({}, "", url.toString());
      } catch (error) {
        console.warn("Failed to fetch from pastebin:", error);
      }
    }
  }

  // Parse options from query params (use current URL state)
  params.delete("code");
  params.delete("paste");
  const query = queryString.parse(url.search, {
    parseBooleans: true,
    parseNumbers: true,
  });

  return {
    sourceCode,
    formatOptions: query as Partial<FormatOptions>,
  };
}

/**
 * Generates a shareable URL for the current playground state
 */
export async function generateShareUrl(
  state: PlaygroundState,
): Promise<{ url: string; usedPastebin: boolean }> {
  // Check if current URL is already suitable (not too long)
  if (window.location.href.length > MAX_URL_LENGTH) {
    // Try to upload to pastebin for long URLs
    const pasteId = await uploadToPastebin(state.sourceCode);
    if (pasteId) {
      // Replace code parameter with paste parameter in current URL
      const url = new URL(window.location.href);
      url.searchParams.delete("code");
      url.searchParams.set("paste", pasteId);
      return { url: url.toString(), usedPastebin: true };
    }
  }

  // If pastebin fails, use current URL as-is
  return { url: window.location.href, usedPastebin: false };
}
