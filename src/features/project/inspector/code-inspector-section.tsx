/**
 * code-inspector-section.tsx — inspector content for script/code asset nodes.
 *
 * Renders:
 *   Class name, namespace (inferred from class_fullname), assembly DLL.
 *   Outgoing references (using statements parsed by backend).
 *   Incoming references — "Coming in v2" placeholder per spec.
 */

import { FileCode, Package, ArrowRight } from "@phosphor-icons/react";
import type { DecompilePayload } from "../../../lib/types";

type CodeInspectorSectionProps = {
  payload: DecompilePayload;
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Split "Foo.Bar.ClassName" → namespace = "Foo.Bar", class = "ClassName". */
function splitFullname(fullname: string): { ns: string | null; cls: string } {
  const dot = fullname.lastIndexOf(".");
  if (dot === -1) return { ns: null, cls: fullname };
  return { ns: fullname.slice(0, dot), cls: fullname.slice(dot + 1) };
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function CodeInspectorSection({ payload }: CodeInspectorSectionProps) {
  const { class_fullname, assembly, references } = payload;

  const { ns, cls } = class_fullname
    ? splitFullname(class_fullname)
    : { ns: null, cls: null };

  return (
    <>
      {/* Identity */}
      <div className="px-4 py-3 border-b border-border-default">
        <div className="mb-2.5">
          <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
            Class
          </span>
        </div>
        <div className="space-y-1.5">
          {cls && (
            <div className="flex items-center gap-2">
              <FileCode size={12} className="text-script flex-shrink-0" />
              <span className="font-mono text-[11.5px] text-primary">{cls}</span>
            </div>
          )}
          {ns && (
            <div className="flex items-center gap-2">
              <Package size={12} className="text-secondary flex-shrink-0" />
              <span className="font-mono text-[11px] text-secondary">{ns}</span>
            </div>
          )}
          {assembly && (
            <div className="flex items-center gap-2">
              <span className="font-mono text-[10px] text-tertiary">Assembly</span>
              <span className="font-mono text-[11px] text-secondary truncate">{assembly}</span>
            </div>
          )}
        </div>
      </div>

      {/* Outgoing references */}
      <div className="px-4 py-3 border-b border-border-default">
        <div className="flex items-center justify-between mb-2.5">
          <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
            Uses
          </span>
          {references.length > 0 && (
            <span className="font-mono text-[10px] text-secondary bg-elevated border border-border-default rounded-[4px] px-1.5 py-[1px]">
              {references.length}
            </span>
          )}
        </div>

        {references.length === 0 ? (
          <span className="font-mono text-[11px] text-tertiary">No using statements found</span>
        ) : (
          <div className="space-y-0.5">
            {references.map((ref, i) => (
              <div
                key={i}
                className="flex items-center gap-2 px-2 py-1 rounded-[var(--radius-md)]"
              >
                <ArrowRight size={10} className="text-tertiary flex-shrink-0" />
                <span className="font-mono text-[11px] text-secondary truncate">
                  {ref.symbol}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Incoming references — deferred to v2 */}
      <div className="px-4 py-3 border-b border-border-default">
        <div className="mb-2.5">
          <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
            Referenced by
          </span>
        </div>
        <span className="font-mono text-[11px] text-tertiary">Coming in v2</span>
      </div>
    </>
  );
}
