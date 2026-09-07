import { useState } from "react";
import { ShieldCheck, ArrowRight, Mic2, KeyRound, FolderLock } from "lucide-react";
import { backend } from "../lib/backend";

export function Onboarding({ onComplete, onSettings, onProjects }: { onComplete: () => void; onSettings: () => void; onProjects: () => void }) {
  const [step, setStep] = useState(0);
  const slides = [
    { icon: ShieldCheck, eyebrow: "LOCAL-FIRST", title: "You remain in control.", copy: "JARVIS cannot scan your Mac. It can access only its app data, its own repository, and individual project folders you explicitly authorize." },
    { icon: KeyRound, eyebrow: "PRIVATE CREDENTIALS", title: "Keys stay out of project code.", copy: "Groq and Fish Audio credentials are stored in JARVIS-specific macOS Keychain records. Saved keys are masked and never returned to the interface.", action: onSettings, label: "OPEN CREDENTIAL SETTINGS" },
    { icon: Mic2, eyebrow: "VOICE PRIVACY", title: "Ambient audio stays local.", copy: "Wake-word mode is off by default. When enabled, local detection listens in memory; cloud transcription starts only after activation." },
    { icon: FolderLock, eyebrow: "EXPLICIT PROJECT ACCESS", title: "No projects authorized.", copy: "Add folders one at a time. Authorizing a folder never authorizes its parent, siblings, Desktop, Documents, Downloads, or your home directory.", action: onProjects, label: "ADD A PROJECT" }
  ];
  const item = slides[step]; const Icon = item.icon;
  const next = async () => { if (step < slides.length - 1) setStep(step + 1); else { await backend.completeOnboarding(); onComplete(); } };
  return <main className="onboarding"><div className="onboarding-grid"/><section><div className="onboarding-logo"><Icon /></div><span>{item.eyebrow} · {step + 1}/{slides.length}</span><h1>{item.title}</h1><p>{item.copy}</p>{item.action && <button className="secondary" onClick={item.action}>{item.label}</button>}<button className="primary next" onClick={next}>{step === slides.length - 1 ? "JARVIS ONLINE" : "CONTINUE"}<ArrowRight size={17}/></button></section><aside><b>PRIVACY BASELINE</b><p>Full Disk Access</p><strong>NOT REQUIRED</strong><p>Accessibility</p><strong>NOT REQUIRED</strong><p>Screen / Camera</p><strong>BLOCKED</strong><p>Permanent deletion</p><strong>UNAVAILABLE</strong></aside></main>;
}
