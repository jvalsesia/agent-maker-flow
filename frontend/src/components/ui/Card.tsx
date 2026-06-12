import type { ReactNode } from "react";

import styles from "./Card.module.css";

interface CardProps {
  /** Optional header title (rendered as a section label). */
  title?: ReactNode;
  /** Header-right actions; only renders the header row when title or actions set. */
  actions?: ReactNode;
  /** Pads the body region. Set false for flush content like tables/scrollers. */
  bodyPadded?: boolean;
  children: ReactNode;
  className?: string;
  bodyClassName?: string;
  /** Heading level for the title, for correct document outline. Default h3. */
  as?: "h2" | "h3";
}

/** Surface container with an optional header row (spec §3.3). */
export function Card({
  title,
  actions,
  bodyPadded = true,
  children,
  className,
  bodyClassName,
  as: Heading = "h3",
}: CardProps) {
  const hasHeader = title != null || actions != null;
  const bodyClasses = [styles.body, bodyPadded && styles.bodyPadded, bodyClassName]
    .filter(Boolean)
    .join(" ");

  return (
    <section className={[styles.card, className].filter(Boolean).join(" ")}>
      {hasHeader && (
        <div className={styles.header}>
          {title != null && <Heading className={styles.title}>{title}</Heading>}
          {actions != null && <div className={styles.actions}>{actions}</div>}
        </div>
      )}
      <div className={bodyClasses}>{children}</div>
    </section>
  );
}
