import { createFileRoute } from "@tanstack/react-router";
import { TitleBar } from "../../components/app-shell/title-bar";
import { Toolbar } from "../../components/app-shell/toolbar";
import { Sidebar } from "../../components/app-shell/sidebar";
import { Inspector } from "../../components/app-shell/inspector";
import { StatusBar } from "../../components/app-shell/status-bar";
import { ResizeHandle } from "../../components/app-shell/resize-handle";
import { useUiStore } from "../../stores/use-ui-store";
import { ProjectRouteContent } from "../../features/project/project-route";

export const Route = createFileRoute("/project/$projectId")({
  component: ProjectRoute,
});

function ProjectRoute() {
  const { projectId } = Route.useParams();
  const { setSidebarWidth, setInspectorWidth } = useUiStore();

  const content = ProjectRouteContent({ projectId });

  const projectName = content.project?.root_path
    ? content.project.root_path.split(/[\\/]/).pop() ?? projectId
    : projectId;

  return (
    <>
      <TitleBar mode="project" projectName={projectName} />
      <Toolbar projectId={projectId} tree={content.tree ?? undefined} />

      {/* Work area */}
      <div className="flex flex-1 min-h-0">
        <Sidebar projectId={projectId} renderTree={content.renderTree} />

        <ResizeHandle onDelta={setSidebarWidth} side="right" />

        {content.renderCenter()}

        <ResizeHandle
          onDelta={(d) =>
            setInspectorWidth(useUiStore.getState().inspectorWidth - d)
          }
          side="left"
        />

        <Inspector renderBody={content.renderInspector} />
      </div>

      <StatusBar
        mode="project"
        backendType={content.project?.scripting_backend === "il2cpp" ? "il2cpp" : "mono"}
        unityVersion={content.project?.engine_version ?? ""}
        assetCount={content.assetCount}
        indexState={
          content.treeLoading
            ? "indexing"
            : content.tree && Object.keys(content.tree.nodes).length > 1
              ? "indexed"
              : "empty"
        }
      />
    </>
  );
}
