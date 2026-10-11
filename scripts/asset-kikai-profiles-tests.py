"""Real pack/content checks; no browser, gameplay or subjective acceptance claim."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("profiles", ROOT / "scripts/asset-kikai-profiles.py")
profiles = importlib.util.module_from_spec(spec)
spec.loader.exec_module(profiles)
SOURCE = ROOT / "assets/concepts/kikai/profiles"


class ProfileArtTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "authored"
        cls.atlas, cls.summary = profiles.author(SOURCE, cls.output)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_generated_sources_are_pinned_and_original_guide_unchanged(self):
        metadata = json.loads((SOURCE / "generation.json").read_text(encoding="utf-8"))
        for kind in ("doro", "talo"):
            self.assertEqual(profiles.sha(SOURCE / f"{kind}-reference.png"), metadata["portraits"][kind]["sha256"])
            self.assertIn("prompt", metadata["portraits"][kind])
            self.assertIsNone(metadata["portraits"][kind]["model"])
        self.assertEqual(profiles.sha(ROOT / metadata["reference"]), metadata["reference_sha256"])

    def test_authored_sources_import_and_indexed_png_roundtrip_exactly(self):
        for kind, atlas in self.atlas["profiles"].items():
            packed = profiles.pack.pack(self.output / kind / "manifest.json")
            for key in ("version", "palette", "sprites", "portrait_roles"):
                self.assertEqual(packed[key], atlas[key])
            manifest = profiles.pack.unpack(atlas, Path(self.temp.name) / f"roundtrip-{kind}")
            restored = profiles.pack.pack(manifest)
            for key in ("version", "palette", "sprites", "portrait_roles"):
                self.assertEqual(restored[key], atlas[key])

    def test_conversion_is_explicit_lossy_fit_not_stretch_or_component_filter(self):
        for kind, atlas in self.atlas["profiles"].items():
            conversion = atlas["asset_metadata"]["provenance"]["conversion"]
            self.assertEqual(conversion["source_dimensions"], [1086, 1448])
            self.assertEqual(conversion["crop"], [0, 0, 1086, 1448])
            self.assertEqual(conversion["alpha_threshold"], 128)
            self.assertEqual(conversion["output_dimensions"], [96, 128])
            self.assertEqual(conversion["component_policy"], "all; no connected-component removal")
            self.assertFalse(conversion["dither"])
            self.assertGreater(conversion["changed_occupied_color_cells"], 5000)
            for sprite in atlas["asset_metadata"]["sprites"].values():
                self.assertEqual(sprite["removed_component_cells"], 0)
                self.assertEqual(sprite["alpha_changed"], 0)
                self.assertFalse(sprite["resized"])
            width, height = conversion["fitted_dimensions"]
            x0, y0, x1, y1 = conversion["tight_bbox"]
            self.assertLess(abs(width / height - (x1 - x0) / (y1 - y0)), .01)

    def test_all_twenty_poses_keep_exact_opaque_mask_and_disjoint_layer_bounds(self):
        for kind, atlas in self.atlas["profiles"].items():
            count = profiles.validate_profile(atlas)
            self.assertEqual(count, self.summary[kind]["composed_occupied_cells"])
            sprites = atlas["sprites"]
            expected = None
            for eye in profiles.EYE_STATES:
                for mouth in profiles.MOUTH_STATES:
                    combined = []
                    for a, b, c in zip(sprites["body"]["indices"], sprites[f"eyes_{eye}"]["indices"], sprites[f"mouth_{mouth}"]["indices"]):
                        self.assertLessEqual(sum(i != 0 for i in (a, b, c)), 1)
                        combined.append(a != 0 or b != 0 or c != 0)
                    if expected is None:
                        expected = combined
                    self.assertEqual(combined, expected)

    def test_doro_talo_and_dora_have_genuinely_different_silhouettes(self):
        dora = json.loads((ROOT / "assets/ui/kikai/atlas.json").read_text(encoding="utf-8"))
        silhouettes = [tuple(c != 0 for c in dora["sprites"]["dora_body"]["indices"])]
        silhouettes.extend(tuple(c != 0 for c in pack["sprites"]["body"]["indices"]) for pack in self.atlas["profiles"].values())
        self.assertEqual(len(set(silhouettes)), 3)
        a, b = silhouettes[1:]
        self.assertGreater(sum(x != y for x, y in zip(a, b)), 1000)
        # Hair occupancy reflects the full spiky crown versus shorter crew cut.
        self.assertGreater(len(self.atlas["profiles"]["doro"]["portrait_roles"]["hair_cells"]),
                           len(self.atlas["profiles"]["talo"]["portrait_roles"]["hair_cells"]) + 500)

    def test_color_roles_live_only_on_body_are_sorted_disjoint_and_genuine(self):
        for atlas in self.atlas["profiles"].values():
            body, palette, roles = atlas["sprites"]["body"]["indices"], atlas["palette"], atlas["portrait_roles"]
            hair, horns, signals = [set(roles[k]) for k in ("hair_cells", "horn_cells", "signal_cells")]
            self.assertFalse(hair & horns)
            self.assertTrue(signals <= horns)
            self.assertGreater(len(hair), 800)
            self.assertGreater(len(horns), 200)
            for i in hair:
                r, g, b, a = profiles.pack.rgba(palette[body[i]])
                self.assertGreater(g, r + 3)
                self.assertGreater(g, b + 2)
                self.assertLess(i // 96, 61, "Neck cables must not follow hair color")
            for i in horns:
                r, g, b, a = profiles.pack.rgba(palette[body[i]])
                self.assertGreater(r, g + 20)
                self.assertGreater(r, b + 10)
                self.assertLess(i // 96, 40)

    def test_states_are_actual_distinct_feature_art_with_authored_gold_eyes(self):
        for atlas in self.atlas["profiles"].values():
            sprites = atlas["sprites"]
            self.assertEqual(len({tuple(sprites[f"eyes_{s}"]["indices"]) for s in profiles.EYE_STATES}), 5)
            self.assertEqual(len({tuple(sprites[f"mouth_{s}"]["indices"]) for s in profiles.MOUTH_STATES}), 4)
            colors = {atlas["palette"][i] for i in sprites["eyes_neutral"]["indices"] if i}
            self.assertIn(profiles.pack.color(profiles.GOLD), colors)
            self.assertIn(profiles.pack.color(profiles.GOLD_LIGHT), colors)

    def test_binary_alpha_palette_and_12288_cell_work_limit(self):
        for atlas in self.atlas["profiles"].values():
            self.assertLessEqual(len(atlas["palette"]), 64)
            self.assertEqual(atlas["palette"][0], 0)
            self.assertTrue(all(c >> 24 == 255 for c in atlas["palette"][1:]))
            self.assertLessEqual(profiles.validate_profile(atlas), 12288)

    def test_reproduction_is_identical_to_checked_in_cells_and_roles(self):
        checked = json.loads((ROOT / "assets/ui/kikai/profiles/atlas.json").read_text(encoding="utf-8"))
        for kind, atlas in self.atlas["profiles"].items():
            for key in ("version", "palette", "sprites", "portrait_roles"):
                self.assertEqual(atlas[key], checked["profiles"][kind][key])

    def test_invalid_source_shape_refuses_implicit_recrop(self):
        path = Path(self.temp.name) / "wrong.png"
        Image.new("RGBA", (30, 40)).save(path)
        with self.assertRaisesRegex(ValueError, "recrop/refit explicitly"):
            profiles.prepare_reference(path)

    def test_validator_rejects_bad_cells_palette_overlap_and_role_contract(self):
        original = self.atlas["profiles"]["doro"]
        for mutate in [
            lambda a: a["palette"].__setitem__(0, 1),
            lambda a: a["palette"].__setitem__(1, 255),
            lambda a: a["sprites"]["body"]["indices"].__setitem__(0, 64),
            lambda a: a["sprites"]["body"].__setitem__("width", 95),
            lambda a: a["portrait_roles"]["hair_cells"].append(a["portrait_roles"]["hair_cells"][0]),
            lambda a: a["portrait_roles"]["signal_cells"].append(a["portrait_roles"]["hair_cells"][0]),
            lambda a: a["portrait_roles"]["horn_cells"].append(a["portrait_roles"]["hair_cells"][0]),
            lambda a: a["sprites"]["eyes_blink"]["indices"].__setitem__(0, 1),
            lambda a: a["sprites"]["mouth_rest"]["indices"].__setitem__(next(i for i, c in enumerate(a["sprites"]["body"]["indices"]) if c), 1),
        ]:
            modified = copy.deepcopy(original)
            mutate(modified)
            with self.assertRaises(ValueError):
                profiles.validate_profile(modified)


if __name__ == "__main__":
    unittest.main(verbosity=2)
