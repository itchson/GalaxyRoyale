"""Scoped source-art proofs; these do not execute the game or its renderer."""
from __future__ import annotations

import copy
import importlib.util
import json
import struct
import tempfile
import unittest
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("raicho_fine", ROOT / "scripts/asset-raicho-fine.py")
art = importlib.util.module_from_spec(spec)
spec.loader.exec_module(art)
ASSETS = ROOT / "assets/pixels/raicho-fine-detail"
COUNTS = {"pan_hikari": (2648, 1205, 31), "renko_hibana": (2752, 1176, 27),
          "hugo_mamori": (3736, 1740, 31)}


def rehash(atlas):
    """Keep generic pack metadata current so adversarial tests reach role rules."""
    for name, sprite in atlas["sprites"].items():
        atlas["asset_metadata"]["sprites"][name]["metrics"] = art.PACK.topology(
            sprite["indices"], sprite["width"], sprite["height"])
    for group in atlas["asset_metadata"]["groups"]:
        group["metrics"] = art.PACK.group_metrics(group, atlas["sprites"], atlas["asset_metadata"]["budgets"])
    core = {k: atlas[k] for k in ("version", "palette", "sprites")}
    atlas["asset_metadata"]["cell_content_sha256"] = art.hashlib.sha256(
        json.dumps(core, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def f32(n):
    return struct.unpack("f", struct.pack("f", n))[0]


def reviewed_source_formula(variant, x, y):
    """Independent scalar transcription of the reviewed Rust f32 silhouette.

    This checks the integer authoring reconstruction, not native execution.
    """
    dx = abs(f32(f32(x) - 23.5))
    if variant == 0:
        width = f32(2.0 + f32(f32(y - 3) * .25)) if y < 18 else f32(9.5 - f32(abs(f32(y - 25)) * f32(.30)))
        return ((3 <= y <= 37 and dx <= width) or (24 <= y <= 40 and 9 <= dx <= 14)
                or (29 <= y <= 32 and dx <= 15) or (38 <= y <= 44 and 3 <= dx <= 6))
    if variant == 1:
        width = f32(1.5 + f32(f32(y) * f32(.17))) if y < 25 else f32(6.5 - f32(f32(y - 25) * f32(.20)))
        return ((2 <= y <= 43 and dx <= width) or
                (18 <= y <= 37 and dx > 7 and dx <= f32(16.0 - f32(abs(f32(y - 27)) * f32(.65))))
                or (12 <= y <= 32 and 11 <= dx <= 13) or (28 <= y <= 30 and dx <= 16)
                or (36 <= y <= 43 and 3 <= dx <= 7))
    width = f32(3.5 + f32(f32(y - 5) * .5)) if y < 16 else 12.5
    return ((5 <= y <= 39 and dx <= width) or (16 <= y <= 36 and 14 <= dx <= 19)
            or (22 <= y <= 27 and dx <= 20.5) or (38 <= y <= 43 and 5 <= dx <= 9))


class RaichoFine(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.atlas = json.loads((ASSETS / "atlas.json").read_text(encoding="utf-8"))
        cls.temporary = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temporary.name)
        cls.regenerated = art.author(cls.output)

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def test_all_committed_outputs_and_png_bytes_regenerate_exactly(self):
        names = [kind + suffix for kind in art.KINDS for suffix in ("", "_fine")]
        files = ["atlas.json", "manifest.json", "provenance.json", "authoring-overview.png", "compact-review.png"]
        files += ["source/" + name + ".png" for name in names]
        for relative in files:
            self.assertEqual((self.output / relative).read_bytes(), (ASSETS / relative).read_bytes(), relative)

    def test_each_source_cell_matches_palette_index_and_hard_alpha(self):
        self.assertEqual(len(self.atlas["palette"]), 32)
        self.assertEqual(self.atlas["palette"][0], 0)
        for name, sprite in self.atlas["sprites"].items():
            with Image.open(ASSETS / "source" / (name + ".png")) as source:
                self.assertEqual(source.size, (96, 96))
                pixels = list(source.convert("RGBA").get_flattened_data())
            rgba = [tuple(self.atlas["palette"][i] >> shift & 255 for shift in (0, 8, 16, 24)) for i in sprite["indices"]]
            self.assertEqual(pixels, rgba, name)
            self.assertLessEqual({c[3] for c in pixels}, {0, 255})

    def test_exact_indexed_png_roundtrip_preserves_palette_and_every_cell(self):
        with tempfile.TemporaryDirectory() as directory:
            manifest = art.PACK.unpack(self.atlas, Path(directory) / "roundtrip")
            packed = art.PACK.pack(manifest)
        self.assertEqual(packed["palette"], self.atlas["palette"])
        self.assertEqual(packed["raicho_detail"], self.atlas["raicho_detail"])
        for name, source in self.atlas["sprites"].items():
            self.assertEqual({k: source[k] for k in ("width", "height", "indices")}, packed["sprites"][name])
        # Generic indexed PNG roundtrip does not promise sprite-extension masks.
        self.assertNotIn("material_cells", packed["sprites"]["pan_hikari"])

    def test_png_import_has_no_resize_quantization_alpha_or_component_loss(self):
        for record in self.atlas["asset_metadata"]["sprites"].values():
            self.assertFalse(record["resized"])
            self.assertEqual(record["alpha_changed"], 0)
            self.assertEqual(record["removed_component_cells"], 0)
            self.assertEqual(record["transparent_rgb_canonicalized"], 0)
            self.assertEqual(record["quantized_color_cells"], 0)

    def test_reference_integer_geometry_matches_reviewed_f32_fields(self):
        for variant, count in enumerate((662, 688, 1040)):
            expected = [reviewed_source_formula(variant, x, y) for y in range(48) for x in range(48)]
            self.assertEqual(art.reference_mask(variant), expected)
            self.assertEqual(sum(expected), count)

    def test_intact_bodies_are_connected_and_only_renko_has_authored_apertures(self):
        for n, kind in enumerate(art.KINDS):
            full = self.atlas["sprites"][kind]["indices"]
            result = art.PACK.topology(full, 96, 96)
            self.assertEqual(result["components"], 1, kind)
            self.assertEqual(result["hole_cells"], 96 if n == 1 else 0, kind)
            coarse = art.PACK.topology(art.reference_mask(n), 48, 48)
            self.assertEqual(result["bounds"], [b * 2 for b in coarse["bounds"]])
        # The existing RENKO cutout is preserved, not a crater/damage mask.
        self.assertEqual(self.atlas["sprites"]["renko_hibana"]["indices"][52*96+34], 0)

    def test_full_and_overlay_surfaces_stay_inside_old_collision_cells(self):
        for n, kind in enumerate(art.KINDS):
            coarse = art.reference_mask(n)
            full = self.atlas["sprites"][kind]
            overlay = self.atlas["sprites"][kind + "_fine"]
            for i, cell in enumerate(full["indices"]):
                if cell:
                    self.assertTrue(coarse[(i // 96 // 2) * 48 + i % 96 // 2])
            self.assertEqual({i for i, cell in enumerate(overlay["indices"]) if cell}, set(full["fine_cells"]))
            for i in full["fine_cells"]:
                self.assertEqual(overlay["indices"][i], full["indices"][i])
        # HUGO is a declared presentation inset, not a full duplicate collision body.
        self.assertLess(COUNTS["hugo_mamori"][0], 4 * 1040)

    def test_damage_mapping_masks_entire_fine_assembly_for_removed_coarse_cell(self):
        for kind in art.KINDS:
            cells = self.atlas["sprites"][kind + "_fine"]["indices"]
            # A real future host must use the surviving48 body, never repair it.
            destroyed = {(x, y) for x in range(21, 27) for y in range(25, 30)}
            surviving = [cell if (i % 96 // 2, i // 96 // 2) not in destroyed else 0 for i, cell in enumerate(cells)]
            self.assertLess(sum(bool(i) for i in surviving), sum(bool(i) for i in cells))
            for i, original in enumerate(cells):
                self.assertEqual(surviving[i], 0 if (i % 96 // 2, i // 96 // 2) in destroyed else original)

    def test_actual_material_landmarks_are_structures_not_noise(self):
        shared = {(47, 30): (9, "glass"), (47, 54): (16, "reactor"),
                  (35, 58): (6, "metal"), (35, 59): (2, "recess")}
        individual = {
            "pan_hikari": {(23, 54): (20, "main_accent"), (20, 53): (8, "fastener"), (19, 64): (6, "metal"), (39, 82): (30, "exhaust")},
            "renko_hibana": {(22, 49): (24, "main_accent"), (23, 33): (8, "metal"), (32, 48): (8, "fastener"), (39, 82): (30, "exhaust")},
            "hugo_mamori": {(13, 41): (28, "main_accent"), (27, 44): (8, "fastener"), (27, 52): (2, "recess"), (32, 82): (30, "exhaust")}}
        for kind in art.KINDS:
            sprite = self.atlas["sprites"][kind]
            for (x, y), (palette, role) in (shared | individual[kind]).items():
                i = y * 96 + x
                self.assertEqual(sprite["indices"][i], palette, (kind, x, y))
                self.assertIn(i, sprite["material_cells"][role])
            self.assertEqual(sprite["indices"][0], 0)

    def test_three_silhouettes_and_fine_color_clusters_are_distinct(self):
        masks = []
        for kind in art.KINDS:
            cells = self.atlas["sprites"][kind]["indices"]
            masks.append(bytes(bool(i) for i in cells))
            varied = sum(len({cells[(y+dy)*96+x+dx] for dx in (0, 1) for dy in (0, 1)}) > 1
                         for y in range(0, 96, 2) for x in range(0, 96, 2))
            self.assertGreater(varied, 450, kind)
        self.assertEqual(len(set(masks)), 3)

    def test_corporate_roles_cannot_tint_glass_or_neutral_metal(self):
        art.validate(self.atlas)
        for n, kind in enumerate(art.KINDS):
            sprite = self.atlas["sprites"][kind]
            masks = sprite["material_cells"]
            accent = set(masks["main_accent"])
            signal = set(sprite["signal_cells"])
            fixed = set(masks["metal"]) | set(masks["glass"]) | set(masks["recess"]) | set(masks["fastener"])
            self.assertFalse(accent & fixed)
            self.assertFalse(signal & fixed)
            self.assertLessEqual(signal, accent | set(masks["reactor"]) | set(masks["exhaust"]))
            self.assertTrue(all(17 + n*4 <= sprite["indices"][i] < 21 + n*4 for i in accent))
            self.assertGreater(len(accent), 75)

    def test_exact_budgets_charge_full_bodies_and_each_repeated_instance(self):
        result = art.validate(self.atlas)
        for kind, (full, fine, signals) in COUNTS.items():
            self.assertEqual(result[kind]["occupied"], full)
            self.assertEqual(result[kind]["signals"], signals)
            self.assertEqual(result[kind + "_fine"]["occupied"], fine)
            self.assertLessEqual(full, 4000)
            self.assertLessEqual(fine, 1800)
        limits = {**self.atlas["asset_metadata"]["budgets"], "group_cells": 60000}
        group = {"name": "reused", "parts": [{"name": "p" + str(n), "sprite": "hugo_mamori"} for n in range(3)]}
        self.assertEqual(art.PACK.group_metrics(group, self.atlas["sprites"], limits)["occupied_cells"], 11208)
        with self.assertRaisesRegex(ValueError, "budget"):
            art.PACK.group_metrics(group, self.atlas["sprites"], {**limits, "group_cells": 11207})

    def test_relative_half_cells_and_full_affine_corners_preserve_mapping(self):
        parent = [[37, -19], [-1.2, .25], [.6, 1.7]]  # Reflection, shear, nonuniform scale.
        child = [[0, 0], [.5, 0], [0, .5]]
        composed = art.PACK.compose(parent, child)
        for kind in art.KINDS:
            coords = []
            for i, index in enumerate(self.atlas["sprites"][kind]["indices"]):
                if not index:
                    continue
                x, y = i % 96, i // 96
                for dx, dy in ((0, 0), (1, 0), (0, 1), (1, 1)):
                    local = [(x+dx) / 2 - 24, (y+dy) / 2 - 24]
                    self.assertTrue(x//2-24 <= local[0] <= x//2-23)
                    self.assertTrue(y//2-24 <= local[1] <= y//2-23)
                    expected = art.PACK.point(parent, local)
                    actual = art.PACK.point(composed, [x+dx-48, y+dy-48])
                    self.assertEqual(actual, expected)
                    coords.append(actual)
            group = {"name": "reflected", "parts": [{"name": "root", "sprite": kind, "transform": {"position": composed[0], "basis_x": composed[1], "basis_y": composed[2]}},
                     {"name": "fine", "sprite": kind + "_fine", "parent": "root"}]}
            # This deliberately repeated overlay costs extra, even at the same transform.
            metrics = art.PACK.group_metrics(group, self.atlas["sprites"], {**art.PACK.LIMITS, "group_cells": 60000})
            self.assertEqual(metrics["transforms"], 2)
            self.assertEqual(metrics["occupied_cells"], COUNTS[kind][0] + COUNTS[kind][1])
            self.assertEqual(metrics["occupied_bounds"], [min(p[0] for p in coords), min(p[1] for p in coords), max(p[0] for p in coords), max(p[1] for p in coords)])

    def test_palette_signal_mask_and_overlay_corruption_fail_closed(self):
        changes = []
        a = copy.deepcopy(self.atlas); a["palette"][7] &= 0x00ffffff; changes.append(a)
        a = copy.deepcopy(self.atlas); a["sprites"]["pan_hikari"]["signal_cells"].append(96*96); rehash(a); changes.append(a)
        a = copy.deepcopy(self.atlas); a["sprites"]["renko_hibana"]["material_cells"]["glass"].append(22+49*96); rehash(a); changes.append(a)
        a = copy.deepcopy(self.atlas); a["sprites"]["hugo_mamori_fine"]["indices"][32+82*96] = 29; rehash(a); changes.append(a)
        a = copy.deepcopy(self.atlas); a["sprites"]["pan_hikari"]["indices"][23+54*96] = 24; rehash(a); changes.append(a)
        a = copy.deepcopy(self.atlas); a["version"] = True; changes.append(a)
        for i, invalid in enumerate(changes):
            with self.subTest(case=i), self.assertRaises(ValueError):
                art.validate(invalid)

    def test_removed_solid_cell_and_added_external_cell_fail_closed(self):
        for x, y, value in ((47, 54, 0), (0, 0, 5)):
            invalid = copy.deepcopy(self.atlas)
            invalid["sprites"]["pan_hikari"]["indices"][y*96+x] = value
            with self.assertRaises(ValueError):
                rehash(invalid)
                art.validate(invalid)

    def test_valid_occupied_role_reassignment_and_missing_overlay_signal_reject(self):
        invalid = copy.deepcopy(self.atlas)
        sprite = invalid["sprites"]["pan_hikari"]
        glass = 47 + 30*96  # Fixed blue glass must not acquire manufacturer tint.
        sprite["material_cells"]["glass"].remove(glass)
        sprite["material_cells"]["main_accent"].append(glass)
        sprite["material_cells"]["main_accent"].sort()
        rehash(invalid)
        with self.assertRaisesRegex(ValueError, "another role"):
            art.validate(invalid)
        invalid = copy.deepcopy(self.atlas)
        invalid["sprites"]["hugo_mamori_fine"]["signal_cells"].remove(32+82*96)
        rehash(invalid)
        with self.assertRaisesRegex(ValueError, "must match"):
            art.validate(invalid)

    def test_software_samples_have_distinct_silhouettes_without_native_claims(self):
        for size in (29, 48, 77):
            masks = []
            for kind in art.KINDS:
                with Image.open(ASSETS / "source" / (kind + ".png")) as source:
                    sampled = source.resize((size, size), Image.Resampling.NEAREST)
                    masks.append(bytes(int(a != 0) for r, g, b, a in sampled.get_flattened_data()))
            self.assertEqual(len(set(masks)), 3, size)
            self.assertGreater(min(map(sum, masks)), size * size // 5)

    def test_code_fingerprints_ignore_checkout_line_endings_only(self):
        with tempfile.TemporaryDirectory() as directory:
            lf, crlf, changed = (Path(directory) / name for name in ("lf.py", "crlf.py", "changed.py"))
            lf.write_bytes(b"x = 1\nprint(x)\n")
            crlf.write_bytes(b"x = 1\r\nprint(x)\r\n")
            changed.write_bytes(b"x = 2\nprint(x)\n")
            self.assertEqual(art.code_digest(lf), art.code_digest(crlf))
            self.assertNotEqual(art.code_digest(lf), art.code_digest(changed))
            self.assertNotEqual(art.digest(lf), art.digest(crlf))


if __name__ == "__main__":
    unittest.main(verbosity=2)
