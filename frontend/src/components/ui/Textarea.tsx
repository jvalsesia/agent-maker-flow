import { forwardRef, type TextareaHTMLAttributes, type ReactNode } from "react";

import { Field, fieldStyles } from "./Field";

interface TextareaProps extends Omit<TextareaHTMLAttributes<HTMLTextAreaElement>, "id"> {
  label?: ReactNode;
  id?: string;
  error?: ReactNode;
  hint?: ReactNode;
  counter?: ReactNode;
  counterOver?: boolean;
  /** Mono typeface for prompts / system-prompt / memory text (spec §2.2). */
  mono?: boolean;
}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(function Textarea(
  { label, id, error, hint, counter, counterOver, mono, required, className, rows = 4, ...rest },
  ref,
) {
  return (
    <Field
      label={label}
      id={id}
      required={required}
      error={error}
      hint={hint}
      counter={counter}
      counterOver={counterOver}
    >
      {(fieldProps) => (
        <textarea
          ref={ref}
          rows={rows}
          required={required}
          className={[
            fieldStyles.control,
            fieldStyles.textarea,
            mono && fieldStyles.mono,
            error != null && fieldStyles.invalid,
            className,
          ]
            .filter(Boolean)
            .join(" ")}
          {...fieldProps}
          {...rest}
        />
      )}
    </Field>
  );
});
