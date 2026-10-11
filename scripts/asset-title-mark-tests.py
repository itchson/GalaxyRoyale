"""Exact native title pack regeneration and explicitly declared raster losses."""
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from PIL import Image

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location("title_mark",ROOT/"scripts/asset-title-mark.py")
title=importlib.util.module_from_spec(spec)
spec.loader.exec_module(title)
FOLDER=ROOT/"assets/ui/title-mark"

class TitleMark(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.atlas=json.loads((FOLDER/"atlas.json").read_text(encoding="utf-8"))
        cls.temp=tempfile.TemporaryDirectory()
        cls.folder=Path(cls.temp.name)
        cls.regenerated,cls.report=title.build(cls.folder)
    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()
    def test_full_independent_regeneration_matches_committed_atlas_bytes(self):
        self.assertEqual((self.folder/"atlas.json").read_bytes(),(FOLDER/"atlas.json").read_bytes())
        self.assertEqual(self.atlas,self.regenerated)
    def test_partition_reconstructs_all_source_rgba_without_duplicate_crown(self):
        output=Image.new("RGBA",(512,192))
        count=0
        for n,(name,sprite) in enumerate(self.atlas["sprites"].items()):
            # Manifest order is8tiles followed by the separately moving crown.
            offset=title.CROWN[:2] if name=="crown" else (int(name[-1])*128,int(name[5])*96)
            for i,index in enumerate(sprite["indices"]):
                if not index: continue
                point=(offset[0]+i%sprite["width"],offset[1]+i//sprite["width"])
                self.assertEqual(output.getpixel(point)[3],0)
                output.putpixel(point,tuple(self.atlas["palette"][index].to_bytes(4,"little")))
                count+=1
        self.assertEqual(count,41662)
        self.assertEqual(output.tobytes(),Image.open(FOLDER/"source-cells.png").convert("RGBA").tobytes())
    def test_role_masks_are_occupied_disjoint_and_explicit(self):
        totals=[0,0]
        for name,roles in self.atlas["title_roles"]["parts"].items():
            occupied=self.atlas["sprites"][name]["indices"]
            a,b=map(set,(roles["energy_cells"],roles["scan_cells"]))
            self.assertFalse(a&b)
            self.assertTrue(all(occupied[i] for i in a|b))
            self.assertEqual(len(a),len(roles["energy_cells"]))
            self.assertEqual(len(b),len(roles["scan_cells"]))
            totals[0]+=len(a);totals[1]+=len(b)
        self.assertEqual(totals,[4799,11520])
    def test_binary_palette_and_conversion_loss_are_honest(self):
        self.assertEqual(len(self.atlas["palette"]),64)
        self.assertEqual(self.atlas["palette"][0],0)
        self.assertTrue(all(c>>24==255 for c in self.atlas["palette"][1:]))
        self.assertEqual(self.report["sampled_alpha_threshold_changes"],49509)
        self.assertGreater(self.report["rgb_channel_mean_absolute_error"],0)
        self.assertFalse(self.report["lossless_reference_claim"])
        self.assertEqual(self.report["components_removed"],0)
    def test_exact_indexed_png_roundtrip_and_group_transform_metrics(self):
        pack=title.module()
        output=self.folder/"roundtrip"
        pack.unpack(self.atlas,output)
        repacked=pack.pack(output/"manifest.json")
        for key in ("version","palette","sprites","title_roles"):
            self.assertEqual(repacked[key],self.atlas[key])
        metrics=self.atlas["asset_metadata"]["groups"][0]["metrics"]
        self.assertEqual((metrics["occupied_cells"],metrics["transforms"]),(41662,9))
    def test_png_encoding_variants_accept_only_equal_decoded_cells(self):
        with Image.open(FOLDER/"crown.png") as canonical:
            image=canonical.convert("RGBA")
        path=self.folder/"variant.png"
        image.save(path,compress_level=0)
        before=path.read_bytes()
        title.canonical_png(path,image)
        self.assertEqual(path.read_bytes(),before)
        changed=image.copy()
        changed.putpixel((0,0),(1,2,3,255))
        with self.assertRaises(AssertionError): title.canonical_png(path,changed)
        changed=image.copy()
        point=next((i%image.width,i//image.width) for i,c in enumerate(image.get_flattened_data()) if c[3])
        c=changed.getpixel(point)
        changed.putpixel(point,(*c[:3],0))
        with self.assertRaises(AssertionError): title.canonical_png(path,changed)
    def test_group_cap_rejects_without_silently_dropping_components(self):
        pack=title.module()
        group=self.atlas["asset_metadata"]["groups"][0]
        with self.assertRaises(pack.AssetError):
            pack.group_metrics(group,self.atlas["sprites"],pack.budget({"group_cells":41661}))

if __name__=="__main__": unittest.main(verbosity=2)
