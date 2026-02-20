import MonacoEditor from "@monaco-editor/react";
import { useImperativeHandle, useMemo, useRef } from "react";
import { getEditorTheme } from "@/config/monaco";
import { useTheme } from "@/hooks";
import type { editor, Monaco } from "@/monaco/types";

export interface CodeEditorProps {
  value: string;
  language: string;
  indentSize: number; // Positive for fixed indent, 0 or negative for auto-detect
  readOnly?: boolean;
  rulers?: number[];
  options?: editor.IStandaloneEditorConstructionOptions;
  onChange?: (value: string | undefined) => void;
  onEditorMount?: (
    editor: editor.IStandaloneCodeEditor,
    monaco: Monaco,
  ) => void;
  ref?: React.Ref<CodeEditorRef>;
}

export interface CodeEditorRef {
  scrollToTop: () => void;
  revealLine: (lineNumber: number) => void;
  getEditor: () => editor.IStandaloneCodeEditor | null;
}

export function CodeEditor({
  value,
  language,
  indentSize,
  readOnly = false,
  rulers,
  options = {},
  onChange,
  onEditorMount,
  ref,
}: CodeEditorProps) {
  const { theme } = useTheme();
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);

  // Expose scroll functionality through ref
  useImperativeHandle(
    ref,
    () => ({
      scrollToTop: () => {
        editorRef.current?.setScrollTop(0);
      },
      revealLine: (lineNumber: number) => {
        editorRef.current?.revealLineInCenter(lineNumber);
      },
      getEditor: () => editorRef.current,
    }),
    [],
  );

  const handleEditorMount = (
    editor: editor.IStandaloneCodeEditor,
    monaco: Monaco,
  ) => {
    editorRef.current = editor;
    // Call the original onEditorMount if provided
    onEditorMount?.(editor, monaco);
  };

  const editorTheme = useMemo(() => getEditorTheme(theme), [theme]);

  const editorOptions: editor.IStandaloneEditorConstructionOptions = {
    readOnly,
    fontSize: 14,
    fontFamily: "Monaco, Menlo, Ubuntu Mono, monospace",
    automaticLayout: true,
    padding: { top: 8, bottom: 8 },
    renderLineHighlight: readOnly ? "none" : "gutter",
    smoothScrolling: true,
    autoIndent: readOnly ? "none" : "full",
    tabSize: indentSize,
    detectIndentation: indentSize <= 0,
    ...(rulers && rulers.length > 0 && { rulers }),
    ...options,
  };
  return (
    <div className="h-full flex-1 overflow-hidden flex flex-col relative bg-base-200">
      <MonacoEditor
        language={language}
        value={value}
        theme={editorTheme}
        onChange={onChange}
        options={editorOptions}
        onMount={handleEditorMount}
      />
    </div>
  );
}
