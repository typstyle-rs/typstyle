import { type RefObject, useEffect, useRef } from "react";
import type { CodeEditorRef } from "@/components/editor";
import type { editor } from "@/monaco/types";
import type { OffsetMapping, SpanMapping } from "@/utils/offset-mapping";
import {
  findContainingSpan,
  findContainingSpanReverse,
  queryMapping,
  queryReverseMapping,
  querySpanMapping,
  querySpanMappingReverse,
} from "@/utils/offset-mapping";

export type CursorSyncMapping =
  | { type: "anchor"; data: OffsetMapping }
  | { type: "span"; data: SpanMapping[] };

interface UseCursorSyncOptions {
  sourceRef: RefObject<CodeEditorRef | null>;
  outputRef: RefObject<CodeEditorRef | null>;
  mapping: CursorSyncMapping | null;
  enabled: boolean;
}

function queryForward(mapping: CursorSyncMapping, offset: number): number {
  if (mapping.type === "anchor") return queryMapping(mapping.data, offset);
  return querySpanMapping(mapping.data, offset);
}

function queryReverse(mapping: CursorSyncMapping, offset: number): number {
  if (mapping.type === "anchor")
    return queryReverseMapping(mapping.data, offset);
  return querySpanMappingReverse(mapping.data, offset);
}

// --- Debug: decoration styles ---
const SRC_HIGHLIGHT_CLASS = "cursor-sync-debug-src";
const OUT_HIGHLIGHT_CLASS = "cursor-sync-debug-out";

function setDecorations(
  editorInst: editor.IStandaloneCodeEditor,
  oldIds: string[],
  startOffset: number,
  endOffset: number,
  className: string,
): string[] {
  const model = editorInst.getModel();
  if (!model) return oldIds;

  const startPos = model.getPositionAt(startOffset);
  const endPos = model.getPositionAt(endOffset);

  return editorInst.deltaDecorations(oldIds, [
    {
      range: {
        startLineNumber: startPos.lineNumber,
        startColumn: startPos.column,
        endLineNumber: endPos.lineNumber,
        endColumn: endPos.column,
      },
      options: {
        className,
        isWholeLine: false,
      },
    },
  ]);
}

function setDecorationsMany(
  editorInst: editor.IStandaloneCodeEditor,
  oldIds: string[],
  ranges: Array<{ startOffset: number; endOffset: number }>,
  className: string,
): string[] {
  const model = editorInst.getModel();
  if (!model) return oldIds;

  const decorations = ranges.map(({ startOffset, endOffset }) => {
    const startPos = model.getPositionAt(startOffset);
    const endPos = model.getPositionAt(endOffset);
    return {
      range: {
        startLineNumber: startPos.lineNumber,
        startColumn: startPos.column,
        endLineNumber: endPos.lineNumber,
        endColumn: endPos.column,
      },
      options: {
        className,
        isWholeLine: false,
      },
    };
  });

  return editorInst.deltaDecorations(oldIds, decorations);
}

function clearDecorations(
  editorInst: editor.IStandaloneCodeEditor | undefined | null,
  ids: string[],
): string[] {
  if (!editorInst || ids.length === 0) return [];
  return editorInst.deltaDecorations(ids, []);
}

/**
 * Hook that synchronizes cursor position between source and output editors
 * based on an offset mapping.
 *
 * Only responds to cursor position changes (clicks / keyboard navigation).
 * Scroll events are NOT tracked — no scroll following.
 *
 * Uses focus-based direction control: only the focused editor's cursor
 * events trigger sync to the other side.
 *
 * Debug features: highlights mapped spans and logs sync details to console.
 */
export function useCursorSync({
  sourceRef,
  outputRef,
  mapping,
  enabled,
}: UseCursorSyncOptions) {
  // Track which editor has focus: "source" | "output" | null
  const focusedEditor = useRef<"source" | "output" | null>(null);

  // Decoration IDs for cleanup
  const srcDecoIds = useRef<string[]>([]);
  const outDecoIds = useRef<string[]>([]);

  // Inject CSS for debug highlight styles (once)
  useEffect(() => {
    const styleId = "cursor-sync-debug-styles";
    if (document.getElementById(styleId)) return;
    const style = document.createElement("style");
    style.id = styleId;
    style.textContent = `
      .${SRC_HIGHLIGHT_CLASS} { background-color: rgba(59, 130, 246, 0.25); }
      .${OUT_HIGHLIGHT_CLASS} { background-color: rgba(34, 197, 94, 0.25); }
    `;
    document.head.appendChild(style);
  }, []);

  useEffect(() => {
    if (!enabled || !mapping) {
      // Clear decorations when sync is disabled or mapping is null
      const srcEditor = sourceRef.current?.getEditor();
      const outEditor = outputRef.current?.getEditor();
      srcDecoIds.current = clearDecorations(srcEditor, srcDecoIds.current);
      outDecoIds.current = clearDecorations(outEditor, outDecoIds.current);
      return;
    }
    if (mapping.data.length === 0) return;

    const srcEditor = sourceRef.current?.getEditor();
    const outEditor = outputRef.current?.getEditor();
    if (!srcEditor || !outEditor) return;

    // Clear stale decorations from previous tab
    srcDecoIds.current = clearDecorations(srcEditor, srcDecoIds.current);
    outDecoIds.current = clearDecorations(outEditor, outDecoIds.current);

    // --- Focus tracking ---
    const onSrcFocus = () => {
      focusedEditor.current = "source";
    };
    const onSrcBlur = () => {
      if (focusedEditor.current === "source") focusedEditor.current = null;
    };
    const onOutFocus = () => {
      focusedEditor.current = "output";
    };
    const onOutBlur = () => {
      if (focusedEditor.current === "output") focusedEditor.current = null;
    };

    // --- Highlight matched span (span mapping only, not anchor) ---
    const highlightSpan = (
      direction: "source→output" | "output→source",
      srcOffset: number,
      outOffset: number,
    ) => {
      // Skip decorations for anchor mapping (Formatted tab)
      if (mapping.type === "anchor") return;

      const matchedSpan =
        direction === "source→output"
          ? findContainingSpan(mapping.data, srcOffset)
          : findContainingSpanReverse(mapping.data, outOffset);
      if (matchedSpan) {
        const sameSrcSpans = mapping.data.filter(
          (span) =>
            span.srcStart === matchedSpan.srcStart &&
            span.srcEnd === matchedSpan.srcEnd,
        );
        srcDecoIds.current = setDecorations(
          srcEditor,
          srcDecoIds.current,
          matchedSpan.srcStart,
          matchedSpan.srcEnd,
          SRC_HIGHLIGHT_CLASS,
        );
        outDecoIds.current = setDecorationsMany(
          outEditor,
          outDecoIds.current,
          sameSrcSpans.map((span) => ({
            startOffset: span.outStart,
            endOffset: span.outEnd,
          })),
          OUT_HIGHLIGHT_CLASS,
        );
      }
    };

    // --- Cursor position sync (click / keyboard navigation) ---
    const syncCursorToOutput = () => {
      if (focusedEditor.current !== "source") return;

      const srcModel = srcEditor.getModel();
      const outModel = outEditor.getModel();
      if (!srcModel || !outModel) return;

      const pos = srcEditor.getPosition();
      if (!pos) return;

      const srcOffset = srcModel.getOffsetAt(pos);
      const outOffset = queryForward(mapping, srcOffset);
      const outPos = outModel.getPositionAt(outOffset);

      highlightSpan("source→output", srcOffset, outOffset);
      outEditor.revealLineInCenter(outPos.lineNumber);
    };

    const syncCursorToSource = () => {
      if (focusedEditor.current !== "output") return;

      const srcModel = srcEditor.getModel();
      const outModel = outEditor.getModel();
      if (!srcModel || !outModel) return;

      const pos = outEditor.getPosition();
      if (!pos) return;

      const outOffset = outModel.getOffsetAt(pos);
      const srcOffset = queryReverse(mapping, outOffset);
      const srcPos = srcModel.getPositionAt(srcOffset);

      highlightSpan("output→source", srcOffset, outOffset);
      srcEditor.revealLineInCenter(srcPos.lineNumber);
    };

    const disposables = [
      // Focus tracking
      srcEditor.onDidFocusEditorWidget(onSrcFocus),
      srcEditor.onDidBlurEditorWidget(onSrcBlur),
      outEditor.onDidFocusEditorWidget(onOutFocus),
      outEditor.onDidBlurEditorWidget(onOutBlur),
      // Cursor sync only (no scroll sync)
      srcEditor.onDidChangeCursorPosition(syncCursorToOutput),
      outEditor.onDidChangeCursorPosition(syncCursorToSource),
    ];

    return () => {
      for (const d of disposables) d.dispose();
      // Clear decorations on cleanup
      srcDecoIds.current = clearDecorations(srcEditor, srcDecoIds.current);
      outDecoIds.current = clearDecorations(outEditor, outDecoIds.current);
    };
  }, [enabled, mapping, sourceRef, outputRef]);
}
