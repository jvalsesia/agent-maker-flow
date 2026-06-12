import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from "react";

import { Spinner } from "./Spinner";
import styles from "./Button.module.css";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";
export type ButtonSize = "sm" | "md" | "lg";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  /** Shows a spinner and disables the button; pass loadingLabel to swap text. */
  loading?: boolean;
  loadingLabel?: string;
  /** Square footprint for icon-only actions (provide aria-label). */
  iconOnly?: boolean;
  fullWidth?: boolean;
  leadingIcon?: ReactNode;
}

/** App-wide button. Replaces every bare <button>; covers default/hover/active/
 * focus-visible/disabled/loading states (spec §3.1). */
export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  {
    variant = "secondary",
    size = "md",
    loading = false,
    loadingLabel,
    iconOnly = false,
    fullWidth = false,
    leadingIcon,
    type = "button",
    disabled,
    className,
    children,
    ...rest
  },
  ref,
) {
  const classes = [
    styles.button,
    styles[variant],
    styles[size],
    iconOnly && styles.iconOnly,
    fullWidth && styles.fullWidth,
    className,
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <button
      ref={ref}
      type={type}
      className={classes}
      disabled={disabled || loading}
      aria-busy={loading || undefined}
      {...rest}
    >
      {loading ? <Spinner size="sm" label="" /> : leadingIcon}
      {loading && loadingLabel ? loadingLabel : children}
    </button>
  );
});
