import { useQuery } from "@tanstack/react-query";
import { ipc } from "../../../lib/ipc";
import { useTranslateStore } from "../../../stores/use-translate-store";
import { TranslateToolbar } from "./translate-toolbar";
import { TranslateList } from "./translate-list";

type Props = {
  projectHandle: string;
};

/**
 * Main translate view — replaces center pane when activeRail === 'strings'.
 * Matches wireframe 04 layout: toolbar (36) + col-header (28) + virtualized rows.
 */
export function TranslateView({ projectHandle }: Props) {
  const { sourceLocale, targetLocale, filter } = useTranslateStore();

  const { data: rows = [], isLoading: rowsLoading } = useQuery({
    queryKey: ["translatable", projectHandle, sourceLocale, targetLocale],
    queryFn: () =>
      ipc.listTranslatable(projectHandle, sourceLocale, targetLocale),
    staleTime: 30_000,
  });

  const { data: locales = [] } = useQuery({
    queryKey: ["locales", projectHandle],
    queryFn: () => ipc.listLocales(projectHandle),
    staleTime: 60_000,
  });

  return (
    <section className="flex-1 flex flex-col min-w-0 bg-base overflow-hidden">
      {/* Toolbar */}
      <TranslateToolbar
        projectHandle={projectHandle}
        rows={rows}
        locales={locales}
      />

      {/* Column header row — 28px */}
      <div
        className="flex border-b border-[#27272A] bg-surface flex-shrink-0"
        style={{ height: 28 }}
      >
        <div
          className="flex items-center px-4 flex-shrink-0"
          style={{ width: 280 }}
        >
          <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
            Key · ID
          </span>
        </div>
        <div className="flex items-center border-l border-[#27272A] px-4 flex-1">
          <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
            {sourceLocale.toUpperCase()}{" "}
            <span className="text-[#27272A]">·</span> Source
          </span>
        </div>
        <div className="flex items-center border-l border-[#27272A] px-4 flex-1">
          <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
            {targetLocale.toUpperCase()}{" "}
            <span className="text-[#27272A]">·</span> Target
          </span>
        </div>
        <div
          className="flex items-center justify-center border-l border-[#27272A] flex-shrink-0"
          style={{ width: 40 }}
        >
          <i className="ph ph-dots-three text-tertiary text-[12px]" />
        </div>
      </div>

      {/* Rows area */}
      {rowsLoading ? (
        <div className="flex-1 flex items-center justify-center">
          <span className="font-mono text-[12px] text-tertiary px-4 py-2">
            Scanning for translatable strings…
          </span>
        </div>
      ) : rows.length === 0 ? (
        <EmptyState />
      ) : (
        <TranslateList
          rows={rows}
          targetLocale={targetLocale}
          filter={filter}
        />
      )}
    </section>
  );
}

/** Shown when no translatable strings were found in the project. */
function EmptyState() {
  return (
    <div className="flex-1 flex flex-col items-center justify-center gap-4 px-8 text-center">
      <div className="w-12 h-12 rounded-xl bg-[#27272A] flex items-center justify-center">
        <i className="ph ph-translate text-secondary text-2xl" />
      </div>
      <div>
        <div className="font-sans text-[14px] font-[500] text-primary mb-1">
          No translatable strings detected
        </div>
        <div className="font-mono text-[11.5px] text-tertiary leading-relaxed max-w-xs">
          Localization files in{" "}
          <code className="text-secondary">Locales/*</code> or{" "}
          <code className="text-secondary">*_en.json</code> patterns will be
          auto-detected when present.
        </div>
      </div>
      <button
        disabled
        className="h-7 px-3 rounded-md bg-elevated border border-[#27272A] text-[12px] text-secondary font-medium opacity-50 cursor-not-allowed"
        title="Coming in v0.2"
      >
        Add manually
      </button>
    </div>
  );
}
