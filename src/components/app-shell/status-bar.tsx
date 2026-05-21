import { CloudSlash, Circuitry, CheckCircle, Warning, Spinner } from "@phosphor-icons/react";
import { StatusDot } from "../ui/status-dot";

type IndexState = "indexed" | "indexing" | "error" | "empty";
type BackendType = "mono" | "il2cpp" | "unknown";

type StatusBarProps =
  | { mode: "welcome" }
  | {
      mode: "project";
      backendType?: BackendType;
      unityVersion?: string;
      assetCount?: number;
      indexState?: IndexState;
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
    backendType = "unknown",
    unityVersion = "",
    assetCount,
    indexState = "indexed",
  } = props;

  const backendDot: "warning" | "info" | "tertiary" =
    backendType === "il2cpp" ? "warning" : backendType === "mono" ? "info" : "tertiary";
  const backendLabel =
    backendType === "il2cpp" ? "IL2CPP" : backendType === "mono" ? "Mono" : "Unknown backend";

  return (
    <footer
      className="flex items-center justify-between border-t border-border-default bg-surface px-3"
      style={{ height: 24, flexShrink: 0 }}
    >
      <div className="flex items-center gap-3 text-[11px] font-mono text-secondary">
        <span className="flex items-center gap-1.5">
          <StatusDot color={backendDot} />
          {backendLabel}
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
        {indexState === "indexing" && (
          <>
            <span className="text-overlay">•</span>
            <span className="flex items-center gap-1.5">
              <Spinner size={11} className="text-info animate-spin" />
              <span className="text-secondary">Indexing…</span>
            </span>
          </>
        )}
        {indexState === "empty" && (
          <>
            <span className="text-overlay">•</span>
            <span className="flex items-center gap-1.5" title="Extraction completed but produced no assets. Check %LOCALAPPDATA%\Unwrap\logs\ for AssetRipper diagnostics.">
              <Warning size={11} className="text-danger" />
              <span className="text-danger">No assets indexed — see logs</span>
            </span>
          </>
        )}
        {indexState === "error" && (
          <>
            <span className="text-overlay">•</span>
            <span className="flex items-center gap-1.5">
              <Warning size={11} className="text-danger" />
              <span className="text-danger">Indexing failed</span>
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
