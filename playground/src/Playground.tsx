import { useEffect, useState } from "react";
import { OutputEditor, SourceEditor } from "./components/editor";
import { FloatingErrorCard } from "./components/FloatingErrorCard";
import { SettingsPanel } from "./components/forms/SettingsPanel";
import { Header } from "./components/Header";
import { MainLayout } from "./components/MainLayout";
import { ShareModal } from "./components/ui/ShareModal";
import { ToastContainer } from "./components/ui/ToastContainer";
import { DEFAULT_FORMAT_OPTIONS } from "./constants";
import { useScreenSize, useShareManager, useTypstFormatter } from "./hooks";
import type { FormatOptions, OutputType } from "./types";
import { cleanUrlAfterLoad, getStateFromUrl } from "./utils";

function Playground() {
  const [sourceCode, setSourceCode] = useState("");
  const [formatOptions, setFormatOptions] = useState<FormatOptions>(
    DEFAULT_FORMAT_OPTIONS,
  );
  const [activeOutput, setActiveOutput] = useState<OutputType>("formatted");

  // Custom hooks
  const screenSize = useScreenSize();
  const formatter = useTypstFormatter(sourceCode, formatOptions, activeOutput);
  const shareManager = useShareManager();

  // Load state from URL on component mount
  useEffect(() => {
    const loadStateFromUrl = async () => {
      const urlState = await getStateFromUrl();
      if (urlState) {
        setSourceCode(urlState.sourceCode || "");
        setFormatOptions({
          ...DEFAULT_FORMAT_OPTIONS,
          ...urlState.formatOptions,
        });
        setActiveOutput(urlState.activeOutput || "formatted");

        // Clean the URL after successfully loading the state
        cleanUrlAfterLoad();
      }
    };

    loadStateFromUrl();
  }, []);

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
      activeOutput,
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
      lineLengthGuide={formatOptions.maxLineLength}
    />
  );
  const formattedPanel = (
    <OutputEditor
      key="output-formatted"
      content={formatter.formattedCode}
      language="typst"
      indentSize={formatOptions.indentSize}
      lineLengthGuide={formatOptions.maxLineLength}
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
