import {
  OPTION_KEYS,
  OPTIONS_SCHEMA,
  type OptionKey,
  type PartialOptions,
} from "@typstyle/options";
import * as lz from "lz-string";
import queryString from "query-string";
import type { OutputType } from "../types";
import { fetchFromPastebin, uploadToPastebin } from "./pastebin";

export interface PlaygroundState {
  sourceCode: string;
  formatOptions: PartialOptions;
  tab: OutputType;
}

export interface PlaygroundUrlState {
  sourceCode?: string;
  formatOptions: PartialOptions;
  tab?: OutputType;
}

/** URL keys used before options were generated from `options.toml`. */
const LEGACY_OPTION_KEYS: Record<string, keyof PartialOptions> = {
  lineWidth: "max_width",
  indentWidth: "tab_spaces",
  collapseMarkupSpaces: "collapse_markup_spaces",
  reorderImportItems: "reorder_import_items",
  wrapMode: "wrap_mode",
};

// Maximum URL length before using pastebin for source code storage
// We use a shorter (< 2048) length to avoid verbosity
const MAX_URL_LENGTH = 1024;

/**
 * Updates the URL with current options and source code
 */
export function updateUrlWithState(
  sourceCode: string,
  options: PartialOptions,
  tab?: OutputType,
): void {
  const currentUrl = new URL(window.location.href);
  const existingTab = parseTabParam(currentUrl.searchParams.get("tab"));
  const tabParam = normalizeTabParam(tab ?? existingTab);

  const query: Record<string, unknown> = {
    ...options,
    code:
      sourceCode === "" ? null : lz.compressToEncodedURIComponent(sourceCode),
    tab: tabParam,
  };
  const qs = queryString.stringify(query, {
    skipNull: true,
    skipEmptyString: true,
    sort: false,
  });

  // Replace the current URL without reloading
  window.history.replaceState({}, "", qs === "" ? location.pathname : `?${qs}`);
}

/**
 * Extracts playground state from the current URL
 */
export async function getStateFromUrl(): Promise<PlaygroundUrlState> {
  const url = new URL(window.location.href);
  const params = url.searchParams;

  let sourceCode: string | undefined;

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
  const query = queryString.parse(
    queryString.pick(url.search, [
      ...OPTION_KEYS,
      ...Object.keys(LEGACY_OPTION_KEYS),
    ]),
    {
      parseBooleans: true,
      parseNumbers: true,
    },
  );
  const formatOptions: Record<string, unknown> = {};
  for (const key of OPTION_KEYS) {
    const value = query[key];
    if (value !== undefined && isValidOptionValue(key, value)) {
      formatOptions[key] = value;
    }
  }
  for (const [legacy, key] of Object.entries(LEGACY_OPTION_KEYS)) {
    const value = query[legacy];
    if (
      !(key in formatOptions) &&
      value !== undefined &&
      isValidOptionValue(key, value)
    ) {
      formatOptions[key] = value;
    }
  }

  const tab = parseTabParam(params.get("tab"));

  return {
    sourceCode,
    formatOptions: formatOptions as PartialOptions,
    tab,
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
      if (state.tab && state.tab !== "formatted") {
        url.searchParams.set("tab", state.tab);
      } else {
        url.searchParams.delete("tab");
      }
      return { url: url.toString(), usedPastebin: true };
    }
  }

  // If pastebin fails, use current URL as-is
  return { url: window.location.href, usedPastebin: false };
}

function isValidOptionValue(key: OptionKey, value: unknown): boolean {
  const property = OPTIONS_SCHEMA.properties[key];
  switch (property.type) {
    case "boolean":
      return typeof value === "boolean";
    case "integer":
      return typeof value === "number" && Number.isInteger(value);
    case "number":
      return typeof value === "number" && Number.isFinite(value);
    case "string":
      return (
        typeof value === "string" &&
        (property.enum === undefined || property.enum.includes(value))
      );
  }
}

function parseTabParam(value: string | null): OutputType | undefined {
  if (value === "ast" || value === "pir") {
    return value;
  }
  return undefined;
}

function normalizeTabParam(tab: OutputType | undefined): OutputType | null {
  if (!tab || tab === "formatted") {
    return null;
  }
  return tab;
}
