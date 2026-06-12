import { forwardRef, type SelectHTMLAttributes, type ReactNode } from "react";

import { Field, fieldStyles } from "./Field";

interface SelectProps extends Omit<SelectHTMLAttributes<HTMLSelectElement>, "id"> {
  label?: ReactNode;
  id?: string;
  error?: ReactNode;
  hint?: ReactNode;
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(function Select(
  { label, id, error, hint, required, className, children, ...rest },
  ref,
) {
  return (
    <Field label={label} id={id} required={required} error={error} hint={hint}>
      {(fieldProps) => (
        <select
          ref={ref}
          required={required}
          className={[
            fieldStyles.control,
            fieldStyles.select,
            error != null && fieldStyles.invalid,
            className,
          ]
            .filter(Boolean)
            .join(" ")}
          {...fieldProps}
          {...rest}
        >
          {children}
        </select>
      )}
    </Field>
  );
});
