import { useCallback, useState } from "react";

export type ToastType = "success" | "error" | "info" | "warning";

export interface ToastState {
  message: string;
  type: ToastType;
  isVisible: boolean;
  id: string;
}

export function useToast() {
  const [toasts, setToasts] = useState<ToastState[]>([]);

  const dismissToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((toast) => toast.id !== id));
  }, []);

  const showToast = useCallback(
    (message: string, type: ToastType = "info") => {
      // Check if there's already a toast with the same message and type
      const existingToast = toasts.find(
        (toast) => toast.message === message && toast.type === type,
      );

      if (existingToast) {
        // If duplicate exists, just return the existing ID
        return existingToast.id;
      }

      // Special handling for share workflow: dismiss "generated" toasts when "copied" appears
      if (message.includes("copied") && type === "success") {
        setToasts((prev) =>
          prev.filter((toast) => !toast.message.includes("generated")),
        );
      }

      const id = Math.random().toString(36).substr(2, 9);
      const newToast: ToastState = {
        message,
        type,
        isVisible: true,
        id,
      };

      setToasts((prev) => {
        // Limit to maximum 3 toasts, remove oldest ones
        const newToasts = [...prev, newToast];
        return newToasts.slice(-3); // Keep only the last 3 toasts
      });

      // Auto-dismiss after 3 seconds
      setTimeout(() => {
        dismissToast(id);
      }, 3000);

      return id;
    },
    [toasts, dismissToast],
  );

  const success = useCallback(
    (message: string) => showToast(message, "success"),
    [showToast],
  );
  const error = useCallback(
    (message: string) => showToast(message, "error"),
    [showToast],
  );
  const info = useCallback(
    (message: string) => showToast(message, "info"),
    [showToast],
  );
  const warning = useCallback(
    (message: string) => showToast(message, "warning"),
    [showToast],
  );

  return {
    toasts,
    showToast,
    dismissToast,
    success,
    error,
    info,
    warning,
  };
}
