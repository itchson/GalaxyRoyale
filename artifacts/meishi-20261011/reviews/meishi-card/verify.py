import hashlib
import json
from pathlib import Path
import subprocess

owned = Path(__file__).resolve().parent
root = owned.parents[1]
manifest = json.loads((owned / "manifest.json").read_text(encoding="utf-8"))
for item in manifest["files"]:
    raw = (owned / item["path"]).read_bytes()
    assert len(raw) == item["bytes"] and hashlib.sha256(raw).hexdigest() == item["sha256"], item["path"]
for item in manifest["inputs"]:
    raw = subprocess.check_output(["git", "show", f"{manifest['source_commit']}:{item['path']}"], cwd=root)
    assert len(raw) == item["bytes"] and hashlib.sha256(raw).hexdigest() == item["sha256"], item["path"]
print(f"Verified {len(manifest['files'])} raw files and {len(manifest['inputs'])} exact committed inputs; no tests rerun")
