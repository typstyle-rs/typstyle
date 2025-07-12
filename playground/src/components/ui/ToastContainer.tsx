import type { ToastState } from "@/hooks/useToast";
import { Toast } from "./Toast";

interface ToastContainerProps {
  toasts: ToastState[];
  onDismiss: (id: string) => void;
}

export function ToastContainer({ toasts, onDismiss }: ToastContainerProps) {
  if (toasts.length === 0) return null;

  return (
    <div className="fixed top-4 right-4 z-50 space-y-2">
      {toasts.map((toast) => (
        <Toast
          key={toast.id}
          message={toast.message}
          type={toast.type}
          isVisible={toast.isVisible}
          onClose={() => onDismiss(toast.id)}
          duration={0} // We'll handle auto-dismiss in the hook
        />
      ))}
    </div>
  );
}
