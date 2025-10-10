// Types for the Typstyle Playground application

import type { EditorSelection } from "./utils/editor-selection";

export type ThemeType = "light" | "dark";

export type ScreenSizeType = "wide" | "thin";

export type OutputType = "formatted" | "ast" | "pir";

export interface RangeFormatterOptions {
  selection: EditorSelection;
  fullText: string;
}
