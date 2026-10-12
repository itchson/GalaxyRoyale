"""Focused exact-mask/palette/pipeline/atlas-span proofs for Bastion40."""
from pathlib import Path
from unittest.mock import patch
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("bastion_detail_author", ROOT / "scripts/asset-bastion-detail.py")
B = importlib.util.module_from_spec(spec)
spec.loader.exec_module(B)


class BastionDetailProof(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.baseline = B.AP.read_json(B.DEFAULT / "baseline.json")
        cls.old = cls.baseline["sprites"]["bastion"]["indices"]
        cls.new = B.authored_indices(cls.baseline)
        cls.raw = subprocess.check_output(["git", "show", f'{cls.baseline["reference_commit"]}:assets/pixels/atlas.json'], cwd=ROOT)
        cls.temporary = ROOT / "work/bastion-material-refinement/test-temp"
        cls.temporary.mkdir(parents=True, exist_ok=True)

    def test_baseline_is_exact_recorded_git_sprite_and_palette(self):
        reference = json.loads(self.raw)
        self.assertEqual(self.baseline["version"], reference["version"])
        self.assertEqual(self.baseline["palette"], reference["palette"])
        self.assertEqual(self.baseline["sprites"], {"bastion": reference["sprites"]["bastion"]})
        self.assertEqual(B.digest(self.raw), self.baseline["source_atlas_sha256_lf"])

    def test_exact938mask40grid_palette_and_83_original_signal_sites(self):
        packed = B.AP.read_json(B.DEFAULT / "pack.json")
        sprite = packed["sprites"]["bastion"]
        self.assertEqual((sprite["width"], sprite["height"]), (40, 40))
        self.assertEqual(sprite["indices"], self.new)
        self.assertEqual(packed["palette"], self.baseline["palette"])
        self.assertEqual(sum(bool(i) for i in self.new), 938)
        self.assertEqual([bool(i) for i in self.old], [bool(i) for i in self.new])
        self.assertEqual(sum(i in B.SIGNALS for i in self.old), 83)
        for signal in (4, 6, 33):
            self.assertEqual([i for i, color in enumerate(self.old) if color == signal],
                             [i for i, color in enumerate(self.new) if color == signal])
        # Runtime animation matches palette colours, so test colour roles too.
        lights = {self.baseline["palette"][index] for index in B.SIGNALS}
        self.assertEqual([self.baseline["palette"][i] in lights for i in self.old],
                         [self.baseline["palette"][i] in lights for i in self.new])

    def test_broad_original_graded_orange_shoulder_fields_are_retained(self):
        cells = [y * 40 + x for y in range(12, 23) for x in range(40)
                 if x <= 15 or x >= 24]
        self.assertEqual([self.old[i] for i in cells], [self.new[i] for i in cells])
        self.assertTrue(any(a != b for a, b in zip(self.old, self.new)))

    def test_surgical_update_retains_all_other_actual_atlas_bytes(self):
        match = B.PATTERN.search(self.raw)
        self.assertIsNotNone(match)
        changed = B.surgical_atlas(self.raw, self.baseline, self.new)
        after = B.PATTERN.search(changed)
        self.assertEqual(self.raw[:match.start(2)], changed[:after.start(2)])
        self.assertEqual(self.raw[match.end(2):], changed[after.end(2):])
        self.assertEqual(B.surgical_atlas(changed, self.baseline, self.new), changed)
        original, candidate = json.loads(self.raw), json.loads(changed)
        self.assertEqual(original["palette"], candidate["palette"])
        for name in original["sprites"]:
            if name != "bastion":
                self.assertEqual(original["sprites"][name], candidate["sprites"][name])

    def test_unreviewed_other_atlas_or_bastion_changes_refuse_overwrite(self):
        with self.assertRaisesRegex(B.AP.AssetError, "Non-Bastion atlas bytes changed"):
            B.surgical_atlas(self.raw.replace(b'"version":1', b'"version":2', 1), self.baseline, self.new)
        match = B.PATTERN.search(self.raw)
        old = self.old.copy()
        old[0] = 1
        altered = self.raw[:match.start(2)] + json.dumps(old, separators=(",", ":")).encode()[1:-1] + self.raw[match.end(2):]
        with self.assertRaisesRegex(B.AP.AssetError, "Unexpected Bastion art"):
            B.surgical_atlas(altered, self.baseline, self.new)

    def test_actual_indexed_png_and_pack_reproduce_all_owned_outputs(self):
        with tempfile.TemporaryDirectory(dir=self.temporary) as folder:
            output = Path(folder)
            self.assertTrue(output.resolve().is_relative_to(self.temporary.resolve()))
            rebuilt, _ = B.author(output)
            self.assertEqual(rebuilt["sprites"]["bastion"]["indices"], self.new)
            for name in ("source.png", "manifest.json", "pack.json", "provenance.json"):
                actual, expected = (output / name).read_bytes(), (B.DEFAULT / name).read_bytes()
                if name.endswith(".json"):
                    actual, expected = actual.replace(b"\r\n", b"\n"), expected.replace(b"\r\n", b"\n")
                self.assertEqual(actual, expected, name)

    def test_crlf_reference_reproduces_same_canonical_outputs(self):
        with tempfile.TemporaryDirectory(dir=self.temporary) as folder:
            task = Path(folder)
            reference = task / "reference"
            reference.mkdir()
            original = B.DEFAULT
            raw = (original / "baseline.json").read_bytes().replace(b"\r\n", b"\n")
            (reference / "baseline.json").write_bytes(raw.replace(b"\n", b"\r\n"))
            with patch.object(B, "DEFAULT", reference):
                B.author(task / "output")
            for name in ("source.png", "manifest.json", "pack.json", "provenance.json"):
                actual, expected = (task / "output" / name).read_bytes(), (original / name).read_bytes()
                if name.endswith(".json"):
                    expected = expected.replace(b"\r\n", b"\n")
                self.assertEqual(actual, expected, name)

    def test_shipped_atlas_is_exact_candidate_with_all_original_other_bytes(self):
        actual = (ROOT / "assets/pixels/atlas.json").read_bytes()
        expected = B.surgical_atlas(self.raw, self.baseline, self.new)
        self.assertEqual(actual.replace(b"\r\n", b"\n"), expected)


if __name__ == "__main__":
    unittest.main()
