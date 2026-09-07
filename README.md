# DKOOs JARVIS

DKOOs JARVIS is a local-first macOS engineering command center. It combines a Tauri/Rust security boundary with a React HUD, Groq inference and speech recognition, Fish Audio speech output, and fully local openWakeWord activation.

The owner—not the model—controls projects, microphone use, networking, and high-impact actions. No computer-wide discovery or home-directory indexing exists.

## Current capabilities

- Native macOS app and optional menu-bar presence
- Explicit native folder picker and per-folder project authorization
- Local file tree/viewer and protected local search
- Git status and diff views
- Controlled test, lint, and build runner with no arbitrary shell
- Groq chat provider and command-only Groq STT
- Bounded Groq agent loop for permission-gated list/read/search/Git tools, enabled only by each project's Cloud Code checkbox
- Hash-checked, Git-aware atomic file editor with a manual Level-2 review dialog
- Fish Audio TTS with configurable reference/voice ID
- Local `hey_jarvis` ONNX wake-word process with socket denial and in-memory ambient audio
- SQLite tasks, structured project memory, conversation history, and redacted action audit
- Emergency stop, lock mode, visible microphone state, start-at-login off by default
- macOS Keychain records dedicated to the two JARVIS credentials
- Two-step JARVIS-data reset that never touches repositories

## Security model

Access is denied by default. Authorizing `/path/ProjectA` does not authorize its parent or siblings. `PermissionGate` resolves canonical paths, rejects `..`, validates symlinks, enforces exact project roots, blocks secrets such as `.env`, PEM/private-key, credential, and token-cache files, and stops immediately after revocation or lock.

JARVIS has no permanent-delete tool. The command runner has no shell interpolation and rejects `sudo`, deletion utilities, disk tools, AppleScript, process killers, force push, hard reset, history rewriting, and shell metacharacters. Git reads and project commands run only with an authorized-root `cwd`, cleared environment, timeout, cancellation, and captured output.

Repository text, web content, logs, and package output are untrusted data. They cannot alter native permissions. No analytics, telemetry, tracking, crash-reporting service, cloud sync, Full Disk Access, Accessibility, Screen Recording, Camera, or Automation permission is included.

See [ARCHITECTURE.md](ARCHITECTURE.md) for the trust boundaries and voice data flow.

## Requirements

- macOS 12 or newer
- Xcode Command Line Tools
- Node.js 20 or newer
- Rust stable (for source builds)
- Python 3.9+ only when rebuilding the local wake-word sidecar

The checked build is Intel (`x86_64-apple-darwin`). Build the sidecar and desktop app on Apple Silicon to produce a native ARM package.

## Development

```sh
npm install
npm run desktop:dev
```

The web-only `npm run dev` preview cannot invoke the native security backend and intentionally reports that limitation.

Create `.env` from `.env.example` only for isolated provider development if needed. `.env` files are ignored. The installed app should be configured in Settings; do not commit or place real credentials in chat.

## Credentials

Open Settings > Credentials and enter Groq and Fish Audio keys. The app stores only its two named records in macOS Keychain and shows only their final four characters. Set the separate Fish Audio voice/reference ID in the same screen. Text mode remains available when Fish Audio is unconfigured.

Environment-variable fallback names for local development are exactly:

```text
dkoo_JARVIS
dkoo_JARVIS_voice
```

## Voice and wake word

Build the Intel local sidecar before packaging:

```sh
./scripts/build-voice-sidecar.sh
```

The script downloads openWakeWord's official `hey_jarvis`, mel-spectrogram, embedding, and VAD model assets at build time and embeds them. Runtime wake detection is offline. The sidecar installs a socket-denying audit hook before loading openWakeWord, never writes ambient frames, and creates audio only after activation. Command capture ends after silence or 15 seconds. Rust removes temporary WAV data immediately after STT.

Voice Listening is off by default. Enabling it is the only action that requests microphone permission. Quitting JARVIS stops the microphone process. Start at Login is also off by default.

## Testing

```sh
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
python3 -c 'import importlib.util; s=importlib.util.spec_from_file_location("v","voice-sidecar/test_voice_privacy.py"); m=importlib.util.module_from_spec(s); s.loader.exec_module(m); [getattr(m,n)() for n in dir(m) if n.startswith("test_")]'
```

Native regression tests cover authorized reads/edits, traversal, absolute outside paths, siblings, home and personal folders, symlink escape, protected secrets/private keys, revocation, lock, deletion absence, `sudo`, hard reset, force push, AppleScript, and shell injection.

## Build and install

```sh
npm run desktop:build -- --bundles app
hdiutil create -volname "DKOOs JARVIS" \
  -srcfolder "src-tauri/target/release/bundle/macos/DKOOs JARVIS.app" \
  -ov -format UDZO \
  "src-tauri/target/release/bundle/dmg/DKOOs JARVIS_0.1.0_x64.dmg"
```

The direct `hdiutil` step intentionally avoids Tauri's decorative Finder/AppleScript DMG layout. The local build is unsigned/not notarized because no Apple Developer certificate was supplied. macOS may require Control-click > Open for local testing. Production distribution requires your own Developer ID signing and notarization.

## Troubleshooting

- `Desktop backend is unavailable`: launch with `npm run desktop:dev`, not the browser preview.
- Voice reports sidecar unavailable: run the sidecar build script before packaging.
- Groq/Fish shows unconfigured: save the corresponding credential in Settings.
- A file is blocked: confirm it is inside the exact authorized project and is not a protected secret type.
- Folder access fails after moving a project: revoke the old record and authorize the new folder explicitly.
- STOP JARVIS shortcut: Command/Control + Shift + `.`

## Provider references

- [Groq Chat Completions and streaming](https://console.groq.com/docs/text-chat)
- [Groq speech-to-text models](https://console.groq.com/docs/speech-to-text)
- [Fish Audio text-to-speech API](https://docs.fish.audio/api-reference/endpoint/openapi-v1/text-to-speech)
- [openWakeWord project and models](https://github.com/dscripka/openWakeWord)
- [Tauri 2 documentation](https://v2.tauri.app/)
