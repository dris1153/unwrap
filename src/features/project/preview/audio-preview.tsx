import { useEffect, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Play, Pause, ArrowsClockwise } from "@phosphor-icons/react";
import WaveSurfer from "wavesurfer.js";
import type { PreviewPayload } from "../../../lib/types";

type AudioPayload = Extract<PreviewPayload, { type: "audio" }>;

function formatMs(ms: number): string {
  const s = Math.floor(ms / 1000);
  const m = Math.floor(s / 60);
  const sec = s % 60;
  return `${m}:${sec.toString().padStart(2, "0")}`;
}

export function AudioPreview({ path, duration_ms }: AudioPayload) {
  const containerRef = useRef<HTMLDivElement>(null);
  const wsRef = useRef<WaveSurfer | null>(null);
  const [playing, setPlaying] = useState(false);
  const [currentMs, setCurrentMs] = useState(0);
  const [ready, setReady] = useState(false);
  const fileName = path.split(/[\\/]/).pop() ?? path;

  useEffect(() => {
    if (!containerRef.current) return;

    const ws = WaveSurfer.create({
      container: containerRef.current,
      url: convertFileSrc(path),
      waveColor: "#3F3F46",
      progressColor: "#10B981",
      cursorColor: "#10B981",
      barWidth: 2,
      barGap: 1,
      barRadius: 2,
      height: 80,
      normalize: true,
    });

    wsRef.current = ws;

    ws.on("ready", () => setReady(true));
    ws.on("play",  () => setPlaying(true));
    ws.on("pause", () => setPlaying(false));
    ws.on("timeupdate", (t) => setCurrentMs(t * 1000));
    ws.on("finish",     () => setPlaying(false));

    return () => {
      ws.destroy();
      wsRef.current = null;
    };
  }, [path]);

  const togglePlay = () => wsRef.current?.playPause();

  return (
    <div className="flex-1 flex flex-col items-center justify-center gap-6 bg-base px-8">
      <div className="flex flex-col items-center gap-1 w-full max-w-2xl">
        <span className="font-mono text-[12px] text-secondary">{fileName}</span>
        <span className="font-mono text-[10px] text-tertiary">
          {formatMs(duration_ms)}
        </span>
      </div>

      {/* Waveform container */}
      <div ref={containerRef} className="w-full max-w-2xl" />

      {/* Controls */}
      <div className="flex items-center gap-4">
        <button
          onClick={togglePlay}
          disabled={!ready}
          className="w-10 h-10 rounded-full bg-accent hover:bg-accent-strong text-base flex items-center justify-center transition-colors disabled:opacity-40"
        >
          {playing ? <Pause size={18} weight="fill" /> : <Play size={18} weight="fill" />}
        </button>
      </div>

      {/* Timeline */}
      <div className="flex items-center gap-2 font-mono text-[11px] text-tertiary">
        <span>{formatMs(currentMs)}</span>
        <span className="text-overlay">/</span>
        <span>{formatMs(duration_ms)}</span>
      </div>

      {/* Loop/speed cosmetic v1 */}
      <div className="flex items-center gap-3 text-tertiary">
        <button className="flex items-center gap-1.5 text-[11px] hover:text-secondary transition-colors">
          <ArrowsClockwise size={13} />
          Loop
        </button>
        <span className="text-overlay">|</span>
        <button className="text-[11px] hover:text-secondary transition-colors">1× Speed</button>
      </div>
    </div>
  );
}
