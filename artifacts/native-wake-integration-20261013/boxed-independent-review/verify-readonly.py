from pathlib import Path
import hashlib,json,subprocess
review=Path(__file__).resolve().parent
author=Path(r"C:/Users/mitch/.codex/worktrees/galaxy-hud-navigation/galaxy-royale")
work=author/"work/boxed-world-review"
candidate="c95c008d22c9b7351463aa1575bf02347659d999"
base="af060d8377389bb0b73de0e09d8f6de58b4f4c3f"
def git(*args): return subprocess.check_output(["git",*args],cwd=author)
def pin(path):
    data=path.read_bytes()
    return {"bytes":len(data),"sha256":hashlib.sha256(data).hexdigest()}
assert git("rev-parse",candidate+"^").decode().strip()==base
assert git("diff","--name-only",base,candidate).decode().splitlines()==["crates/game_core/src/co_op_tests.rs"]
assert git("diff","--numstat",base,candidate).decode().splitlines()==["277\t0\tcrates/game_core/src/co_op_tests.rs"]
manifest=json.loads((work/"MANIFEST.json").read_text(encoding="utf-8"))
source=json.loads((work/"source-pins.json").read_text(encoding="utf-8"))
assert manifest["candidate"]==candidate and source["candidate"]==candidate
for item in manifest["evidence"]:
    assert pin(work/item["path"])=={"bytes":item["bytes"],"sha256":item["sha256"]},item["path"]
for item in source["source"]:
    data=git("show",candidate+":"+item["path"])
    assert data==(work/item["snapshot"]).read_bytes(),item["path"]
    assert len(data)==item["bytes"] and hashlib.sha256(data).hexdigest()==item["sha256"]
    assert git("rev-parse",candidate+":"+item["path"]).decode().strip()==item["git_blob"]
    old=git("show",source["reference_root"]+":"+item["path"])
    assert (data==old)==item["identical_to_reference_root"],item["path"]
    if not item["identical_to_reference_root"]:
        assert old==(work/item["reference_root_snapshot"]).read_bytes()
logs={name:(work/name).read_text(encoding="utf-8-sig") for name in ["co-op-final.log","guest-bridge.log","clippy-final.log","fmt-final.log","native-first.log","native-first-exact.log","native-second.log"]}
assert "23 passed; 0 failed; 1 ignored" in logs["co-op-final.log"]
assert "boxed_solo_world_restores_ecs_transients_and_deterministic_continuation_after_teardown ... ok" in logs["co-op-final.log"]
assert "4 passed; 0 failed; 0 ignored" in logs["guest-bridge.log"]
assert "test result: ok. 0 passed;" in logs["native-first.log"]
assert "assertion failed: !game.chunks.is_empty()" in logs["native-first-exact.log"]
assert "left: Err(StaleMembership)" in logs["native-second.log"]
assert "Finished `dev` profile" in logs["clippy-final.log"] and "error:" not in logs["clippy-final.log"]
assert "PASS: owned test rustfmt and git diff --check" in logs["fmt-final.log"]
world=Path(r"C:/Users/mitch/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hecs-0.11.2/src/world.rs")
results={"scope":"Independent commit, canonical source/snapshot and raw evidence verification only. No Rust tests, compilation, Clippy, formatting, native simulation, browser, RTC or render execution rerun.","candidate":candidate,"parent":base,"changed_files":["crates/game_core/src/co_op_tests.rs"],"added_test_only_lines":277,"verified_author_evidence_files":len(manifest["evidence"]),"verified_canonical_source_files":len(source["source"]),"runtime_inputs_identical_to_root5e":sum(i["identical_to_reference_root"] for i in source["source"]),"recorded_author_native_coop":{"passed":23,"failed":0,"ignored":1},"recorded_author_guest_bridge":{"passed":4,"failed":0,"ignored":0},"author_zero_test_attempt_preserved":True,"author_fixture_failures_preserved":2,"author_manifest_pin":pin(work/"MANIFEST.json"),"author_handoff_pin":pin(work/"HANDOFF.md"),"author_source_pins_pin":pin(work/"source-pins.json"),"local_hecs_primary_source":{"path":str(world),**pin(world)},"inspected_candidate_files":[{"path":i["path"],"canonical_sha256":i["sha256"],"git_blob":i["git_blob"]} for i in source["source"]]}
(review/"verification.json").write_text(json.dumps(results,indent=2)+"\n",encoding="utf-8")
(review/"candidate.diff").write_bytes(git("diff",base,candidate,"--","crates/game_core/src/co_op_tests.rs"))
print(json.dumps({k:results[k] for k in ["candidate","parent","added_test_only_lines","verified_author_evidence_files","verified_canonical_source_files","runtime_inputs_identical_to_root5e","recorded_author_native_coop","recorded_author_guest_bridge","scope"]},indent=2))