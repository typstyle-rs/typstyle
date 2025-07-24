import diff from "fast-diff";
import { useRef } from "react";
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

  const handleEditorMount = (
    editor: editor.IStandaloneCodeEditor,
    monaco: Monaco,
  ) => {
    editorRef.current = editor;

    // Add keyboard shortcuts
    editor.addCommand(
      monaco.KeyMod.Alt | monaco.KeyMod.Shift | monaco.KeyCode.KeyF,
      () => formatDocument(),
      "Format Document",
    );

    editor.addCommand(
      monaco.KeyMod.CtrlCmd | monaco.KeyMod.Alt | monaco.KeyCode.KeyF,
      () => formatSelection(),
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
      run: () => formatDocument(),
    });

    editor.addAction({
      id: "format-selection",
      label: "Format Selection",
      keybindings: [
        monaco.KeyMod.CtrlCmd | monaco.KeyMod.Alt | monaco.KeyCode.KeyF,
      ],
      contextMenuGroupId: "1_modification",
      contextMenuOrder: 1.6,
      run: () => formatSelection(),
    });
  };

  const withEditorAndModel = (
    callback: (
      editor: editor.IStandaloneCodeEditor,
      model: editor.ITextModel,
    ) => void,
  ) => {
    if (!editorRef.current) return;
    const model = editorRef.current.getModel();
    if (!model) return;
    callback(editorRef.current, model);
  };

  const applyFormattingEdits = (
    editor: editor.IStandaloneCodeEditor,
    model: editor.ITextModel,
    originalText: string,
    formattedText: string,
    baseOffset: number = 0,
  ) => {
    if (originalText === formattedText) return;

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

  const formatDocument = () => {
    withEditorAndModel((editor, model) => {
      try {
        const currentValue = model.getValue();
        const config = formatOptionsToConfig(formatOptions);
        const formatted = typstyle.format(currentValue, config);
        applyFormattingEdits(editor, model, currentValue, formatted);
      } catch (error) {
        console.error("Failed to format document:", error);
      }
    });
  };

  const formatSelection = () => {
    withEditorAndModel((editor, model) => {
      const selection = editor.getSelection();
      if (!selection || selection.isEmpty()) return;

      try {
        const fullText = model.getValue();
        const start = model.getOffsetAt({
          lineNumber: selection.startLineNumber,
          column: selection.startColumn,
        });
        const end = model.getOffsetAt({
          lineNumber: selection.endLineNumber,
          column: selection.endColumn,
        });

        const config = formatOptionsToConfig(formatOptions);
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
  };

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

const computeDiffEdits = (
  originalText: string,
  newText: string,
  baseOffset: number,
  model: editor.ITextModel,
): editor.IIdentifiedSingleEditOperation[] => {
  if (originalText === newText) return [];

  const changes = diff(originalText, newText);
  const edits: editor.IIdentifiedSingleEditOperation[] = [];
  let currentOffset = 0;

  for (const [operation, text] of changes) {
    if (operation === diff.EQUAL) {
      currentOffset += text.length;
    } else if (operation === diff.DELETE) {
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
    } else if (operation === diff.INSERT) {
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
    }
  }

  return edits;
};
