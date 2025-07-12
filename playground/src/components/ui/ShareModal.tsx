import type { ShareState } from "@/hooks/useShareManager";

interface ShareModalProps {
  shareState: ShareState;
  onCopy: () => Promise<boolean>;
  onClose: () => void;
}

export function ShareModal({ shareState, onCopy, onClose }: ShareModalProps) {
  const handleCopy = async () => {
    await onCopy();
  };

  const handleOverlayClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  if (!shareState.isOpen) return null;

  return (
    <div
      className="fixed inset-0 bg-black/20 backdrop-blur-sm flex items-center justify-center z-40"
      onClick={handleOverlayClick}
    >
      <div className="bg-base-100 rounded-lg p-6 max-w-md w-full mx-4">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-bold">Share Playground</h2>
          <button
            type="button"
            onClick={onClose}
            className="btn btn-ghost btn-circle"
            title="Close"
          >
            ✕
          </button>
        </div>

        <p className="text-sm text-base-content/70 mb-4">
          Copy this link to share your current playground state:
        </p>

        {shareState.usedPastebin && (
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
            value={shareState.url}
            readOnly
            placeholder="Share link"
            title="Share link"
            className="input input-bordered flex-1 text-sm"
            onClick={(e) => e.currentTarget.select()}
          />
          <button
            type="button"
            onClick={handleCopy}
            className={`btn ${
              shareState.copied ? "btn-success" : "btn-primary"
            }`}
          >
            {shareState.copied ? "Copied!" : "Copy"}
          </button>
        </div>
      </div>
    </div>
  );
}
