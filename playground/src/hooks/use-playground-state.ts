import { useDeferredValue, useEffect, useRef, useState } from "react";
import {
  DEFAULT_FORMAT_OPTIONS,
  type FormatOptions,
  filterNonDefaultOptions,
} from "@/utils/formatter";
import { getStateFromUrl, updateUrlWithState } from "@/utils/url";

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
        console.log("load state", urlState);
        if (urlState) {
          setSourceCode(urlState.sourceCode);
          setFormatOptions({
            ...DEFAULT_FORMAT_OPTIONS,
            ...urlState.formatOptions,
          });
        }
      } catch (error) {
        console.error("Error loading state:", error);
      }
      initialized.current = true;
    };

    loadState();
  }, []);

  // Save state to localStorage and update URL (only non-default options)
  useEffect(() => {
    if (!initialized.current) {
      // To avoid url state being overridden
      return;
    }
    const nondefaultOptions = filterNonDefaultOptions(formatOptions);

    // Update URL with current state
    updateUrlWithState(sourceCode, nondefaultOptions);
  }, [deferredSourceCode, formatOptions]);

  return {
    state: { sourceCode, formatOptions },
    setSourceCode,
    setFormatOptions,
  };
}
