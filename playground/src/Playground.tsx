import { useRef, useState } from "react";
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
import {
  useEditorSelection,
  usePlaygroundState,
  useScreenSize,
  useShareManager,
  useTypstFormatter,
} from "./hooks";
import type { RangeFormatterOptions } from "./types";

function Playground() {
  const {
    state: {
      sourceCode,
      deferredSourceCode,
      formatOptions,
      isInitializing,
      activeOutput,
    },
    setSourceCode,
    setFormatOptions,
    setActiveOutput,
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
      language="json"
      indentSize={2}
    />
  );
  const irPanel = (
    <OutputEditor
      ref={irEditorRef}
      key="output-ir"
      content={formatter.irOutput}
      language="python"
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
