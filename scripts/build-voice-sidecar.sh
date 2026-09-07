#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
python3 -m venv voice-sidecar/.venv
voice-sidecar/.venv/bin/pip install -r voice-sidecar/requirements.txt pyinstaller
voice-sidecar/.venv/bin/python -c 'from openwakeword.utils import download_models; download_models(["hey_jarvis"])'
voice-sidecar/.venv/bin/pyinstaller --noconfirm --clean --onedir --distpath voice-sidecar/dist --workpath voice-sidecar/build --specpath voice-sidecar --name jarvis-wakeword --collect-data openwakeword voice-sidecar/jarvis_wakeword.py
mkdir -p src-tauri/binaries
mkdir -p src-tauri/binaries/jarvis-wakeword
cp -R voice-sidecar/dist/jarvis-wakeword/. src-tauri/binaries/jarvis-wakeword/
