import { useQuery } from "@tanstack/react-query";
import { ipc } from "../../../lib/ipc";
import { useTabStore } from "../../../stores/use-tab-store";
import { useTreeStore } from "../../../stores/use-tree-store";
import { IdentityCard } from "./identity-card";
import { PropertiesSection } from "./properties-section";
import { OriginSection } from "./origin-section";
import { ReferencesSection } from "./references-section";
import { ActionsSection } from "./actions-section";
import { CodeInspectorSection } from "./code-inspector-section";
import { buildPathLabel } from "../tree/build-path-label";
import type { PreviewPayload } from "../../../lib/types";

type InspectorPanelProps = {
  projectId: string;
  projectHandle: string;
};

/**
 * Inspector body — populates from the active tab's node + preview payload.
 */
export function InspectorPanel({ projectId, projectHandle }: InspectorPanelProps) {
  const { tabs, activeTabId } = useTabStore();
  const activeTab = tabs.find((t) => t.id === activeTabId) ?? null;

  const selectedNode = useTreeStore((s) => s.selectedNode[projectId]);

  const { data: tree } = useQuery({
    queryKey: ["tree", projectId],
    queryFn: () => ipc.tree(projectHandle),
    staleTime: 60_000,
  });

  const { data: payload } = useQuery<PreviewPayload>({
    queryKey: ["preview", activeTab?.projectId, activeTab?.nodeId],
    queryFn: () => ipc.preview(projectHandle, activeTab!.nodeId),
    enabled: !!activeTab,
    staleTime: 5 * 60 * 1000,
  });

  const node = selectedNode ?? (activeTab && tree ? tree.nodes[activeTab.nodeId] : null);
  if (!node) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <span className="font-sans text-[13px] text-tertiary">Select an asset to inspect.</span>
      </div>
    );
  }

  const pathLabel = tree ? buildPathLabel(tree, node.id["0"]) : node.name;
  const meta = node.metadata as Record<string, unknown>;
  const guid = meta.guid ? String(meta.guid) : undefined;

  // For script nodes, the decompile query (keyed identically to CodePreview) is already
  // cached by React Query — this read is free after CodePreview has populated it.
  const isScriptNode = node.kind === "script";
  const { data: decompileData } = useQuery({
    queryKey: ["decompile", projectHandle, activeTab?.nodeId],
    queryFn: () => ipc.decompile(JSON.parse(projectHandle), activeTab!.nodeId),
    enabled: isScriptNode && !!activeTab,
    staleTime: 5 * 60 * 1000,
  });

  // Extract references from code payload (fallback for non-decompiled scripts)
  const refs = payload?.type === "code" ? payload.references : undefined;

  return (
    <div className="flex-1 overflow-y-auto scroll-thin">
      <IdentityCard node={node} pathLabel={pathLabel} />
      <PropertiesSection node={node} payload={payload} />
      <OriginSection node={node} />

      {/* Code-specific inspector — shown when decompile data is available */}
      {isScriptNode && decompileData ? (
        <CodeInspectorSection payload={decompileData} />
      ) : (
        <ReferencesSection references={refs} />
      )}

      {/* In animations — mock v1 */}
      <div className="px-4 py-3 border-b border-border-default">
        <div className="mb-2.5">
          <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
            In animations
          </span>
        </div>
        <span className="font-mono text-[11px] text-tertiary">
          Anim parsing deferred to v2.
        </span>
      </div>

      <ActionsSection node={node} guid={guid} />
    </div>
  );
}
