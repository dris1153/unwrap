import { cn } from "../../lib/cn";

type DotColor = "accent" | "info" | "warning" | "danger" | "success" | "tertiary";

const colorMap: Record<DotColor, string> = {
  accent:   "bg-accent",
  info:     "bg-info",
  warning:  "bg-warning",
  danger:   "bg-danger",
  success:  "bg-success",
  tertiary: "bg-tertiary",
};

export function StatusDot({
  color = "tertiary",
  pulse = false,
  className,
}: {
  color?: DotColor;
  pulse?: boolean;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-block w-[6px] h-[6px] rounded-full flex-shrink-0",
        colorMap[color],
        pulse && "status-pulse",
        className,
      )}
    />
  );
}
