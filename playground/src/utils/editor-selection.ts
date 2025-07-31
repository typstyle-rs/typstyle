import type { editor, monaco } from "@/monaco/types";

/**
 * Selection types for range-based operations
 */
export interface EditorSelection {
  startOffset: number;
  endOffset: number;
  isEmpty: boolean;
}

/**
 * Enhanced editor selection with line/column information
 */
export interface DetailedEditorSelection extends EditorSelection {
  startLine: number;
  startColumn: number;
  endLine: number;
  endColumn: number;
  lineCount: number;
  characterCount: number;
}

/**
 * Convert Monaco selection to character offsets and detailed information
 * @param selection Monaco selection object
 * @param model Monaco text model for offset calculations
 */
export function convertSelectionToOffsets(
  selection: monaco.Selection,
  model: editor.ITextModel,
): DetailedEditorSelection {
  const startOffset = model.getOffsetAt({
    lineNumber: selection.startLineNumber,
    column: selection.startColumn,
  });
  const endOffset = model.getOffsetAt({
    lineNumber: selection.endLineNumber,
    column: selection.endColumn,
  });

  const lineCount = selection.endLineNumber - selection.startLineNumber + 1;
  const characterCount = endOffset - startOffset;

  return {
    startOffset,
    endOffset,
    isEmpty: selection.isEmpty(),
    startLine: selection.startLineNumber,
    startColumn: selection.startColumn,
    endLine: selection.endLineNumber,
    endColumn: selection.endColumn,
    lineCount,
    characterCount,
  };
}
