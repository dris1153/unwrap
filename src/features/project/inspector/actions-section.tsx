import { DownloadSimple, Copy, Graph } from "@phosphor-icons/react";
import type { AssetNode } from "../../../lib/types";

type ActionsSectionProps = {
  node: AssetNode;
  guid?: string;
};

export function ActionsSection({ node, guid }: ActionsSectionProps) {
  const isTexture = node.kind === "texture";

  const copyGuid = () => {
    if (guid) {
      navigator.clipboard.writeText(guid).catch(() => {/* non-fatal */});
    }
  };

  return (
    <div className="px-4 py-3">
      <div className="mb-2.5">
        <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
          Actions
        </span>
      </div>
      <div className="space-y-1.5">
        {isTexture && (
          <button className="w-full h-8 px-2.5 rounded-[var(--radius-md)] bg-accent text-[#09090B] text-[12.5px] font-[600] hover:bg-accent-strong active:translate-y-[1px] transition-all flex items-center justify-between">
            <span className="flex items-center gap-2">
              <DownloadSimple size={14} />
              Export PNG
            </span>
            <span className="font-mono text-[10px] text-[#09090B]/70">⌘E</span>
          </button>
        )}

        <button
          onClick={copyGuid}
          disabled={!guid}
          className="w-full h-8 px-2.5 rounded-[var(--radius-md)] bg-elevated border border-border-default hover:border-border-strong text-primary text-[12.5px] font-[500] hover:bg-overlay transition-all flex items-center gap-2 disabled:opacity-40"
        >
          <Copy size={14} className="text-secondary" />
          Copy GUID
        </button>

        <button className="w-full h-8 px-2.5 rounded-[var(--radius-md)] bg-elevated border border-border-default hover:border-border-strong text-primary text-[12.5px] font-[500] hover:bg-overlay transition-all flex items-center justify-between">
          <span className="flex items-center gap-2">
            <Graph size={14} className="text-secondary" />
            Reveal references
          </span>
          <span className="font-mono text-[10px] text-tertiary">⌘R</span>
        </button>
      </div>
    </div>
  );
}
