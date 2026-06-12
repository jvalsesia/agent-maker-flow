import styles from "./Skeleton.module.css";

interface SkeletonProps {
  width?: string | number;
  height?: string | number;
  className?: string;
}

/** A single shimmering placeholder block. */
export function Skeleton({ width, height, className }: SkeletonProps) {
  return (
    <span
      className={[styles.skeleton, className].filter(Boolean).join(" ")}
      style={{ width, height }}
      aria-hidden="true"
    />
  );
}

interface SkeletonRowsProps {
  count?: number;
  className?: string;
}

/** Stack of full-width rows for list/table loading states (spec §3.8). */
export function SkeletonRows({ count = 5, className }: SkeletonRowsProps) {
  return (
    <div
      className={[styles.rows, className].filter(Boolean).join(" ")}
      role="status"
      aria-label="Loading"
    >
      {Array.from({ length: count }, (_, i) => (
        <Skeleton key={i} className={styles.row} />
      ))}
    </div>
  );
}
