import { useCallback, useRef } from "react";

type ResizeHandleProps = {
  /** Called with the pixel delta as the user drags */
  onDelta: (delta: number) => void;
  /** Which edge this handle sits on (affects cursor) */
  side?: "right" | "left";
};

/**
 * 1px visible / 8px hit-area resize handle.
 * Uses pointermove for touch compat.
 */
export function ResizeHandle({ onDelta, side = "right" }: ResizeHandleProps) {
  const dragging = useRef(false);
  const lastX = useRef(0);

  const onPointerDown = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      e.preventDefault();
      dragging.current = true;
      lastX.current = e.clientX;
      (e.target as HTMLElement).setPointerCapture(e.pointerId);
    },
    [],
  );

  const onPointerMove = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      if (!dragging.current) return;
      const delta = e.clientX - lastX.current;
      lastX.current = e.clientX;
      onDelta(side === "right" ? delta : -delta);
    },
    [onDelta, side],
  );

  const onPointerUp = useCallback(() => {
    dragging.current = false;
  }, []);

  return (
    <div
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      style={{
        width: 8,
        cursor: "col-resize",
        flexShrink: 0,
        position: "relative",
        zIndex: 10,
      }}
      className="group"
    >
      {/* 1px visible strip centered in the 8px hit area */}
      <div
        className="absolute inset-y-0 group-hover:bg-accent/40 transition-colors"
        style={{ left: 3, width: 1, background: "transparent" }}
      />
    </div>
  );
}
