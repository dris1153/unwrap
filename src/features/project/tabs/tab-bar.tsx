import { Plus, Columns, DotsThreeVertical } from "@phosphor-icons/react";
import { useTabStore } from "../../../stores/use-tab-store";
import { TabItem } from "./tab";

/**
 * 32px tab bar with Framer animated active underline.
 */
export function TabBar() {
  const { tabs, activeTabId, setActive, closeTab } = useTabStore();

  return (
    <div
      className="flex items-center bg-surface border-b border-border-default flex-shrink-0 overflow-hidden"
      style={{ height: 32 }}
      role="tablist"
    >
      {tabs.map((tab) => (
        <TabItem
          key={tab.id}
          tab={tab}
          isActive={tab.id === activeTabId}
          onActivate={() => setActive(tab.id)}
          onClose={(e) => {
            e.stopPropagation();
            closeTab(tab.id);
          }}
        />
      ))}

      {/* New tab button */}
      <button
        className="w-8 h-full flex items-center justify-center text-tertiary hover:text-primary hover:bg-overlay flex-shrink-0"
        title="New tab"
        onClick={() => {/* no-op v1 */}}
      >
        <Plus size={12} />
      </button>

      <div className="flex-1" />

      {/* Split / more (cosmetic v1) */}
      <div className="flex items-center gap-0.5 px-2 flex-shrink-0">
        <button
          className="w-7 h-7 rounded-[var(--radius-md)] hover:bg-overlay text-tertiary hover:text-primary flex items-center justify-center"
          title="Split right"
        >
          <Columns size={14} />
        </button>
        <button
          className="w-7 h-7 rounded-[var(--radius-md)] hover:bg-overlay text-tertiary hover:text-primary flex items-center justify-center"
          title="More"
        >
          <DotsThreeVertical size={14} />
        </button>
      </div>
    </div>
  );
}
