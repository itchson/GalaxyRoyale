"""Actual authoring/pack boundary checks, not browser visual acceptance."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

ROOT=Path(__file__).resolve().parent.parent
spec=importlib.util.spec_from_file_location('star_author',ROOT/'scripts/asset-shooting-star.py')
star=importlib.util.module_from_spec(spec)
spec.loader.exec_module(star)


class ShootingStarTests(unittest.TestCase):
    def test_authored_cells_roundtrip_without_loss_and_keep_real_facet_palette(self):
        with tempfile.TemporaryDirectory() as folder:
            output=Path(folder)
            atlas=star.author(output/'original')
            again=star.asset_pack.pack(star.asset_pack.unpack(atlas,output/'unpacked'))
            for key in ['version','palette','sprites']:
                self.assertEqual(atlas[key],again[key])
            head=atlas['sprites']['nucleus']
            self.assertEqual((head['width'],head['height']),(48,32))
            self.assertEqual(sum(i!=0 for i in head['indices']),644)
            self.assertGreaterEqual(len({i for i in head['indices'] if i}),8)
            for sprite in atlas['asset_metadata']['sprites'].values():
                self.assertEqual(sprite['alpha_changed'],0)
                self.assertEqual(sprite['removed_component_cells'],0)
                self.assertFalse(sprite['resized'])

    def test_original_generation_is_deterministic_and_has_explicit_authorship(self):
        with tempfile.TemporaryDirectory() as folder:
            first=star.author(Path(folder)/'first')
            second=star.author(Path(folder)/'second')
            self.assertEqual(first,second)
            self.assertFalse(first['asset_metadata']['provenance']['ai_generated'])
            self.assertEqual(first['palette'][0],0)
            self.assertLessEqual(len(first['palette']),64)
            self.assertEqual(sum(i!=0 for i in first['sprites']['coma']['indices']),176)


if __name__=='__main__':unittest.main(verbosity=2)
