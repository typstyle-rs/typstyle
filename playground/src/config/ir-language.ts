import type { Monaco } from "@/monaco/types";

const IR_LANGUAGE_ID = "typstyle-ir";

/**
 * Monarch tokenizer for typstyle Pretty IR debug output.
 *
 * IR format example:
 *   Group([
 *     "#",
 *     "let",
 *     Nest(2, [
 *       LineOrNil,
 *       "size",
 *       WhenBreak(
 *         ",",
 *       ),
 *     ]),
 *     HardLine,
 *   ])
 */
export const registerIrLanguage = (monaco: Monaco) => {
  monaco.languages.register({ id: IR_LANGUAGE_ID });

  monaco.languages.setMonarchTokensProvider(IR_LANGUAGE_ID, {
    keywords: [
      "Nil",
      "Fail",
      "HardLine",
      "ExpandParent",
      "LineOrSpace",
      "LineOrNil",
      "SoftLineOrSpace",
      "SoftLineOrNil",
    ],

    compounds: [
      "Group",
      "Nest",
      "Flatten",
      "DedentToRoot",
      "Align",
      "LineSuffix",
      "WhenBreak",
      "WhenFlat",
      "FlatOrBreak",
      "Union",
      "PartialUnion",
    ],

    tokenizer: {
      root: [
        // String literals (double-quoted, with escapes)
        [/"(?:[^"\\]|\\.)*"/, "string"],

        // Numeric arguments (e.g. in Nest(2, ...))
        [/-?\b\d+\b/, "number"],

        // Standalone keywords (leaf nodes like HardLine, Nil)
        // and compound keywords (Group, Nest, etc.)
        [
          /[A-Z][a-zA-Z]*/,
          {
            cases: {
              "@keywords": "keyword",
              "@compounds": "type",
              "@default": "identifier",
            },
          },
        ],

        // Brackets, parens, commas
        [/[[\]]/, "delimiter.bracket"],
        [/[()]/, "delimiter.parenthesis"],
        [/,/, "delimiter.comma"],
      ],
    },
  });

  monaco.languages.setLanguageConfiguration(IR_LANGUAGE_ID, {
    brackets: [
      ["[", "]"],
      ["(", ")"],
    ],
    autoClosingPairs: [
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: '"', close: '"' },
    ],
    surroundingPairs: [
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: '"', close: '"' },
    ],
  });
};

export { IR_LANGUAGE_ID };
