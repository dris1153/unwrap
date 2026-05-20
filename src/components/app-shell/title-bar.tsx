import { CaretRight, CaretDown } from "@phosphor-icons/react";
import { WindowControls } from "./window-controls";

type TitleBarProps =
  | { mode: "welcome" }
  | { mode: "project"; projectName: string };

/**
 * Custom 36px title bar.
 * The outer container has data-tauri-drag-region so the whole bar is draggable.
 * WindowControls buttons must NOT have that attribute.
 */
export function TitleBar(props: TitleBarProps) {
  return (
    <header
      data-tauri-drag-region
      className="flex items-center justify-between bg-surface border-b border-border-default"
      style={{ height: 36, flexShrink: 0 }}
    >
      {/* Left: brand mark + context */}
      <div className="flex items-center gap-2 px-3" data-tauri-drag-region>
        {/* Monogram mark */}
        <div className="w-[18px] h-[18px] rounded-[4px] bg-accent/15 border border-accent/40 flex items-center justify-center pointer-events-none">
          <div className="w-[8px] h-[8px] bg-accent rounded-[1px]" />
        </div>

        <span className="font-sans text-[12px] font-medium tracking-tightish text-primary pointer-events-none">
          Unwrap
        </span>

        {props.mode === "welcome" ? (
          <>
            <span className="text-tertiary text-[12px] pointer-events-none">·</span>
            <span className="font-mono text-[11px] text-tertiary pointer-events-none">
              welcome
            </span>
          </>
        ) : (
          <>
            <CaretRight size={10} className="text-tertiary pointer-events-none" />
            {/* Project dropdown (visual only — no behavior this phase) */}
            <button
              className="flex items-center gap-1.5 px-2 py-[3px] rounded-[4px] hover:bg-overlay text-primary text-[12px] font-medium"
              style={{ pointerEvents: "auto" }}
            >
              {props.projectName}
              <CaretDown size={10} className="text-tertiary" />
            </button>
          </>
        )}
      </div>

      {/* Center drag region */}
      <div className="flex-1" data-tauri-drag-region />

      {/* Right: window controls — NO drag-region */}
      <WindowControls />
    </header>
  );
}
