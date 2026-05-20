import { useState } from "react";
import type { LocaleEntry, TranslatableRowWithDraft } from "../../../lib/types";
import { useTranslateStore, type TranslateFilter } from "../../../stores/use-translate-store";
import { LocalePicker } from "./locale-picker";
import { ProgressStrip } from "./progress-strip";
import { StatusDot } from "./status-dot";
import { ipc } from "../../../lib/ipc";

type Props = {
  projectHandle: string;
  rows: TranslatableRowWithDraft[];
  locales: LocaleEntry[];
};

/**
 * Translate toolbar: locale pickers, progress strip, legend, filter, export, auto-translate.
 * Matches wireframe 04 lines 469-520.
 */
export function TranslateToolbar({ projectHandle, rows, locales }: Props) {
  const { sourceLocale, targetLocale, filter, setSourceLocale, setTargetLocale, setFilter } =
    useTranslateStore();
  const [showAutoModal, setShowAutoModal] = useState(false);
  const [exportLoading, setExportLoading] = useState(false);

  // Compute progress from rows.
  const translated = rows.filter((r) => r.translation?.status === "Translated").length;
  const total = rows.length;

  async function handleExport(format: "csv" | "json") {
    if (exportLoading) return;
    setExportLoading(true);
    try {
      // Use a fixed export path in user's downloads equivalent; frontend Blob fallback.
      const destPath = `${projectHandle}_translations_${targetLocale}.${format}`;
      await ipc.exportTranslations({
        handleId: projectHandle,
        targetLocale,
        format,
        destPath,
      });
    } catch (e) {
      console.error("export failed", e);
    } finally {
      setExportLoading(false);
    }
  }

  return (
    <>
      <div
        className="flex items-center justify-between px-4 border-b border-[#27272A] bg-base flex-shrink-0"
        style={{ height: 36 }}
      >
        <div className="flex items-center gap-3">
          {/* Locale pickers */}
          <div className="flex items-center gap-1.5">
            <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
              Locale
            </span>
            <LocalePicker
              value={sourceLocale}
              locales={locales.filter((l) => l.kind === "source")}
              onChange={setSourceLocale}
            />
            <i className="ph ph-arrow-right text-tertiary text-[11px]" />
            <LocalePicker
              value={targetLocale}
              locales={locales}
              onChange={setTargetLocale}
            />
          </div>

          <span className="text-[#27272A]">·</span>

          {/* Progress strip */}
          <ProgressStrip translated={translated} total={total} />

          <span className="text-[#27272A]">·</span>

          {/* Legend */}
          <div className="flex items-center gap-3 font-mono text-[10px] text-tertiary">
            <span className="flex items-center gap-1.5">
              <StatusDot status="Translated" size={6} />
              translated
            </span>
            <span className="flex items-center gap-1.5">
              <StatusDot status="Pending" size={6} />
              pending
            </span>
            <span className="flex items-center gap-1.5">
              <StatusDot status="Review" size={6} />
              review
            </span>
          </div>
        </div>

        {/* Right controls */}
        <div className="flex items-center gap-1">
          <FilterDropdown value={filter} onChange={setFilter} />

          <ExportDropdown onExport={handleExport} loading={exportLoading} />

          <div className="w-px h-4 bg-[#27272A] mx-1" />

          <button
            onClick={() => setShowAutoModal(true)}
            className="h-7 px-2.5 rounded-md bg-accent text-[#09090B] hover:bg-[#059669] active:translate-y-[1px] text-[11.5px] font-[600] flex items-center gap-1.5 transition-all"
          >
            <i className="ph ph-sparkle text-[13px]" />
            Auto-translate
          </button>
        </div>
      </div>

      {/* Auto-translate coming soon modal */}
      {showAutoModal && (
        <AutoTranslateModal onClose={() => setShowAutoModal(false)} />
      )}
    </>
  );
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

const FILTER_LABELS: Record<TranslateFilter, string> = {
  all: "All",
  translated: "Translated",
  pending: "Pending",
  review: "Review",
};

function FilterDropdown({
  value,
  onChange,
}: {
  value: TranslateFilter;
  onChange: (f: TranslateFilter) => void;
}) {
  const [open, setOpen] = useState(false);

  return (
    <div className="relative">
      <button
        onClick={() => setOpen((o) => !o)}
        className="h-7 px-2.5 rounded-md hover:bg-overlay text-secondary hover:text-primary text-[11.5px] font-medium flex items-center gap-1.5 transition-colors"
      >
        <i className="ph ph-funnel text-[13px]" />
        Filter: {FILTER_LABELS[value]}
        <i className="ph ph-caret-down text-[9px] text-tertiary" />
      </button>

      {open && (
        <div
          className="absolute top-8 right-0 z-50 w-36 bg-elevated border border-[#27272A] rounded-md shadow-xl overflow-hidden"
          onMouseLeave={() => setOpen(false)}
        >
          {(["all", "translated", "pending", "review"] as TranslateFilter[]).map((f) => (
            <button
              key={f}
              onClick={() => { onChange(f); setOpen(false); }}
              className={`w-full px-3 py-1.5 text-left text-[12px] hover:bg-overlay transition-colors ${
                f === value ? "text-accent" : "text-primary"
              }`}
            >
              {FILTER_LABELS[f]}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function ExportDropdown({
  onExport,
  loading,
}: {
  onExport: (fmt: "csv" | "json") => void;
  loading: boolean;
}) {
  const [open, setOpen] = useState(false);

  return (
    <div className="relative">
      <button
        disabled={loading}
        onClick={() => setOpen((o) => !o)}
        className="h-7 px-2.5 rounded-md hover:bg-overlay text-secondary hover:text-primary text-[11.5px] font-medium flex items-center gap-1.5 transition-colors disabled:opacity-50"
      >
        <i className="ph ph-export text-[13px]" />
        Export…
      </button>

      {open && (
        <div
          className="absolute top-8 right-0 z-50 w-32 bg-elevated border border-[#27272A] rounded-md shadow-xl overflow-hidden"
          onMouseLeave={() => setOpen(false)}
        >
          {(["csv", "json"] as const).map((fmt) => (
            <button
              key={fmt}
              onClick={() => { onExport(fmt); setOpen(false); }}
              className="w-full px-3 py-1.5 text-left text-[12px] text-primary hover:bg-overlay transition-colors uppercase font-mono"
            >
              {fmt.toUpperCase()}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function AutoTranslateModal({ onClose }: { onClose: () => void }) {
  return (
    <div
      className="fixed inset-0 z-[100] flex items-center justify-center bg-black/60"
      onClick={onClose}
    >
      <div
        className="bg-elevated border border-[#27272A] rounded-xl p-6 w-80 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-3 mb-4">
          <div className="w-10 h-10 rounded-lg bg-accent/15 border border-accent/30 flex items-center justify-center">
            <i className="ph ph-sparkle text-accent text-xl" />
          </div>
          <div>
            <div className="font-sans text-[15px] font-[600] text-primary">Auto-translate</div>
            <div className="font-mono text-[10.5px] text-tertiary">AI-powered</div>
          </div>
        </div>
        <p className="font-sans text-[13px] text-secondary leading-relaxed mb-5">
          AI translation is coming in{" "}
          <span className="text-accent font-[600]">v0.2</span>. It will
          automatically translate all pending strings using a language model of
          your choice.
        </p>
        <button
          onClick={onClose}
          className="w-full h-8 rounded-md bg-accent text-[#09090B] text-[12.5px] font-[600] hover:bg-[#059669] transition-colors"
        >
          Got it
        </button>
      </div>
    </div>
  );
}
