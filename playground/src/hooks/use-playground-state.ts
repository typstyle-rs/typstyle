import { useDeferredValue, useEffect, useState } from "react";
import { DEFAULT_FORMAT_OPTIONS } from "../constants";
import type { FormatOptions } from "../types";
import { cleanUrlAfterLoad, getStateFromUrl } from "../utils";

const STORAGE_KEY = "typstyle-playground-state";

export interface PlaygroundState {
  sourceCode: string;
  formatOptions: FormatOptions;
}

export function usePlaygroundState() {
  const [sourceCode, setSourceCode] = useState("");
  const [formatOptions, setFormatOptions] = useState(DEFAULT_FORMAT_OPTIONS);

  // Use deferred value for source code throttling
  const deferredSourceCode = useDeferredValue(sourceCode);

  // Load state on mount
  useEffect(() => {
    const loadState = async () => {
      try {
        const urlState = await getStateFromUrl();
        if (urlState) {
          setSourceCode(urlState.sourceCode || "");
          setFormatOptions({
            ...DEFAULT_FORMAT_OPTIONS,
            ...urlState.formatOptions,
          });
          cleanUrlAfterLoad();
          return;
        }

        const savedState = localStorage.getItem(STORAGE_KEY);
        if (savedState) {
          const parsed = JSON.parse(savedState);
          setSourceCode(parsed.sourceCode || "");
          setFormatOptions({
            ...DEFAULT_FORMAT_OPTIONS,
            ...parsed.formatOptions,
          });
        }
      } catch (error) {
        console.error("Error loading state:", error);
      }
    };

    loadState();
  }, []);

  // Save state to localStorage (throttled for source code, only non-default options)
  useEffect(() => {
    // Build state with only non-default options
    const stateToSave: {
      sourceCode?: string;
      formatOptions?: Partial<FormatOptions>;
    } = {};

    if (deferredSourceCode !== "") {
      stateToSave.sourceCode = deferredSourceCode;
    }

    // Only include format options that differ from defaults
    const nondefaultOptions: Partial<FormatOptions> = Object.fromEntries(
      Object.entries(formatOptions).filter(
        ([key, value]) =>
          value !==
          DEFAULT_FORMAT_OPTIONS[key as keyof typeof DEFAULT_FORMAT_OPTIONS],
      ),
    );
    if (Object.keys(nondefaultOptions).length > 0) {
      stateToSave.formatOptions = nondefaultOptions;
    }

    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(stateToSave));
    } catch (error) {
      console.error("Error saving to localStorage:", error);
    }
  }, [deferredSourceCode, formatOptions]);

  return {
    state: { sourceCode, formatOptions },
    setSourceCode,
    setFormatOptions,
  };
}
