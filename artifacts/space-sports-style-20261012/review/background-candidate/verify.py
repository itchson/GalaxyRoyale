"""Read-only verification of this independent review's frozen inputs/evidence."""

import hashlib
import json
from pathlib import Path
import subprocess

base = Path(__file__).resolve().parent
manifest = json.loads((base / "frozen-hashes.json").read_text(encoding="utf-8"))
repo = Path(manifest["source_repository"])
for item in manifest["source_inputs"]:
    raw = subprocess.check_output(
        ["git", "show", manifest["commit"] + ":" + item["path"]], cwd=repo
    )
    assert len(raw) == item["bytes"], item["path"]
    assert hashlib.sha256(raw).hexdigest() == item["sha256"], item["path"]
for item in manifest["raw_files"]:
    raw = Path(item["path"]).read_bytes()
    assert len(raw) == item["bytes"], item["path"]
    assert hashlib.sha256(raw).hexdigest() == item["sha256"], item["path"]
print(
    f"PASS {len(manifest['source_inputs'])} committed source pins / "
    f"{len(manifest['raw_files'])} raw review/evidence pins; "
    "no native tests, exporter or browser execution"
)
