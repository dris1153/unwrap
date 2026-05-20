import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/cn";

const buttonVariants = cva(
  // base
  "inline-flex items-center justify-center gap-1.5 font-sans font-medium transition-all active:translate-y-[1px] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[rgba(16,185,129,0.20)] focus-visible:border-accent disabled:opacity-50 disabled:pointer-events-none",
  {
    variants: {
      variant: {
        primary:
          "bg-accent text-[#09090B] hover:bg-accent-strong rounded-[var(--radius-md)]",
        secondary:
          "bg-elevated text-primary border border-border-default hover:bg-overlay rounded-[var(--radius-md)]",
        ghost:
          "text-secondary hover:text-primary hover:bg-overlay rounded-[var(--radius-md)]",
        danger:
          "text-danger hover:bg-[rgba(239,68,68,0.10)] rounded-[var(--radius-md)]",
      },
      size: {
        sm: "h-7 px-2.5 text-[12px]",
        md: "h-8 px-3 text-[13px]",
        lg: "h-9 px-4 text-[14px]",
        icon: "h-8 w-8 p-0",
        "icon-sm": "h-7 w-7 p-0",
      },
    },
    defaultVariants: {
      variant: "ghost",
      size: "md",
    },
  },
);

export type ButtonProps = React.ButtonHTMLAttributes<HTMLButtonElement> &
  VariantProps<typeof buttonVariants>;

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => (
    <button
      ref={ref}
      className={cn(buttonVariants({ variant, size }), className)}
      {...props}
    />
  ),
);
Button.displayName = "Button";
