import { useId } from "react";
import { DEFAULT_FORMAT_OPTIONS, type FormatOptions } from "@/utils/formatter";

interface SettingsPanelProps {
  formatOptions: FormatOptions;
  setFormatOptions: React.Dispatch<React.SetStateAction<FormatOptions>>;
}

export function SettingsPanel({
  formatOptions,
  setFormatOptions,
}: SettingsPanelProps) {
  const lineWidthSelectId = useId();
  const lineWidthInputId = useId();
  const indentWidthSelectId = useId();
  const indentWidthInputId = useId();
  const collapseMarkupSpacesId = useId();
  const reorderImportItemsId = useId();
  const wrapTextId = useId();

  const handleReset = () => {
    setFormatOptions(DEFAULT_FORMAT_OPTIONS);
  };

  return (
    <div className="p-2 overflow-y-auto flex flex-wrap gap-3 text-sm">
      <div className="flex items-center justify-between w-full">
        <label htmlFor={lineWidthSelectId}>Line Width:</label>
        <div className="flex gap-1 flex-shrink-0">
          <select
            id={lineWidthSelectId}
            name="lineWidth"
            className="select w-16 px-3"
            value={
              [40, 60, 80, 100, 120].includes(formatOptions.lineWidth)
                ? formatOptions.lineWidth
                : "custom"
            }
            onChange={(e) => {
              if (e.target.value !== "custom") {
                setFormatOptions((prev) => ({
                  ...prev,
                  lineWidth: Number.parseInt(e.target.value),
                }));
              }
            }}
          >
            <option value="custom" disabled>
              Custom
            </option>
            <option value={0}>0</option>
            <option value={20}>20</option>
            <option value={40}>40</option>
            <option value={60}>60</option>
            <option value={80}>80</option>
            <option value={100}>100</option>
            <option value={120}>120</option>
          </select>
          <input
            id={lineWidthInputId}
            type="number"
            className="input w-16"
            min="0"
            max="200"
            aria-label="Custom Line Width"
            value={formatOptions.lineWidth}
            onChange={(e) =>
              setFormatOptions((prev) => ({
                ...prev,
                lineWidth: Number.parseInt(e.target.value),
              }))
            }
          />
        </div>
      </div>

      <div className="flex items-center justify-between w-full">
        <label htmlFor={indentWidthSelectId}>Indent:</label>
        <div className="flex gap-1 flex-shrink-0">
          <select
            id={indentWidthSelectId}
            name="indentSize"
            className="select w-16 px-3"
            value={
              [2, 4, 8].includes(formatOptions.indentWidth)
                ? formatOptions.indentWidth
                : "custom"
            }
            onChange={(e) => {
              setFormatOptions((prev) => ({
                ...prev,
                indentWidth: Number.parseInt(e.target.value),
              }));
            }}
          >
            <option value="custom" disabled>
              Custom
            </option>
            <option value={2}>2</option>
            <option value={4}>4</option>
            <option value={8}>8</option>
          </select>
          <input
            id={indentWidthInputId}
            type="number"
            className="input w-16"
            min="1"
            max="16"
            aria-label="Custom Indent Size"
            value={formatOptions.indentWidth}
            onChange={(e) =>
              setFormatOptions((prev) => ({
                ...prev,
                indentWidth: Number.parseInt(e.target.value),
              }))
            }
          />
        </div>
      </div>

      <div className="flex items-center justify-between w-full">
        <label htmlFor={collapseMarkupSpacesId}>Collapse Markup Spaces:</label>
        <input
          id={collapseMarkupSpacesId}
          type="checkbox"
          className="checkbox"
          checked={formatOptions.collapseMarkupSpaces}
          onChange={(e) =>
            setFormatOptions((prev) => ({
              ...prev,
              collapseMarkupSpaces: e.target.checked,
            }))
          }
        />
      </div>

      <div className="flex items-center justify-between w-full">
        <label htmlFor={reorderImportItemsId}>Reorder Import Items:</label>
        <input
          id={reorderImportItemsId}
          type="checkbox"
          className="checkbox"
          checked={formatOptions.reorderImportItems}
          onChange={(e) =>
            setFormatOptions((prev) => ({
              ...prev,
              reorderImportItems: e.target.checked,
            }))
          }
        />
      </div>

      <div className="flex items-center justify-between w-full">
        <label htmlFor={wrapTextId}>Wrap Text:</label>
        <input
          id={wrapTextId}
          type="checkbox"
          className="checkbox"
          checked={formatOptions.wrapText}
          onChange={(e) =>
            setFormatOptions((prev) => ({
              ...prev,
              wrapText: e.target.checked,
            }))
          }
        />
      </div>

      <button type="button" className="btn w-full" onClick={handleReset}>
        🔄 Reset to Defaults
      </button>
    </div>
  );
}
