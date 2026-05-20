import { CloudSlash, Circuitry, CheckCircle } from "@phosphor-icons/react";
import { StatusDot } from "../ui/status-dot";

type StatusBarProps =
  | { mode: "welcome" }
  | {
      mode: "project";
      backendType?: "mono" | "il2cpp";
      unityVersion?: string;
      assetCount?: number;
      indexState?: "indexed" | "indexing" | "error";
    };

/**
 * 24px status bar — two variants by mode.
 */
export function StatusBar(props: StatusBarProps) {
  if (props.mode === "welcome") {
    return (
      <footer
        className="flex items-center justify-between border-t border-border-default bg-surface px-3"
        style={{ height: 24, flexShrink: 0 }}
      >
        <div className="flex items-center gap-3 text-[11px] font-mono text-secondary">
          <span className="flex items-center gap-1.5">
            <StatusDot color="tertiary" />
            Ready
          </span>
          <span className="text-overlay">•</span>
          <span className="text-tertiary">No project open</span>
        </div>
        <div className="flex items-center gap-3 text-[11px] font-mono text-tertiary">
          <span className="flex items-center gap-1.5">
            <CloudSlash size={11} />
            Offline mode
          </span>
          <span className="text-overlay">•</span>
          <span>v0.1.0-alpha</span>
          <span className="text-overlay">•</span>
          <span className="flex items-center gap-1.5">
            <Circuitry size={11} />
            Win 11 · WebView2
          </span>
        </div>
      </footer>
    );
  }

  const {
    backendType = "mono",
    unityVersion = "",
    assetCount,
    indexState = "indexed",
  } = props;

  return (
    <footer
      className="flex items-center justify-between border-t border-border-default bg-surface px-3"
      style={{ height: 24, flexShrink: 0 }}
    >
      <div className="flex items-center gap-3 text-[11px] font-mono text-secondary">
        <span className="flex items-center gap-1.5">
          <StatusDot color={backendType === "il2cpp" ? "warning" : "info"} />
          {backendType === "il2cpp" ? "IL2CPP" : "Mono"}
        </span>
        {unityVersion && (
          <>
            <span className="text-overlay">•</span>
            <span>{unityVersion}</span>
          </>
        )}
        {assetCount !== undefined && (
          <>
            <span className="text-overlay">•</span>
            <span className="text-tertiary">{assetCount.toLocaleString()} assets</span>
          </>
        )}
        {indexState === "indexed" && (
          <>
            <span className="text-overlay">•</span>
            <span className="flex items-center gap-1.5">
              <CheckCircle size={11} className="text-accent" />
              <span className="text-secondary">Indexed</span>
            </span>
          </>
        )}
      </div>
      <div className="flex items-center gap-3 text-[11px] font-mono text-tertiary">
        <span>v0.1.0</span>
      </div>
    </footer>
  );
}
