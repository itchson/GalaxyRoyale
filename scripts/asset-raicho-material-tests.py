"""Independent authoring/source/PNG checks; no native renderer claim."""
import argparse
import copy
import importlib.util
import json
import math
from pathlib import Path
import unittest

from PIL import Image

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("pan_material_authorer", HERE / "asset-raicho-material-proof.py")
AUTHOR = importlib.util.module_from_spec(spec)
spec.loader.exec_module(AUTHOR)
PROOF = None


def pixels(image):
    raw = image.convert("RGBA").tobytes()
    return list(zip(raw[0::4],raw[1::4],raw[2::4],raw[3::4]))


class MaterialAuthoringTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.atlas = json.loads(AUTHOR.ATLAS.read_text(encoding="utf-8"))
        cls.sidecar, cls.heights, cls.roles = AUTHOR.author(cls.atlas)
        AUTHOR.validate(cls.sidecar, cls.atlas)

    def test_exact_field_dimensions_and_authoring_counts(self):
        for name, width, count in (("coarse48",48,662),("fine96",96,1205),("full96_authoring_only",96,2648)):
            grid = self.sidecar[name]
            self.assertEqual((grid["width"],grid["height"],len(grid["cells"])),(width,width,width*width))
            self.assertEqual(sum(c is not None for c in grid["cells"]),count)

    def test_normals_are_varied_finite_and_not_a_flat_placeholder(self):
        cells = [c for c in self.sidecar["full96_authoring_only"]["cells"] if c is not None]
        self.assertGreater(len({tuple(c[:3]) for c in cells}),100)
        self.assertTrue(all(all(math.isfinite(v) for v in c) and abs(sum(v*v for v in c[:3])-1)<1e-7 for c in cells))
        self.assertTrue(any(math.hypot(c[0],c[1])>.5 for c in cells))
        self.assertTrue(any(c[0]>.1 for c in cells) and any(c[0]<-.1 for c in cells))
        self.assertTrue(any(c[1]>.1 for c in cells) and any(c[1]<-.1 for c in cells))

    def test_painted_color_changes_cannot_change_authored_height_or_normals(self):
        changed = copy.deepcopy(self.atlas)
        changed["palette"] = [0] + [int.from_bytes(bytes((255,1,251,255)),"little") for _ in changed["palette"][1:]]
        other, heights, _ = AUTHOR.author(changed)
        self.assertEqual(heights,self.heights)
        for name in ("coarse48","fine96","full96_authoring_only"):
            self.assertEqual(other[name],self.sidecar[name])

    def test_nonfinite_zero_and_wrong_sized_masks_are_rejected(self):
        index = next(i for i,c in enumerate(self.sidecar["fine96"]["cells"]) if c is not None)
        for normal in ((float("nan"),0,1),(0,0,0),(2,0,1)):
            changed = copy.deepcopy(self.sidecar)
            changed["fine96"]["cells"][index][:3] = normal
            with self.assertRaises(ValueError):
                AUTHOR.validate(changed,self.atlas)
        changed = copy.deepcopy(self.sidecar)
        changed["fine96"]["cells"].pop()
        with self.assertRaises(ValueError):
            AUTHOR.validate(changed,self.atlas)

    def test_role_overlap_and_empty_source_material_are_rejected(self):
        changed = copy.deepcopy(self.atlas)
        full = changed["sprites"]["pan_hikari"]
        full["material_cells"]["glass"] = sorted(full["material_cells"]["glass"] + [full["material_cells"]["metal"][0]])
        with self.assertRaises(ValueError):
            AUTHOR.author(changed)
        changed = copy.deepcopy(self.atlas)
        index = changed["sprites"]["pan_hikari"]["material_cells"]["reactor"][0]
        changed["sprites"]["pan_hikari"]["indices"][index] = 0
        with self.assertRaises(ValueError):
            AUTHOR.author(changed)

    def test_sparse_emission_is_only_existing_signals_and_no_coarse_double_glow(self):
        signal = set(self.atlas["sprites"]["pan_hikari"]["signal_cells"])
        actual = {i for i,c in enumerate(self.sidecar["fine96"]["cells"]) if c is not None and c[3]}
        self.assertEqual(actual,signal)
        self.assertEqual(len(actual),31)
        self.assertTrue(all(c is None or c[3]==0 for c in self.sidecar["coarse48"]["cells"]))
        self.assertEqual({self.sidecar["fine96"]["cells"][i][3] for i in actual},{255})

    def test_rendered_source_and_every_mask_keep_exact_alpha(self):
        full = self.atlas["sprites"]["pan_hikari"]
        palette = [tuple(c.to_bytes(4,"little")) for c in self.atlas["palette"]]
        expected = [palette[p] for p in full["indices"]]
        self.assertEqual(pixels(Image.open(PROOF/"pan-source.png")),expected)
        for name in ("height","normal","emissive","flat-left","normal-left","normal-right","normal-emission"):
            image = Image.open(PROOF/f"pan-{name}.png").convert("RGBA")
            self.assertEqual(image.size,(96,96))
            self.assertEqual([p[3] for p in pixels(image)],[p[3] for p in expected])
        flat = pixels(Image.open(PROOF/"pan-flat-left.png"))
        normal = pixels(Image.open(PROOF/"pan-normal-left.png"))
        self.assertGreater(sum(a[:3]!=b[:3] for a,b in zip(flat,normal,strict=True)),100)

    def test_authoring_damage_fixture_removes_exact_coarse_owned_fine_block(self):
        # This confirms the existing coordinate/ownership contract offline;
        # it does not claim an actual projectile hit or native frame execution.
        coarse = (23,16)
        removed = {i for i,c in enumerate(self.sidecar["fine96"]["cells"]) if c is not None and (i%96//2,i//96//2)==coarse}
        self.assertEqual(removed,{32*96+46,32*96+47,33*96+46,33*96+47})
        intact = pixels(Image.open(PROOF/"pan-normal-emission.png"))
        damaged = pixels(Image.open(PROOF/"pan-damage-fixture.png"))
        self.assertEqual({i for i,(a,b) in enumerate(zip(intact,damaged,strict=True)) if a!=b},removed)
        self.assertTrue(all(damaged[i]==(0,0,0,0) for i in removed))


if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--proof",type=Path,required=True)
    args=parser.parse_args()
    if not args.proof.is_absolute():
        parser.error("absolute actual authoring proof directory required")
    PROOF=args.proof
    unittest.main(argv=[__file__],verbosity=2)
