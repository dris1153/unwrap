import * as React from "react";
import { cn } from "../../lib/cn";

export type InputProps = React.InputHTMLAttributes<HTMLInputElement> & {
  leadingIcon?: React.ReactNode;
  trailingNode?: React.ReactNode;
};

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, leadingIcon, trailingNode, ...props }, ref) => (
    <div
      className={cn(
        "flex items-center gap-2 bg-base border border-border-default rounded-[var(--radius-md)] h-8 px-2.5",
        "hover:border-border-strong focus-within:border-accent focus-within:ring-2 focus-within:ring-[rgba(16,185,129,0.20)]",
        "transition-all",
        className,
      )}
    >
      {leadingIcon && (
        <span className="text-tertiary text-[14px] flex-shrink-0">
          {leadingIcon}
        </span>
      )}
      <input
        ref={ref}
        className="flex-1 bg-transparent text-[13px] text-primary placeholder:text-tertiary outline-none font-sans min-w-0"
        {...props}
      />
      {trailingNode && (
        <span className="flex-shrink-0">{trailingNode}</span>
      )}
    </div>
  ),
);
Input.displayName = "Input";
