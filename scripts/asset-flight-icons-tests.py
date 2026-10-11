"""Actual authored64-cell icon invariants and committed-output reproduction."""
from __future__ import annotations

import copy
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("flight_icons", ROOT / "scripts/asset-flight-icons.py")
icons = importlib.util.module_from_spec(spec)
spec.loader.exec_module(icons)
ASSETS = ROOT / "assets/ui/flight-icons"

COUNTS = {"status": 1291, "map": 2036, "settings": 1802, "save": 1503, "close": 1289,
          "resume": 1362, "restart": 1373, "title": 1591, "quit": 1322, "fullscreen": 1140}
# Hand-picked spatial landmarks describe the visible designs, not role-list output.
LANDMARKS = {
    "status": {"solid": [(32, 5), (8, 42)], "open": [(3, 32)], "emitter": [(26, 58), (37, 58)]},
    "map": {"solid": [(30, 18)], "open": [(21, 31), (41, 31)], "emitter": [(12, 37), (50, 24)]},
    "settings": {"solid": [(8, 31), (31, 9)], "open": [(31, 31)], "emitter": [(17, 31), (47, 31)]},
    "save": {"solid": [(30, 44)], "open": [(25, 15), (30, 39)], "emitter": [(46, 30)]},
    "close": {"solid": [(10, 10)], "open": [(31, 10), (10, 31)], "emitter": [(31, 31)]},
    "resume": {"solid": [(14, 31), (56, 32)], "open": [(50, 12)], "emitter": [(24, 25)]},
    "restart": {"solid": [(22, 15), (9, 38)], "open": [(31, 31), (30, 10)], "emitter": [(51, 30)]},
    "title": {"solid": [(31, 10), (18, 45)], "open": [(31, 50)], "emitter": [(31, 32)]},
    "quit": {"solid": [(50, 26)], "open": [(31, 39), (24, 10)], "emitter": [(31, 10)]},
    "fullscreen": {"solid": [(10, 10), (53, 10), (10, 53), (53, 53)], "open": [(31, 31)], "emitter": [(9, 13), (54, 13), (9, 50), (54, 50)]},
}


class FlightIcons(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.atlas = json.loads((ASSETS / "atlas.json").read_text(encoding="utf-8"))
        cls.temporary = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temporary.name)
        cls.regenerated = icons.author(cls.output)

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def test_regeneration_matches_entire_committed_pack_and_all_sources(self):
        for relative in ["atlas.json", "manifest.json", "provenance.json", "authoring-overview.png", "compact-readability.png"]:
            self.assertEqual((self.output / relative).read_bytes(), (ASSETS / relative).read_bytes(), relative)
        for kind in icons.KINDS:
            relative = "source/icon_" + kind + ".png"
            self.assertEqual((self.output / relative).read_bytes(), (ASSETS / relative).read_bytes(), relative)

    def test_source_png_decodes_to_every_original_indexed_rgba_cell(self):
        for kind in icons.KINDS:
            sprite = self.atlas["sprites"]["icon_" + kind]
            with Image.open(ASSETS / "source" / ("icon_" + kind + ".png")) as source:
                self.assertEqual(source.size, (64, 64))
                actual = list(source.convert("RGBA").get_flattened_data())
            expected = [tuple(c >> shift & 255 for shift in (0, 8, 16, 24)) for c in (self.atlas["palette"][i] for i in sprite["indices"])]
            self.assertEqual(actual, expected)

    def test_all_exact_conversion_policies_remain_lossless(self):
        metadata = self.atlas["asset_metadata"]["sprites"]
        for record in metadata.values():
            self.assertFalse(record["resized"])
            self.assertEqual(record["alpha_changed"], 0)
            self.assertEqual(record["removed_component_cells"], 0)
            self.assertEqual(record["transparent_rgb_canonicalized"], 0)

    def test_budgets_use_actual_occupied_cells_and_repeated_instances(self):
        measured = icons.validate(self.atlas)
        self.assertEqual({k.removeprefix("icon_"): v["occupied"] for k, v in measured.items()}, COUNTS)
        self.assertEqual(sum(COUNTS.values()), 14709)
        self.assertEqual(sum(COUNTS[k] for k in ("status", "map", "settings", "save")), 6632)
        self.assertLess(34594 + 15588 + 6632, 60000)
        # Reused instances still cost occupied cells each, never unique sprites only.
        self.assertEqual(COUNTS["map"] * 3, 6108)

    def test_authored_visual_landmarks_and_open_negative_space(self):
        for kind, roles in LANDMARKS.items():
            sprite = self.atlas["sprites"]["icon_" + kind]
            for role, points in roles.items():
                for x, y in points:
                    i = y * 64 + x
                    with self.subTest(icon=kind, role=role, point=(x, y)):
                        if role == "open":
                            self.assertEqual(sprite["indices"][i], 0)
                        else:
                            self.assertNotEqual(sprite["indices"][i], 0)
                        if role == "emitter":
                            self.assertIn(i, sprite["emitter_cells"])

    def test_every_silhouette_is_distinct_and_has_fine_not_doubled_cells(self):
        masks = []
        for kind in icons.KINDS:
            cells = self.atlas["sprites"]["icon_" + kind]["indices"]
            masks.append(bytes(bool(i) for i in cells))
            varied = sum(len({cells[(y+dy)*64+x+dx] for dx in (0, 1) for dy in (0, 1)}) > 1
                         for y in range(0, 64, 2) for x in range(0, 64, 2))
            self.assertGreater(varied, 170, kind)
        self.assertEqual(len(set(masks)), 10)

    def test_masks_are_occupied_sorted_and_restrict_animation_to_accents(self):
        icons.validate(self.atlas)
        for sprite in self.atlas["sprites"].values():
            accent, emitter = map(set, (sprite["accent_cells"], sprite["emitter_cells"]))
            self.assertLess(len(accent), sum(bool(i) for i in sprite["indices"]) // 3)
            self.assertTrue(emitter < accent)
            fixed_metal = [i for i, p in enumerate(sprite["indices"]) if p and p < icons.CYAN_DARK]
            self.assertFalse(set(fixed_metal) & accent)

    def test_declared_fourteen_pixel_samples_have_distinct_bounded_silhouettes(self):
        silhouettes = []
        for kind in icons.KINDS:
            with Image.open(ASSETS / "source" / ("icon_" + kind + ".png")) as image:
                sample = image.resize((14, 14), Image.Resampling.NEAREST)
                alpha = bytes(int(a != 0) for r, g, b, a in sample.get_flattened_data())
            self.assertGreater(sum(alpha), 45, kind)
            self.assertLess(sum(alpha), 150, kind)
            silhouettes.append(alpha)
        self.assertEqual(len(set(silhouettes)), 10)

    def test_palette_and_mask_fail_closed(self):
        cases = []
        atlas = copy.deepcopy(self.atlas)
        atlas["palette"][1] &= 0x00ffffff
        cases.append(atlas)
        atlas = copy.deepcopy(self.atlas)
        atlas["sprites"]["icon_map"]["emitter_cells"].append(4096)
        cases.append(atlas)
        atlas = copy.deepcopy(self.atlas)
        atlas["sprites"]["icon_save"]["accent_cells"].append(25*64+15)
        cases.append(atlas)
        atlas = copy.deepcopy(self.atlas)
        atlas["sprites"]["icon_close"]["indices"][0] = 1
        cases.append(atlas)
        atlas = copy.deepcopy(self.atlas)
        atlas["version"] = True
        cases.append(atlas)
        for value in cases:
            with self.assertRaises(ValueError):
                icons.validate(value)

    def test_metadata_pins_exact_sources_and_manifest(self):
        records = self.atlas["asset_metadata"]["sprites"]
        self.assertEqual(self.atlas["asset_metadata"]["manifest_sha256"], icons.digest(ASSETS / "manifest.json"))
        for name, record in records.items():
            self.assertEqual(record["source_sha256"], icons.digest(ASSETS / "source" / (name + ".png")))

    def test_stored_png_encoder_is_exact_rgba_for_multiple_block_sizes(self):
        for size in ((1, 1), (64, 64), (129, 129)):
            source = Image.new("RGBA", size)
            source.putdata([(i % 256, i // 256 % 256, 17, 0 if i % 7 == 0 else 255) for i in range(size[0]*size[1])])
            path = self.output / (str(size[0]) + ".png")
            icons.write_png(path, source)
            original = path.read_bytes()
            with Image.open(path) as image:
                self.assertEqual(image.convert("RGBA").tobytes(), source.tobytes())
            icons.write_png(path, source)
            self.assertEqual(original, path.read_bytes())


if __name__ == "__main__":
    unittest.main()
