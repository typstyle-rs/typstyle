import { useEffect, useState } from "react";
import * as typstyle from "typstyle-wasm";
import type { OutputType } from "@/types";
import { type FormatOptions, formatOptionsToConfig } from "@/utils/formatter";
import { useAsyncError } from "./useErrorHandler";

export interface Formatter {
  formattedCode: string;
  astOutput: string;
  irOutput: string;
  error: string | null;
  update: () => void;
}

export function useTypstFormatter(
  sourceCode: string,
  formatOptions: FormatOptions,
  activeOutput: OutputType,
): Formatter {
  const [formattedCode, setFormattedCode] = useState("");
  const [astOutput, setAstOutput] = useState("");
  const [irOutput, setIrOutput] = useState("");
  const [error, setError] = useState<string | null>(null);
  const captureAsyncError = useAsyncError();

  const formatCode = async () => {
    const config = formatOptionsToConfig(formatOptions);

    try {
      // Only call the WASM function for the currently active output
      switch (activeOutput) {
        case "formatted": {
          const formatted = typstyle.format(sourceCode, config);
          setFormattedCode(formatted);

          // Check for convergence on formatted output
          const secondFormatted = typstyle.format(formatted, config);
          if (secondFormatted !== formatted) {
            setError(
              "Format doesn't converge! " +
                "This means formatting the output again will result in a different output. " +
                "This is a bug in the formatter. " +
                "Please report it to https://github.com/typstyle-rs/typstyle with the input code.",
            );
          } else {
            setError(null);
          }
          break;
        }
        case "ast": {
          const ast = typstyle.parse(sourceCode);
          setAstOutput(ast);
          setError(null);
          break;
        }
        case "ir": {
          const formatIr = typstyle.format_ir(sourceCode, config);
          setIrOutput(formatIr);
          setError(null);
          break;
        }
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      setError(errorMessage);

      // Report critical WASM errors to error boundary in development
      if (error instanceof Error && error.message.includes("RuntimeError")) {
        captureAsyncError(error);
      }

      // Keep previous outputs on error
    }
  };

  useEffect(() => {
    formatCode();
  }, [sourceCode, formatOptions, activeOutput]);

  return {
    formattedCode,
    astOutput,
    irOutput,
    error,
    update: formatCode,
  };
}
