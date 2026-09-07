import { Keyboard, LockKeyhole, Mic, Send, ShieldAlert, Volume2 } from "lucide-react";
import { FormEvent, useState } from "react";

export function CommandBar({ disabled, microphone, onSend, onMic, onStop, onLock }: { disabled: boolean; microphone: string; onSend: (value: string) => Promise<void>; onMic: () => void; onStop: () => void; onLock: () => void }) {
  const [value, setValue] = useState("");
  const submit = async (event: FormEvent) => { event.preventDefault(); const command = value.trim(); if (!command) return; setValue(""); await onSend(command); };
  return <form className="command-bar" onSubmit={submit}>
    <div className="command-input"><span className="prompt">›</span><textarea aria-label="JARVIS command" value={value} disabled={disabled} onChange={e => setValue(e.target.value)} onKeyDown={e => { if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); e.currentTarget.form?.requestSubmit(); } }} placeholder={disabled ? "JARVIS is locked" : "Issue a command…"} rows={1}/><span className="key"><Keyboard size={13}/> ⌘K</span></div>
    <button type="button" className={microphone !== "OFF" ? "control active" : "control"} onClick={onMic} aria-label="Toggle microphone"><Mic size={18}/><span>{microphone}</span></button>
    <button type="button" className="control" aria-label="Voice output"><Volume2 size={18}/><span>VOICE</span></button>
    <button type="submit" className="send" disabled={disabled || !value.trim()} aria-label="Send command"><Send size={18}/></button>
    <button type="button" className="danger" onClick={onStop}><ShieldAlert size={18}/><span>STOP JARVIS</span></button>
    <button type="button" className="lock" onClick={onLock}><LockKeyhole size={17}/><span>LOCK</span></button>
  </form>;
}

