import markUrl from "../assets/unwrap-mark.svg";

export function BrandHello() {
  return (
    <div className="min-h-[100dvh] bg-base text-primary flex flex-col items-center justify-center gap-6 font-sans">
      <img src={markUrl} alt="Unwrap" className="w-16 h-16" />
      <div className="font-sans font-bold text-[32px] tracking-[-0.02em]">Unwrap</div>
      <div className="font-mono text-[14px] text-secondary">Hello, build.</div>
    </div>
  );
}
