import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useTypstFormatter } from "@/hooks/use-typst-formatter";
import type { OutputType } from "@/types";
import { DEFAULT_FORMAT_OPTIONS } from "@/utils/formatter";

// Mock the error handler
vi.mock("@/hooks/useErrorHandler", () => ({
  useAsyncError: () => vi.fn(),
}));

describe("Playground Integration Tests", () => {
  describe("useTypstFormatter Hook", () => {
    it("should format code correctly", async () => {
      const testCode = '#hello("world")';

      const { result } = renderHook(() =>
        useTypstFormatter(testCode, DEFAULT_FORMAT_OPTIONS, "formatted"),
      );

      // Wait for the formatting to complete
      await waitFor(() => {
        expect(result.current.formattedCode).toBeTruthy();
      });

      expect(result.current.formattedCode).toContain('#hello("world")');
      expect(result.current.error).toBeNull();
    });

    it("should handle AST output", async () => {
      const testCode = '#hello("world")';

      const { result } = renderHook(() =>
        useTypstFormatter(testCode, DEFAULT_FORMAT_OPTIONS, "ast"),
      );

      await waitFor(() => {
        expect(result.current.astOutput).toBeTruthy();
      });

      expect(result.current.astOutput.length).toBeGreaterThan(0);
    });

    it("should handle IR output", async () => {
      const testCode = '#hello("world")';

      const { result } = renderHook(() =>
        useTypstFormatter(testCode, DEFAULT_FORMAT_OPTIONS, "pir"),
      );

      await waitFor(() => {
        expect(result.current.irOutput).toBeTruthy();
      });

      expect(result.current.irOutput.length).toBeGreaterThan(0);
    });

    it("should handle formatting errors gracefully", async () => {
      const invalidCode = "#invalid(syntax here";

      const { result } = renderHook(() =>
        useTypstFormatter(invalidCode, DEFAULT_FORMAT_OPTIONS, "formatted"),
      );

      // The hook should handle errors without crashing
      await waitFor(() => {
        // Either we get a formatted result or an error, but not a crash
        expect(
          result.current.formattedCode !== "" || result.current.error !== null,
        ).toBe(true);
      });
    });

    it("should update when source code changes", async () => {
      const initialCode = '#hello("world")';
      const updatedCode = '#goodbye("world")';

      const { result, rerender } = renderHook(
        ({ code }) =>
          useTypstFormatter(code, DEFAULT_FORMAT_OPTIONS, "formatted"),
        { initialProps: { code: initialCode } },
      );

      // Wait for initial format
      await waitFor(() => {
        expect(result.current.formattedCode).toContain("hello");
      });

      // Update the code
      rerender({ code: updatedCode });

      // Wait for updated format
      await waitFor(() => {
        expect(result.current.formattedCode).toContain("goodbye");
      });
    });

    it("should update when output type changes", async () => {
      const testCode = '#hello("world")';

      const { result, rerender } = renderHook(
        ({ outputType }) =>
          useTypstFormatter(testCode, DEFAULT_FORMAT_OPTIONS, outputType),
        { initialProps: { outputType: "formatted" as OutputType } },
      );

      // Wait for formatted output
      await waitFor(() => {
        expect(result.current.formattedCode).toBeTruthy();
      });

      const initialFormatted = result.current.formattedCode;

      // Change to AST output
      rerender({ outputType: "ast" as OutputType });

      // Wait for AST output
      await waitFor(() => {
        expect(result.current.astOutput).toBeTruthy();
      });

      expect(result.current.astOutput).not.toBe(initialFormatted);
    });
  });
});
