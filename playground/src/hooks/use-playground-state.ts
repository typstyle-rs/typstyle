import { useDeferredValue, useEffect, useRef, useState } from "react";
import type { OutputType } from "@/types";
import {
  DEFAULT_FORMAT_OPTIONS,
  type FormatOptions,
  filterNonDefaultOptions,
} from "@/utils/formatter";
import { getStateFromUrl, updateUrlWithState } from "@/utils/url";
import { useAsyncError } from "./useErrorHandler";

const STORAGE_KEY = "playground-code";

export interface PlaygroundState {
  sourceCode: string;
  formatOptions: FormatOptions;
  activeOutput: OutputType;
}

export function usePlaygroundState() {
  const [sourceCode, setSourceCode] = useState("");
  const [formatOptions, setFormatOptions] = useState(DEFAULT_FORMAT_OPTIONS);
  const [activeOutput, setActiveOutput] = useState<OutputType>("formatted");
  const [cursorSyncEnabled, setCursorSyncEnabled] = useState(true);
  const [isInitializing, setIsInitializing] = useState(true);
  const captureAsyncError = useAsyncError();

  // Use deferred value for source code throttling
  const deferredSourceCode = useDeferredValue(sourceCode);

  const initialized = useRef(false);

  // Load state on mount
  useEffect(() => {
    const loadState = async () => {
      try {
        const urlState = await getStateFromUrl();
        console.debug("load state", urlState);

        // If source code is not provided in URL, use that in LocalStorage. Otherwise empty.
        setSourceCode(urlState.sourceCode ?? loadSourceCode() ?? "");
        setFormatOptions({
          ...DEFAULT_FORMAT_OPTIONS,
          ...urlState.formatOptions,
        });
        setActiveOutput(urlState.tab ?? "formatted");
      } catch (error) {
        console.error("Error loading state:", error);

        // If it's a critical error (like network failure for pastebin), report it
        if (
          error instanceof Error &&
          error.message.includes("Failed to fetch")
        ) {
          captureAsyncError(error);
        }
      } finally {
        setIsInitializing(false);
        initialized.current = true;
      }
    };

    loadState();
  }, [captureAsyncError]);

  // Save state to URL (only non-default options)
  useEffect(() => {
    if (!initialized.current) {
      // To avoid url state being overridden
      return;
    }
    const nondefaultOptions = filterNonDefaultOptions(formatOptions);

    // Update URL with current state
    updateUrlWithState(deferredSourceCode, nondefaultOptions, activeOutput);
  }, [deferredSourceCode, formatOptions, activeOutput]);

  // Save code to local storage
  useEffect(() => {
    if (!initialized.current) {
      return;
    }

    storeSourceCode(deferredSourceCode);
  }, [deferredSourceCode]);

  return {
    state: {
      sourceCode,
      deferredSourceCode,
      formatOptions,
      isInitializing,
      activeOutput,
      cursorSyncEnabled,
    },
    setSourceCode,
    setFormatOptions,
    setActiveOutput,
    setCursorSyncEnabled,
  };
}

/**
 * Load source code from LocalStorage.
 */
function loadSourceCode() {
  return localStorage.getItem(STORAGE_KEY);
}

/**
 * Store source code to LocalStorage.
 */
function storeSourceCode(sourceCode: string) {
  localStorage.setItem(STORAGE_KEY, sourceCode);
}
