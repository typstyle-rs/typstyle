/**
 * Global error handlers for unhandled errors and promise rejections
 * This should be imported in the main entry point
 */

// Handle unhandled promise rejections
window.addEventListener("unhandledrejection", (event) => {
  console.error("Unhandled promise rejection:", event.reason);

  // Prevent the default browser behavior (logging to console)
  event.preventDefault();

  // In development, we might want to be more aggressive
  if (process.env.NODE_ENV === "development") {
    console.error("Unhandled promise rejection details:", {
      promise: event.promise,
      reason: event.reason,
      stack: event.reason?.stack,
    });
  }

  // You could also report to an error tracking service here
  // For example: Sentry.captureException(event.reason);
});

// Handle uncaught JavaScript errors
window.addEventListener("error", (event) => {
  console.error("Uncaught error:", event.error);

  if (process.env.NODE_ENV === "development") {
    console.error("Uncaught error details:", {
      message: event.message,
      filename: event.filename,
      lineno: event.lineno,
      colno: event.colno,
      error: event.error,
      stack: event.error?.stack,
    });
  }

  // You could also report to an error tracking service here
  // For example: Sentry.captureException(event.error);
});

console.log("Global error handlers initialized");
