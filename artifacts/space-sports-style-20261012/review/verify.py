"""Read-only verification of exact Git inputs and reviewed evidence."""

import hashlib
import json
from pathlib import Path
import subprocess


def digest(data):
    return hashlib.sha256(data).hexdigest()


owned = Path(__file__).resolve().parent
repo = owned.parents[1]
manifest = json.loads((owned / "frozen-hashes.json").read_text(encoding="utf-8"))
baseline = manifest["baseline"]
for item in manifest["committed_inputs"]:
    data = subprocess.check_output(
        ["git", "show", f"{baseline}:{item['path']}"], cwd=repo
    )
    assert len(data) == item["bytes"], item["path"]
    assert digest(data) == item["sha256"], item["path"]
for item in manifest["owned_files"]:
    data = (owned / item["path"]).read_bytes()
    assert len(data) == item["bytes"], item["path"]
    assert digest(data) == item["sha256"], item["path"]
for item in manifest["reviewed_evidence"]:
    data = Path(item["path"]).read_bytes()
    assert len(data) == item["bytes"], item["path"]
    assert digest(data) == item["sha256"], item["path"]
for path in manifest["peer_source_parity"]:
    baseline_data = subprocess.check_output(
        ["git", "show", f"{baseline}:{path}"], cwd=repo
    )
    peer_data = subprocess.check_output(
        ["git", "show", f"{manifest['peer_commit']}:{path}"], cwd=repo
    )
    assert baseline_data == peer_data, path
print(json.dumps({
    "baseline": baseline,
    "committed_inputs_verified": len(manifest["committed_inputs"]),
    "owned_files_verified": len(manifest["owned_files"]),
    "reviewed_evidence_verified": len(manifest["reviewed_evidence"]),
    "peer_blob_parity_verified": len(manifest["peer_source_parity"]),
    "native_browser_tests_executed": False,
}, indent=2))
