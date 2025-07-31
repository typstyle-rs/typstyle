import type { DetailedEditorSelection } from "@/utils/editor-selection";

interface StatusBarProps {
  detailedSelection: DetailedEditorSelection | null;
}

export function StatusBar({ detailedSelection }: StatusBarProps) {
  if (!detailedSelection) {
    return (
      <div className="flex items-center justify-between px-2 py-0.5 text-[10px] bg-base-200 border-t border-base-300">
        <div className="opacity-60">Ready</div>
      </div>
    );
  }

  const position = `Ln ${detailedSelection.startLine}, Col ${detailedSelection.startColumn}`;

  const selectionInfo = detailedSelection.isEmpty
    ? ""
    : detailedSelection.lineCount > 1
      ? ` (${detailedSelection.lineCount} lines, ${detailedSelection.characterCount} chars selected)`
      : ` (${detailedSelection.characterCount} chars selected)`;

  return (
    <div className="flex items-center justify-between px-2 py-0.5 text-[10px] bg-base-200 border-t border-base-300">
      <div className="opacity-60">
        {position}
        {selectionInfo}
      </div>
      {!detailedSelection.isEmpty && (
        <div className="opacity-60">
          Range: {detailedSelection.startLine}:{detailedSelection.startColumn}-
          {detailedSelection.endLine}:{detailedSelection.endColumn}
        </div>
      )}
    </div>
  );
}
