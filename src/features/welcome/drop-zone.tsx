import { useEffect, useRef, useState } from "react";
import { FolderSimplePlus, FolderOpen, FileArrowUp, ShieldCheck } from "@phosphor-icons/react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useRouter } from "@tanstack/react-router";
import { ipc } from "../../lib/ipc";
import type { ProgressPayload } from "../../lib/types";
import { LoadingModal } from "./loading-modal";
import { Tooltip } from "../../components/ui/tooltip";

/**
 * Drop zone for the Welcome screen.
 * Listens for both HTML drag-drop events and Tauri native drag-drop events.
 * On drop: detect → open_project → navigate to /project/$projectId.
 */
export function DropZone() {
  const router = useRouter();
  const [opening, setOpening] = useState(false);
  const [progress, setProgress] = useState<ProgressPayload | null>(null);
  // Ref holds the current operation id so the cancel handler is always fresh.
  const opIdRef = useRef<string | null>(null);
  // Unlisten function for progress events.
  const unlistenRef = useRef<(() => void) | null>(null);

  // Register Tauri native drag-drop listener once on mount.
  useEffect(() => {
    let cleanupNative: (() => void) | null = null;

    getCurrentWindow()
      .onDragDropEvent((event) => {
        if (event.payload.type === "drop") {
          const paths: string[] = event.payload.paths ?? [];
          if (paths.length > 0) {
            void handleOpen(paths[0]);
          }
        }
      })
      .then((unlisten) => {
        cleanupNative = unlisten;
      })
      .catch((err) => {
        console.warn("[DropZone] onDragDropEvent registration failed:", err);
      });

    return () => {
      cleanupNative?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function handleOpen(path: string) {
    if (opening) return;

    setOpening(true);
    setProgress(null);

    // Derive operation id (matches Rust: sha256(path)[..16]).
    // We don't have sha256 on the frontend so we use path as a key for the
    // progress subscription — the Rust side emits with its own computed op id.
    // Subscribe to ALL progress events and display them while this modal is open.
    const unlisten = await ipc.subscribeProgress("__any__", () => {}).catch(() => () => {});

    // Re-subscribe with a real handler now that we have the function reference.
    const actualUnlisten = await ipc
      .subscribeProgress("__any__", (p) => setProgress(p))
      .catch(() => () => {});

    unlistenRef.current = actualUnlisten;
    // Clean up the dummy subscription immediately.
    unlisten();

    try {
      // Detect first so we can route chooser vs auto.
      const detection = await ipc.detectProject(path);

      let handle;
      if (detection.type === "auto" || detection.type === "chooser") {
        // For both auto and chooser (only one handler registered) open directly.
        handle = await ipc.openProject(path);
      } else {
        throw new Error("Unrecognised detection result");
      }

      // Navigate to the project route using the unwrapped id string.
      const projectId =
        typeof handle.id === "object" && "0" in handle.id
          ? (handle.id as { "0": string })["0"]
          : String(handle.id);

      router.navigate({ to: "/project/$projectId", params: { projectId } });
    } catch (err) {
      console.error("[DropZone] open_project failed:", err);
      setOpening(false);
      setProgress(null);
    } finally {
      actualUnlisten();
      unlistenRef.current = null;
    }
  }

  function handleCancel() {
    if (opIdRef.current) {
      ipc.cancel(opIdRef.current).catch(() => {});
    }
    unlistenRef.current?.();
    unlistenRef.current = null;
    setOpening(false);
    setProgress(null);
  }

  async function handleBrowse() {
    if (opening) return;
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Unity build folder",
      });
      if (typeof selected === "string" && selected.length > 0) {
        await handleOpen(selected);
      }
    } catch (err) {
      console.error("[DropZone] browse failed:", err);
    }
  }

  // ⌘O / Ctrl+O keyboard shortcut — same flow as Browse button.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "o" && !opening) {
        e.preventDefault();
        void handleBrowse();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [opening]);

  // HTML drag-drop fallback (browser dragging files into the webview).
  function handleDragOver(e: React.DragEvent) {
    e.preventDefault();
    e.stopPropagation();
  }

  function handleDrop(e: React.DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0) {
      // `webkitRelativePath` or `path` is available in Tauri's webview for local files.
      // In practice the Tauri native event fires first; this is a safety fallback.
      const file = files[0] as File & { path?: string };
      const path = file.path ?? (file as unknown as { webkitRelativePath: string }).webkitRelativePath;
      if (path) void handleOpen(path);
    }
  }

  return (
    <>
      {opening && (
        <LoadingModal progress={progress} onCancel={handleCancel} />
      )}

      <div
        onDragOver={handleDragOver}
        onDrop={handleDrop}
        className="drop-pulse rounded-2xl border border-dashed"
        style={{
          padding: "44px 40px",
          borderWidth: "1.5px",
          borderColor: "rgba(16,185,129,0.40)",
          background: "rgba(16,185,129,0.015)",
          maxWidth: 560,
        }}
      >
        <div className="flex items-start gap-5">
          {/* Icon */}
          <div className="shrink-0 w-[52px] h-[52px] rounded-xl bg-accent/15 border border-accent/30 flex items-center justify-center">
            <FolderOpen size={28} weight="duotone" className="text-accent" />
          </div>

          {/* Text + buttons */}
          <div className="flex-1">
            <div className="font-sans text-[18px] leading-[24px] font-semibold text-primary mb-1.5">
              Drop a Unity build folder here
            </div>
            <div className="font-sans text-[13px] leading-[20px] text-secondary mb-4">
              Point at the folder containing{" "}
              <span className="font-mono text-[12px] text-primary/80">*.exe</span>{" "}
              and{" "}
              <span className="font-mono text-[12px] text-primary/80">*_Data/</span>{" "}
              — or browse manually.
            </div>

            <div className="flex items-center gap-3">
              <button
                onClick={() => void handleBrowse()}
                className="h-8 px-3 rounded-[var(--radius-md)] bg-accent text-[#09090B] text-[13px] font-semibold hover:bg-accent-strong active:translate-y-[1px] transition-all flex items-center gap-1.5"
              >
                <FolderSimplePlus size={14} />
                Browse files…
              </button>
              <span className="font-mono text-[12px] text-tertiary">or</span>
              <Tooltip content="Coming in v0.2 — archive extraction" side="top">
                <button
                  onClick={(e) => e.preventDefault()}
                  aria-label="Open archive (coming in v0.2)"
                  className="h-8 px-3 rounded-[var(--radius-md)] text-tertiary cursor-not-allowed opacity-60 text-[13px] font-medium flex items-center gap-1.5"
                >
                  <FileArrowUp size={14} />
                  Open archive (.zip)
                </button>
              </Tooltip>
            </div>
          </div>
        </div>

        {/* Bottom meta row */}
        <div className="mt-5 pt-4 border-t border-accent/15 flex items-center gap-4 text-[11px] font-mono text-tertiary">
          <span className="flex items-center gap-1.5">
            <ShieldCheck size={12} className="text-accent/70" />
            Local only · nothing uploaded
          </span>
          <span className="text-overlay">·</span>
          <span>Drag any folder · ⌘O to browse</span>
        </div>
      </div>
    </>
  );
}
