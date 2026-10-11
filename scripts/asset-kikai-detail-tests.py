"""Real192-grid source/import checks; no native, GPU or anatomy certification."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("detail", ROOT / "scripts/asset-kikai-detail.py")
detail = importlib.util.module_from_spec(spec)
spec.loader.exec_module(detail)
KEYS = ("version", "palette", "sprites", "portrait_roles")


class DetailArtTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "authored"
        cls.atlas, cls.summary = detail.author(cls.output)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_sources_pin_generated_edits_and_full_resolution_original_dora(self):
        metadata, records = detail.source_records()
        self.assertEqual(set(records), {"dora", "doro", "talo"})
        self.assertEqual(metadata["mode"], "built-in edit")
        self.assertIsNone(metadata["model"])
        for kind, record in records.items():
            self.assertEqual(detail.sha(ROOT / record["path"]), record["sha256"])
            if kind != "dora":
                self.assertIn("FLAT", record["prompt"])
                self.assertEqual(detail.sha(ROOT / record["input"]), record["input_sha256"])
        self.assertEqual(records["dora"]["path"], "assets/concepts/kikai/dora-reference-expression-sheet.png")
        self.assertEqual(records["dora"]["sha256"], "b558e6572e9d065319ff7957fcdc7faadb1bc38f6ae7cdc39d3d826cc5f83742")

    def test_committed_atlas_exactly_matches_full_source_regeneration(self):
        checked_path = ROOT / "assets/ui/kikai/detail/atlas.json"
        checked = json.loads(checked_path.read_text(encoding="utf-8"))
        self.assertEqual(checked_path.read_bytes(),
                         (json.dumps(checked, ensure_ascii=False, separators=(",", ":")) + "\n").encode("utf-8"))
        self.assertEqual(checked_path.read_bytes(), (self.output / "atlas.json").read_bytes())
        self.assertLess(checked_path.stat().st_size, 4_000_000, "Runtime payload must not include pretty-array padding")
        for kind in ("dora", "doro", "talo"):
            committed_manifest = ROOT / "assets/ui/kikai/detail" / kind / "manifest.json"
            self.assertNotIn(b"\r", committed_manifest.read_bytes())
            self.assertEqual(detail.sha(committed_manifest), checked["profiles"][kind]["asset_metadata"]["manifest_sha256"])
            self.assertEqual(committed_manifest.read_bytes(), (self.output / kind / "manifest.json").read_bytes())
        self.assertEqual(checked["version"], 1)
        self.assertEqual(set(checked["profiles"]), {"dora", "doro", "talo"})
        for kind, atlas in self.atlas["profiles"].items():
            for key in KEYS:
                self.assertEqual(atlas[key], checked["profiles"][kind][key], f"Changed {kind}/{key}")
            self.assertEqual(atlas["asset_metadata"], checked["profiles"][kind]["asset_metadata"])
        evidence = json.loads((ROOT / "assets/ui/kikai/detail/authoring-evidence.json").read_text(encoding="utf-8"))
        self.assertEqual(evidence["profiles"], self.summary)
        for filename in ("authoring-overview.png", "authoring-contact-sheet.png", "corporate-role-review.png"):
            self.assertEqual(detail.sha(self.output / filename), detail.sha(ROOT / "assets/ui/kikai/detail" / filename))

    def test_authored_png_import_and_indexed_png_roundtrip_preserve_every_cell(self):
        for kind, atlas in self.atlas["profiles"].items():
            imported = detail.pack.pack(self.output / kind / "manifest.json")
            for key in KEYS:
                self.assertEqual(imported[key], atlas[key])
            manifest = detail.pack.unpack(atlas, Path(self.temp.name) / f"roundtrip-{kind}")
            roundtrip = detail.pack.pack(manifest)
            for key in KEYS:
                self.assertEqual(roundtrip[key], atlas[key])

    def test_conversion_loss_is_explicit_and_never_an_upscaled96_pack(self):
        for kind, atlas in self.atlas["profiles"].items():
            conversion = atlas["asset_metadata"]["provenance"]["conversion"]
            expected = [1881, 836] if kind == "dora" else [1086, 1448]
            self.assertEqual(conversion["source_dimensions"], expected)
            self.assertEqual(conversion["output_dimensions"], [192, 256])
            self.assertFalse(conversion["source_is_old_96cell_pack"])
            self.assertEqual(conversion["alpha_threshold"], 128)
            self.assertEqual(conversion["component_policy"], "all; no connected-component removal")
            self.assertEqual(conversion["quantize_colors"], 55)
            self.assertEqual(conversion["palette_eye_region_extra_weight"], 20)
            self.assertGreater(conversion["source_gold_mask_cells"], 20)
            self.assertFalse(conversion["dither"])
            self.assertGreater(conversion["changed_occupied_color_cells"], 20000)
            x0, y0, x1, y1 = conversion["tight_bbox"]
            width, height = conversion["fitted_dimensions"]
            self.assertLess(abs(width / height - (x1 - x0) / (y1 - y0)), .004)
            for imported in atlas["asset_metadata"]["sprites"].values():
                self.assertEqual(imported["removed_component_cells"], 0)
                self.assertEqual(imported["alpha_changed"], 0)
                self.assertFalse(imported["resized"])

    def test_all60_composed_poses_have_disjoint_identical_occupied_positions(self):
        for kind, atlas in self.atlas["profiles"].items():
            count = detail.validate_profile(atlas)
            self.assertEqual(count, self.summary[kind]["composed_occupied_cells"])
            expected = None
            sprites = atlas["sprites"]
            for eye in detail.EYE_STATES:
                for mouth in detail.MOUTH_STATES:
                    composed = []
                    for a, b, c in zip(sprites["body"]["indices"], sprites[f"eyes_{eye}"]["indices"], sprites[f"mouth_{mouth}"]["indices"]):
                        self.assertLessEqual(sum(i != 0 for i in (a, b, c)), 1)
                        composed.append(a != 0 or b != 0 or c != 0)
                    if expected is None:
                        expected = composed
                    self.assertEqual(composed, expected)

    def test_neutral_expression_and_all_torso_cells_preserve_converted_reference(self):
        _, records = detail.source_records()
        for kind, record in records.items():
            base, _ = detail.prepare_reference(ROOT / record["path"], kind)
            layers = detail.portrait_layers(base, kind)
            pose = detail.compose(layers)
            # Neutral/rest preserves the actual detailed face; only six tiny
            # catchlight/pupil accents are authored across its two irises.
            differences = [(x, y) for y in range(256) for x in range(192)
                           if pose.getpixel((x, y)) != base.getpixel((x, y))]
            self.assertLessEqual(len(differences), 6)
            self.assertTrue(all(y < 130 for _, y in differences))
            self.assertEqual(list(pose.crop((0, 180, 192, 256)).get_flattened_data()),
                             list(base.crop((0, 180, 192, 256)).get_flattened_data()))

    def test_192_states_are_authored_distinct_fine_features_with_gold_iris_colors(self):
        for atlas in self.atlas["profiles"].values():
            sprites = atlas["sprites"]
            self.assertEqual(len({tuple(sprites[f"eyes_{s}"]["indices"]) for s in detail.EYE_STATES}), 5)
            self.assertEqual(len({tuple(sprites[f"mouth_{s}"]["indices"]) for s in detail.MOUTH_STATES}), 4)
            colors = {atlas["palette"][i] for i in sprites["eyes_neutral"]["indices"] if i}
            for color in (detail.GOLD, detail.GOLD_LIGHT, detail.GOLD_SHADOW, detail.GOLD_WARM):
                self.assertIn(detail.pack.color(color), colors)
            authoring = atlas["asset_metadata"]["provenance"]["manual_authoring"]
            self.assertEqual(authoring["fine_feature_line_width"], 1)
            self.assertTrue(authoring["neutral_reference_iris_retained"])

    def test_all_types_have_distinct_real_body_and_hair_masks(self):
        masks = [tuple(c != 0 for c in p["sprites"]["body"]["indices"]) for p in self.atlas["profiles"].values()]
        self.assertEqual(len(set(masks)), 3)
        doro, talo = masks[1:]
        self.assertGreater(sum(a != b for a, b in zip(doro, talo)), 4000)
        profiles = self.atlas["profiles"]
        self.assertGreater(len(profiles["doro"]["portrait_roles"]["hair_cells"]),
                           len(profiles["talo"]["portrait_roles"]["hair_cells"]) + 1000)
        self.assertGreater(len(profiles["dora"]["portrait_roles"]["hair_cells"]), 12000)

    def test_roles_are_sorted_body_cells_independent_hair_and_horn_masks(self):
        for kind, atlas in self.atlas["profiles"].items():
            body, palette, roles = atlas["sprites"]["body"]["indices"], atlas["palette"], atlas["portrait_roles"]
            hair, horns, signals = [set(roles[k]) for k in ("hair_cells", "horn_cells", "signal_cells")]
            self.assertFalse(hair & horns)
            self.assertTrue(signals <= horns)
            self.assertGreater(len(hair), 3000)
            self.assertGreater(len(horns), 700)
            for index in hair:
                r, g, b, a = detail.pack.rgba(palette[body[index]])
                self.assertGreater(g, r + 3)
                self.assertGreater(g, b + 2)
                if kind != "dora": self.assertLess(index // 192, 124, "Neck cables must not be hair roles")
            for index in horns:
                r, g, b, a = detail.pack.rgba(palette[body[index]])
                self.assertGreater(r, g + 20)
                self.assertGreater(r, b + 10)
                self.assertLess(index // 192, 85)

    def test_actual_forehead_skin_never_inherits_corporate_color_and_crystals_still_do(self):
        # Coordinates are the actual fixed192 anatomy observed in Root's
        # TALO-RENKO-HUGO and DORO-HUGO-PAN WebGPU failures, not a generic y cap.
        skin = {"dora": [((105, 80), (246, 214, 192, 255))],
                "doro": [((105, 80), (249, 216, 189, 255)), ((115, 75), (249, 216, 189, 255))],
                "talo": [((105, 80), (250, 217, 194, 255)), ((75, 84), (249, 218, 196, 255))]}
        crystals = {"dora": [(45, 20), (58, 50)],
                    "doro": [(49, 20), (62, 50)], "talo": [(49, 20), (59, 40)]}
        for kind, atlas in self.atlas["profiles"].items():
            body, palette, roles = atlas["sprites"]["body"]["indices"], atlas["palette"], atlas["portrait_roles"]
            hair, horns, signals = [set(roles[k]) for k in ("hair_cells", "horn_cells", "signal_cells")]
            for (x, y), expected in skin[kind]:
                index = y * 192 + x
                self.assertEqual(detail.pack.rgba(palette[body[index]]), expected)
                self.assertNotIn(index, hair | horns | signals, f"{kind} forehead skin must retain its source color")
                for main in ("PAN", "RENKO", "HUGO"):
                    for sub in ("PAN", "RENKO", "HUGO"):
                        self.assertEqual(detail.corporate_preview(atlas, main, sub).getpixel((x, y)), expected)
            for x, y in crystals[kind]:
                self.assertIn(y * 192 + x, horns, f"{kind} real crystal facet must still recolor")
            # All authored eye/eyebrow/mouth cells and the central neck region
            # stay outside the body-owned color masks, including future hues.
            for rect in detail.CORPORATE_REGIONS[kind]["eyebrows"] + [detail.CORPORATE_REGIONS[kind]["neck"]]:
                self.assertFalse(any(y * 192 + x in hair | horns | signals
                                     for y in range(rect[1], rect[3]) for x in range(rect[0], rect[2])))

    def test_binary_alpha_palette_and_49152_cell_bound_are_exact(self):
        self.assertEqual({kind: summary["composed_occupied_cells"] for kind, summary in self.summary.items()},
                         {"dora": 34594, "doro": 28524, "talo": 24262})
        for atlas in self.atlas["profiles"].values():
            self.assertLessEqual(len(atlas["palette"]), 64)
            self.assertEqual(atlas["palette"][0], 0)
            self.assertTrue(all(c >> 24 == 255 for c in atlas["palette"][1:]))
            self.assertLessEqual(detail.validate_profile(atlas), 49152)
        self.assertGreater(self.summary["dora"]["composed_occupied_cells"] + self.summary["doro"]["composed_occupied_cells"], 60000)

    def test_another_reference_size_refuses_implicit_recrop_or_upscale(self):
        path = Path(self.temp.name) / "wrong.png"
        Image.new("RGBA", (96, 128), (255, 255, 255, 255)).save(path)
        for kind in detail.AUTHORING:
            with self.assertRaisesRegex(ValueError, "recrop explicitly"):
                detail.prepare_reference(path, kind)

    def test_bad_dimensions_palette_indices_roles_and_pose_overlap_fail_closed(self):
        original = self.atlas["profiles"]["doro"]
        for mutate in [
            lambda a: a["palette"].__setitem__(0, 1),
            lambda a: a["palette"].__setitem__(1, 255),
            lambda a: a["sprites"]["body"]["indices"].__setitem__(0, 64),
            lambda a: a["sprites"]["body"]["indices"].__setitem__(0, True),
            lambda a: a["sprites"]["body"].__setitem__("width", 191),
            lambda a: a["portrait_roles"].__setitem__("source_size", [96, 128]),
            lambda a: a["portrait_roles"]["hair_cells"].append(a["portrait_roles"]["hair_cells"][0]),
            lambda a: a["portrait_roles"]["signal_cells"].append(a["portrait_roles"]["hair_cells"][0]),
            lambda a: a["sprites"]["eyes_blink"]["indices"].__setitem__(0, 1),
            lambda a: a["sprites"]["mouth_rest"]["indices"].__setitem__(next(i for i, c in enumerate(a["sprites"]["body"]["indices"]) if c), 1),
        ]:
            modified = copy.deepcopy(original)
            mutate(modified)
            with self.assertRaises(ValueError):
                detail.validate_profile(modified)


if __name__ == "__main__":
    unittest.main(verbosity=2)
