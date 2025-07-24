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
      monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS,
      () => formatDocument(),
      "Format Document",
    );

    editor.addCommand(
      monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyS,
      () => formatSelection(),
      "Format Selection",
    );

    // Add context menu actions
    editor.addAction({
      id: "format-document",
      label: "Format Document",
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS],
      contextMenuGroupId: "1_modification",
      contextMenuOrder: 1.5,
      run: () => formatDocument(),
    });

    editor.addAction({
      id: "format-selection",
      label: "Format Selection",
      keybindings: [
        monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyS,
      ],
      contextMenuGroupId: "1_modification",
      contextMenuOrder: 1.6,
      run: () => formatSelection(),
    });
  };

  const formatDocument = () => {
    if (!editorRef.current) return;

    const model = editorRef.current.getModel();
    if (!model) return;

    const currentValue = model.getValue();
    const config: Partial<typstyle.Config> =
      formatOptionsToConfig(formatOptions);

    try {
      const formatted = typstyle.format(currentValue, config);
      if (formatted !== "") {
        onChange(formatted);
      }
    } catch (error) {
      console.error("Failed to format document:", error);
    }
  };

  const formatSelection = () => {
    if (!editorRef.current) return;

    const selection = editorRef.current.getSelection();
    if (!selection || selection.isEmpty()) return;

    const model = editorRef.current.getModel();
    if (!model) return;

    // Get the full text to calculate byte offsets
    const fullText = model.getValue();

    // Get character offsets from Monaco
    const start = model.getOffsetAt({
      lineNumber: selection.startLineNumber,
      column: selection.startColumn,
    });
    const end = model.getOffsetAt({
      lineNumber: selection.endLineNumber,
      column: selection.endColumn,
    });

    const config = formatOptionsToConfig(formatOptions);

    try {
      const result = typstyle.format_range(fullText, start, end, config);

      if (result.text !== "") {
        const actualStartChar = result.start;
        const actualEndChar = result.end;

        const startPos = model.getPositionAt(actualStartChar);
        const endPos = model.getPositionAt(actualEndChar);

        const edit: editor.IIdentifiedSingleEditOperation = {
          range: {
            startLineNumber: startPos.lineNumber,
            startColumn: startPos.column,
            endLineNumber: endPos.lineNumber,
            endColumn: endPos.column,
          },
          text: result.text,
        };

        editorRef.current.executeEdits("format-selection", [edit]);
      }
    } catch (error) {
      console.error("Failed to format selection:", error);
    }
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
