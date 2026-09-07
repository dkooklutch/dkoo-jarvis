# Architecture decision record

## Desktop shell

JARVIS uses Tauri 2 with a React 19/TypeScript interface and a Rust trust boundary. This was selected over Electron because the installed application can use the macOS WebKit already present on the machine, keeps the native binary small, supports a narrow capability manifest, and places filesystem, process, credential, and network enforcement outside frontend JavaScript.

The app is sandboxed on macOS. Its entitlements are limited to outbound network access, microphone input, and user-selected read/write folders. It does not request Full Disk Access, Accessibility, Automation, Screen Recording, Camera, Contacts, Calendar, or Photos.

## Trust boundaries

The React process renders state and gathers explicit user choices. It never receives a saved API key. Native commands enter `PermissionGate`, which canonicalizes targets, rejects traversal and symlink escapes, applies protected-file rules, checks the exact authorized root, observes lock state, and writes a redacted audit record.

Project commands are direct process argument arrays. There is no shell-string execution surface. The allowlist rejects privilege escalation, deletion, history rewriting, force push, system tools, AppleScript, shell metacharacters, and non-JARVIS process control. Child environments are cleared and rebuilt from a four-variable allowlist.

## Storage

SQLite in the Tauri application-data directory stores project grants, structured memory, tasks, conversations, settings, and append-oriented actions. The two provider credentials use only these macOS Keychain accounts:

- service `ai.dkoo.jarvis`, account `dkoo_JARVIS`
- service `ai.dkoo.jarvis`, account `dkoo_JARVIS_voice`

The application does not enumerate Keychain.

## Providers

Rust traits isolate `AIProvider`, `SpeechToTextProvider`, `VoiceProvider`, and `WakeWordProvider`. Current implementations use Groq Chat Completions with `openai/gpt-oss-120b`, Groq transcription with `whisper-large-v3-turbo`, Fish Audio `/v1/tts` with the `s2-pro` header, and openWakeWord's `hey_jarvis` ONNX model. Model and endpoint choices were checked against provider documentation in September 2026.

## Voice privacy sequence

The bundled wake-word process has a runtime Python audit hook that denies every socket operation. Ambient 16 kHz frames are held in a bounded in-memory queue and passed to openWakeWord. Only after detection does VAD collect command frames and create a temporary WAV in JARVIS app data. The Rust host validates that path, submits it to Groq STT, and attempts deletion immediately after the transcription call regardless of success. Fish Audio receives only a concise version of the final response. STOP JARVIS kills the child process and frontend playback.

## Approval levels

Safe reads operate after project authorization. Controlled test/lint/build actions are exposed as explicit buttons. High-impact Git and deletion operations are absent from the generic runner. Local-data reset has two manual UI confirmations plus an exact backend confirmation phrase. Project deletion is not implemented; project access can only be revoked.

