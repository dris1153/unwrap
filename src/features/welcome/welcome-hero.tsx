import { DropZone } from "./drop-zone";

/**
 * Left 60% of the Welcome screen — eyebrow, display headline, sub body,
 * drop zone, supports line.
 */
export function WelcomeHero() {
  return (
    <section className="flex flex-col" style={{ width: "60%", paddingRight: 48 }}>
      {/* Eyebrow */}
      <div className="flex items-center gap-2 mb-7">
        <span
          className="w-[6px] h-[6px] rounded-full bg-accent"
          style={{ boxShadow: "0 0 8px rgba(16,185,129,0.6)" }}
        />
        <span className="font-mono text-[11px] tracking-[0.04em] uppercase text-secondary">
          Reverse engineering
          <span className="text-tertiary mx-2">•</span>
          v0.1
        </span>
      </div>

      {/* Display headline */}
      <h1
        className="font-sans font-bold text-primary mb-6"
        style={{
          fontSize: 56,
          lineHeight: "58px",
          letterSpacing: "-0.02em",
        }}
      >
        Decode anything
        <br />
        <span className="relative">
          you own.
          {/* accent dot at end */}
          <span className="absolute -right-3 bottom-3 w-[6px] h-[6px] rounded-full bg-accent" />
        </span>
      </h1>

      {/* Sub body */}
      <p className="font-sans text-[15px] leading-[24px] text-secondary mb-9" style={{ maxWidth: 480 }}>
        Drop a Unity build to extract assets, decompile scripts, and surface
        translatable text — without leaving your desktop.
      </p>

      <DropZone />

      {/* Supports line */}
      <div className="mt-5 font-mono text-[11px] text-tertiary">
        Supports:{" "}
        <span className="text-secondary">Unity 3.5 – 6000.x</span>
        <span className="mx-2 text-overlay">•</span>
        <span className="text-secondary">Mono + IL2CPP</span>
        <span className="mx-2 text-overlay">•</span>
        <span className="text-secondary">Windows 11 / 10</span>
      </div>
    </section>
  );
}
