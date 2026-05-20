/**
 * monaco-theme.ts — registers the "unwrap-dark" editor theme.
 *
 * Colors match wireframe 03:
 *   keywords  → emerald  #10B981
 *   types     → blue     #60A5FA
 *   strings   → pink     #F472B6
 *   numbers   → amber    #FBBF24
 *   comments  → gray     #71717A (italic)
 *   bg        → near-black #0C0C0E
 *
 * Call `registerUnwrapTheme(monaco)` once before the editor mounts.
 */

import type { Monaco } from "@monaco-editor/react";

export function registerUnwrapTheme(monaco: Monaco): void {
  monaco.editor.defineTheme("unwrap-dark", {
    base: "vs-dark",
    inherit: false,
    rules: [
      { token: "keyword",   foreground: "10B981", fontStyle: "bold" },
      { token: "type",      foreground: "60A5FA" },
      { token: "string",    foreground: "F472B6" },
      { token: "number",    foreground: "FBBF24" },
      { token: "comment",   foreground: "71717A", fontStyle: "italic" },
      { token: "delimiter", foreground: "A1A1AA" },
    ],
    colors: {
      "editor.background":                  "#0C0C0E",
      "editor.foreground":                  "#F4F4F5",
      "editorLineNumber.foreground":        "#52525B",
      "editorLineNumber.activeForeground":  "#10B981",
      "editor.lineHighlightBackground":     "#18181B",
      "editorCursor.foreground":            "#10B981",
      "editor.selectionBackground":         "#10B98130",
      "editorIndentGuide.background1":      "#18181B",
      "editorIndentGuide.activeBackground1":"#27272A",
    },
  });
}
