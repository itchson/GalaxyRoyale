"""Read-only pins and exact decoded native/PixelRig/CameraView proof checks."""
import hashlib
import importlib.util
import json
import math
from pathlib import Path

spec = importlib.util.spec_from_file_location("dock_archives", Path(__file__).with_name("archive_256.py"))
A = importlib.util.module_from_spec(spec)
spec.loader.exec_module(A)
N, M = A.N, A.N.M


def verify():
    record = M.AP.read_json(N.DEFAULT / "validation.json")
    for relative, expected in record["input_sha256"].items():
        data = (M.ROOT / relative).read_bytes()
        if relative in record["utf8_lf_source_inputs"]:
            data = data.replace(b"\r\n", b"\n")
        M.AP.require(hashlib.sha256(data).hexdigest() == expected, f"Input hash differs: {relative}")
    pack = M.AP.read_json(N.DEFAULT / "pack.json")
    M.AP.validate(pack)
    source = A.read(N.DEFAULT, "native-extraction.json")
    rig = A.read(N.DEFAULT, "rig-extraction.json")
    zoom = A.read(N.DEFAULT, "zoom-extraction.json")
    lamps = set(pack["pixel_rig_tiles"]["lamp_cells_source"])
    M.AP.require(len(lamps) == 1528, "Lamp mask differs")
    for n in range(3):
        ordered = []
        for tile in pack["pixel_rig_tiles"]["tiles"]:
            sprite = pack["sprites"][tile["frame_sprites"][n]]
            x0, y0 = [v + 64 for v in tile["position"]]
            for role in range(4):
                ordered.extend(((y0 + i // 128) * 256 + x0 + i % 128, index, role)
                               for i, index in enumerate(sprite["indices"])
                               if index and tile["roles"][i] == role)
        M.AP.require(len(ordered) == 28549, "Source cell count differs")
        body, frame = source["frames"][n], rig["frames"][n]
        for captured in (body, frame, zoom["frames"][n]):
            M.AP.require(captured["occupied_cells"] == len(captured["cells"]) == 28549,
                         "Native submitted cell count differs")
            M.AP.require(captured["transforms"] == 16 and captured["batches"] == 1,
                         "Native transform/batch count differs")
        M.AP.require(body["sample_time_ms"] == n * 700, "Sample-time seam differs")
        for offset, ((i, index, role), raw, actual) in enumerate(zip(ordered, body["cells"], frame["cells"])):
            x, y = (i % 256 - 128) * .5, (i // 256 - 128) * .5
            wanted = [[x + dx * .5, y + dy * .5] for dx, dy in ((0,0),(1,0),(1,1),(0,1))]
            M.AP.require(raw["corners"] == wanted == actual["corners"], "All-corner source mapping differs")
            M.AP.require(raw["color"] == pack["palette"][index], "Exact PixelBody source color differs")
            M.AP.require(all(math.isfinite(v) for p in actual["corners"] for v in p), "Nonfinite native corner")
            M.AP.require(0 <= actual["light_response"] <= 1 and 0 <= actual["emission"] <= 1,
                         "Native material out of range")
            if role != 3:
                M.AP.require(actual["color"] == raw["color"], "Immutable alloy/deck/screen color changed")
            if n:
                baseline = rig["frames"][0]["cells"][offset]
                M.AP.require(actual["corners"] == baseline["corners"], "Lamp state geometry moved")
                M.AP.require(actual["color"] == baseline["color"] or i in lamps,
                             "Non-lamp state color changed")
    M.AP.require(zoom["logical_viewport"] == [320, 240], "Projection viewport differs")
    for z, captured in zip(zoom["zooms"], zoom["frames"]):
        for raw, actual in zip(rig["frames"][0]["cells"], captured["cells"]):
            for p, q in zip(raw["corners"], actual["corners"]):
                expected = [160 + p[0] * z, 120 + p[1] * z]
                M.AP.require(all(math.isfinite(v) for v in q) and
                             max(abs(a-b) for a,b in zip(expected,q)) < .0001,
                             "Actual CameraView projection differs")
            M.AP.require(all(raw[k] == actual[k] for k in ("color", "light_response", "emission")),
                         "Projection changed cell color/material")
    audit = M.AP.read_json(N.DEFAULT / "native-audit.json")
    M.AP.require(audit["timeline_sample_times_verified"] and
                 all(f["maximum_corner_error_world"] == 0 for f in audit["frames"]),
                 "Existing native source audit differs")
    print(json.dumps({"input_hashes": len(record["input_sha256"]), "actual_body_frames": 3,
                      "actual_rig_frames": 3, "actual_camera_views": 3, "cells_each": 28549,
                      "transforms_each": 16, "decoded_evidence": "exact bytes verified",
                      "scope": "Offline source/native/flat software only; live Game/GPU/phone pending"}, indent=2))


if __name__ == "__main__":
    verify()
