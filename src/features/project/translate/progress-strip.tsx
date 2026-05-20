type Props = {
  translated: number;
  total: number;
};

/** Progress bar + count matching wireframe 04 toolbar strip. */
export function ProgressStrip({ translated, total }: Props) {
  const pct = total > 0 ? (translated / total) * 100 : 0;
  const pctStr = pct.toFixed(1);

  return (
    <div className="flex items-center gap-2">
      {/* 120px track */}
      <div
        className="rounded-full overflow-hidden bg-[#27272A]"
        style={{ width: 120, height: 6 }}
      >
        <div
          className="h-full bg-accent shadow-[0_0_6px_rgba(16,185,129,0.45)] transition-all"
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="font-mono text-[11px] text-primary">
        <span className="text-accent">{translated}</span>
        <span className="text-tertiary">/{total}</span>
      </span>
      <span className="font-mono text-[10.5px] text-tertiary">({pctStr}%)</span>
    </div>
  );
}
