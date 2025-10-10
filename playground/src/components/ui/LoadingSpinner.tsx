interface LoadingSpinnerProps {
  size?: "sm" | "md" | "lg";
  title: string;
  description?: string;
  error?: string | null;
  onRetry?: () => void;
}

export function LoadingSpinner({
  size = "lg",
  title,
  description,
  error,
  onRetry,
}: LoadingSpinnerProps) {
  if (error) {
    return (
      <div className="h-screen flex flex-col items-center justify-center bg-base-100">
        <div className="flex flex-col items-center gap-4 max-w-md text-center">
          <div className="text-error text-5xl">⚠️</div>
          <div>
            <h2 className="text-xl font-semibold mb-2 text-error">
              Loading Failed
            </h2>
            <p className="text-base-content/70 mb-4">{title}</p>
            <p className="text-sm bg-error/10 p-3 rounded border text-error">
              {error}
            </p>
            {onRetry && (
              <button
                type="button"
                className="btn btn-primary mt-4"
                onClick={onRetry}
              >
                Retry
              </button>
            )}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col items-center justify-center bg-base-100">
      <div className="flex flex-col items-center gap-4">
        <span className={`loading loading-spinner loading-${size}`}></span>
        <div className="text-center">
          <h2 className="text-xl font-semibold mb-2">{title}</h2>
          {description && <p className="text-base-content/70">{description}</p>}
        </div>
      </div>
    </div>
  );
}
