// features/command-palette/shortcuts.ts
// React hook component that wires global keyboard shortcuts for the palette.

import { useEffect } from "react";
import { usePaletteStore } from "./use-palette-store";

/**
 * Mounts global keydown listeners for:
 *   Cmd/Ctrl+K → toggle palette
 *   Escape      → close palette (if open)
 *
 * Returns null — mount once in the root layout.
 */
export function GlobalShortcuts(): null {
  const toggle = usePaletteStore((s) => s.toggle);
  const close = usePaletteStore((s) => s.close);
  const open = usePaletteStore((s) => s.open);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        toggle();
        return;
      }
      if (e.key === "Escape" && open) {
        e.preventDefault();
        close();
      }
    };

    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [toggle, close, open]);

  return null;
}
