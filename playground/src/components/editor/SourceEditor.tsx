import diff from "fast-diff";
import { useEffect, useRef } from "react";
import * as typstyle from "typstyle-wasm";
import type { editor, Monaco } from "@/monaco/types";
import { type FormatOptions, formatOptionsToConfig } from "@/utils/formatter";
import { CodeEditor } from "./CodeEditor";

export interface SourceEditorProps {
  value: string;
  onChange: (value: string | undefined) => void;
  lineLengthGuide?: number;
  formatOptions: FormatOptions;
}

export function SourceEditor({
  value,
  onChange,
  lineLengthGuide,
  formatOptions,
}: SourceEditorProps) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);
  // Keep latest options in a ref to ensure formatting functions always use current values
  const formatOptionsRef = useRef(formatOptions);
  useEffect(() => {
    formatOptionsRef.current = formatOptions;
  }, [formatOptions]);

  const handleEditorMount = (
    editor: editor.IStandaloneCodeEditor,
    monaco: Monaco,
  ) => {
    editorRef.current = editor;

    // Add keyboard shortcuts
    editor.addCommand(
      monaco.KeyMod.Alt | monaco.KeyMod.Shift | monaco.KeyCode.KeyF,
      formatDocument,
      "Format Document",
    );

    editor.addCommand(
      monaco.KeyMod.CtrlCmd | monaco.KeyMod.Alt | monaco.KeyCode.KeyF,
      formatSelection,
      "Format Selection",
    );

    // Add context menu actions
    editor.addAction({
      id: "format-document",
      label: "Format Document",
      keybindings: [
        monaco.KeyMod.Alt | monaco.KeyMod.Shift | monaco.KeyCode.KeyF,
      ],
      contextMenuGroupId: "1_modification",
      contextMenuOrder: 1.5,
      run: formatDocument,
    });

    editor.addAction({
      id: "format-selection",
      label: "Format Selection",
      keybindings: [
        monaco.KeyMod.CtrlCmd | monaco.KeyMod.Alt | monaco.KeyCode.KeyF,
      ],
      contextMenuGroupId: "1_modification",
      contextMenuOrder: 1.6,
      run: formatSelection,
    });
  };

  const withEditorState = (
    callback: (
      editor: editor.IStandaloneCodeEditor,
      model: editor.ITextModel,
      fullText: string,
      config: Partial<typstyle.Config>,
    ) => void,
  ) => {
    if (!editorRef.current) return;
    const model = editorRef.current.getModel();
    if (!model) return;
    const fullText = model.getValue();
    // Use ref to ensure we always have the latest format options
    const config = formatOptionsToConfig(formatOptionsRef.current);
    callback(editorRef.current, model, fullText, config);
  };

  /**
   * Apply fine-grained edits to the editor using diff-based changes
   * Only applies actual changes to preserve cursor position and undo history
   */
  const applyFormattingEdits = (
    editor: editor.IStandaloneCodeEditor,
    model: editor.ITextModel,
    originalText: string,
    formattedText: string,
    baseOffset: number = 0,
  ) => {
    if (originalText === formattedText) return;

    // Compute minimal edits using fast-diff algorithm
    const edits = computeDiffEdits(
      originalText,
      formattedText,
      baseOffset,
      model,
    );
    if (edits.length > 0) {
      editor.executeEdits("format", edits);
    }
  };

  /**
   * Format the entire document in place
   * Uses fine-grained edits to maintain cursor position and undo history
   */
  const formatDocument = () =>
    withEditorState((editor, model, fullText, config) => {
      try {
        const formatted = typstyle.format(fullText, config);
        applyFormattingEdits(editor, model, fullText, formatted);
      } catch (error) {
        console.error("Failed to format document:", error);
      }
    });

  /**
   * Format the current selection in place
   * Uses fine-grained edits to maintain cursor position and undo history
   */
  const formatSelection = () =>
    withEditorState((editor, model, fullText, config) => {
      const selection = editor.getSelection();
      if (!selection || selection.isEmpty()) return;

      try {
        // Convert Monaco line/column positions to character offsets
        const start = model.getOffsetAt({
          lineNumber: selection.startLineNumber,
          column: selection.startColumn,
        });
        const end = model.getOffsetAt({
          lineNumber: selection.endLineNumber,
          column: selection.endColumn,
        });

        const result = typstyle.format_range(fullText, start, end, config);
        const originalRangeText = fullText.slice(result.start, result.end);
        applyFormattingEdits(
          editor,
          model,
          originalRangeText,
          result.text,
          result.start,
        );
      } catch (error) {
        console.error("Failed to format selection:", error);
      }
    });

  return (
    <CodeEditor
      value={value}
      language="typst"
      indentSize={0}
      readOnly={false}
      options={{
        wordWrap: "on",
        minimap: { enabled: true },
        rulers: lineLengthGuide ? [lineLengthGuide] : [],
      }}
      onChange={onChange}
      onEditorMount={handleEditorMount}
    />
  );
}

/**
 * Compute fine-grained edits between original and formatted text using diff algorithm
 * Returns Monaco edit operations that represent minimal changes needed
 *
 * @param originalText - The original text to compare
 * @param newText - The formatted text
 * @param baseOffset - Character offset in the full document where this text starts
 * @param model - Monaco text model for position calculations
 * @returns Array of Monaco edit operations
 */
const computeDiffEdits = (
  originalText: string,
  newText: string,
  baseOffset: number,
  model: editor.ITextModel,
): editor.IIdentifiedSingleEditOperation[] => {
  if (originalText === newText) return [];

  // Use fast-diff to compute minimal changes between texts
  const changes = diff(originalText, newText);
  const edits: editor.IIdentifiedSingleEditOperation[] = [];
  let currentOffset = 0;

  // Convert diff changes to Monaco edit operations
  for (const [operation, text] of changes) {
    switch (operation) {
      case diff.EQUAL:
        // Skip unchanged text - just advance the offset
        currentOffset += text.length;
        break;
      case diff.DELETE: {
        // Create delete operation for removed text
        const startOffset = baseOffset + currentOffset;
        const endOffset = baseOffset + currentOffset + text.length;

        const startPos = model.getPositionAt(startOffset);
        const endPos = model.getPositionAt(endOffset);

        edits.push({
          range: {
            startLineNumber: startPos.lineNumber,
            startColumn: startPos.column,
            endLineNumber: endPos.lineNumber,
            endColumn: endPos.column,
          },
          text: "",
        });

        currentOffset += text.length;
        break;
      }
      case diff.INSERT: {
        // Create insert operation for new text
        const startOffset = baseOffset + currentOffset;
        const pos = model.getPositionAt(startOffset);

        edits.push({
          range: {
            startLineNumber: pos.lineNumber,
            startColumn: pos.column,
            endLineNumber: pos.lineNumber,
            endColumn: pos.column,
          },
          text: text,
        });
        break;
      }
    }
  }

  return edits;
};
