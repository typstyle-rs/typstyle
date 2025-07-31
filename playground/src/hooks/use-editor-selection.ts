import { useCallback, useState } from "react";
import type {
  DetailedEditorSelection,
  EditorSelection,
} from "@/utils/editor-selection";

export function useEditorSelection() {
  const [detailedSelection, setDetailedSelection] =
    useState<DetailedEditorSelection | null>(null);

  // Derive basic selection directly - no need for memoization since it changes frequently
  const selection: EditorSelection = detailedSelection
    ? {
        startOffset: detailedSelection.startOffset,
        endOffset: detailedSelection.endOffset,
        isEmpty: detailedSelection.isEmpty,
      }
    : { startOffset: 0, endOffset: 0, isEmpty: true };

  const updateSelection = useCallback(
    (newSelection: DetailedEditorSelection) => {
      setDetailedSelection(newSelection);
    },
    [],
  );

  return {
    selection,
    detailedSelection,
    updateSelection,
  };
}
