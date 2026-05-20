import { FileCode, Square } from "@phosphor-icons/react";
import type { CodeRef } from "../../../lib/types";

type ReferencesSectionProps = {
  references?: CodeRef[];
};

/**
 * Shows code references from PreviewPayload::Code.references.
 * Shows "No references yet" placeholder when empty — full xref index deferred to v2.
 */
export function ReferencesSection({ references }: ReferencesSectionProps) {
  const refs = references ?? [];

  return (
    <div className="px-4 py-3 border-b border-border-default">
      <div className="flex items-center justify-between mb-2.5">
        <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
          References
        </span>
        {refs.length > 0 && (
          <span className="font-mono text-[10px] text-secondary bg-elevated border border-border-default rounded-[4px] px-1.5 py-[1px]">
            {refs.length}
          </span>
        )}
      </div>

      {refs.length === 0 ? (
        <span className="font-mono text-[11px] text-tertiary">No references yet</span>
      ) : (
        <div className="space-y-1">
          {refs.map((ref, i) => (
            <button
              key={i}
              className="w-full flex items-center gap-2 px-2 py-1.5 rounded-[var(--radius-md)] hover:bg-overlay text-left transition-colors"
            >
              {ref.file?.endsWith(".cs") ? (
                <FileCode size={13} className="text-script flex-shrink-0" />
              ) : (
                <Square size={13} className="text-secondary flex-shrink-0" />
              )}
              <span className="font-mono text-[11.5px] text-primary flex-1 truncate">
                {ref.symbol}
              </span>
              {ref.line !== null && (
                <span className="font-mono text-[10px] text-tertiary flex-shrink-0">
                  line {ref.line}
                </span>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
