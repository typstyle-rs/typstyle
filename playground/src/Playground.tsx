import { useState } from "react";
import { OutputEditor, SourceEditor } from "./components/editor";
import { FloatingErrorCard } from "./components/FloatingErrorCard";
import { SettingsPanel } from "./components/forms/SettingsPanel";
import { Header } from "./components/Header";
import { MainLayout } from "./components/MainLayout";
import { LoadingSpinner } from "./components/ui";
import { ShareModal } from "./components/ui/ShareModal";
import { ToastContainer } from "./components/ui/ToastContainer";
import {
  usePlaygroundState,
  useScreenSize,
  useShareManager,
  useTypstFormatter,
} from "./hooks";
import type { OutputType } from "./types";

function Playground() {
  const {
    state: { sourceCode, deferredSourceCode, formatOptions, isInitializing },
    setSourceCode,
    setFormatOptions,
  } = usePlaygroundState();
  const [activeOutput, setActiveOutput] = useState<OutputType>("formatted");

  // Custom hooks
  const screenSize = useScreenSize();
  const formatter = useTypstFormatter(
    deferredSourceCode,
    formatOptions,
    activeOutput,
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
  };

  const handleActiveTabChange = (tabId: string) => {
    // Only update activeOutput if the tab is an output type
    if (tabId === "formatted" || tabId === "ast" || tabId === "ir") {
      setActiveOutput(tabId);
    }
  };

  const handleShareClick = async () => {
    const playgroundState = {
      sourceCode,
      formatOptions,
    };
    await shareManager.generateShare(playgroundState);
  };

  const handleShareModalClose = () => {
    shareManager.closeModal();
  };

  const optionsPanel = (
    <SettingsPanel
      formatOptions={formatOptions}
      setFormatOptions={setFormatOptions}
    />
  );
  const sourcePanel = (
    <SourceEditor
      key="source-editor"
      value={sourceCode}
      onChange={handleEditorChange}
      lineLengthGuide={formatOptions.lineWidth}
      formatOptions={formatOptions}
    />
  );
  const formattedPanel = (
    <OutputEditor
      key="output-formatted"
      content={formatter.formattedCode}
      language="typst"
      indentSize={formatOptions.indentWidth}
      lineLengthGuide={formatOptions.lineWidth}
    />
  );
  const astPanel = (
    <OutputEditor
      key="output-ast"
      content={formatter.astOutput}
      language="json"
      indentSize={4}
    />
  );
  const irPanel = (
    <OutputEditor
      key="output-ir"
      content={formatter.irOutput}
      language="python"
      indentSize={4}
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
        onActiveTabChange={handleActiveTabChange}
      />

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
