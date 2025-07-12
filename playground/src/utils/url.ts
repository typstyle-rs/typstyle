import type { FormatOptions, OutputType } from "../types";

export interface PlaygroundState {
  sourceCode: string;
  formatOptions: FormatOptions;
  activeOutput: OutputType;
}

// Maximum URL length before using pastebin (browsers generally support 2000+ chars safely)
const MAX_URL_LENGTH = 2000;

// shz.al API base URL
const PASTEBIN_API = "https://shz.al";

/**
 * Upload content to shz.al pastebin
 */
async function uploadToPastebin(content: string): Promise<string | null> {
  try {
    const formData = new FormData();
    formData.append("c", content);

    const response = await fetch(PASTEBIN_API, {
      method: "POST",
      body: formData,
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const result = await response.json();
    if (result.url) {
      // Extract the paste ID from the URL (e.g., "https://shz.al/abcd" -> "abcd")
      const urlParts = result.url.split("/");
      return urlParts[urlParts.length - 1];
    }

    return null;
  } catch (error) {
    console.error("Error uploading to pastebin:", error);
    return null;
  }
}

/**
 * Fetch content from shz.al pastebin
 */
async function fetchFromPastebin(pasteId: string): Promise<string | null> {
  try {
    const response = await fetch(`${PASTEBIN_API}/${pasteId}`);

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    return await response.text();
  } catch (error) {
    console.error("Error fetching from pastebin:", error);
    return null;
  }
}

/**
 * Unicode-safe base64 encoding using native browser APIs
 */
function unicodeToBase64(str: string): string {
  return btoa(unescape(encodeURIComponent(str)));
}

/**
 * Unicode-safe base64 decoding using native browser APIs
 */
function base64ToUnicode(base64: string): string {
  return decodeURIComponent(escape(atob(base64)));
}

/**
 * Compresses a string using URL-safe base64 encoding for sharing.
 */
export function compressForUrl(data: string): string {
  try {
    // Unicode-safe base64 encoding
    const base64 = unicodeToBase64(data);

    // Make it URL-safe by replacing characters
    return base64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
  } catch (error) {
    console.error("Error compressing data for URL:", error);
    return "";
  }
}

/**
 * Decompresses a URL-safe string back to the original data
 */
export function decompressFromUrl(compressed: string): string {
  try {
    // Restore base64 characters
    let base64 = compressed.replace(/-/g, "+").replace(/_/g, "/");

    // Add padding if needed
    while (base64.length % 4) {
      base64 += "=";
    }

    // Unicode-safe base64 decoding
    return base64ToUnicode(base64);
  } catch (error) {
    console.error("Error decompressing data from URL:", error);
    return "";
  }
}

/**
 * Encodes the playground state into a URL parameter
 */
export function encodePlaygroundState(state: PlaygroundState): string {
  try {
    // Create a compact representation
    const compactState = {
      c: state.sourceCode,
      f: state.formatOptions,
      o: state.activeOutput,
    };

    const stateString = JSON.stringify(compactState);
    return compressForUrl(stateString);
  } catch (error) {
    console.error("Error encoding playground state:", error);
    return "";
  }
}

/**
 * Decodes a URL parameter back to playground state
 */
export function decodePlaygroundState(encoded: string): PlaygroundState | null {
  try {
    const decompressed = decompressFromUrl(encoded);
    if (!decompressed) return null;

    const parsed = JSON.parse(decompressed);

    // Handle compact format
    if (parsed.c !== undefined) {
      return {
        sourceCode: parsed.c || "",
        formatOptions: parsed.f || {},
        activeOutput: parsed.o || "formatted",
      };
    }

    // Handle legacy format (if any)
    return {
      sourceCode: parsed.sourceCode || "",
      formatOptions: parsed.formatOptions || {},
      activeOutput: parsed.activeOutput || "formatted",
    };
  } catch (error) {
    console.error("Error decoding playground state:", error);
    return null;
  }
}

/**
 * Generates a shareable URL for the current playground state
 * Automatically uses pastebin if URL becomes too long
 */
export async function generateShareUrl(
  state: PlaygroundState,
): Promise<{ url: string; usedPastebin: boolean }> {
  const encoded = encodePlaygroundState(state);
  if (!encoded) return { url: window.location.href, usedPastebin: false };

  // Try normal URL encoding first
  const baseUrl = new URL(window.location.href);
  baseUrl.searchParams.delete("share");
  baseUrl.searchParams.delete("paste");
  baseUrl.searchParams.set("share", encoded);
  const normalUrl = baseUrl.toString();

  // If URL is within acceptable length, use it
  if (normalUrl.length <= MAX_URL_LENGTH) {
    return { url: normalUrl, usedPastebin: false };
  }

  // URL is too long, upload to pastebin
  const stateString = JSON.stringify({
    c: state.sourceCode,
    f: state.formatOptions,
    o: state.activeOutput,
  });

  const pasteId = await uploadToPastebin(stateString);
  if (!pasteId) {
    // Fallback to normal URL if pastebin fails
    return { url: normalUrl, usedPastebin: false };
  }

  // Create pastebin URL
  const pastebinUrl = new URL(window.location.href);
  pastebinUrl.searchParams.delete("share");
  pastebinUrl.searchParams.delete("paste");
  pastebinUrl.searchParams.set("paste", pasteId);

  return { url: pastebinUrl.toString(), usedPastebin: true };
}

/**
 * Extracts playground state from the current URL
 * Supports both direct encoding and pastebin URLs
 */
export async function getStateFromUrl(): Promise<PlaygroundState | null> {
  const url = new URL(window.location.href);

  // Check for direct share parameter first
  const shareParam = url.searchParams.get("share");
  if (shareParam) {
    return decodePlaygroundState(shareParam);
  }

  // Check for pastebin parameter
  const pasteParam = url.searchParams.get("paste");
  if (pasteParam) {
    const content = await fetchFromPastebin(pasteParam);
    if (!content) return null;

    try {
      const parsed = JSON.parse(content);
      // Handle the compact format from pastebin
      return {
        sourceCode: parsed.c || "",
        formatOptions: parsed.f || {},
        activeOutput: parsed.o || "formatted",
      };
    } catch (error) {
      console.error("Error parsing pastebin content:", error);
      return null;
    }
  }

  return null;
}

/**
 * Copies text to the clipboard using modern API
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text);
      return true;
    }
    // For modern browsers, fallback should rarely be needed
    throw new Error("Clipboard API not available");
  } catch (error) {
    console.error("Error copying to clipboard:", error);
    return false;
  }
}
