// Generic theme registration utilities for Monaco Editor

import type { editor, Monaco } from "./types";

// Custom token color rules for AST/IR Monarch languages.
// CDN themes only define colors for TextMate scopes; Monarch tokens
// (type, keyword, tag, etc.) need explicit rules to be visible.
export const LIGHT_TOKEN_RULES: editor.ITokenThemeRule[] = [
  { token: "type", foreground: "0070C1" }, // blue — container/compound nodes
  { token: "keyword", foreground: "AF00DB" }, // purple — keyword nodes
  { token: "tag", foreground: "808080" }, // grey — punctuation nodes
  { token: "string", foreground: "A31515" }, // red — string literals
  { token: "number", foreground: "098658" }, // green — numeric values
  { token: "delimiter", foreground: "383A42" }, // dark grey — brackets/commas
];

export const DARK_TOKEN_RULES: editor.ITokenThemeRule[] = [
  { token: "type", foreground: "4EC9B0" }, // teal — container/compound nodes
  { token: "keyword", foreground: "C586C0" }, // purple — keyword nodes
  { token: "tag", foreground: "808080" }, // grey — punctuation nodes
  { token: "string", foreground: "CE9178" }, // orange — string literals
  { token: "number", foreground: "B5CEA8" }, // green — numeric values
  { token: "delimiter", foreground: "ABB2BF" }, // light grey — brackets/commas
];

// Fetches a converted theme from CDN
// NOTE: The actual Monaco theme data is nested under the "data" property
// This structure may change if we use different theme sources or formats
const fetchTheme = async (
  url: string,
): Promise<editor.IStandaloneThemeData> => {
  const response = await fetch(url);
  const themeData = await response.json();
  return themeData.data; // Extract Monaco theme from converted format
};

export const registerTheme = async (
  monaco: Monaco,
  name: string,
  url: string,
  extraRules?: editor.ITokenThemeRule[],
): Promise<void> => {
  try {
    const theme = await fetchTheme(url);
    if (extraRules) {
      theme.rules = [...extraRules, ...theme.rules];
    }
    monaco.editor.defineTheme(name, theme);
  } catch (err) {
    console.error(`Failed to register monaco theme ${name} from ${url}:`, err);
  }
};

export const registerThemes = async (
  monaco: Monaco,
  themes: Array<{ name: string; url: string; extraRules?: editor.ITokenThemeRule[] }>,
): Promise<void> => {
  await Promise.all(
    themes.map(({ name, url, extraRules }) =>
      registerTheme(monaco, name, url, extraRules),
    ),
  );
};
