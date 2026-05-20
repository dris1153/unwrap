import type { TranslationStatus } from "../../../lib/types";

type Props = {
  status: TranslationStatus;
  size?: number;
};

/** Colored status indicator dot matching wireframe 04 palette. */
export function StatusDot({ status, size = 8 }: Props) {
  const cls =
    status === "Translated"
      ? "bg-[#22C55E] shadow-[0_0_6px_rgba(34,197,94,0.55)]"
      : status === "Review"
        ? "bg-[#F59E0B] shadow-[0_0_6px_rgba(245,158,11,0.55)]"
        : "bg-[#52525B]";

  return (
    <span
      className={`inline-block flex-shrink-0 rounded-full ${cls}`}
      style={{ width: size, height: size }}
    />
  );
}
