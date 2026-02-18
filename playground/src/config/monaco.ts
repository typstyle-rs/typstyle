// Playground-specific Monaco setup

import { initMonacoLoader, registerThemes } from "@/monaco";
import {
  DARK_TOKEN_RULES,
  LIGHT_TOKEN_RULES,
} from "@/monaco/theme-registry";
import type { ThemeType } from "@/types";
import { registerAstLanguage } from "./ast-language";
import { registerIrLanguage } from "./ir-language";
import { registerTypstLanguage } from "./typst-language";

const DEFAULT_LIGHT_THEME = "play-light";
const DEFAULT_DARK_THEME = "play-dark";

export const initMonaco = async () => {
  const monaco = await initMonacoLoader();

  // Register themes with custom token rules for AST/IR Monarch languages
  await registerThemes(monaco, [
    {
      name: DEFAULT_LIGHT_THEME,
      url: "https://cdn.jsdelivr.net/npm/@react-monaco/assets/assets/themes/atom-one-light.json",
      extraRules: LIGHT_TOKEN_RULES,
    },
    {
      name: DEFAULT_DARK_THEME,
      url: "https://cdn.jsdelivr.net/npm/@react-monaco/assets/assets/themes/csb-default.json",
      extraRules: DARK_TOKEN_RULES,
    },
  ]);

  // Register languages (Typst is async/TextMate, AST/IR are sync/Monarch)
  registerTypstLanguage(monaco);
  registerAstLanguage(monaco);
  registerIrLanguage(monaco);
};

export const getEditorTheme = (theme: ThemeType): string => {
  return theme === "light" ? DEFAULT_LIGHT_THEME : DEFAULT_DARK_THEME;
};
