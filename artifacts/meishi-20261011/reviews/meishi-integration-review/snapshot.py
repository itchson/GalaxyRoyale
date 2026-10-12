import hashlib
import json
from pathlib import Path
import subprocess

root = Path("C:/Users/mitch/Documents/ChatGPT/galaxy-royale")
owned = Path(__file__).resolve().parent
paths = [
    "apps/game/src/main.ts", "apps/game/src/pixel-ui.ts", "apps/game/index.html",
    "apps/game/src/cargo.ts", "apps/game/src/cargo.css", "apps/game/src/style.css",
    "apps/game/src/meishi.ts", "apps/game/src/meishi.css", "apps/game/src/flight-menu.ts",
    "apps/game/src/kikai-portrait.ts", "apps/game/src/campaign.ts", "apps/game/src/loadout.ts",
    "crates/game_web/src/ui_scene.rs", "crates/game_web/src/kikai_art.rs",
    "crates/game_core/src/campaign_runtime.rs", "crates/game_core/src/ship_progression.rs",
    "crates/game_core/src/lib.rs",
]
head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root).decode().strip()
items = []
for name in paths:
    raw = (root / name).read_bytes()
    target = owned / "source" / name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(raw)
    items.append({"path": name, "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()})
for item in items:
    raw = (root / item["path"]).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == item["sha256"], "source changed during snapshot"
assert head == subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root).decode().strip()
(owned / "source-pins.json").write_text(json.dumps({"root_head": head,
    "domain": "actual working-tree raw bytes; not claimed committed or build inputs",
    "inputs": items}, indent=2) + "\n", encoding="utf-8")
print(f"Pinned {len(items)} stable actual working-source files at Root HEAD {head}")
