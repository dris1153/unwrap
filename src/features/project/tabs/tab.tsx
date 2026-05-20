import { motion } from "framer-motion";
import { X, Image, Waveform, Cube, FileCode, FileText, File } from "@phosphor-icons/react";
import { cn } from "../../../lib/cn";
import type { Tab } from "../../../stores/use-tab-store";
import type { AssetKind } from "../../../lib/types";

type TabProps = {
  tab: Tab;
  isActive: boolean;
  onActivate: () => void;
  onClose: (e: React.MouseEvent) => void;
};

function TabIcon({ kind }: { kind: AssetKind }) {
  const props = { size: 12, className: "flex-shrink-0" };
  switch (kind) {
    case "texture":  return <Image {...props} className={cn(props.className, "text-tex")} />;
    case "audio":    return <Waveform {...props} className={cn(props.className, "text-aud")} />;
    case "mesh":     return <Cube {...props} className={cn(props.className, "text-mesh")} />;
    case "script":   return <FileCode {...props} className={cn(props.className, "text-script")} />;
    case "text":     return <FileText {...props} className={cn(props.className, "text-txt")} />;
    default:         return <File {...props} className={cn(props.className, "text-secondary")} />;
  }
}

export function TabItem({ tab, isActive, onActivate, onClose }: TabProps) {
  return (
    <div
      role="tab"
      aria-selected={isActive}
      onClick={onActivate}
      className={cn(
        "relative flex items-center gap-2 h-full px-3 border-r border-border-default cursor-default",
        "flex-shrink-0 max-w-[260px]",
        isActive
          ? "bg-base text-primary"
          : "text-secondary hover:bg-base/60 transition-colors",
      )}
    >
      <TabIcon kind={tab.kind} />

      <span className="font-mono text-[11.5px] font-[500] truncate">
        {tab.pathLabel}
      </span>

      {/* Modified dot slot (always rendered, always inactive v1) */}
      <span
        className="w-[6px] h-[6px] rounded-full flex-shrink-0 opacity-0"
        aria-hidden
      />

      <button
        onClick={onClose}
        className="ml-1 w-4 h-4 rounded-[3px] hover:bg-overlay text-tertiary hover:text-primary flex items-center justify-center flex-shrink-0"
        aria-label={`Close ${tab.name}`}
      >
        <X size={10} />
      </button>

      {/* Active underline — animated via Framer layoutId */}
      {isActive && (
        <motion.div
          layoutId="active-tab-underline"
          className="absolute bottom-0 left-0 right-0 h-[2px] bg-accent"
          transition={{ type: "spring", stiffness: 500, damping: 30 }}
        />
      )}
    </div>
  );
}
