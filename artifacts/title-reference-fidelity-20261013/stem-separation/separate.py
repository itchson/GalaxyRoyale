"""Work-only pinned source separation; all user audio stays local."""
from pathlib import Path
import sys
import json
import hashlib
import subprocess
import time
import os

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / "dependencies"))
os.environ["HF_HOME"] = "C:/AI/models/huggingface"
import numpy as np
import torch
from scipy.io import wavfile
from huggingface_hub import hf_hub_download
from safetensors import safe_open
from demucs.hf import load_safetensors_model
from demucs.apply import apply_model
import demucs

SOURCE = Path("C:/Users/mitch/Downloads/Galaxy Royale (3).mp3")
SOURCE_SHA = "95fcb149bb55da752c4d93c18fcc1913a23b70ccafecb52d8db5548dc64755d6"
REPO = "adefossez/HTDemucs"
REVISION = "cbc8a9b1a87023b7fd74e7b3412e6321c0eab003"

def pin(path):
    path = Path(path)
    data = path.read_bytes()
    return {"path": str(path).replace("\\", "/"), "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}

assert pin(SOURCE)["sha256"] == SOURCE_SHA
assert torch.cuda.is_available()
torch.set_num_threads(4)
torch.manual_seed(0)
torch.cuda.reset_peak_memory_stats()
print(json.dumps({"phase": "dependencies", "torch": torch.__version__, "demucs": demucs.__version__, "cuda": torch.version.cuda, "device": torch.cuda.get_device_name(0)}), flush=True)
model_file = hf_hub_download(REPO, "955717e8.safetensors", revision=REVISION)
with safe_open(model_file, framework="pt") as archive:
    metadata = archive.metadata()
    assert metadata["klass"] == "demucs.htdemucs.HTDemucs"
    assert "structure" not in metadata  # no serialized arbitrary class metadata
model = load_safetensors_model(model_file)
assert model.sources == ["drums", "bass", "other", "vocals"]
assert model.samplerate == 44100 and model.audio_channels == 2
model.eval()
print(json.dumps({"phase": "model-loaded", "model": pin(model_file), "sources": model.sources}), flush=True)
command = ["ffmpeg", "-v", "error", "-i", str(SOURCE), "-vn", "-ac", "2", "-ar", "44100", "-f", "f32le", "pipe:1"]
raw = subprocess.check_output(command)
original = np.frombuffer(raw, dtype="<f4").reshape(-1, 2).copy()
assert original.shape == (1882188, 2) and np.isfinite(original).all()
wav = torch.from_numpy(original.T.copy())
ref = wav.mean(0)
mean, std = ref.mean(), ref.std() + 1e-8
started = time.perf_counter()
with torch.inference_mode():
    result = apply_model(model, ((wav - mean) / std)[None], device="cuda", shifts=0, split=True, segment=7, overlap=.25, progress=True, num_workers=0)
    result = result * std + mean
torch.cuda.synchronize()
elapsed = time.perf_counter() - started
arrays = result[0].cpu().numpy().transpose(0, 2, 1)
assert arrays.shape == (4, *original.shape) and np.isfinite(arrays).all()
folder = ROOT / "stems"
folder.mkdir(exist_ok=True)
records = []
for name, values in zip(model.sources, arrays):
    out = folder / f"{name}.wav"
    wavfile.write(out, model.samplerate, values.astype(np.float32))
    records.append({"name": name, "wav": pin(out), "rms": float(np.sqrt(np.mean(values.astype(np.float64) ** 2))), "peak": float(np.max(np.abs(values)))})
summed = arrays.sum(axis=0)
residual = original - summed
record = {
    "scope": "Actual work-only GPU model-separated estimates from supplied MP3; not original stems, a recovered score, source-waveform runtime playback or listening/song-match acceptance.",
    "source": pin(SOURCE), "repository": REPO, "revision": REVISION, "model": pin(model_file), "script": pin(__file__),
    "torch": torch.__version__, "demucs": demucs.__version__, "cuda": torch.version.cuda, "device": torch.cuda.get_device_name(0),
    "decode_command": command, "decoded_float32_sha256": hashlib.sha256(raw).hexdigest(), "sample_rate": 44100,
    "samples": len(original), "seconds": len(original) / 44100, "channels": 2,
    "settings": {"shifts": 0, "split": True, "segment_seconds": 7, "overlap": .25, "workers": 0, "seed": 0},
    "elapsed_separation_seconds": elapsed, "peak_torch_allocated_bytes": torch.cuda.max_memory_allocated(),
    "peak_torch_reserved_bytes": torch.cuda.max_memory_reserved(), "nonfinite": 0,
    "stems": records, "sum_residual_rms": float(np.sqrt(np.mean(residual.astype(np.float64) ** 2))),
    "source_rms": float(np.sqrt(np.mean(original.astype(np.float64) ** 2))),
    "remaining": ["Synthetic arcade/glitch separation bleed review", "Source-supported beat/note/envelope extraction from estimates", "Instrument parameter synthesis and tempo integration", "Human same-song acceptance"]}
(ROOT / "separation.json").write_text(json.dumps(record, indent=2, allow_nan=False) + "\n", encoding="utf-8")
print(json.dumps(record, indent=2), flush=True)
