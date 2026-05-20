import { useCallback } from "react";
import { useDebouncedCallback } from "use-debounce";
import type { TranslatableRowWithDraft, TranslationStatus } from "../../../lib/types";
import { ipc } from "../../../lib/ipc";
import { useTranslateStore } from "../../../stores/use-translate-store";
import { StatusDot } from "./status-dot";

type Props = {
  item: TranslatableRowWithDraft;
  targetLocale: string;
  style?: React.CSSProperties;
};

/**
 * Single translate row matching wireframe 04.
 * Uses <input> (not textarea) for better perf in long lists (see risk table).
 * Active/expanded state is local — clicking into the input expands.
 */
export function TranslateRow({ item, targetLocale, style }: Props) {
  const { dirty, updateDraft } = useTranslateStore();

  // Draft text: optimistic local override > DB translation > empty
  const draftText =
    dirty[item.id] !== undefined
      ? dirty[item.id]
      : (item.translation?.text ?? "");

  const dbStatus: TranslationStatus = item.translation?.status ?? "Pending";
  const effectiveStatus: TranslationStatus =
    dirty[item.id] !== undefined
      ? dirty[item.id].length > 0
        ? "Translated"
        : "Pending"
      : dbStatus;

  // Debounced persist — 500ms after last keystroke.
  const debouncedSave = useDebouncedCallback(
    async (rowId: string, text: string) => {
      try {
        await ipc.saveTranslation({
          rowId,
          targetLocale,
          text,
          status: text.length > 0 ? "Translated" : "Pending",
        });
      } catch (e) {
        console.error("save translation failed", e);
      }
    },
    500,
  );

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const text = e.target.value;
      updateDraft(item.id, text);
      debouncedSave(item.id, text);
    },
    [item.id, updateDraft, debouncedSave],
  );

  const isPending = effectiveStatus === "Pending";
  const isReview = effectiveStatus === "Review";

  // Row background + left border for active/review states.
  const rowClass = isReview
    ? "flex border-b border-[#27272A]/40 hover:bg-base/60"
    : "flex border-b border-[#27272A]/40 hover:bg-base/60";

  return (
    <div className={rowClass} style={{ ...style, minHeight: 76 }}>
      {/* Key cell — 280px */}
      <div className="flex items-start gap-2 px-4 py-3 flex-shrink-0" style={{ width: 280 }}>
        <StatusDot status={effectiveStatus} size={8} />
        <div className="min-w-0 flex-1 mt-[-1px]">
          <div className="font-mono text-[12px] font-[500] text-primary truncate">
            {item.key}
          </div>
          <div className="font-mono text-[10px] text-tertiary mt-0.5 truncate">
            {item.source_path.split(/[/\\]/).pop()}
            {item.comment && (
              <span className="ml-1 text-tertiary">· {item.comment}</span>
            )}
          </div>
        </div>
      </div>

      {/* Source EN cell */}
      <div className="border-l border-[#27272A]/60 px-4 py-3 flex-1 min-w-0">
        <div className="font-mono text-[12.5px] text-secondary leading-[1.55] break-words">
          {item.source_text}
        </div>
        <div className="font-mono text-[10px] text-tertiary mt-1.5">
          {item.source_text.length} char
        </div>
      </div>

      {/* Target VI cell */}
      <div
        className={`border-l border-[#27272A]/60 px-4 py-3 flex-1 min-w-0 ${
          isReview ? "shadow-[inset_2px_0_0_0_#F59E0B]" : ""
        }`}
      >
        {isPending && draftText.length === 0 ? (
          // Pending placeholder — clicking focuses the input via label trick
          <label className="block cursor-text">
            <span className="font-mono text-[12.5px] text-[#52525B] italic leading-[1.55]">
              Add translation…
            </span>
            <input
              type="text"
              value={draftText}
              onChange={handleChange}
              className="sr-only"
              aria-label={`Translation for ${item.key}`}
            />
          </label>
        ) : (
          <input
            type="text"
            value={draftText}
            onChange={handleChange}
            placeholder="Add translation…"
            aria-label={`Translation for ${item.key}`}
            className="w-full bg-transparent border-none outline-none font-mono text-[12.5px] text-primary leading-[1.55] placeholder:text-[#52525B] placeholder:italic focus:ring-0"
          />
        )}
        <div className="font-mono text-[10px] text-tertiary mt-1.5 flex items-center gap-1.5">
          {isReview && (
            <>
              <i className="ph ph-warning text-[#F59E0B] text-[11px]" />
              <span className="text-[#F59E0B]">needs review</span>
              <span className="text-[#27272A]">·</span>
            </>
          )}
          {isPending && draftText.length === 0 && (
            <>
              <span className="text-[#F59E0B]">pending</span>
              <span className="text-[#27272A]">·</span>
            </>
          )}
          <span>{draftText.length} char</span>
        </div>
      </div>

      {/* Actions cell — 40px */}
      <div className="border-l border-[#27272A]/60 flex items-start justify-center pt-3 flex-shrink-0" style={{ width: 40 }}>
        <button
          className="w-5 h-5 rounded-[4px] hover:bg-overlay text-tertiary hover:text-primary flex items-center justify-center"
          title="Row actions"
          onClick={() => {
            // Copy source text to clipboard
            navigator.clipboard.writeText(item.source_text).catch(() => {});
          }}
        >
          <i className="ph ph-dots-three-vertical text-[11px]" />
        </button>
      </div>
    </div>
  );
}
