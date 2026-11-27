import {
  type Options,
  type PartialOptions,
  resolveOptions,
} from "@typstyle/options";
import { useDeferredValue, useEffect, useMemo, useRef, useState } from "react";
import type { OutputType } from "@/types";
import { getStateFromUrl, updateUrlWithState } from "@/utils/url";
import { useAsyncError } from "./useErrorHandler";

const STORAGE_KEY = "playground-code";

export interface PlaygroundState {
  sourceCode: string;
  formatOptions: Options;
  optionOverrides: PartialOptions;
  activeOutput: OutputType;
}

export function usePlaygroundState() {
  const [sourceCode, setSourceCode] = useState("");
  const [optionOverrides, setOptionOverrides] = useState<PartialOptions>({});
  const [activeOutput, setActiveOutput] = useState<OutputType>("formatted");
  const formatOptions = useMemo(
    () => resolveOptions(optionOverrides),
    [optionOverrides],
  );
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
        setOptionOverrides(urlState.formatOptions);
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
    // Update URL with current state
    updateUrlWithState(deferredSourceCode, optionOverrides, activeOutput);
  }, [deferredSourceCode, optionOverrides, activeOutput]);

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
      optionOverrides,
      isInitializing,
      activeOutput,
      cursorSyncEnabled,
    },
    setSourceCode,
    setOptionOverrides,
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
