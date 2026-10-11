import hashlib
import json
from pathlib import Path

owned = Path(__file__).resolve().parent
root = Path("C:/Users/mitch/Documents/ChatGPT/galaxy-royale")
manifest = json.loads((owned / "manifest.json").read_text(encoding="utf-8"))
for item in manifest["files"]:
    raw = (owned / item["path"]).read_bytes()
    assert len(raw) == item["bytes"] and hashlib.sha256(raw).hexdigest() == item["sha256"], item["path"]
inputs = json.loads((owned / "source-pins.json").read_text(encoding="utf-8"))["inputs"]
for item in inputs:
    raw = (root / item["path"]).read_bytes()
    assert len(raw) == item["bytes"] and hashlib.sha256(raw).hexdigest() == item["sha256"], item["path"]
print(f"Verified {len(manifest['files'])} raw files and {len(inputs)} current Root input hashes; no tests run")
