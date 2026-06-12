import { useId, type ReactNode } from "react";

import styles from "./Field.module.css";

export interface FieldRenderProps {
  id: string;
  "aria-invalid": boolean | undefined;
  "aria-describedby": string | undefined;
}

interface FieldProps {
  label?: ReactNode;
  /** Provide a stable id; otherwise one is generated. */
  id?: string;
  required?: boolean;
  error?: ReactNode;
  hint?: ReactNode;
  /** Right-aligned counter text (e.g. "1,204 / 8,000"); turns danger when over. */
  counter?: ReactNode;
  counterOver?: boolean;
  className?: string;
  /** Render-prop receiving the wired id + aria attributes for the control. */
  children: (props: FieldRenderProps) => ReactNode;
}

/** Label-above-field layout with mandatory htmlFor pairing, an error slot
 * (role="alert"), and a hint/counter footer (spec §3.2). */
export function Field({
  label,
  id,
  required,
  error,
  hint,
  counter,
  counterOver,
  className,
  children,
}: FieldProps) {
  const generated = useId();
  const fieldId = id ?? generated;
  const errorId = `${fieldId}-error`;
  const hintId = `${fieldId}-hint`;
  const describedBy =
    [error ? errorId : null, hint ? hintId : null].filter(Boolean).join(" ") || undefined;
  const showFooter = error != null || hint != null || counter != null;

  return (
    <div className={[styles.field, className].filter(Boolean).join(" ")}>
      {label != null && (
        <label className={styles.label} htmlFor={fieldId}>
          {label}
          {required && (
            <span className={styles.required} aria-hidden="true">
              *
            </span>
          )}
        </label>
      )}
      {children({
        id: fieldId,
        "aria-invalid": error != null ? true : undefined,
        "aria-describedby": describedBy,
      })}
      {showFooter && (
        <div className={styles.footer}>
          {error != null ? (
            <span id={errorId} className={styles.error} role="alert">
              {error}
            </span>
          ) : hint != null ? (
            <span id={hintId} className={styles.hint}>
              {hint}
            </span>
          ) : (
            <span />
          )}
          {counter != null && (
            <span className={[styles.counter, counterOver && styles.counterOver].filter(Boolean).join(" ")}>
              {counter}
            </span>
          )}
        </div>
      )}
    </div>
  );
}

export { styles as fieldStyles };
