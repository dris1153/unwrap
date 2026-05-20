import React from "react";
import {
  CaretRight,
  Folder,
  FolderOpen,
  Image,
  Waveform,
  Cube,
  FileCode,
  FileText,
  MapTrifold,
  PaintBrush,
  Lightning,
  FilmStrip,
  File,
  FileDashed,
} from "@phosphor-icons/react";
import { cn } from "../../../lib/cn";
import type { AssetKind } from "../../../lib/types";
import type { FlatRow } from "./use-flattened-tree";

type TreeRowProps = {
  row: FlatRow;
  isSelected: boolean;
  isExpanded: boolean;
  onSelect: () => void;
  onToggleExpand: () => void;
};

/** File-type icon + color by AssetKind */
function AssetIcon({ kind, isExpanded }: { kind: AssetKind; isExpanded: boolean }) {
  const iconProps = { size: 13, className: "flex-shrink-0" };
  switch (kind) {
    case "folder":
      return isExpanded ? (
        <FolderOpen {...iconProps} className={cn(iconProps.className, "text-secondary")} weight="fill" />
      ) : (
        <Folder {...iconProps} className={cn(iconProps.className, "text-secondary")} />
      );
    case "texture":
      return <Image {...iconProps} className={cn(iconProps.className, "text-tex")} />;
    case "audio":
      return <Waveform {...iconProps} className={cn(iconProps.className, "text-aud")} />;
    case "mesh":
      return <Cube {...iconProps} className={cn(iconProps.className, "text-mesh")} />;
    case "script":
      return <FileCode {...iconProps} className={cn(iconProps.className, "text-script")} />;
    case "text":
      return <FileText {...iconProps} className={cn(iconProps.className, "text-txt")} />;
    case "scene":
      return <MapTrifold {...iconProps} className={cn(iconProps.className, "text-txt")} />;
    case "material":
      return <PaintBrush {...iconProps} className={cn(iconProps.className, "text-mesh")} />;
    case "shader":
      return <Lightning {...iconProps} className={cn(iconProps.className, "text-secondary")} />;
    case "animation":
      return <FilmStrip {...iconProps} className={cn(iconProps.className, "text-secondary")} />;
    case "prefab":
      return <File {...iconProps} className={cn(iconProps.className, "text-secondary")} />;
    case "binary":
    default:
      return <FileDashed {...iconProps} className={cn(iconProps.className, "text-bin")} />;
  }
}

function formatSize(bytes: number): string {
  if (bytes === 0) return "";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export const TreeRow = React.memo(function TreeRow({
  row,
  isSelected,
  isExpanded,
  onSelect,
  onToggleExpand,
}: TreeRowProps) {
  const { node, depth, hasChildren } = row;
  const isFolder = node.kind === "folder";
  const paddingLeft = 8 + depth * 16;

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (isFolder && hasChildren) {
      onToggleExpand();
    }
    onSelect();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onSelect();
    } else if (e.key === "ArrowRight" && isFolder && hasChildren && !isExpanded) {
      e.preventDefault();
      onToggleExpand();
    } else if (e.key === "ArrowLeft" && isExpanded) {
      e.preventDefault();
      onToggleExpand();
    }
  };

  return (
    <div
      role="treeitem"
      aria-selected={isSelected}
      aria-expanded={isFolder && hasChildren ? isExpanded : undefined}
      tabIndex={isSelected ? 0 : -1}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      className={cn(
        "h-6 flex items-center gap-1.5 cursor-default select-none",
        "font-mono text-[12px] transition-colors",
        isSelected
          ? "bg-accent-muted text-primary [box-shadow:inset_2px_0_0_0_#10B981]"
          : "hover:bg-[#1c1c1f] text-primary",
      )}
      style={{ paddingLeft }}
    >
      {/* Chevron slot (always 12px wide) */}
      {isFolder && hasChildren ? (
        <CaretRight
          size={10}
          className={cn(
            "text-tertiary flex-shrink-0 transition-transform duration-150",
            isExpanded && "rotate-90",
          )}
          style={{ width: 12 }}
        />
      ) : (
        <span style={{ width: 12, display: "inline-block", flexShrink: 0 }} />
      )}

      <AssetIcon kind={node.kind} isExpanded={isExpanded} />

      <span className={cn("truncate flex-1", isFolder ? "font-[500]" : "")}>
        {node.name}
      </span>

      {node.size > 0 && !isFolder && (
        <span className="ml-auto pr-3 text-tertiary text-[10px] flex-shrink-0">
          {formatSize(node.size)}
        </span>
      )}
    </div>
  );
});
