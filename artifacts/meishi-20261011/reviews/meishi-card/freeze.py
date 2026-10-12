import hashlib
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[2]
owned = Path(__file__).resolve().parent
commit = "1a6bf7c80b5f426880e79506d58ae18af22f482b"
paths = [
    "apps/game/src/meishi.ts", "apps/game/src/meishi.css", "scripts/test/meishi.test.mjs",
    "apps/game/src/kikai-portrait.ts", "apps/game/src/kikai.ts", "apps/game/src/campaign.ts",
    "apps/game/src/loadout.ts", "apps/game/src/pixel-ui.ts", "apps/game/src/style.css",
    "crates/game_core/src/campaign_runtime.rs", "crates/game_core/src/identity.rs",
    "crates/game_web/src/ui_scene.rs", "crates/game_web/src/kikai_art.rs",
    "package.json", "tsconfig.json",
]
inputs = []
for name in paths:
    raw = subprocess.check_output(["git", "show", f"{commit}:{name}"], cwd=root)
    target = owned / "source" / name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(raw)
    inputs.append({"path": name, "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()})
platform = {"node": subprocess.check_output(["node", "--version"], cwd=root).decode().strip(),
            "commit": commit, "parent": subprocess.check_output(["git", "rev-parse", f"{commit}^"], cwd=root).decode().strip(),
            "scope": "actual TS DOM-adapter tests; no browser/native extraction"}
(owned / "platform.json").write_text(json.dumps(platform, indent=2) + "\n", encoding="utf-8")
files = []
for path in sorted(owned.rglob("*")):
    if not path.is_file() or path.name == "manifest.json":
        continue
    raw = path.read_bytes()
    files.append({"path": path.relative_to(owned).as_posix(), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()})
(owned / "manifest.json").write_text(json.dumps({"version": 1, "source_commit": commit,
    "domain": "exact committed blobs and raw owned evidence; no line normalization", "inputs": inputs, "files": files}, indent=2) + "\n", encoding="utf-8")
print(f"Frozen {len(inputs)} Git inputs and {len(files)} raw files")
