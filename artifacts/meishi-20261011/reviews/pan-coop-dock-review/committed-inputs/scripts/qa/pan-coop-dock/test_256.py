"""Independent decoded-grid/tile/frame tests for the detailed candidate."""
import importlib.util
import hashlib
from pathlib import Path
import tempfile
import unittest

spec=importlib.util.spec_from_file_location("dock256",Path(__file__).with_name("convert_256.py"))
N=importlib.util.module_from_spec(spec);spec.loader.exec_module(N)
M=N.M
archive_spec=importlib.util.spec_from_file_location("dock_archives",Path(__file__).with_name("archive_256.py"))
A=importlib.util.module_from_spec(archive_spec);archive_spec.loader.exec_module(A)


def assemble(pack,state):
    result=[0]*65536
    for name,x0,y0 in N.TILES:
        s=pack["sprites"][f"{name}-body-{state}"]
        for y in range(128):
            result[(y0+y)*256+x0:(y0+y)*256+x0+128]=s["indices"][y*128:(y+1)*128]
    return result


class TiledDockTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.pack=M.AP.read_json(N.DEFAULT/"pack.json")
        cls.manifest=M.AP.read_json(N.DEFAULT/"manifest.json")

    def test_all_committed_authoring_bytes_match_regeneration(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder=Path(tmp);N.author(folder)
            for p in folder.iterdir():
                self.assertEqual(p.read_bytes(),(N.DEFAULT/p.name).read_bytes(),p.name)

    def test_shared_palette_exact_and_all_bodies_fit_actual128_contract(self):
        prior=M.AP.read_json(M.DEFAULT/"pack.json")
        self.assertEqual(self.pack["palette"],prior["palette"])
        self.assertEqual(len(self.pack["sprites"]),60)
        for s in self.pack["sprites"].values():
            self.assertEqual((s["width"],s["height"]),(128,128))
        M.AP.validate(self.pack)

    def test_full256_sample_mask_is_retained_without_tile_seams_or_duplication(self):
        with M.Image.open(M.DEFAULT/"source.png") as im:
            raw=list(M.sample(im.convert("RGBA"),256).get_flattened_data())
        merged=assemble(self.pack,0)
        self.assertEqual([bool(c[3]) for c in raw],[bool(c) for c in merged])
        self.assertEqual(sum(bool(c) for c in merged),28549)
        self.assertEqual(M.AP.topology(merged,256,256)["components"],1)
        self.assertEqual(M.AP.topology(merged,256,256)["hole_cells"],0)

    def test_each_quad_material_partition_emits_same_color_once(self):
        for n in range(3):
            for name,_,_ in N.TILES:
                full=self.pack["sprites"][f"{name}-body-{n}"]["indices"]
                for i,index in enumerate(full):
                    parts=[self.pack["sprites"][f"{name}-{p}-{n}"]["indices"][i] for p in M.PARTS]
                    self.assertEqual(sum(c!=0 for c in parts),int(index!=0))
                    self.assertEqual(sum(parts),index)

    def test_real_same_occupancy_lamp_states_and_fixed_central_core(self):
        states=[assemble(self.pack,n) for n in range(3)]
        lamps=set(self.pack["pixel_rig_tiles"]["lamp_cells_source"])
        self.assertEqual(len(lamps),1528)
        self.assertEqual(len({tuple(s) for s in states}),3)
        for state in states[1:]:
            self.assertEqual([bool(c) for c in state],[bool(c) for c in states[0]])
            self.assertTrue(all(i in lamps for i,(a,b) in enumerate(zip(states[0],state)) if a!=b))
        for y in range(110,146):
            for x in range(110,146):
                self.assertEqual(len({s[y*256+x] for s in states}),1)

    def test_whole_group_16_instances_and_half_scale_bounds(self):
        for group in self.pack["asset_metadata"]["groups"]:
            self.assertEqual(group["metrics"]["occupied_cells"],28549)
            self.assertEqual(group["metrics"]["transforms"],16)
            self.assertEqual(group["metrics"]["occupied_bounds"],[-63,-63,63,63])
            roots=[p for p in group["parts"] if "parent" not in p]
            self.assertEqual(len(roots),4)
            for p in roots:
                self.assertEqual(p["transform"]["basis_x"],[.5,0])
                self.assertEqual(p["transform"]["basis_y"],[0,.5])

    def test_native_pngs_roundtrip_and_body_full_review_pngs_agree(self):
        for name,s in self.pack["sprites"].items():
            with M.Image.open(N.DEFAULT/f"{name}.png") as im:
                self.assertEqual(im.convert("RGBA").tobytes(),M.image(s,self.pack["palette"]).tobytes())
        for n in range(3):
            with M.Image.open(N.DEFAULT/f"full-state-{n}.png") as im:
                self.assertEqual(im.tobytes(),M.image({"width":256,"height":256,"indices":assemble(self.pack,n)},self.pack["palette"]).tobytes())

    def test_higher_detail_source_landmarks_preserve_alloy_recess_and_crystal(self):
        # Concrete fine source landmarks corresponding to the retained coarse
        # north runway, central crystal and cyan terminal, plus open exterior.
        full=assemble(self.pack,0)
        for x,y in ((0,0),(40,40),(88,20),(48,48)):
            self.assertEqual(full[y*256+x],0)
        for x,y in ((128,10),(128,24),(128,62),(128,128),(10,128),(240,128)):
            self.assertNotEqual(full[y*256+x],0)
        colors={M.AP.rgba(self.pack["palette"][i]) for i in full if i}
        self.assertGreaterEqual(len(colors),55)

    def test_transform_pixel_limits_remain_local_and_reject_short_budget(self):
        g=self.manifest["groups"][0]
        for limits in ({"group_cells":28548,"group_transforms":16},{"group_cells":32000,"group_transforms":15}):
            with self.assertRaises(M.AP.AssetError):
                M.AP.group_metrics(g,self.pack["sprites"],M.AP.budget(limits))

    def test_retained_gzip_proofs_decode_exactly_with_bounded_sizes(self):
        records=M.AP.read_json(N.DEFAULT/"archive.json")["files"]
        for name in A.NAMES:
            encoded=(N.DEFAULT/(name+".gz")).read_bytes()
            self.assertEqual(encoded[4:8],b"\0"*4)
            raw=A.decode(encoded,records[name])
            self.assertEqual(hashlib.sha256(raw).hexdigest(),records[name]["decoded_sha256"])
            frames=A.read(N.DEFAULT,name)["frames"]
            self.assertEqual(len(frames),3)
            self.assertTrue(all(f["occupied_cells"]==len(f["cells"])==28549 for f in frames))

    def test_archive_corruption_and_changed_decoded_cell_fail_closed(self):
        record=M.AP.read_json(N.DEFAULT/"archive.json")["files"][A.NAMES[0]]
        encoded=(N.DEFAULT/(A.NAMES[0]+".gz")).read_bytes()
        corrupt=bytearray(encoded);corrupt[len(corrupt)//2]^=1
        with self.assertRaises(A.N.M.AP.AssetError):
            A.decode(corrupt,record)
        raw=A.decode(encoded,record)
        # Preserve valid JSON/length but alter a recorded coordinate digit.
        changed=raw.replace(b"-63.0",b"-62.0",1)
        self.assertNotEqual(changed,raw)
        replacement=A.encode(changed)
        claimed=dict(record,gzip_sha256=A.digest(replacement),gzip_bytes=len(replacement))
        with self.assertRaises(A.N.M.AP.AssetError):
            A.decode(replacement,claimed)


if __name__=="__main__":
    unittest.main()
