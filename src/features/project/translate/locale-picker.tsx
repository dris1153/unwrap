import { useState, useRef, useEffect } from "react";
import type { LocaleEntry } from "../../../lib/types";

type Props = {
  value: string;
  locales: LocaleEntry[];
  onChange: (code: string) => void;
  disabled?: boolean;
};

/**
 * Compact locale picker — shows current code, opens a searchable popover
 * listing source locales first then ISO-639 fallback list.
 */
export function LocalePicker({ value, locales, onChange, disabled }: Props) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const ref = useRef<HTMLDivElement>(null);

  // Close on outside click
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
        setSearch("");
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  const filtered = locales.filter((l) =>
    l.code.toLowerCase().includes(search.toLowerCase()),
  );

  const sourceLocales = filtered.filter((l) => l.kind === "source");
  const targetLocales = filtered.filter((l) => l.kind === "target");

  return (
    <div className="relative" ref={ref}>
      <button
        disabled={disabled}
        onClick={() => setOpen((o) => !o)}
        className="h-6 px-2 rounded-[4px] bg-elevated border border-[#27272A] hover:border-[#3F3F46] flex items-center gap-1.5 transition-colors disabled:opacity-50"
      >
        <span className="font-mono text-[10px] font-[700] text-primary tracking-wider">
          {value.toUpperCase()}
        </span>
        <i className="ph ph-caret-down text-tertiary text-[9px]" />
      </button>

      {open && (
        <div className="absolute top-8 left-0 z-50 w-48 bg-elevated border border-[#27272A] rounded-md shadow-xl overflow-hidden">
          {/* Search */}
          <div className="px-2 py-1.5 border-b border-[#27272A]">
            <input
              autoFocus
              type="text"
              placeholder="Search locale…"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="w-full bg-base border border-[#27272A] rounded-[4px] px-2 h-6 text-[11px] font-mono text-primary placeholder:text-tertiary outline-none focus:border-accent"
            />
          </div>

          <div className="max-h-48 overflow-y-auto">
            {sourceLocales.length > 0 && (
              <>
                <div className="px-2 py-1 font-mono text-[9px] text-tertiary tracking-widest uppercase">
                  Detected
                </div>
                {sourceLocales.map((l) => (
                  <LocaleOption
                    key={l.code}
                    entry={l}
                    active={l.code === value}
                    onSelect={() => {
                      onChange(l.code);
                      setOpen(false);
                      setSearch("");
                    }}
                  />
                ))}
              </>
            )}

            {targetLocales.length > 0 && (
              <>
                <div className="px-2 py-1 font-mono text-[9px] text-tertiary tracking-widest uppercase border-t border-[#27272A] mt-0.5 pt-1.5">
                  ISO-639
                </div>
                {targetLocales.map((l) => (
                  <LocaleOption
                    key={l.code}
                    entry={l}
                    active={l.code === value}
                    onSelect={() => {
                      onChange(l.code);
                      setOpen(false);
                      setSearch("");
                    }}
                  />
                ))}
              </>
            )}

            {filtered.length === 0 && (
              <div className="px-2 py-3 text-center font-mono text-[11px] text-tertiary">
                No match
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function LocaleOption({
  entry,
  active,
  onSelect,
}: {
  entry: LocaleEntry;
  active: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      onClick={onSelect}
      className={`w-full flex items-center justify-between px-2 py-1 text-left hover:bg-overlay transition-colors ${
        active ? "text-accent" : "text-primary"
      }`}
    >
      <span className="font-mono text-[11px]">{entry.code}</span>
      {entry.count > 0 && (
        <span className="font-mono text-[10px] text-tertiary">{entry.count}</span>
      )}
    </button>
  );
}
