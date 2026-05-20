import {
  Image,
  Waveform,
  Cube,
  FileCode,
  FileText,
  File,
} from "@phosphor-icons/react";
import { cn } from "../../../lib/cn";
import type { AssetKind, AssetNode } from "../../../lib/types";

type IdentityCardProps = {
  node: AssetNode;
  pathLabel: string;
};

function KindIcon({ kind }: { kind: AssetKind }) {
  const cls = "flex-shrink-0";
  switch (kind) {
    case "texture": return <Image size={20} className={cn(cls, "text-tex")} />;
    case "audio":   return <Waveform size={20} className={cn(cls, "text-aud")} />;
    case "mesh":    return <Cube size={20} className={cn(cls, "text-mesh")} />;
    case "script":  return <FileCode size={20} className={cn(cls, "text-script")} />;
    case "text":    return <FileText size={20} className={cn(cls, "text-txt")} />;
    default:        return <File size={20} className={cn(cls, "text-secondary")} />;
  }
}

function kindLabel(kind: AssetKind): string {
  const map: Partial<Record<AssetKind, string>> = {
    texture: "Texture2D",
    audio: "AudioClip",
    mesh: "Mesh",
    script: "MonoScript",
    text: "TextAsset",
    scene: "SceneAsset",
    material: "Material",
    shader: "Shader",
    animation: "AnimationClip",
    prefab: "Prefab",
    binary: "Binary",
  };
  return map[kind] ?? kind;
}

function kindColor(kind: AssetKind): string {
  switch (kind) {
    case "texture": return "text-tex bg-tex/10 border-tex/20";
    case "audio":   return "text-aud bg-aud/10 border-aud/20";
    case "mesh":    return "text-mesh bg-mesh/10 border-mesh/20";
    case "script":  return "text-script bg-script/10 border-script/20";
    case "text":    return "text-txt bg-txt/10 border-txt/20";
    default:        return "text-secondary bg-elevated border-border-default";
  }
}

function iconBg(kind: AssetKind): string {
  switch (kind) {
    case "texture": return "bg-tex/15 border-tex/30";
    case "audio":   return "bg-aud/15 border-aud/30";
    case "mesh":    return "bg-mesh/15 border-mesh/30";
    case "script":  return "bg-script/15 border-script/30";
    case "text":    return "bg-txt/15 border-txt/30";
    default:        return "bg-elevated border-border-default";
  }
}

export function IdentityCard({ node, pathLabel }: IdentityCardProps) {
  const ext = node.name.includes(".")
    ? node.name.split(".").pop()?.toUpperCase()
    : null;

  return (
    <div className="px-4 pt-4 pb-4 border-b border-border-default">
      <div className="flex items-start gap-3">
        <div
          className={cn(
            "w-[40px] h-[40px] rounded-lg border flex items-center justify-center flex-shrink-0",
            iconBg(node.kind),
          )}
        >
          <KindIcon kind={node.kind} />
        </div>

        <div className="min-w-0 flex-1">
          <div className="font-sans text-[15px] font-[600] text-primary tracking-tight mb-0.5 truncate">
            {node.name}
          </div>
          <div className="font-mono text-[10.5px] text-tertiary mb-2 truncate">
            {pathLabel}
          </div>
          <div className="flex items-center gap-1.5 flex-wrap">
            <span
              className={cn(
                "font-mono text-[10px] rounded-[4px] border px-1.5 py-[1px]",
                kindColor(node.kind),
              )}
            >
              {kindLabel(node.kind)}
            </span>
            {ext && (
              <span className="font-mono text-[10px] text-secondary bg-elevated border border-border-default rounded-[4px] px-1.5 py-[1px]">
                {ext}
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
