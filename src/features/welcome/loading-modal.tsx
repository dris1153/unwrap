import { useEffect, useRef } from "react";
import type { ProgressPayload } from "../../lib/types";

interface LoadingModalProps {
  /** Current progress snapshot, or null when not yet started. */
  progress: ProgressPayload | null;
  /** Called when the user requests cancellation via ESC or the button. */
  onCancel: () => void;
}

/**
 * Full-screen modal shown while open_project is in progress.
 * Design: zinc-900 backdrop, centered card, emerald progress bar.
 */
export function LoadingModal({ progress, onCancel }: LoadingModalProps) {
  const cancelRef = useRef(onCancel);
  cancelRef.current = onCancel;

  // ESC to cancel
  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if (e.key === "Escape") cancelRef.current();
    }
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, []);

  const percent = progress?.percent ?? 0;
  const phase = progress?.phase ?? "preparing";
  const message = progress?.message ?? "Starting…";

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Opening project"
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 9999,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "rgba(9,9,11,0.80)",
        backdropFilter: "blur(4px)",
      }}
    >
      <div
        style={{
          background: "var(--color-surface, #18181b)",
          border: "1px solid rgba(255,255,255,0.08)",
          borderRadius: 16,
          padding: "32px 36px",
          minWidth: 360,
          maxWidth: 480,
          width: "100%",
          boxShadow: "0 24px 64px rgba(0,0,0,0.6)",
        }}
      >
        {/* Header */}
        <div
          style={{
            fontFamily: "var(--font-sans, sans-serif)",
            fontSize: 16,
            fontWeight: 600,
            color: "var(--color-primary, #fafafa)",
            marginBottom: 8,
          }}
        >
          Opening project
        </div>

        {/* Phase label */}
        <div
          style={{
            fontFamily: "var(--font-mono, monospace)",
            fontSize: 12,
            color: "var(--color-secondary, #a1a1aa)",
            marginBottom: 20,
            textTransform: "lowercase",
          }}
        >
          {phase}
        </div>

        {/* Progress bar track */}
        <div
          style={{
            height: 6,
            borderRadius: 999,
            background: "rgba(255,255,255,0.08)",
            overflow: "hidden",
            marginBottom: 12,
          }}
        >
          <div
            style={{
              height: "100%",
              borderRadius: 999,
              background: "var(--color-accent, #10b981)",
              width: `${Math.max(2, Math.min(100, percent))}%`,
              transition: "width 0.3s ease",
            }}
          />
        </div>

        {/* Percent + message row */}
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            marginBottom: 24,
          }}
        >
          <span
            style={{
              fontFamily: "var(--font-mono, monospace)",
              fontSize: 12,
              color: "var(--color-accent, #10b981)",
            }}
          >
            {Math.round(percent)}%
          </span>
          <span
            style={{
              fontFamily: "var(--font-sans, sans-serif)",
              fontSize: 12,
              color: "var(--color-secondary, #a1a1aa)",
              maxWidth: 300,
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
            }}
          >
            {message}
          </span>
        </div>

        {/* Cancel button */}
        <div style={{ display: "flex", justifyContent: "flex-end" }}>
          <button
            onClick={onCancel}
            style={{
              height: 32,
              padding: "0 12px",
              borderRadius: 8,
              border: "1px solid rgba(255,255,255,0.12)",
              background: "transparent",
              color: "var(--color-secondary, #a1a1aa)",
              fontFamily: "var(--font-sans, sans-serif)",
              fontSize: 13,
              fontWeight: 500,
              cursor: "pointer",
            }}
          >
            Cancel  <kbd style={{ opacity: 0.5, fontSize: 11 }}>ESC</kbd>
          </button>
        </div>
      </div>
    </div>
  );
}
