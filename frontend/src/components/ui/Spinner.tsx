import styles from "./Spinner.module.css";

interface SpinnerProps {
  size?: "sm" | "md";
  /** Accessible label; defaults to "Loading". Pass "" to hide from SR when an
   * adjacent label already conveys the loading state. */
  label?: string;
  className?: string;
}

/** Inline indeterminate spinner (inherits `currentColor`). */
export function Spinner({ size = "sm", label = "Loading", className }: SpinnerProps) {
  const classes = [styles.spinner, styles[size], className].filter(Boolean).join(" ");
  return (
    <span
      className={classes}
      role={label ? "status" : undefined}
      aria-label={label || undefined}
      aria-hidden={label ? undefined : true}
    />
  );
}
