import { FileDashed } from "@phosphor-icons/react";

/**
 * Shown when no tab is active or payload kind is unknown.
 */
export function EmptyPreview({ message }: { message?: string }) {
  return (
    <div className="flex-1 flex items-center justify-center bg-base">
      <div className="flex flex-col items-center gap-3 text-tertiary">
        <FileDashed size={32} />
        <span className="font-mono text-[12px]">
          {message ?? "No file open"}
        </span>
      </div>
    </div>
  );
}
