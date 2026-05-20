import * as React from "react";
import { cn } from "../../lib/cn";

/** Keyboard hint pill — e.g. ⌘K */
export function Kbd({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 font-mono text-[10px] text-tertiary",
        "bg-base border border-border-default rounded-[var(--radius-sm)]",
        "px-1.5 py-[1px]",
        className,
      )}
    >
      {children}
    </span>
  );
}
