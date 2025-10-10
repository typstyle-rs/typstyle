import { Component, type ErrorInfo, type ReactNode } from "react";

interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
  errorInfo?: ErrorInfo;
}

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: (
    error: Error,
    errorInfo: ErrorInfo,
    retry: () => void,
  ) => ReactNode;
}

export class ErrorBoundary extends Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    // Update state so the next render will show the fallback UI
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log the error to the console and any error reporting service
    console.error("Error Boundary caught an error:", error, errorInfo);

    this.setState({
      error,
      errorInfo,
    });

    // You can also log the error to an error reporting service here
    // For example: Sentry.captureException(error, { contexts: { react: errorInfo } });
  }

  handleRetry = () => {
    this.setState({ hasError: false, error: undefined, errorInfo: undefined });
  };

  render() {
    if (this.state.hasError && this.state.error) {
      // Custom fallback UI if provided
      if (this.props.fallback && this.state.errorInfo) {
        return this.props.fallback(
          this.state.error,
          this.state.errorInfo,
          this.handleRetry,
        );
      }

      // Default fallback UI
      return (
        <div className="h-screen flex flex-col items-center justify-center bg-base-100">
          <div className="flex flex-col items-center gap-6 max-w-2xl text-center p-6">
            <div className="text-error text-6xl">💥</div>
            <div>
              <h1 className="text-2xl font-bold mb-3 text-error">
                Something went wrong
              </h1>
              <p className="text-base-content/80 mb-4">
                The application encountered an unexpected error. This is likely
                a bug in the code.
              </p>

              {/* Error details */}
              <div className="bg-error/10 border border-error/20 rounded-lg p-4 mb-4 text-left">
                <h3 className="font-semibold text-error mb-2">
                  Error Details:
                </h3>
                <p className="text-sm font-mono text-error break-all">
                  {this.state.error.name}: {this.state.error.message}
                </p>
                {this.state.error.stack && (
                  <details className="mt-3">
                    <summary className="cursor-pointer text-sm text-error/80 hover:text-error">
                      Show stack trace
                    </summary>
                    <pre className="text-xs mt-2 overflow-auto max-h-32 text-error/70 whitespace-pre-wrap">
                      {this.state.error.stack}
                    </pre>
                  </details>
                )}
              </div>

              {/* Action buttons */}
              <div className="flex gap-3 justify-center">
                <button
                  type="button"
                  className="btn btn-primary"
                  onClick={this.handleRetry}
                >
                  Try Again
                </button>
                <button
                  type="button"
                  className="btn btn-ghost"
                  onClick={() => window.location.reload()}
                >
                  Reload Page
                </button>
              </div>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
