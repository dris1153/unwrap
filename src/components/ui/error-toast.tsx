import { useEffect, useState } from "react";
import { Warning, X, Copy, Check } from "@phosphor-icons/react";
import { useErrorStore } from "../../stores/use-error-store";

const AUTO_DISMISS_MS = 10_000;

/**
 * Top-right error banner mounted at app root.
 * Displays the most recent error from `useErrorStore` with auto-dismiss
 * and a manual close button. If the error includes a `logHint`, renders
 * a "Copy path" button so the user can paste the path into their file
 * explorer for debugging.
 */
export function ErrorToast() {
  const current = useErrorStore((s) => s.current);
  const dismiss = useErrorStore((s) => s.dismiss);
  const [copied, setCopied] = useState(false);

  // Auto-dismiss timer, reset whenever the error id changes.
  useEffect(() => {
    if (!current) return;
    setCopied(false);
    const t = setTimeout(() => dismiss(), AUTO_DISMISS_MS);
    return () => clearTimeout(t);
  }, [current?.id, dismiss]);

  if (!current) return null;

  async function handleCopy() {
    if (!current?.logHint) return;
    try {
      await navigator.clipboard.writeText(current.logHint);
      setCopied(true);
    } catch {
      // Ignore — older WebView2 may not expose clipboard.writeText.
    }
  }

  return (
    <div
      role="alert"
      className="fixed top-12 right-4 z-50 max-w-md rounded-lg border border-danger/40 bg-surface shadow-lg"
      style={{ animation: "fadeIn 200ms ease-out" }}
    >
      <div className="flex items-start gap-3 p-4">
        <Warning size={20} className="text-danger shrink-0 mt-0.5" weight="fill" />

        <div className="flex-1 min-w-0">
          <div className="font-sans text-[13px] font-semibold text-primary leading-tight">
            {current.title}
          </div>
          <div className="mt-1 font-mono text-[12px] text-secondary break-words">
            {current.message}
          </div>
          {current.logHint && (
            <div className="mt-2 flex items-center gap-2">
              <code className="font-mono text-[11px] text-tertiary bg-overlay/30 px-1.5 py-0.5 rounded truncate">
                {current.logHint}
              </code>
              <button
                onClick={() => void handleCopy()}
                className="text-tertiary hover:text-primary transition-colors"
                aria-label="Copy log path"
              >
                {copied ? <Check size={12} /> : <Copy size={12} />}
              </button>
            </div>
          )}
        </div>

        <button
          onClick={dismiss}
          className="text-tertiary hover:text-primary transition-colors shrink-0"
          aria-label="Dismiss"
        >
          <X size={14} />
        </button>
      </div>
    </div>
  );
}
