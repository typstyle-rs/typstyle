import type { FormatOptions } from "../types";

export interface PlaygroundState {
  sourceCode: string;
  formatOptions: FormatOptions;
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
      return (result.url as string).split("/").pop() ?? null;
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
 * Generates a shareable URL for the current playground state
 * Automatically uses pastebin if URL becomes too long
 */
export async function generateShareUrl(
  state: PlaygroundState,
): Promise<{ url: string; usedPastebin: boolean }> {
  const stateString = JSON.stringify({
    c: state.sourceCode,
    f: state.formatOptions,
  });
  const encoded = encodeURIComponent(stateString);

  // Try normal URL encoding first
  const baseUrl = new URL(window.location.href);
  baseUrl.searchParams.delete("paste");
  baseUrl.searchParams.set("share", encoded);
  const normalUrl = baseUrl.toString();

  // If URL is within acceptable length, use it
  if (normalUrl.length <= MAX_URL_LENGTH) {
    return { url: normalUrl, usedPastebin: false };
  }

  // URL is too long, upload to pastebin
  const pasteId = await uploadToPastebin(stateString);
  if (!pasteId) {
    // Fallback to normal URL if pastebin fails
    return { url: normalUrl, usedPastebin: false };
  }

  // Create pastebin URL
  const pastebinUrl = new URL(window.location.href);
  pastebinUrl.searchParams.delete("share");
  pastebinUrl.searchParams.set("paste", pasteId);

  return { url: pastebinUrl.toString(), usedPastebin: true };
}

/**
 * Extracts playground state from the current URL
 * Supports both direct encoding and pastebin URLs
 */
export async function getStateFromUrl(): Promise<PlaygroundState | null> {
  const url = new URL(window.location.href);

  let stateString: string | null = null;

  // Check for direct share parameter first
  const shareParam = url.searchParams.get("share");
  if (shareParam) {
    stateString = decodeURIComponent(shareParam);
  } else {
    // Check for pastebin parameter
    const pasteParam = url.searchParams.get("paste");
    if (pasteParam) {
      stateString = await fetchFromPastebin(pasteParam);
    }
  }

  if (!stateString) {
    return null;
  }

  try {
    const parsed = JSON.parse(stateString);
    // Handle the compact format from pastebin
    return {
      sourceCode: parsed.c ?? "",
      formatOptions: parsed.f ?? {},
    };
  } catch (error) {
    console.error("Error parsing pastebin content:", error);
    return null;
  }
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

/**
 * Removes share parameters from the current URL to clean up the address bar
 */
export function cleanUrlAfterLoad(): void {
  const url = new URL(window.location.href);
  if (url.searchParams.has("share") || url.searchParams.has("paste")) {
    url.searchParams.delete("share");
    url.searchParams.delete("paste");

    // Update the URL without triggering a page reload
    window.history.replaceState({}, "", url.toString());
  }
}
