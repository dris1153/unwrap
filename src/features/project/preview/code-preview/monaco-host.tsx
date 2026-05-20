/**
 * monaco-host.tsx — lazy-loaded Monaco editor wrapper with the unwrap-dark theme.
 *
 * Lazy-loaded via React.lazy at the call site so the Monaco JS bundle (~2.5MB)
 * is excluded from the welcome-screen bundle.
 *
 * Props:
 *   content  — full C# source text
 *   language — monaco language id (always "csharp" for decompile; "plaintext" fallback)
 */

import Editor, { type OnMount } from "@monaco-editor/react";
import { useRef } from "react";
import { registerUnwrapTheme } from "./monaco-theme";

type MonacoHostProps = {
  content: string;
  language?: string;
};

export function MonacoHost({ content, language = "csharp" }: MonacoHostProps) {
  const themeRegistered = useRef(false);

  const handleMount: OnMount = (_, monaco) => {
    if (!themeRegistered.current) {
      registerUnwrapTheme(monaco);
      monaco.editor.setTheme("unwrap-dark");
      themeRegistered.current = true;
    }
  };

  return (
    <div className="flex-1 min-h-0 overflow-hidden">
      <Editor
        value={content}
        language={language}
        theme="unwrap-dark"
        onMount={handleMount}
        options={{
          readOnly: true,
          minimap: { enabled: true },
          fontFamily: "JetBrains Mono, monospace",
          fontSize: 13,
          lineHeight: 20,
          scrollBeyondLastLine: false,
          renderLineHighlight: "line",
          automaticLayout: true,
          wordWrap: "off",
          folding: true,
          lineNumbers: "on",
          glyphMargin: false,
          contextmenu: true,
        }}
        height="100%"
        loading={
          <div className="flex-1 flex items-center justify-center bg-[#0C0C0E] h-full">
            <span className="font-mono text-[12px] text-zinc-500">Decompiling…</span>
          </div>
        }
      />
    </div>
  );
}
