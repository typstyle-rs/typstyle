import type { ReactNode } from "react";
import type { OutputType, ScreenSizeType } from "@/types";
import { Tab, Tabs } from "./ui";

interface MainLayoutProps {
  screenSize: ScreenSizeType;
  optionsPanel: ReactNode;
  sourcePanel: ReactNode;
  formattedPanel: ReactNode;
  astPanel: ReactNode;
  irPanel: ReactNode;
  activeOutputTab?: OutputType;
  onActiveTabChange?: (activeTab: string) => void;
  cursorSyncEnabled?: boolean;
  onCursorSyncToggle?: (enabled: boolean) => void;
}

export function MainLayout({
  screenSize,
  optionsPanel,
  sourcePanel,
  formattedPanel,
  astPanel,
  irPanel,
  activeOutputTab,
  onActiveTabChange,
  cursorSyncEnabled = true,
  onCursorSyncToggle,
}: MainLayoutProps) {
  return (
    <div className="flex overflow-hidden min-h-0 h-full p-2 gap-2">
      {/* Wide Layout: 3 Columns */}
      {screenSize === "wide" && (
        <>
          <div className="panel flex-none max-w-[280px] card card-border">
            <div className="panel-header font-semibold">Settings</div>
            <div className="panel-content">{optionsPanel}</div>
          </div>
          <div className="panel flex-1 min-w-0 card card-border">
            <div className="panel-header font-semibold flex items-center justify-between">
              <span>Source</span>
              <button
                type="button"
                className={`btn btn-xs btn-ghost gap-1 ${cursorSyncEnabled ? "btn-active" : "opacity-50"}`}
                title={
                  cursorSyncEnabled
                    ? "Cursor sync enabled"
                    : "Cursor sync disabled"
                }
                onClick={() => onCursorSyncToggle?.(!cursorSyncEnabled)}
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                  className="w-4 h-4"
                >
                  <path d="M12.232 4.232a2.5 2.5 0 0 1 3.536 3.536l-1.225 1.224a.75.75 0 0 0 1.061 1.06l1.224-1.224a4 4 0 0 0-5.656-5.656l-3 3a4 4 0 0 0 .225 5.865.75.75 0 0 0 .977-1.138 2.5 2.5 0 0 1-.142-3.667l3-3Z" />
                  <path d="M11.603 7.963a.75.75 0 0 0-.977 1.138 2.5 2.5 0 0 1 .142 3.667l-3 3a2.5 2.5 0 0 1-3.536-3.536l1.225-1.224a.75.75 0 0 0-1.061-1.06l-1.224 1.224a4 4 0 1 0 5.656 5.656l3-3a4 4 0 0 0-.225-5.865Z" />
                </svg>
                Sync
              </button>
            </div>
            <div className="panel-content">{sourcePanel}</div>
          </div>
          <div className="panel flex-1 min-w-0 card card-border">
            <div className="panel-content">
              <Tabs
                defaultActiveTab="formatted"
                activeTab={activeOutputTab}
                className="bg-base-300"
                tabClassName="font-semibold flex-1"
                contentClassName="bg-base-100 border-base-300"
                onTabChange={onActiveTabChange}
              >
                <Tab tid="formatted" label="Formatted">
                  {formattedPanel}
                </Tab>
                <Tab tid="ast" label="AST">
                  {astPanel}
                </Tab>
                <Tab tid="pir" label="Pretty IR">
                  {irPanel}
                </Tab>
              </Tabs>
            </div>
          </div>
        </>
      )}

      {/* Thin Layout: 1 Column (Full Width) */}
      {screenSize === "thin" && (
        <div className="panel flex-1 min-w-0 card card-border">
          <div className="panel-content">
            <Tabs
              defaultActiveTab={
                activeOutputTab && activeOutputTab !== "formatted"
                  ? activeOutputTab
                  : "source"
              }
              className="bg-base-300"
              tabClassName="font-semibold flex-1"
              contentClassName="bg-base-100 border-base-300"
              onTabChange={onActiveTabChange}
            >
              <Tab tid="options" label="Settings">
                {optionsPanel}
              </Tab>
              <Tab tid="source" label="Source">
                {sourcePanel}
              </Tab>
              <Tab tid="formatted" label="Formatted">
                {formattedPanel}
              </Tab>
              <Tab tid="ast" label="AST">
                {astPanel}
              </Tab>
              <Tab tid="pir" label="Pretty IR">
                {irPanel}
              </Tab>
            </Tabs>
          </div>
        </div>
      )}
    </div>
  );
}
