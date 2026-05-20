import type { AssetNode, PreviewPayload } from "../../../lib/types";

type Row = { label: string; value: string; accent?: boolean };

function buildRows(node: AssetNode, payload: PreviewPayload | null | undefined): Row[] {
  const meta = node.metadata as Record<string, unknown>;
  const rows: Row[] = [];

  if (payload?.type === "image") {
    rows.push({ label: "Dimensions", value: `${payload.width} × ${payload.height} px` });
    if (meta.format)     rows.push({ label: "Format",     value: String(meta.format) });
    if (meta.mip_count)  rows.push({ label: "Mipmaps",    value: `Yes (${meta.mip_count})`, accent: true });
    if (meta.compressed !== undefined)
      rows.push({ label: "Compressed", value: meta.compressed ? "Yes" : "No (raw)" });
    if (meta.filter_mode) rows.push({ label: "Filter mode", value: String(meta.filter_mode) });
    if (meta.wrap_mode)   rows.push({ label: "Wrap mode",   value: String(meta.wrap_mode) });
  }

  if (payload?.type === "audio") {
    const s = Math.round(payload.duration_ms / 1000);
    rows.push({ label: "Duration", value: `${s}s (${payload.duration_ms}ms)` });
    if (meta.channels)    rows.push({ label: "Channels",    value: String(meta.channels) });
    if (meta.sample_rate) rows.push({ label: "Sample rate", value: `${meta.sample_rate} Hz` });
  }

  if (node.size > 0) {
    rows.push({ label: "Size on disk", value: formatSize(node.size) });
  }

  if (meta.pixel_hash) {
    rows.push({ label: "Pixel hash", value: String(meta.pixel_hash) });
  }

  return rows;
}

function formatSize(b: number): string {
  if (b < 1024) return `${b} B`;
  if (b < 1024 * 1024) return `${(b / 1024).toFixed(0)} KB`;
  return `${(b / (1024 * 1024)).toFixed(2)} MB`;
}

type PropertiesSectionProps = {
  node: AssetNode;
  payload?: PreviewPayload | null;
};

export function PropertiesSection({ node, payload }: PropertiesSectionProps) {
  const rows = buildRows(node, payload);
  if (rows.length === 0) return null;

  return (
    <div className="px-4 py-3 border-b border-border-default">
      <div className="flex items-center justify-between mb-2">
        <span className="font-mono text-[10px] tracking-[0.12em] uppercase text-tertiary">
          Properties
        </span>
      </div>
      <div className="space-y-1.5">
        {rows.map((r) => (
          <div key={r.label} className="flex items-center justify-between">
            <span className="font-sans text-[12px] text-secondary">{r.label}</span>
            <span
              className={`font-mono text-[11.5px] ${r.accent ? "text-accent" : "text-primary"}`}
            >
              {r.value}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
