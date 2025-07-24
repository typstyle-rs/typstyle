import * as typstyle from "typstyle-wasm";

export interface FormatOptions {
  lineWidth: number;
  indentWidth: number;
  collapseMarkupSpaces: boolean;
  reorderImportItems: boolean;
  wrapText: boolean;
}

// Default format style options
export const DEFAULT_FORMAT_OPTIONS: FormatOptions = {
  lineWidth: 80,
  indentWidth: 2,
  collapseMarkupSpaces: false,
  reorderImportItems: true,
  wrapText: false,
};

/**
 * Filters format options to only include non-default values
 */
export function filterNonDefaultOptions(
  options: FormatOptions,
): Partial<FormatOptions> {
  return Object.fromEntries(
    Object.entries(options).filter(
      ([key, value]) =>
        value !==
        DEFAULT_FORMAT_OPTIONS[key as keyof typeof DEFAULT_FORMAT_OPTIONS],
    ),
  );
}

/**
 * Converts FormatOptions to typstyle configuration
 */
export function formatOptionsToConfig(
  options: FormatOptions,
): Partial<typstyle.Config> {
  return {
    max_width: options.lineWidth,
    tab_spaces: options.indentWidth,
    collapse_markup_spaces: options.collapseMarkupSpaces,
    reorder_import_items: options.reorderImportItems,
    wrap_text: options.wrapText,
  };
}
