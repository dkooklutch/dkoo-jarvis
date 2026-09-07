import type { Health, Project, ActionEntry } from "../types";

const Tone = ({ value }: { value: string }) => <span className={`status-value ${["ERROR", "LOCKED"].includes(value) ? "bad" : ["UNCONFIGURED", "OFF", "NONE"].includes(value) ? "muted" : "good"}`}>{value}</span>;

export function StatusPanel({ health, project, actions }: { health: Health; project?: Project; actions: ActionEntry[] }) {
  return <aside className="right-rail" aria-label="System status">
    <section className="hud-panel project-panel">
      <header><span>01</span>CURRENT PROJECT</header>
      {project ? <><strong>{project.name}</strong><small title={project.root}>{project.root}</small><dl><dt>BRANCH</dt><dd>{project.branch ?? "Checking…"}</dd><dt>CLOUD CODE</dt><dd>{project.cloudCodeAllowed ? "ALLOWED" : "BLOCKED"}</dd></dl></> : <p className="empty">No project access granted.</p>}
    </section>
    <section className="hud-panel health-panel">
      <header><span>02</span>HEALTH</header>
      <dl>
        <dt>JARVIS CORE</dt><dd><Tone value={health.core} /></dd>
        <dt>GROQ</dt><dd><Tone value={health.groq} /></dd>
        <dt>FISH AUDIO</dt><dd><Tone value={health.fishAudio} /></dd>
        <dt>WAKE WORD</dt><dd><Tone value={health.wakeWord} /></dd>
        <dt>MICROPHONE</dt><dd><Tone value={health.microphone} /></dd>
        <dt>PROJECT</dt><dd><Tone value={health.project} /></dd>
      </dl>
    </section>
    <section className="hud-panel recent-panel">
      <header><span>03</span>RECENT ACTIONS</header>
      <div className="recent-list">{actions.slice(0, 5).map(a => <div key={a.id}><time>{new Date(a.timestamp).toLocaleTimeString([], { hour12: false })}</time><b>{a.actor}</b><span>{a.result}</span></div>)}{actions.length === 0 && <p className="empty">No actions this session.</p>}</div>
    </section>
    <section className="hud-panel permission-panel">
      <header><span>04</span>PRIVACY BOUNDARY</header>
      <p><i className="shield-dot" /> DENY BY DEFAULT</p>
      <small>Full Disk Access · NOT REQUIRED</small>
      <small>Screen / Camera · BLOCKED</small>
    </section>
  </aside>;
}

