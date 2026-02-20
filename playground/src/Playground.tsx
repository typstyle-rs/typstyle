import { useMemo, useRef, useState } from "react";
import type { CodeEditorRef } from "./components/editor";
import { OutputEditor, SourceEditor } from "./components/editor";
import { FloatingErrorCard } from "./components/FloatingErrorCard";
import { RangeControls } from "./components/forms/RangeControls";
import { SettingsPanel } from "./components/forms/SettingsPanel";
import { Header } from "./components/Header";
import { MainLayout } from "./components/MainLayout";
import { LoadingSpinner, StatusBar } from "./components/ui";
import { ShareModal } from "./components/ui/ShareModal";
import { ToastContainer } from "./components/ui/ToastContainer";
import { AST_LANGUAGE_ID } from "./config/ast-language";
import { IR_LANGUAGE_ID } from "./config/ir-language";
import {
  useCursorSync,
  useEditorSelection,
  usePlaygroundState,
  useScreenSize,
  useShareManager,
  useTypstFormatter,
} from "./hooks";
import type { RangeFormatterOptions } from "./types";
import { buildOffsetMapping } from "./utils/offset-mapping";

function Playground() {
  const {
    state: {
      sourceCode,
      deferredSourceCode,
      formatOptions,
      isInitializing,
      activeOutput,
      cursorSyncEnabled,
    },
    setSourceCode,
    setFormatOptions,
    setActiveOutput,
    setCursorSyncEnabled,
  } = usePlaygroundState();
  const [isRangeMode, setIsRangeMode] = useState(false);

  // Editor refs for scrolling
  const sourceEditorRef = useRef<CodeEditorRef>(null);
  const formattedEditorRef = useRef<CodeEditorRef>(null);
  const astEditorRef = useRef<CodeEditorRef>(null);
  const irEditorRef = useRef<CodeEditorRef>(null);

  // Custom hooks
  const screenSize = useScreenSize();
  const editorSelection = useEditorSelection();

  // Create range options for the formatter
  const rangeOptions: RangeFormatterOptions | undefined = isRangeMode
    ? {
        selection: editorSelection.selection,
        fullText: sourceCode,
      }
    : undefined;

  const formatter = useTypstFormatter(
    deferredSourceCode,
    formatOptions,
    activeOutput,
    rangeOptions,
    isRangeMode,
  );
  const shareManager = useShareManager();

  // Build offset mapping for cursor sync
  const cursorSyncMapping = useMemo(() => {
    if (activeOutput === "formatted") {
      if (!formatter.formattedCode || formatter.error) return null;
      const anchors = buildOffsetMapping(
        deferredSourceCode,
        formatter.formattedCode,
      );
      if (anchors.length === 0) return null;
      return { type: "anchor" as const, data: anchors };
    }
    // AST: use WASM span mappings
    if (
      activeOutput === "ast" &&
      formatter.astMapping &&
      formatter.astMapping.length > 0
    ) {
      return { type: "span" as const, data: formatter.astMapping };
    }
    return null;
  }, [
    deferredSourceCode,
    formatter.formattedCode,
    formatter.error,
    formatter.astMapping,
    activeOutput,
  ]);

  // Determine which output editor ref to sync with
  const activeOutputRef =
    activeOutput === "formatted"
      ? formattedEditorRef
      : activeOutput === "ast"
        ? astEditorRef
        : irEditorRef;

  // Cursor sync (wide layout only)
  useCursorSync({
    sourceRef: sourceEditorRef,
    outputRef: activeOutputRef,
    mapping: cursorSyncMapping,
    enabled: cursorSyncEnabled && screenSize === "wide",
  });

  // Show loading while playground state is initializing
  if (isInitializing) {
    return (
      <LoadingSpinner
        title="Loading Typstyle Playground"
        description="Initializing states..."
      />
    );
  }

  const handleEditorChange = (value: string | undefined) => {
    if (value !== undefined) {
      setSourceCode(value);
    }
  };

  const handleSampleSelect = (content: string) => {
    setSourceCode(content);
    // Scroll all editors to top
    sourceEditorRef.current?.scrollToTop();
    formattedEditorRef.current?.scrollToTop();
    astEditorRef.current?.scrollToTop();
    irEditorRef.current?.scrollToTop();
  };

  const handleActiveTabChange = (tabId: string) => {
    // Only update activeOutput if the tab is an output type
    if (tabId === "formatted" || tabId === "ast" || tabId === "pir") {
      setActiveOutput(tabId);
    }
  };

  const handleShareClick = async () => {
    const playgroundState = {
      sourceCode,
      formatOptions,
      tab: activeOutput,
    };
    await shareManager.generateShare(playgroundState);
  };

  const handleShareModalClose = () => {
    shareManager.closeModal();
  };

  const optionsPanel = (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-hidden">
        <SettingsPanel
          formatOptions={formatOptions}
          setFormatOptions={setFormatOptions}
        />
      </div>
      <RangeControls
        isRangeMode={isRangeMode}
        onRangeModeChange={setIsRangeMode}
      />
    </div>
  );
  const sourcePanel = (
    <SourceEditor
      ref={sourceEditorRef}
      key="source-editor"
      value={sourceCode}
      onChange={handleEditorChange}
      lineLengthGuide={formatOptions.lineWidth}
      formatOptions={formatOptions}
      onSelectionChange={editorSelection.updateSelection}
    />
  );
  const formattedPanel = (
    <OutputEditor
      ref={formattedEditorRef}
      key="output-formatted"
      content={formatter.formattedCode}
      language="typst"
      indentSize={formatOptions.indentWidth}
      lineLengthGuide={formatOptions.lineWidth}
    />
  );
  const astPanel = (
    <OutputEditor
      ref={astEditorRef}
      key="output-ast"
      content={formatter.astOutput}
      language={AST_LANGUAGE_ID}
      indentSize={2}
    />
  );
  const irPanel = (
    <OutputEditor
      ref={irEditorRef}
      key="output-ir"
      content={formatter.irOutput}
      language={IR_LANGUAGE_ID}
      indentSize={2}
    />
  );

  return (
    <div className="h-screen flex flex-col">
      <Header
        onSampleSelect={handleSampleSelect}
        onShareClick={handleShareClick}
        isGeneratingShare={shareManager.shareState.isGenerating}
      />

      <MainLayout
        screenSize={screenSize}
        optionsPanel={optionsPanel}
        sourcePanel={sourcePanel}
        formattedPanel={formattedPanel}
        astPanel={astPanel}
        irPanel={irPanel}
        activeOutputTab={activeOutput}
        onActiveTabChange={handleActiveTabChange}
        cursorSyncEnabled={cursorSyncEnabled}
        onCursorSyncToggle={setCursorSyncEnabled}
      />

      {/* Status Bar */}
      <StatusBar detailedSelection={editorSelection.detailedSelection} />

      {/* Global floating error card */}
      <FloatingErrorCard error={formatter.error} />

      {/* Toast notifications */}
      <ToastContainer
        toasts={shareManager.toasts}
        onDismiss={shareManager.dismissToast}
      />

      {/* Share Modal */}
      <ShareModal
        shareState={shareManager.shareState}
        onCopy={shareManager.copyShareUrl}
        onClose={handleShareModalClose}
      />
    </div>
  );
}

export default Playground;
