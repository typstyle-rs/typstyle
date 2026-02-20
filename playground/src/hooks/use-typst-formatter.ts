import { useEffect, useState } from "react";
import * as typstyle from "typstyle-wasm";
import type { OutputType, RangeFormatterOptions } from "@/types";
import type { SpanMapping } from "@/utils/offset-mapping";
import { type FormatOptions, formatOptionsToConfig } from "@/utils/formatter";
import { useAsyncError } from "./useErrorHandler";

/** Result from WASM parse_with_mapping API. */
interface MappingResult {
  text: string;
  mapping: SpanMapping[];
}

export interface Formatter {
  formattedCode: string;
  astOutput: string;
  irOutput: string;
  astMapping: SpanMapping[] | null;
  irMapping: SpanMapping[] | null;
  error: string | null;
  update: () => void;
}

export function useTypstFormatter(
  sourceCode: string,
  formatOptions: FormatOptions,
  activeOutput: OutputType,
  rangeOptions?: RangeFormatterOptions,
  isRangeMode: boolean = false,
): Formatter {
  const [formattedCode, setFormattedCode] = useState("");
  const [astOutput, setAstOutput] = useState("");
  const [irOutput, setIrOutput] = useState("");
  const [astMapping, setAstMapping] = useState<SpanMapping[] | null>(null);
  const [irMapping, setIrMapping] = useState<SpanMapping[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const captureAsyncError = useAsyncError();

  const formatCode = async () => {
    const config = formatOptionsToConfig(formatOptions);

    try {
      // Determine if we should use range operations
      const useRange =
        isRangeMode && rangeOptions && !rangeOptions.selection.isEmpty;

      // Only call the WASM function for the currently active output
      switch (activeOutput) {
        case "formatted": {
          const formatted = useRange
            ? typstyle.format_range(
                rangeOptions.fullText,
                rangeOptions.selection.startOffset,
                rangeOptions.selection.endOffset,
                config,
              ).text
            : typstyle.format(sourceCode, config);

          setFormattedCode(formatted);

          // Check for convergence on formatted output (only for full document)
          if (!useRange) {
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
          } else {
            setError(null);
          }
          break;
        }
        case "ast": {
          // Try parse_with_mapping if available (provides scroll sync mapping)
          if (!useRange && "parse_with_mapping" in typstyle) {
            const parseWithMapping = (typstyle as unknown as {
              parse_with_mapping: (text: string) => MappingResult;
            }).parse_with_mapping;
            const result = parseWithMapping(sourceCode);
            setAstOutput(result.text);
            setAstMapping(result.mapping ?? null);
          } else {
            const ast = useRange
              ? typstyle.get_range_ast(
                  rangeOptions.fullText,
                  rangeOptions.selection.startOffset,
                  rangeOptions.selection.endOffset,
                )
              : typstyle.parse(sourceCode);
            setAstOutput(ast);
            setAstMapping(null);
          }
          setError(null);
          break;
        }
        case "pir": {
          const maybeFormatIrWithMapping = Reflect.get(
            typstyle as unknown as Record<string, unknown>,
            "format_ir_with_mapping",
          );
          if (!useRange && typeof maybeFormatIrWithMapping === "function") {
            const result = (
              maybeFormatIrWithMapping as (text: string, config: unknown) => MappingResult
            )(sourceCode, config);
            setIrOutput(result.text);
            setIrMapping(result.mapping ?? null);
          } else {
            const formatIr = useRange
              ? typstyle.format_range_ir(
                  rangeOptions.fullText,
                  rangeOptions.selection.startOffset,
                  rangeOptions.selection.endOffset,
                  config,
                )
              : typstyle.format_ir(sourceCode, config);

            setIrOutput(formatIr);
            setIrMapping(null);
          }
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

  // biome-ignore lint/correctness/useExhaustiveDependencies: spurious
  useEffect(() => {
    formatCode();
  }, [sourceCode, formatOptions, activeOutput, rangeOptions, isRangeMode]);

  return {
    formattedCode,
    astOutput,
    irOutput,
    astMapping,
    irMapping,
    error,
    update: formatCode,
  };
}
