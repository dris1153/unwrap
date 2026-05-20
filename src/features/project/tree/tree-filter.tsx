import React, { useEffect, useRef, useState } from "react";
import { MagnifyingGlass } from "@phosphor-icons/react";

type TreeFilterProps = {
  value: string;
  onChange: (v: string) => void;
  resultCount?: number;
};

/**
 * Debounced filter input for the asset tree. Fires onChange 150ms after user stops typing.
 */
export function TreeFilter({ value, onChange, resultCount }: TreeFilterProps) {
  const [local, setLocal] = useState(value);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Sync external resets (e.g. project change)
  useEffect(() => {
    setLocal(value);
  }, [value]);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const v = e.target.value;
    setLocal(v);
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => onChange(v), 150);
  };

  return (
    <div className="px-2 py-2 border-b border-border-default">
      <div className="flex items-center gap-2 bg-base border border-border-default rounded-[var(--radius-md)] h-7 px-2.5 hover:border-border-strong transition-colors focus-within:border-accent focus-within:shadow-[0_0_0_2px_rgba(16,185,129,0.20)]">
        <MagnifyingGlass size={12} className="text-tertiary flex-shrink-0" />
        <input
          type="text"
          placeholder="Filter tree…"
          value={local}
          onChange={handleChange}
          className="flex-1 bg-transparent text-[12px] placeholder:text-tertiary outline-none font-mono"
        />
        {local && resultCount !== undefined && (
          <span className="font-mono text-[10px] text-tertiary flex-shrink-0">
            {resultCount}
          </span>
        )}
      </div>
    </div>
  );
}
