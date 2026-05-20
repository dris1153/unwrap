import { Minus, Square, X } from "@phosphor-icons/react";
import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * Custom Windows-style window control buttons (minimize / maximize / close).
 * These must NOT have data-tauri-drag-region — that attribute would prevent
 * click events from reaching the buttons.
 */
export function WindowControls() {
  async function handleMinimize() {
    try {
      await getCurrentWindow().minimize();
    } catch (e) {
      console.error("minimize failed", e);
    }
  }

  async function handleMaximize() {
    try {
      await getCurrentWindow().toggleMaximize();
    } catch (e) {
      console.error("toggleMaximize failed", e);
    }
  }

  async function handleClose() {
    try {
      await getCurrentWindow().close();
    } catch (e) {
      console.error("close failed", e);
    }
  }

  return (
    <div className="flex items-center">
      <button
        onClick={handleMinimize}
        aria-label="Minimize"
        className="w-[44px] h-[36px] flex items-center justify-center text-secondary hover:bg-overlay hover:text-primary transition-colors"
      >
        <Minus size={14} />
      </button>
      <button
        onClick={handleMaximize}
        aria-label="Maximize"
        className="w-[44px] h-[36px] flex items-center justify-center text-secondary hover:bg-overlay hover:text-primary transition-colors"
      >
        <Square size={12} />
      </button>
      <button
        onClick={handleClose}
        aria-label="Close"
        className="w-[44px] h-[36px] flex items-center justify-center text-secondary hover:bg-danger hover:text-white transition-colors"
      >
        <X size={14} />
      </button>
    </div>
  );
}
