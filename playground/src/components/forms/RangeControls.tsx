import { useId } from "react";

interface RangeControlsProps {
  isRangeMode: boolean;
  onRangeModeChange: (enabled: boolean) => void;
}

export function RangeControls({
  isRangeMode,
  onRangeModeChange,
}: RangeControlsProps) {
  const rangeModeId = useId();

  return (
    <div className="p-2 border-t border-base-300 text-sm">
      <div className="flex items-center justify-between w-full">
        <label htmlFor={rangeModeId} className="flex flex-col">
          <span className="font-medium">Range Only Mode</span>
          <small className="text-xs opacity-70">
            Format/AST/IR only selected text
          </small>
        </label>
        <input
          id={rangeModeId}
          type="checkbox"
          className="checkbox"
          checked={isRangeMode}
          onChange={(e) => onRangeModeChange(e.target.checked)}
        />
      </div>
    </div>
  );
}
