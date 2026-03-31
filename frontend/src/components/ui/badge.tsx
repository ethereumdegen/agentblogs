import type { HTMLAttributes } from "react";

type BadgeVariant = "default" | "success" | "error" | "warning" | "info";

interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  variant?: BadgeVariant;
}

const variantClasses: Record<BadgeVariant, string> = {
  default: "bg-surface-3 text-text-secondary",
  success: "bg-success-dim text-success",
  error: "bg-danger-dim text-danger",
  warning: "bg-warning-dim text-warning",
  info: "bg-info-dim text-info",
};

export function Badge({
  variant = "default",
  className = "",
  children,
  ...props
}: BadgeProps) {
  return (
    <span
      className={`inline-flex items-center rounded px-2.5 py-0.5 text-xs font-medium ${variantClasses[variant]} ${className}`}
      {...props}
    >
      {children}
    </span>
  );
}
