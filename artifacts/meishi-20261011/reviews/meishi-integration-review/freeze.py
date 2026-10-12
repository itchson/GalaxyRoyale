import hashlib
import json
from pathlib import Path

owned = Path(__file__).resolve().parent
files = []
for path in sorted(owned.rglob("*")):
    if path.is_file() and path.name != "manifest.json":
        raw = path.read_bytes()
        files.append({"path": path.relative_to(owned).as_posix(), "bytes": len(raw),
                      "sha256": hashlib.sha256(raw).hexdigest()})
(owned / "manifest.json").write_text(json.dumps({"version": 1,
    "domain": "exact raw working-source snapshots and own notes; no normalization",
    "files": files}, indent=2) + "\n", encoding="utf-8")
print(f"Frozen {len(files)} raw review files")
