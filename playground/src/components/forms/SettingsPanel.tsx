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
  const formatDocCommentsId = useId();
  const wrapDocCommentsId = useId();
  const DocCommentWidthId = useId();

  const lineWidthValues = [0, 20, 40, 60, 80, 100, 120];

  const handleReset = () => {
    setFormatOptions(DEFAULT_FORMAT_OPTIONS);
  };

  return (
    <div className="p-2 overflow-y-auto flex flex-wrap gap-3 text-sm *:flex *:items-center *:justify-between *:w-full">
      <div>
        <label htmlFor={lineWidthSelectId}>Line Width:</label>
        <div className="flex gap-1 flex-shrink-0">
          <select
            id={lineWidthSelectId}
            name="lineWidth"
            className="select w-16 px-3"
            value={
              lineWidthValues.includes(formatOptions.lineWidth)
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
            {lineWidthValues.map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
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

      <div>
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

      <div>
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

      <div>
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

      <div>
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

      <div>
        <label htmlFor={formatDocCommentsId}>Format doc comments:</label>
        <input
          id={formatDocCommentsId}
          type="checkbox"
          className="checkbox"
          checked={formatOptions.formatDocComments}
          onChange={(e) =>
            setFormatOptions((prev) => ({
              ...prev,
              formatDocComments: e.target.checked,
            }))
          }
        />
      </div>

      <div>
        <label htmlFor={wrapDocCommentsId}>Wrap doc comments:</label>
        <input
          id={wrapDocCommentsId}
          type="checkbox"
          className="checkbox"
          checked={formatOptions.wrapDocComments}
          onChange={(e) =>
            setFormatOptions((prev) => ({
              ...prev,
              wrapDocComments: e.target.checked,
            }))
          }
        />
      </div>

      <div>
        <label htmlFor={DocCommentWidthId}>Doc comment Width:</label>
        <div className="flex gap-1 flex-shrink-0">
          <input
            id={lineWidthInputId}
            type="number"
            className="input w-16"
            min="0"
            max="200"
            aria-label="Custom Doc Comment Width"
            value={formatOptions.docCommentWidth}
            onChange={(e) =>
              setFormatOptions((prev) => ({
                ...prev,
                docCommentWidth: Number.parseInt(e.target.value),
              }))
            }
          />
        </div>
      </div>

      <button type="button" className="btn w-full" onClick={handleReset}>
        🔄 Reset to Defaults
      </button>
    </div>
  );
}
