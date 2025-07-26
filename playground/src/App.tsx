import { useEffect, useState } from "react";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { LoadingSpinner } from "./components/ui";
import { initMonaco } from "./config/monaco";
import { ThemeProvider } from "./contexts";
import Playground from "./Playground";

function AppContent() {
  const [isMonacoLoaded, setIsMonacoLoaded] = useState(false);
  const [loadingError, setLoadingError] = useState<string | null>(null);

  useEffect(() => {
    const initializeMonaco = async () => {
      try {
        await initMonaco();
        setIsMonacoLoaded(true);
      } catch (error) {
        console.error("Failed to initialize Monaco:", error);
        setLoadingError(
          error instanceof Error ? error.message : "Failed to load editor",
        );
      }
    };

    initializeMonaco();
  }, []);

  if (!isMonacoLoaded) {
    return (
      <LoadingSpinner
        title="Loading Typstyle Playground"
        description="Setting up editor environment..."
        error={loadingError}
        onRetry={loadingError ? () => window.location.reload() : undefined}
      />
    );
  }

  return (
    <ErrorBoundary>
      <Playground />
    </ErrorBoundary>
  );
}

function App() {
  return (
    <ErrorBoundary>
      <ThemeProvider>
        <AppContent />
      </ThemeProvider>
    </ErrorBoundary>
  );
}

export default App;
