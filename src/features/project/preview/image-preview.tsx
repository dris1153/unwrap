import React, { useRef, useState, useCallback, useEffect } from "react";
import { TransformWrapper, TransformComponent, useControls } from "react-zoom-pan-pinch";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Minus, Plus, FrameCorners, Checkerboard } from "@phosphor-icons/react";
import type { PreviewPayload } from "../../../lib/types";
import { Filmstrip } from "../filmstrip/filmstrip";

type ImagePayload = Extract<PreviewPayload, { type: "image" }>;

type Props = ImagePayload & {
  projectId: string;
  nodeId: string;
};

/** Zoom controls that live inside TransformWrapper context */
function ZoomControls({ onToggleChecker }: {
  onToggleChecker: () => void;
}) {
  const { zoomIn, zoomOut, resetTransform } = useControls();
  const [pct, setPct] = useState(100);

  return (
    <div className="absolute bottom-4 right-4 flex items-center gap-1 bg-elevated/90 backdrop-blur-md border border-border-default rounded-lg p-1 shadow-xl z-10">
      <button
        onClick={() => { zoomOut(); setPct((p) => Math.max(10, p - 25)); }}
        className="w-7 h-7 rounded-[var(--radius-md)] hover:bg-overlay text-secondary hover:text-primary flex items-center justify-center transition-colors"
      >
        <Minus size={12} />
      </button>
      <div className="px-2 font-mono text-[11px] text-primary min-w-[44px] text-center">
        {pct}%
      </div>
      <button
        onClick={() => { zoomIn(); setPct((p) => Math.min(1600, p + 25)); }}
        className="w-7 h-7 rounded-[var(--radius-md)] hover:bg-overlay text-secondary hover:text-primary flex items-center justify-center transition-colors"
      >
        <Plus size={12} />
      </button>
      <div className="w-px h-4 bg-border-default" />
      <button
        onClick={() => { resetTransform(); setPct(100); }}
        title="Fit"
        className="w-7 h-7 rounded-[var(--radius-md)] hover:bg-overlay text-secondary hover:text-primary flex items-center justify-center transition-colors"
      >
        <FrameCorners size={12} />
      </button>
      <button
        onClick={onToggleChecker}
        title="Toggle alpha checker"
        className="w-7 h-7 rounded-[var(--radius-md)] hover:bg-overlay text-secondary hover:text-primary flex items-center justify-center transition-colors"
      >
        <Checkerboard size={12} />
      </button>
    </div>
  );
}

export function ImagePreview({ path, width, height, projectId, nodeId }: Props) {
  const [showChecker, setShowChecker] = useState(true);
  const [coords, setCoords] = useState<{ x: number; y: number } | null>(null);
  const [pixelHex, setPixelHex] = useState<string | null>(null);
  const imgRef = useRef<HTMLImageElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const fileName = path.split(/[\\/]/).pop() ?? path;
  const srcUrl = convertFileSrc(path);

  // Draw image into off-screen canvas for pixel readback
  const initCanvas = useCallback(() => {
    const img = imgRef.current;
    const canvas = canvasRef.current;
    if (!img || !canvas) return;
    canvas.width = img.naturalWidth || width;
    canvas.height = img.naturalHeight || height;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    try {
      ctx.drawImage(img, 0, 0);
    } catch {
      // Cross-origin safety — degraded gracefully
    }
  }, [width, height]);

  const handleMouseMove = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      const img = imgRef.current;
      const canvas = canvasRef.current;
      if (!img) return;
      const rect = img.getBoundingClientRect();
      const x = Math.floor(((e.clientX - rect.left) / rect.width) * (img.naturalWidth || width));
      const y = Math.floor(((e.clientY - rect.top) / rect.height) * (img.naturalHeight || height));
      setCoords({ x, y });
      // Pixel readback
      if (canvas) {
        const ctx = canvas.getContext("2d");
        if (ctx) {
          try {
            const [r, g, b] = ctx.getImageData(x, y, 1, 1).data;
            setPixelHex(
              `#${r.toString(16).padStart(2, "0")}${g.toString(16).padStart(2, "0")}${b.toString(16).padStart(2, "0")}`.toUpperCase(),
            );
          } catch {
            setPixelHex(null);
          }
        }
      }
    },
    [width, height],
  );

  useEffect(() => {
    const img = imgRef.current;
    if (img?.complete) initCanvas();
  }, [initCanvas]);

  return (
    <div className="flex flex-col flex-1 min-h-0">
      {/* Preview canvas area */}
      <div
        className={showChecker ? "checker relative flex-1 overflow-hidden" : "relative flex-1 overflow-hidden bg-base"}
        onMouseMove={handleMouseMove}
        onMouseLeave={() => { setCoords(null); setPixelHex(null); }}
      >
        {/* Hidden canvas for pixel readback */}
        <canvas ref={canvasRef} className="hidden" />

        <TransformWrapper
          minScale={0.1}
          maxScale={16}
          doubleClick={{ mode: "zoomIn" }}
        >
          {() => (
            <>
              <TransformComponent
                wrapperStyle={{ width: "100%", height: "100%", position: "absolute", inset: 0 }}
                contentStyle={{ display: "flex", alignItems: "center", justifyContent: "center" }}
              >
                <img
                  ref={imgRef}
                  src={srcUrl}
                  alt={fileName}
                  onLoad={initCanvas}
                  draggable={false}
                  style={{ imageRendering: "pixelated", maxWidth: "80%", maxHeight: "80%" }}
                />
              </TransformComponent>

              <ZoomControls
                onToggleChecker={() => setShowChecker((v) => !v)}
              />
            </>
          )}
        </TransformWrapper>

        {/* Coordinates HUD top-right */}
        {coords && (
          <div className="absolute top-3 right-3 px-2 py-1 bg-elevated/80 backdrop-blur-md border border-border-default rounded-[var(--radius-md)] font-mono text-[10px] text-tertiary z-10 pointer-events-none">
            x: <span className="text-secondary">{coords.x}</span>
            <span className="mx-1 text-overlay">·</span>
            y: <span className="text-secondary">{coords.y}</span>
            {pixelHex && (
              <>
                <span className="mx-1 text-overlay">·</span>
                <span className="text-tex">{pixelHex}</span>
              </>
            )}
          </div>
        )}

        {/* File label top-left */}
        <div className="absolute top-3 left-3 flex items-center gap-2 px-2 py-1 bg-elevated/80 backdrop-blur-md border border-border-default rounded-[var(--radius-md)] z-10 pointer-events-none">
          <span className="font-mono text-[10px] text-secondary">{fileName}</span>
        </div>
      </div>

      {/* Filmstrip */}
      <Filmstrip projectId={projectId} nodeId={nodeId} />
    </div>
  );
}
