"""Local-only wake-word and command capture sidecar.

This process has no networking code. Ambient 16 kHz microphone frames are passed
directly to openWakeWord in memory. Only post-activation command audio is written
to the app-provided temporary directory, and the native host deletes it after STT.
"""
from __future__ import annotations
import argparse
import json
import queue
import signal
import sys
import tempfile
import time
import wave
from pathlib import Path
import numpy as np
import sounddevice as sd

def _deny_network(event: str, _args: object) -> None:
    # Defense in depth: even transitive packages cannot open a socket in this process.
    if event.startswith("socket."):
        raise PermissionError("wake-word process is local-only")

sys.addaudithook(_deny_network)

from openwakeword.model import Model

SAMPLE_RATE = 16_000
FRAME_SAMPLES = 1_280
SILENCE_RMS = 260.0
END_SILENCE_SECONDS = 1.15
MAX_COMMAND_SECONDS = 15.0

def emit(event: str, **data: object) -> None:
    print(json.dumps({"event": event, **data}), flush=True)

class LocalWakeWord:
    def __init__(self, temp_dir: Path, threshold: float) -> None:
        self.temp_dir = temp_dir
        self.threshold = threshold
        self.frames: queue.Queue[np.ndarray] = queue.Queue(maxsize=24)
        self.running = True
        self.model = Model(wakeword_models=["hey_jarvis"], inference_framework="onnx")

    def callback(self, indata: np.ndarray, _frames: int, _time: object, status: object) -> None:
        if status:
            emit("warning", message=str(status))
        try:
            self.frames.put_nowait(indata[:, 0].copy().astype(np.int16))
        except queue.Full:
            try: self.frames.get_nowait()
            except queue.Empty: pass

    def capture_command(self) -> Path | None:
        emit("listening")
        chunks: list[np.ndarray] = []
        heard_speech = False
        silent_frames = 0
        silence_limit = int(END_SILENCE_SECONDS * SAMPLE_RATE / FRAME_SAMPLES)
        max_frames = int(MAX_COMMAND_SECONDS * SAMPLE_RATE / FRAME_SAMPLES)
        for _ in range(max_frames):
            frame = self.frames.get(timeout=2)
            rms = float(np.sqrt(np.mean(frame.astype(np.float32) ** 2)))
            chunks.append(frame)
            if rms >= SILENCE_RMS:
                heard_speech = True; silent_frames = 0
            elif heard_speech:
                silent_frames += 1
                if silent_frames >= silence_limit: break
        if not heard_speech:
            emit("ready"); return None
        self.temp_dir.mkdir(parents=True, exist_ok=True)
        handle = tempfile.NamedTemporaryFile(prefix="jarvis-command-", suffix=".wav", dir=self.temp_dir, delete=False)
        path = Path(handle.name); handle.close()
        with wave.open(str(path), "wb") as wav:
            wav.setnchannels(1); wav.setsampwidth(2); wav.setframerate(SAMPLE_RATE)
            wav.writeframes(np.concatenate(chunks).tobytes())
        emit("command", path=str(path)); return path

    def run(self) -> None:
        emit("ready", model="openWakeWord hey_jarvis", local_only=True)
        with sd.InputStream(samplerate=SAMPLE_RATE, channels=1, dtype="int16", blocksize=FRAME_SAMPLES, callback=self.callback):
            while self.running:
                frame = self.frames.get()
                scores = self.model.predict(frame)
                score = max((float(v) for k, v in scores.items() if "jarvis" in k.lower()), default=0.0)
                if score >= self.threshold:
                    emit("wake", score=round(score, 4)); self.model.reset(); self.capture_command(); emit("ready")

def main() -> None:
    parser = argparse.ArgumentParser(); parser.add_argument("--temp-dir", required=True); parser.add_argument("--threshold", type=float, default=0.55); args = parser.parse_args()
    service = LocalWakeWord(Path(args.temp_dir), args.threshold)
    signal.signal(signal.SIGTERM, lambda *_: setattr(service, "running", False))
    signal.signal(signal.SIGINT, lambda *_: setattr(service, "running", False))
    try: service.run()
    finally: emit("stopped")

if __name__ == "__main__": main()
