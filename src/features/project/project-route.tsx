import { useQuery } from "@tanstack/react-query";
import { ipc } from "../../lib/ipc";
import { useTreeStore } from "../../stores/use-tree-store";
import { useTabStore } from "../../stores/use-tab-store";
import { useUiStore } from "../../stores/use-ui-store";
import { AssetTree } from "./tree/asset-tree";
import { TabBar } from "./tabs/tab-bar";
import { TabContent } from "./tabs/tab-content";
import { InspectorPanel } from "./inspector/inspector-panel";
import { TranslateView } from "./translate/translate-view";

type ProjectRouteProps = {
  projectId: string;
};

/**
 * Composes the full project UI: tree (in sidebar), tabs + preview (center), inspector (right).
 * Receives projectId from the route, fetches tree and wires stores.
 */
export function ProjectRouteContent({ projectId }: ProjectRouteProps) {
  // projectId IS the handle id string for ipc calls
  const projectHandle = projectId;

  const { data: tree, isLoading: treeLoading } = useQuery({
    queryKey: ["tree", projectId],
    queryFn: () => ipc.tree(projectHandle),
    staleTime: 60_000,
  });

  const { data: project } = useQuery({
    queryKey: ["project", projectId],
    queryFn: () => ipc.getProject(projectId),
    staleTime: 60_000,
  });

  // Expose project data for status bar via store subscription (read via zustand)
  const selectedNode = useTreeStore((s) => s.selectedNode[projectId]);
  const tabs = useTabStore((s) => s.tabs);
  const activeRail = useUiStore((s) => s.activeRail);
  const assetCount = tree ? Object.keys(tree.nodes).length : undefined;

  return {
    tree: tree ?? null,
    treeLoading,
    project: project ?? null,
    selectedNode,
    assetCount,
    tabs,
    projectHandle,

    // Component factories used by the shell
    renderTree: () =>
      treeLoading ? (
        <div className="flex-1 flex items-center justify-center">
          <span className="font-mono text-[12px] text-tertiary shimmer px-3 py-1.5 rounded">
            Indexing…
          </span>
        </div>
      ) : tree ? (
        <AssetTree projectId={projectId} tree={tree} />
      ) : (
        <div className="flex-1 flex items-center justify-center">
          <span className="font-mono text-[12px] text-tertiary">No project open</span>
        </div>
      ),

    renderCenter: () => {
      // Switch center pane based on active rail tab (captured above).
      if (activeRail === "strings") {
        return <TranslateView projectHandle={projectHandle} />;
      }
      return (
        <section className="flex-1 flex flex-col min-w-0 bg-base">
          <TabBar />
          <TabContent projectHandle={projectHandle} />
        </section>
      );
    },

    renderInspector: () => (
      <InspectorPanel projectId={projectId} projectHandle={projectHandle} />
    ),
  };
}
