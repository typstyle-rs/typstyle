import { useCallback, useState } from "react";

/**
 * Custom hook that provides error boundary functionality for functional components
 * Use this when you need to programmatically trigger error boundary from within a component
 */
export function useErrorHandler() {
  const [, setError] = useState<Error>();

  const throwError = useCallback((error: Error | string) => {
    const errorObj = error instanceof Error ? error : new Error(error);
    setError(() => {
      throw errorObj;
    });
  }, []);

  return throwError;
}

/**
 * Custom hook to capture and handle async errors that might not be caught by error boundaries
 */
export function useAsyncError() {
  const throwError = useErrorHandler();

  const captureAsyncError = useCallback(
    (error: Error | string) => {
      console.error("Async error captured:", error);

      // For development, you might want to throw immediately
      if (process.env.NODE_ENV === "development") {
        throwError(error);
      } else {
        // In production, you might want to report to an error service
        // and show a more graceful error UI
        console.error("Production async error:", error);
      }
    },
    [throwError],
  );

  return captureAsyncError;
}
