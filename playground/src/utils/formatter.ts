export interface FormatOptions {
  maxLineLength: number;
  indentSize: number;
  collapseMarkupSpaces: boolean;
  reorderImportItems: boolean;
  wrapText: boolean;
}

// Default format style options
export const DEFAULT_FORMAT_OPTIONS: FormatOptions = {
  maxLineLength: 80,
  indentSize: 2,
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
