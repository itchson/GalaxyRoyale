"""Read-only candidate input/packet verification; does not run generators."""
import hashlib
import importlib.util
import json
import math
from pathlib import Path

spec = importlib.util.spec_from_file_location("dock_conversion", Path(__file__).with_name("convert.py"))
M = importlib.util.module_from_spec(spec)
spec.loader.exec_module(M)


def verify():
    record = M.AP.read_json(M.DEFAULT / "validation.json")
    for relative, expected in record["input_sha256"].items():
        path = M.ROOT / relative
        data = path.read_bytes()
        if relative in record["utf8_lf_source_inputs"]:
            data = data.replace(b"\r\n", b"\n")
        M.AP.require(hashlib.sha256(data).hexdigest() == expected, f"Input hash differs: {relative}")
    pack = M.AP.read_json(M.DEFAULT / "pack.json")
    M.AP.validate(pack)
    rig = M.AP.read_json(M.DEFAULT / "rig-extraction.json")
    lamps = set(pack["pixel_rig"]["lamp_cells"])
    for n, frame in enumerate(rig["frames"]):
        full = pack["sprites"][f"dock-{n}"]["indices"]
        ordered = [(i, index) for part in range(4) for i, index in enumerate(full)
                   if index and pack["pixel_rig"]["roles"][i] == part]
        M.AP.require(len(ordered) == frame["occupied_cells"] == 7100, "Rig cell count differs")
        M.AP.require(frame["transforms"] == 4, "Rig transform count differs")
        transform = M.AP.affine(rig["parent"])
        for offset, ((i, index), cell) in enumerate(zip(ordered, frame["cells"])):
            x, y = i % 128 - 64, i // 128 - 64
            corners = [M.AP.point(transform, [x+dx, y+dy]) for dx, dy in ((0,0),(1,0),(1,1),(0,1))]
            for wanted, actual in zip(corners, cell["corners"]):
                M.AP.require(all(math.isfinite(v) for v in actual), "Nonfinite native corner")
                M.AP.require(max(abs(a-b) for a,b in zip(wanted, actual)) < .0001, "Native rig corner differs")
            M.AP.require(0 <= cell["light_response"] <= 1 and 0 <= cell["emission"] <= 1, "Material out of range")
            if pack["pixel_rig"]["roles"][i] != 3:
                M.AP.require(cell["color"] == pack["palette"][index], "Immutable metal/screen color changed")
            if n:
                baseline = rig["frames"][0]["cells"][offset]
                M.AP.require(cell["corners"] == baseline["corners"], "State geometry moved")
                M.AP.require(cell["color"] == baseline["color"] or i in lamps, "Non-lamp state color changed")
    audit = M.AP.read_json(M.DEFAULT / "native-audit.json")
    M.AP.require(audit["timeline_sample_times_verified"] and all(f["maximum_corner_error_world"] == 0 for f in audit["frames"]), "Source extraction proof differs")
    print(json.dumps({"input_hashes": len(record["input_sha256"]), "actual_rig_frames": 3,
                      "cells_each": 7100, "immutable_metal_and_geometry": "verified",
                      "scope": "Read-only source/native packets; no GPU or live Game acceptance"}, indent=2))


if __name__ == "__main__":
    verify()
