import * as typstyle from "typstyle-wasm";
import { beforeAll, describe, expect, it } from "vitest";
import {
  DEFAULT_FORMAT_OPTIONS,
  formatOptionsToConfig,
} from "@/utils/formatter";

describe("WASM Binding Tests", () => {
  // Helper function to create proper config
  const createConfig = (indentWidth = 2, lineWidth = 80) =>
    formatOptionsToConfig({
      ...DEFAULT_FORMAT_OPTIONS,
      indentWidth,
      lineWidth,
    });

  // Test data
  const simpleTypstCode = `
#set page(width: 10cm, height: auto)
#set heading(numbering: "1.")
= Fibonacci sequence
The Fibonacci sequence is defined through the recurrence relation $F_n = F_(n-1) + F_(n-2)$.
It can also be expressed in _closed form:_
$ F_n = round(1 / sqrt(5) phi.alt^n), quad
  phi.alt = (1 + sqrt(5)) / 2 $
#let count = 8
#let nums = range(1, count + 1)
#let fib(n) = (
  if n <= 2 { 1 }
  else { fib(n - 1) + fib(n - 2) }
)
The first #count numbers of the sequence are:
#align(center, table(
  columns: count,
  ..nums.map(n => $F_#n$),
  ..nums.map(n => str(fib(n))),
))
`;

  const invalidTypstCode = "#invalid(syntax here";

  beforeAll(async () => {
    // Ensure WASM module is properly initialized
    try {
      // Test that the module is loaded
      expect(typstyle).toBeDefined();
      expect(typeof typstyle.format).toBe("function");
    } catch (error) {
      console.error("Failed to initialize WASM module:", error);
      throw error;
    }
  });

  describe("Basic WASM Module Loading", () => {
    it("should load the WASM module successfully", () => {
      expect(typstyle).toBeDefined();
      expect(typeof typstyle.format).toBe("function");
      expect(typeof typstyle.parse).toBe("function");
      expect(typeof typstyle.format_ir).toBe("function");
    });
  });

  describe("Format Function", () => {
    it("should format simple Typst code", () => {
      const config = createConfig(2, 80);
      const result = typstyle.format(simpleTypstCode, config);

      expect(typeof result).toBe("string");
      expect(result).toContain("= Fibonacci sequence");
    });

    it("should handle different formatting options", () => {
      const configs = [
        createConfig(2, 80),
        createConfig(4, 120),
        createConfig(2, 40),
      ];

      for (const config of configs) {
        const result = typstyle.format(simpleTypstCode, config);
        expect(typeof result).toBe("string");
        expect(result.length).toBeGreaterThan(0);
      }
    });

    it("should be idempotent (formatting twice gives same result)", () => {
      const config = createConfig(2, 80);
      const firstFormat = typstyle.format(simpleTypstCode, config);
      const secondFormat = typstyle.format(firstFormat, config);

      expect(firstFormat).toBe(secondFormat);
    });

    it("should handle empty string", () => {
      const config = createConfig(2, 80);
      const result = typstyle.format("", config);

      // Empty strings may return a newline character
      expect(typeof result).toBe("string");
      expect(result.length).toBeLessThanOrEqual(1);
    });
  });

  describe("Parse Function", () => {
    it("should parse simple Typst code and return AST", () => {
      const result = typstyle.parse(simpleTypstCode);

      expect(typeof result).toBe("string");
      expect(result.length).toBeGreaterThan(0);

      // The result appears to be AST representation but not necessarily valid JSON
      // Just check that it contains expected content
      expect(result).toContain("Markup");
    });

    it("should handle empty string in parse", () => {
      const result = typstyle.parse("");

      expect(typeof result).toBe("string");
      expect(result).toContain("Markup");
    });
  });

  describe("Format IR Function", () => {
    it("should return IR representation", () => {
      const config = createConfig(2, 80);
      const result = typstyle.format_ir(simpleTypstCode, config);

      expect(typeof result).toBe("string");
      expect(result.length).toBeGreaterThan(0);
    });
  });

  describe("Error Handling", () => {
    it("should handle invalid syntax gracefully", () => {
      const config = createConfig(2, 80);

      // Invalid syntax should throw an error, which is expected behavior
      expect(() => {
        typstyle.format(invalidTypstCode, config);
      }).toThrow();
    });

    it("should handle malformed config objects", () => {
      // Test with various config variations that should work
      const validConfigs = [
        formatOptionsToConfig(DEFAULT_FORMAT_OPTIONS),
        createConfig(2, 80),
        createConfig(4, 120),
      ];

      for (const config of validConfigs) {
        expect(() => {
          const result = typstyle.format(simpleTypstCode, config);
          expect(typeof result).toBe("string");
        }).not.toThrow();
      }

      // Test that negative values work fine (they get handled internally)
      // The createConfig function doesn't validate, it just passes through
      expect(() => {
        typstyle.format(simpleTypstCode, createConfig(-1, 80));
      }).toThrow(); // This should throw when passed to the WASM function
    });
  });

  describe("Range Formatting", () => {
    it("should format specific ranges", () => {
      const config = createConfig(2, 80);
      const testCode = '#hello("world")\n#goodbye("world")';

      // Test formatting a range (first line only)
      const result = typstyle.format_range(testCode, 0, 15, config);

      expect(result).toBeDefined();
      expect(typeof result.text).toBe("string");
      expect(typeof result.start).toBe("number");
      expect(typeof result.end).toBe("number");
    });

    it("should handle range IR formatting", () => {
      const config = createConfig(2, 80);
      const testCode = '#hello("world")\n#goodbye("world")';

      const result = typstyle.format_range_ir(testCode, 0, 15, config);

      expect(result).toBeDefined();
      expect(typeof result).toBe("string");
    });
  });
});
