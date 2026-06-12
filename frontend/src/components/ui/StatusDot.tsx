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
  /** Status label for screen readers. Defaults to the status name; pass "" to
   * make the dot decorative when a sibling already names the state (spec §7). */
  label?: string;
  className?: string;
}

export function StatusDot({ status, size = "sm", label, className }: StatusDotProps) {
  const classes = [styles.dot, styles[size], styles[status], className]
    .filter(Boolean)
    .join(" ");
  const name = label === "" ? undefined : (label ?? LABELS[status]);
  return (
    <span
      className={classes}
      role={name ? "img" : undefined}
      aria-label={name}
      aria-hidden={name ? undefined : true}
    />
  );
}
