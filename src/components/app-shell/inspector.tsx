import React from "react";
import { PushPin, ArrowsInSimple } from "@phosphor-icons/react";
import { useUiStore } from "../../stores/use-ui-store";

type InspectorProps = {
  /** Render prop supplying the inspector body content (from ProjectRouteContent) */
  renderBody?: () => React.ReactNode;
};

/**
 * 320px right inspector panel, collapsible to 0 via useUiStore.
 * When renderBody is provided, its output replaces the placeholder body.
 */
export function Inspector({ renderBody }: InspectorProps = {}) {
  const { inspectorWidth, toggleInspector } = useUiStore();

  if (inspectorWidth === 0) return null;

  return (
    <aside
      className="border-l border-border-default bg-surface flex flex-col flex-shrink-0"
      style={{ width: inspectorWidth }}
    >
      {/* Header 36px */}
      <div
        className="flex items-center justify-between px-4 border-b border-border-default"
        style={{ height: 36 }}
      >
        <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
          Inspector
        </span>
        <div className="flex items-center gap-0.5">
          <button className="w-5 h-5 rounded-[4px] hover:bg-overlay text-tertiary hover:text-primary flex items-center justify-center">
            <PushPin size={12} />
          </button>
          <button
            onClick={toggleInspector}
            title="Collapse inspector"
            className="w-5 h-5 rounded-[4px] hover:bg-overlay text-tertiary hover:text-primary flex items-center justify-center"
          >
            <ArrowsInSimple size={12} />
          </button>
        </div>
      </div>

      {/* Body — delegated to renderBody when provided */}
      {renderBody ? (
        renderBody()
      ) : (
        <div className="flex-1 flex items-center justify-center">
          <span className="font-sans text-[13px] text-tertiary">
            Select an asset to inspect.
          </span>
        </div>
      )}
    </aside>
  );
}
