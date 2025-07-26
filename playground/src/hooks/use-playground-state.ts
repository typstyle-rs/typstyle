import { useDeferredValue, useEffect, useRef, useState } from "react";
import {
  DEFAULT_FORMAT_OPTIONS,
  type FormatOptions,
  filterNonDefaultOptions,
} from "@/utils/formatter";
import { getStateFromUrl, updateUrlWithState } from "@/utils/url";

const STORAGE_KEY = "playground-code";

export interface PlaygroundState {
  sourceCode: string;
  formatOptions: FormatOptions;
}

export function usePlaygroundState() {
  const [sourceCode, setSourceCode] = useState("");
  const [formatOptions, setFormatOptions] = useState(DEFAULT_FORMAT_OPTIONS);

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
      } catch (error) {
        console.error("Error loading state:", error);
      }
      initialized.current = true;
    };

    loadState();
  }, []);

  // Save state to URL (only non-default options)
  useEffect(() => {
    if (!initialized.current) {
      // To avoid url state being overridden
      return;
    }
    const nondefaultOptions = filterNonDefaultOptions(formatOptions);

    // Update URL with current state
    updateUrlWithState(sourceCode, nondefaultOptions);
  }, [deferredSourceCode, formatOptions]);

  // Save code to local storage
  useEffect(() => {
    if (!initialized.current) {
      return;
    }

    storeSourceCode(deferredSourceCode);
  }, [deferredSourceCode]);

  return {
    state: { sourceCode, deferredSourceCode, formatOptions },
    setSourceCode,
    setFormatOptions,
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
