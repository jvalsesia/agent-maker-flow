import type { ReactNode } from "react";

import styles from "./EmptyState.module.css";

interface EmptyStateProps {
  icon?: ReactNode;
  title: ReactNode;
  description?: ReactNode;
  action?: ReactNode;
  className?: string;
}

/** Centered icon + heading + guidance + primary action (spec §3.9). */
export function EmptyState({ icon, title, description, action, className }: EmptyStateProps) {
  return (
    <div className={[styles.empty, className].filter(Boolean).join(" ")}>
      {icon != null && <div className={styles.icon}>{icon}</div>}
      <div className={styles.title}>{title}</div>
      {description != null && <p className={styles.description}>{description}</p>}
      {action != null && <div className={styles.action}>{action}</div>}
    </div>
  );
}
