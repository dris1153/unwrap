import React, { Suspense } from "react";
import { useQuery } from "@tanstack/react-query";
import { ipc } from "../../../lib/ipc";
import { useTabStore } from "../../../stores/use-tab-store";
import { EmptyPreview } from "../preview/empty-preview";
import { ImageSubToolbar } from "../preview/image-sub-toolbar";
import type { PreviewPayload } from "../../../lib/types";

// Lazy-loaded renderers — keeps initial bundle small
const ImagePreview  = React.lazy(() => import("../preview/image-preview").then((m) => ({ default: m.ImagePreview })));
const AudioPreview  = React.lazy(() => import("../preview/audio-preview").then((m) => ({ default: m.AudioPreview })));
const Model3dPreview = React.lazy(() => import("../preview/model3d-preview").then((m) => ({ default: m.Model3dPreview })));
const TextPreview   = React.lazy(() => import("../preview/text-preview").then((m) => ({ default: m.TextPreview })));
const CodePreview   = React.lazy(() => import("../preview/code-preview").then((m) => ({ default: m.CodePreview })));
const HexPreview    = React.lazy(() => import("../preview/hex-preview").then((m) => ({ default: m.HexPreview })));

function PreviewFallback() {
  return (
    <div className="flex-1 flex items-center justify-center bg-base">
      <span className="font-mono text-[12px] text-tertiary shimmer px-4 py-2 rounded">
        Loading…
      </span>
    </div>
  );
}

function PreviewContent({
  payload,
  projectId,
  nodeId,
  projectHandle,
}: {
  payload: PreviewPayload;
  projectId: string;
  nodeId: string;
  projectHandle: string;
}) {
  return (
    <Suspense fallback={<PreviewFallback />}>
      {(() => {
        switch (payload.type) {
          case "image":
            return (
              <>
                <ImageSubToolbar width={payload.width} height={payload.height} />
                <ImagePreview {...payload} projectId={projectId} nodeId={nodeId} />
              </>
            );
          case "audio":
            return <AudioPreview {...payload} />;
          case "model3_d":
            return <Model3dPreview {...payload} />;
          case "text":
            return <TextPreview {...payload} />;
          case "code":
            // CodePreview owns the decompile query — receives handle+nodeId, not payload spread.
            return <CodePreview projectHandle={projectHandle} nodeId={nodeId} />;
          case "hex":
            return <HexPreview {...payload} />;
          case "empty":
          default:
            return <EmptyPreview message="No preview available" />;
        }
      })()}
    </Suspense>
  );
}

type TabContentProps = {
  projectHandle: string;
};

/**
 * Fetches PreviewPayload for the active tab and routes to the correct renderer.
 */
export function TabContent({ projectHandle }: TabContentProps) {
  const { tabs, activeTabId } = useTabStore();
  const activeTab = tabs.find((t) => t.id === activeTabId) ?? null;

  const { data: payload, isLoading, isError } = useQuery({
    queryKey: ["preview", activeTab?.projectId, activeTab?.nodeId],
    queryFn: () =>
      ipc.preview(projectHandle, activeTab!.nodeId),
    enabled: !!activeTab,
    staleTime: 5 * 60 * 1000,
    gcTime: 10 * 60 * 1000,
  });

  if (!activeTab) {
    return <EmptyPreview />;
  }

  if (isLoading) {
    return <PreviewFallback />;
  }

  if (isError || !payload) {
    return <EmptyPreview message="Failed to load preview" />;
  }

  return (
    <PreviewContent
      payload={payload}
      projectId={activeTab.projectId}
      nodeId={activeTab.nodeId}
      projectHandle={projectHandle}
    />
  );
}
