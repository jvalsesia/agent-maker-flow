import type { ReactNode } from "react";

import styles from "./Badge.module.css";

export type BadgeVariant =
  | "neutral"
  | "accent"
  | "success"
  | "warning"
  | "danger"
  | "running";

interface BadgeProps {
  variant?: BadgeVariant;
  /** Render a leading colored dot (uses the badge's text color). */
  dot?: boolean;
  children: ReactNode;
  className?: string;
  title?: string;
}

/** Small pill for status, provider/model tags, and the node "Root" marker. */
export function Badge({ variant = "neutral", dot = false, children, className, title }: BadgeProps) {
  const classes = [styles.badge, styles[variant], className].filter(Boolean).join(" ");
  return (
    <span className={classes} title={title}>
      {dot && <span className={styles.dot} aria-hidden="true" />}
      {children}
    </span>
  );
}
