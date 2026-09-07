import { Activity, BrainCircuit, Code2, Files, FolderKanban, GitBranch, ListTodo, MemoryStick, Search, Settings, ShieldCheck, TerminalSquare } from "lucide-react";

const items = [
  ["Projects", FolderKanban], ["Files", Files], ["Search", Search], ["Tasks", ListTodo], ["Git", GitBranch],
  ["Terminal", TerminalSquare], ["Memory", MemoryStick], ["Activity", Activity], ["Privacy", ShieldCheck], ["Settings", Settings]
] as const;

export function Sidebar({ active, onSelect }: { active: string; onSelect: (view: string) => void }) {
  return <aside className="sidebar"><div className="side-mark"><BrainCircuit /><span>DKOO</span></div><nav aria-label="Primary navigation">
    {items.map(([label, Icon], index) => <button key={label} className={active === label ? "active" : ""} onClick={() => onSelect(label)} title={label}><span>{String(index + 1).padStart(2, "0")}</span><Icon size={17} aria-hidden="true"/><b>{label}</b></button>)}
  </nav><div className="side-foot"><Code2 size={16}/><span>ENGINEERING<br/>COMMAND SYSTEM</span></div></aside>;
}

