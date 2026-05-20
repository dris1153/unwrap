import type { AssetNode } from "../../../lib/types";

type OriginSectionProps = {
  node: AssetNode;
};

export function OriginSection({ node }: OriginSectionProps) {
  const meta = node.metadata as Record<string, unknown>;
  const bundle = meta.source_bundle ? String(meta.source_bundle) : node.source_path || null;
  const pathId = meta.path_id ? String(meta.path_id) : null;
  const guid   = meta.guid     ? String(meta.guid)    : null;

  if (!bundle && !pathId && !guid) return null;

  return (
    <div className="px-4 py-3 border-b border-border-default">
      <div className="mb-2">
        <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
          Origin
        </span>
      </div>
      <div className="space-y-1.5">
        {bundle && (
          <div className="flex items-center justify-between">
            <span className="font-sans text-[12px] text-secondary">Source bundle</span>
            <span className="font-mono text-[11.5px] text-primary truncate max-w-[160px]" title={bundle}>
              {bundle}
            </span>
          </div>
        )}
        {pathId && (
          <div className="flex items-center justify-between">
            <span className="font-sans text-[12px] text-secondary">PathID</span>
            <span className="font-mono text-[11.5px] text-primary">{pathId}</span>
          </div>
        )}
        {guid && (
          <div className="flex items-center justify-between">
            <span className="font-sans text-[12px] text-secondary">Asset GUID</span>
            <span className="font-mono text-[10.5px] text-tertiary truncate max-w-[120px]" title={guid}>
              {guid.length > 12 ? `${guid.slice(0, 4)}…${guid.slice(-2)}` : guid}
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
