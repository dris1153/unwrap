import Editor from "@monaco-editor/react";
import type { PreviewPayload } from "../../../lib/types";

type TextPayload = Extract<PreviewPayload, { type: "text" }>;

export function TextPreview({ content, language }: TextPayload) {
  return (
    <div className="flex-1 min-h-0 overflow-hidden">
      <Editor
        value={content}
        language={language ?? "plaintext"}
        theme="vs-dark"
        options={{
          readOnly: true,
          minimap: { enabled: false },
          fontFamily: "JetBrains Mono",
          fontSize: 13,
          lineHeight: 20,
          scrollBeyondLastLine: false,
          renderLineHighlight: "none",
          automaticLayout: true,
          wordWrap: "on",
        }}
        height="100%"
        loading={
          <div className="flex-1 flex items-center justify-center bg-[#1e1e1e]">
            <span className="font-mono text-[12px] text-tertiary">Loading…</span>
          </div>
        }
      />
    </div>
  );
}
