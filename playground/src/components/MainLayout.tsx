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
            <div className="panel-header font-semibold">Source</div>
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
