import type { JarvisState } from "../types";

export function CoreHud({ state }: { state: JarvisState }) {
  const slug = state.toLowerCase().replaceAll(" ", "-");
  return (
    <section className={`core-hud state-${slug}`} aria-label={`JARVIS state: ${state}`}>
      <div className="core-crosshair" aria-hidden="true" />
      <div className="orbit orbit-a"><i /><i /><i /></div>
      <div className="orbit orbit-b"><i /><i /></div>
      <div className="orbit orbit-c" />
      <div className="core-radar" />
      <div className="core-center">
        <span className="core-kicker">JARVIS CORE</span>
        <strong>{state}</strong>
        <span className="core-signal">{state === "LOCKED" ? "SYSTEM ISOLATED" : "SECURE LOCAL CONTROL"}</span>
      </div>
      <div className="core-ticks" aria-hidden="true" />
    </section>
  );
}

