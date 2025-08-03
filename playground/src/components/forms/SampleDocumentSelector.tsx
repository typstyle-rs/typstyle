import { useState } from "react";
import { loadSample, SAMPLES } from "@/utils/sample-loader";

interface SampleDocumentSelectorProps {
  onSampleSelect: (content: string) => void;
}

export function SampleDocumentSelector({
  onSampleSelect,
}: SampleDocumentSelectorProps) {
  const [selectedSample, setSelectedSample] = useState<string>("");
  const [error, setError] = useState<string | null>(null);

  const loadSampleDocument = async (sampleKey: string) => {
    setError(null);

    await loadSample(sampleKey, {
      onSuccess: (content) => {
        onSampleSelect(content);
        setSelectedSample(sampleKey);
      },
      onError: (errorMessage) => {
        setError(errorMessage);
      },
    });
  };

  const handleSampleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    if (value && value in SAMPLES) {
      loadSampleDocument(value);
    } else {
      setSelectedSample("");
      setError(null);
    }
  };

  return (
    <div className="flex items-center gap-2">
      <select
        value={selectedSample}
        onChange={handleSampleChange}
        className="select w-48"
        aria-label="Select a sample Typst document"
      >
        <option value="" disabled>
          Select a sample...
        </option>
        {Object.entries(SAMPLES).map(([key, sample]) => (
          <option key={key} value={key}>
            {sample.name}
          </option>
        ))}
      </select>

      <button
        type="button"
        onClick={() => {
          setSelectedSample("");
          setError(null);
          onSampleSelect("");
        }}
        className="btn btn-square"
        title="Clear document and start fresh"
      >
        🗑️
      </button>

      {/* Error message */}
      {error && (
        <div
          role="alert"
          className="alert alert-error alert-outline text-xs py-1 px-2"
        >
          <span>⚠️ {error}</span>
        </div>
      )}
    </div>
  );
}
