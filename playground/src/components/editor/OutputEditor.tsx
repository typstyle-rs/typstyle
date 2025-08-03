import { CodeEditor, type CodeEditorRef } from "./CodeEditor";

export interface OutputEditorProps {
  content: string;
  language: string;
  indentSize: number;
  lineLengthGuide?: number;
  ref?: React.Ref<CodeEditorRef>;
}

export function OutputEditor({
  content,
  language,
  indentSize,
  lineLengthGuide,
  ref,
}: OutputEditorProps) {
  return (
    <CodeEditor
      ref={ref}
      value={content}
      indentSize={indentSize}
      language={language}
      readOnly={true}
      options={{
        rulers: lineLengthGuide ? [lineLengthGuide] : [],
      }}
    />
  );
}
