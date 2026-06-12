import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { createPortal } from "react-dom";

import styles from "./Toast.module.css";

export type ToastVariant = "success" | "error" | "info";

interface ToastItem {
  id: number;
  message: ReactNode;
  variant: ToastVariant;
}

interface ToastContextValue {
  /** Show an ephemeral toast; auto-dismisses after ~4s (spec §3.7). */
  showToast: (message: ReactNode, variant?: ToastVariant) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

const ICONS: Record<ToastVariant, ReactNode> = {
  success: <path d="M5 12.5l4 4 10-10" strokeLinecap="round" strokeLinejoin="round" />,
  error: <path d="M12 8v5m0 3h.01M12 3a9 9 0 100 18 9 9 0 000-18z" strokeLinecap="round" strokeLinejoin="round" />,
  info: <path d="M12 8h.01M11 12h1v4h1" strokeLinecap="round" strokeLinejoin="round" />,
};

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([]);
  const nextId = useRef(0);

  const dismiss = useCallback((id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const showToast = useCallback(
    (message: ReactNode, variant: ToastVariant = "success") => {
      const id = nextId.current++;
      setToasts((prev) => [...prev, { id, message, variant }]);
      window.setTimeout(() => dismiss(id), 4000);
    },
    [dismiss],
  );

  const value = useMemo(() => ({ showToast }), [showToast]);

  return (
    <ToastContext.Provider value={value}>
      {children}
      {toasts.length > 0 &&
        createPortal(
          <div className={styles.viewport} aria-live="polite" aria-atomic="false">
            {toasts.map((t) => (
              <div key={t.id} className={[styles.toast, styles[t.variant]].join(" ")} role="status">
                <svg className={styles.icon} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
                  {ICONS[t.variant]}
                </svg>
                <span className={styles.message}>{t.message}</span>
                <button type="button" className={styles.dismiss} onClick={() => dismiss(t.id)} aria-label="Dismiss">
                  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
                    <path strokeLinecap="round" d="M6 6l12 12M18 6L6 18" />
                  </svg>
                </button>
              </div>
            ))}
          </div>,
          document.body,
        )}
    </ToastContext.Provider>
  );
}

export function useToast(): ToastContextValue {
  const ctx = useContext(ToastContext);
  if (!ctx) {
    throw new Error("useToast must be used within a ToastProvider");
  }
  return ctx;
}
