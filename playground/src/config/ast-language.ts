import type { Monaco } from "@/monaco/types";

const AST_LANGUAGE_ID = "typstyle-ast";

/**
 * Monarch tokenizer for typstyle AST debug output.
 *
 * AST format example:
 *   Markup: 33 [
 *     Hash: "#",
 *     LetBinding: 9 [
 *       Let: "let",
 *       Space: " ",
 *       Ident: "x",
 *     ],
 *   ]
 */
export const registerAstLanguage = (monaco: Monaco) => {
  monaco.languages.register({ id: AST_LANGUAGE_ID });

  monaco.languages.setMonarchTokensProvider(AST_LANGUAGE_ID, {
    // Keyword AST nodes (purple)
    keywords: [
      "Let",
      "Set",
      "Show",
      "Import",
      "Include",
      "As",
      "In",
      "If",
      "Else",
      "For",
      "While",
      "Return",
      "Break",
      "Continue",
      "Not",
      "And",
      "Or",
      "None",
      "Auto",
      "True",
      "False",
      "Context",
      "Contextual",
    ],

    // Punctuation AST nodes (grey/tag)
    punctuation: [
      "LeftParen",
      "RightParen",
      "LeftBracket",
      "RightBracket",
      "LeftBrace",
      "RightBrace",
      "Comma",
      "Semicolon",
      "Colon",
      "Star",
      "Underscore",
      "Dollar",
      "Plus",
      "Minus",
      "Slash",
      "Hat",
      "Dot",
      "Eq",
      "EqEq",
      "ExclEq",
      "Lt",
      "LtEq",
      "Gt",
      "GtEq",
      "PlusEq",
      "HyphEq",
      "StarEq",
      "SlashEq",
      "DotDot",
      "Arrow",
      "Root",
      "Prime",
      "Hash",
    ],

    tokenizer: {
      root: [
        // String literals (double-quoted, with escapes)
        [/"(?:[^"\\]|\\.)*"/, "string"],

        // Numeric byte lengths (e.g. `: 33 [`)
        [/\b\d+\b/, "number"],

        // Node type names: capitalized identifiers before `:`
        [
          /[A-Z][a-zA-Z]*(?=\s*:)/,
          {
            cases: {
              "@keywords": "keyword",
              "@punctuation": "tag",
              "@default": "type",
            },
          },
        ],

        // Brackets and commas
        [/[[\]]/, "delimiter.bracket"],
        [/,/, "delimiter.comma"],
        [/:/, "delimiter"],
      ],
    },
  });

  monaco.languages.setLanguageConfiguration(AST_LANGUAGE_ID, {
    brackets: [["[", "]"]],
    autoClosingPairs: [
      { open: "[", close: "]" },
      { open: '"', close: '"' },
    ],
    surroundingPairs: [
      { open: "[", close: "]" },
      { open: '"', close: '"' },
    ],
  });
};

export { AST_LANGUAGE_ID };
