"""Candidate conversion/schema tests; no GPU, co-op or human acceptance."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("dock_conversion", Path(__file__).with_name("convert.py"))
M = importlib.util.module_from_spec(spec)
spec.loader.exec_module(M)


class DockConversionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.pack = M.AP.read_json(M.DEFAULT / "pack.json")
        cls.loss = M.AP.read_json(M.DEFAULT / "conversion-losses.json")

    def test_reference_identity_and_actual_dimensions(self):
        self.assertEqual(M.sha(M.DEFAULT / "source.png"), M.SOURCE_SHA)
        with M.Image.open(M.DEFAULT / "source.png") as source:
            self.assertEqual(source.size, (1254, 1254))

    def test_regeneration_matches_all_committed_conversion_bytes(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            M.author(folder)
            # Full raw pack/provenance bytes, every material/frame/mask PNG,
            # comparisons and measured losses; not just temp self round-trip.
            for generated in folder.iterdir():
                self.assertEqual(generated.read_bytes(), (M.DEFAULT / generated.name).read_bytes(), generated.name)

    def test_pack_exact_grid_palette_binary_alpha_and_budget(self):
        summary = M.AP.validate(self.pack)
        self.assertEqual(len(self.pack["palette"]), 64)
        self.assertEqual(self.pack["palette"][0], 0)
        self.assertTrue(all(M.AP.rgba(c)[3] == 255 for c in self.pack["palette"][1:]))
        for sprite in self.pack["sprites"].values():
            self.assertEqual((sprite["width"], sprite["height"]), (128, 128))
        for group in self.pack["asset_metadata"]["groups"]:
            self.assertEqual(group["metrics"]["occupied_cells"], 7100)
            self.assertEqual(group["metrics"]["transforms"], 4)
        self.assertEqual(summary["dock-0"]["occupied"], 7100)

    def test_material_partition_preserves_union_once_and_exact_colors(self):
        for state in range(3):
            body = self.pack["sprites"][f"dock-{state}"]["indices"]
            for i, index in enumerate(body):
                parts = [self.pack["sprites"][f"{p}-{state}"]["indices"][i] for p in M.PARTS]
                self.assertEqual(sum(x != 0 for x in parts), int(index != 0))
                self.assertEqual(sum(parts), index)
        self.assertEqual(self.loss["part_counts"], {"deck": 4611, "alloy": 1786, "screens": 182, "signals": 521})

    def test_all_components_retained_and_open_pad_cutouts_stay_transparent(self):
        indices = self.pack["sprites"]["dock-0"]["indices"]
        topology = M.AP.topology(indices, 128, 128)
        self.assertEqual((topology["components"], topology["hole_cells"]), (1, 0))
        # Actual exposed-space probes beside cardinal docking arms and between
        # diagonal rim assemblies; no arbitrary largest-component filter.
        for x, y in ((0, 0), (20, 20), (44, 10), (24, 24), (34, 31)):
            self.assertEqual(indices[y * 128 + x], 0)
        for x, y in ((64, 5), (64, 120), (5, 64), (120, 64)):
            self.assertNotEqual(indices[y * 128 + x], 0)

    def test_concrete_deck_crystal_screen_and_alloy_landmarks(self):
        roles = self.pack["pixel_rig"]["roles"]
        for x, y, role in ((64, 12, 0), (64, 31, 1), (65, 58, 2), (64, 64, 3)):
            self.assertEqual(roles[y * 128 + x], role)
            self.assertNotEqual(self.pack["sprites"]["dock-0"]["indices"][y * 128 + x], 0)
        ix = self.pack["sprites"]["dock-0"]["indices"][64 * 128 + 64]
        r, g, b, _ = M.AP.rgba(self.pack["palette"][ix])
        self.assertGreater(g, r + 20)
        self.assertGreater(g, b + 20)

    def test_three_real_lamp_states_same_occupancy_and_only_explicit_mask_changes(self):
        states = [self.pack["sprites"][f"dock-{n}"]["indices"] for n in range(3)]
        lamps = set(self.pack["pixel_rig"]["lamp_cells"])
        self.assertEqual(len(lamps), 433)
        self.assertEqual(len({tuple(s) for s in states}), 3)
        for state in states[1:]:
            self.assertEqual([bool(x) for x in states[0]], [bool(x) for x in state])
            self.assertTrue(all(i in lamps for i, (a, b) in enumerate(zip(states[0], state)) if a != b))
        for x, y in ((64, 64), (65, 58), (64, 12)):
            self.assertEqual(len({s[y * 128 + x] for s in states}), 1)

    def test_exact_png_rgba_and_indexed_roundtrip_no_component_drop(self):
        for name, sprite in self.pack["sprites"].items():
            with M.Image.open(M.DEFAULT / f"{name}.png") as source:
                committed = source.convert("RGBA")
            self.assertEqual(committed.tobytes(), M.image(sprite, self.pack["palette"]).tobytes())
        with tempfile.TemporaryDirectory() as tmp:
            manifest = M.AP.unpack(self.pack, Path(tmp) / "unpacked")
            recovered = M.AP.pack(manifest)
            self.assertEqual(recovered["palette"], self.pack["palette"])
            self.assertEqual(recovered["sprites"], self.pack["sprites"])

    def test_loss_measures_are_not_lossless_or_exact1280_claim(self):
        for side, count in ((128, 7100), (192, 16085), (256, 28549)):
            row = self.loss["grid_comparison"][str(side)]
            self.assertEqual(row["native_topology"]["occupied"], count)
            self.assertEqual(row["source_dimensions"], [1254, 1254])
            self.assertGreater(row["quantized_occupied_color_cells"], 0)
            self.assertGreater(row["alpha_values_changed_before_sampling"], 0)
            self.assertLess(row["source_mask_vs_native_nearest_reprojection"]["intersection_over_union"], 1)

    def test_pack_rejects_cell_and_transform_budget_underflow(self):
        manifest = M.AP.read_json(M.DEFAULT / "manifest.json")
        for key, value in (("group_cells", 7099), ("group_transforms", 3)):
            changed = copy.deepcopy(manifest)
            changed["budgets"][key] = value
            with self.assertRaises(M.AP.AssetError):
                M.AP.group_metrics(changed["groups"][0], self.pack["sprites"], M.AP.budget(changed["budgets"]))

    def test_group_parent_is_finite_earlier_and_counted_not_free(self):
        group = copy.deepcopy(self.pack["asset_metadata"]["groups"][0])
        limits = self.pack["asset_metadata"]["budgets"]
        self.assertEqual(M.AP.group_metrics(group, self.pack["sprites"], limits)["transforms"], 4)
        group["parts"][0]["transform"]["position"][0] = float("nan")
        with self.assertRaises(M.AP.AssetError):
            M.AP.group_metrics(group, self.pack["sprites"], limits)
        group = copy.deepcopy(self.pack["asset_metadata"]["groups"][0])
        group["parts"][0]["parent"] = "signals"
        with self.assertRaises(M.AP.AssetError):
            M.AP.group_metrics(group, self.pack["sprites"], limits)

    def test_unknown_reference_fails_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            changed = Path(tmp) / "changed.png"
            M.PNG.write_png(changed, M.Image.new("RGBA", (1254, 1254), (0, 0, 0, 0)))
            with self.assertRaises(M.AP.AssetError):
                M.author(Path(tmp) / "candidate", changed)


if __name__ == "__main__":
    unittest.main()
