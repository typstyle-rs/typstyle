import type { FormatOptions, OutputType } from "../types";

export interface PlaygroundState {
  sourceCode: string;
  formatOptions: FormatOptions;
  activeOutput: OutputType;
}

/**
 * Compresses a string using a simple run-length encoding approach
 * combined with URL-safe base64 encoding for sharing.
 */
export function compressForUrl(data: string): string {
  try {
    // Convert to JSON string first
    const jsonString = JSON.stringify(data);

    // Simple compression by encoding as base64
    const base64 = btoa(jsonString);

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

    // Decode from base64
    const jsonString = atob(base64);

    // Parse back to original data
    return JSON.parse(jsonString);
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
 */
export function generateShareUrl(state: PlaygroundState): string {
  const encoded = encodePlaygroundState(state);
  if (!encoded) return window.location.href;

  const url = new URL(window.location.href);
  url.searchParams.set("share", encoded);
  return url.toString();
}

/**
 * Extracts playground state from the current URL
 */
export function getStateFromUrl(): PlaygroundState | null {
  const url = new URL(window.location.href);
  const shareParam = url.searchParams.get("share");

  if (!shareParam) return null;

  return decodePlaygroundState(shareParam);
}

/**
 * Copies text to the clipboard
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text);
      return true;
    } else {
      // Fallback for older browsers
      const textArea = document.createElement("textarea");
      textArea.value = text;
      textArea.style.position = "fixed";
      textArea.style.opacity = "0";
      document.body.appendChild(textArea);
      textArea.focus();
      textArea.select();

      const successful = document.execCommand("copy");
      document.body.removeChild(textArea);
      return successful;
    }
  } catch (error) {
    console.error("Error copying to clipboard:", error);
    return false;
  }
}
