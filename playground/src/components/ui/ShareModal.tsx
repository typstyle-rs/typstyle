import { useState } from "react";
import { copyToClipboard } from "@/utils";

interface ShareModalProps {
  isOpen: boolean;
  onClose: () => void;
  shareUrl: string;
  usedPastebin?: boolean;
}

export function ShareModal({
  isOpen,
  onClose,
  shareUrl,
  usedPastebin = false,
}: ShareModalProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    const success = await copyToClipboard(shareUrl);
    if (success) {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleOverlayClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      onClick={handleOverlayClick}
    >
      <div className="bg-base-100 rounded-lg p-6 max-w-md w-full mx-4">
        <h2 className="text-xl font-bold mb-4">Share Playground</h2>

        <p className="text-sm text-base-content/70 mb-4">
          Copy this link to share your current playground state:
        </p>

        {usedPastebin && (
          <div className="alert alert-info mb-4">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              className="stroke-current shrink-0 w-6 h-6"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              ></path>
            </svg>
            <span className="text-sm">
              Large content was uploaded to pastebin (shz.al) for sharing.
            </span>
          </div>
        )}

        <div className="flex gap-2 mb-4">
          <input
            type="text"
            value={shareUrl}
            readOnly
            placeholder="Share link"
            title="Share link"
            className="input input-bordered flex-1 text-sm"
            onClick={(e) => e.currentTarget.select()}
          />
          <button
            type="button"
            onClick={handleCopy}
            className={`btn ${copied ? "btn-success" : "btn-primary"}`}
          >
            {copied ? "Copied!" : "Copy"}
          </button>
        </div>

        <div className="flex justify-end gap-2">
          <button type="button" onClick={onClose} className="btn btn-ghost">
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
