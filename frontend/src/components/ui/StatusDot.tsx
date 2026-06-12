import styles from "./StatusDot.module.css";

/** Structurally matches `NodeStatus` from lib/runStream so node/run state maps
 * directly, while keeping the ui layer dependency-free. */
export type StatusKind = "idle" | "running" | "complete" | "error" | "skipped";

const LABELS: Record<StatusKind, string> = {
  idle: "Idle",
  running: "Running",
  complete: "Complete",
  error: "Error",
  skipped: "Skipped",
};

interface StatusDotProps {
  status: StatusKind;
  size?: "sm" | "md";
  /** Visually-hidden status label for screen readers. Omit if a sibling text
   * node already names the state (color is never the sole signal — spec §7). */
  label?: string;
  className?: string;
}

export function StatusDot({ status, size = "sm", label, className }: StatusDotProps) {
  const classes = [styles.dot, styles[size], styles[status], className]
    .filter(Boolean)
    .join(" ");
  return (
    <span className={classes} role="img" aria-label={label ?? LABELS[status]} />
  );
}
