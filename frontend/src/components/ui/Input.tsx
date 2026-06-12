import { forwardRef, type InputHTMLAttributes, type ReactNode } from "react";

import { Field, fieldStyles } from "./Field";

interface InputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "id"> {
  label?: ReactNode;
  id?: string;
  error?: ReactNode;
  hint?: ReactNode;
  counter?: ReactNode;
  counterOver?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(function Input(
  { label, id, error, hint, counter, counterOver, required, className, ...rest },
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
        <input
          ref={ref}
          required={required}
          className={[fieldStyles.control, error != null && fieldStyles.invalid, className]
            .filter(Boolean)
            .join(" ")}
          {...fieldProps}
          {...rest}
        />
      )}
    </Field>
  );
});
