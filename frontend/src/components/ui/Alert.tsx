import type { ReactNode } from "react";

import styles from "./Alert.module.css";

export type AlertVariant = "info" | "success" | "warning" | "danger";

interface AlertProps {
  variant?: AlertVariant;
  title?: ReactNode;
  children?: ReactNode;
  /** Right-aligned actions (e.g. a Retry button). */
  actions?: ReactNode;
  /** Override the live-region role. Errors should use "alert" (assertive). */
  role?: "alert" | "status" | "note";
  className?: string;
  /** Hide the leading icon (e.g. inside dense rows). */
  hideIcon?: boolean;
}

const ICONS: Record<AlertVariant, ReactNode> = {
  info: <path d="M12 8h.01M11 12h1v4h1" strokeLinecap="round" strokeLinejoin="round" />,
  success: <path d="M5 12.5l4 4 10-10" strokeLinecap="round" strokeLinejoin="round" />,
  warning: <path d="M12 9v4m0 3h.01M12 3l9 16H3l9-16z" strokeLinecap="round" strokeLinejoin="round" />,
  danger: <path d="M12 8v5m0 3h.01M12 3a9 9 0 100 18 9 9 0 000-18z" strokeLinecap="round" strokeLinejoin="round" />,
};

/** Persistent contextual banner — distinct from the ephemeral Toast (spec §3.10). */
export function Alert({
  variant = "info",
  title,
  children,
  actions,
  role = variant === "danger" ? "alert" : "status",
  className,
  hideIcon = false,
}: AlertProps) {
  return (
    <div className={[styles.alert, styles[variant], className].filter(Boolean).join(" ")} role={role}>
      {!hideIcon && (
        <svg className={styles.icon} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
          {ICONS[variant]}
        </svg>
      )}
      <div className={styles.content}>
        {title != null && <span className={styles.title}>{title}</span>}
        {children != null && <span className={styles.body}>{children}</span>}
      </div>
      {actions != null && <div className={styles.actions}>{actions}</div>}
    </div>
  );
}
