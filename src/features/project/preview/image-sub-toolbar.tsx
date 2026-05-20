import { Eye, ChartBar, DownloadSimple, Image } from "@phosphor-icons/react";
import type { PreviewPayload } from "../../../lib/types";

type ImagePayload = Extract<PreviewPayload, { type: "image" }>;

/** 36px sub-toolbar shown above image preview — matches wireframe 02 lines 509-535. */
export function ImageSubToolbar({ width, height }: Pick<ImagePayload, "width" | "height">) {
  return (
    <div
      className="flex items-center justify-between px-4 border-b border-border-default bg-base flex-shrink-0"
      style={{ height: 36 }}
    >
      <div className="flex items-center gap-2 font-mono text-[11px] text-secondary">
        <Image size={12} className="text-tex" />
        <span>Texture2D</span>
        <span className="text-overlay">·</span>
        <span>{width} × {height}</span>
        <span className="text-overlay">·</span>
        <span>RGBA32</span>
      </div>
      <div className="flex items-center gap-1">
        <button className="h-7 px-2.5 rounded-[var(--radius-md)] hover:bg-overlay text-secondary hover:text-primary text-[11.5px] font-medium flex items-center gap-1.5 transition-colors">
          <Eye size={13} />
          Channels: RGBA
        </button>
        <button className="h-7 px-2.5 rounded-[var(--radius-md)] hover:bg-overlay text-secondary hover:text-primary text-[11.5px] font-medium flex items-center gap-1.5 transition-colors">
          <ChartBar size={13} />
          Histogram
        </button>
        <div className="w-px h-4 bg-border-default mx-1" />
        <button className="h-7 px-2.5 rounded-[var(--radius-md)] bg-elevated border border-border-default hover:border-border-strong text-primary text-[11.5px] font-medium flex items-center gap-1.5 transition-colors">
          <DownloadSimple size={13} />
          Export PNG
        </button>
      </div>
    </div>
  );
}
