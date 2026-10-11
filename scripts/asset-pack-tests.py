"""Meaningful offline import/rejection/real-atlas roundtrip tests."""
import copy
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path
from PIL import Image
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('asset_pack', Path(__file__).with_name('asset-pack.py'))
ap = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ap)
ROOT = Path(__file__).resolve().parents[1]

class PackTests(unittest.TestCase):

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.folder = Path(self.temp.name)

    def tearDown(self):
        self.temp.cleanup()

    def png(self, name='sprite.png', size=(3, 3), pixels=None):
        image = Image.new('RGBA', size)
        image.putdata(pixels or [(33, 77, 99, 255)] * (size[0] * size[1]))
        image.save(self.folder / name)
        return name

    def run_pack(self, manifest):
        path = self.folder / 'manifest.json'
        ap.write_json(path, manifest)
        return ap.pack(path)

    def manifest(self, **sprite):
        return {'schema': 1, 'sprites': [{'name': 'ship', 'source': self.png(), **sprite}]}

    def assert_rejected(self, manifest, message=None):
        with self.assertRaises(ap.AssetError) as caught:
            self.run_pack(manifest)
        if message:
            self.assertIn(message, str(caught.exception))

    def test_exact_native_cells_and_shared_palette_roundtrip(self):
        m = self.manifest(topology='solid')
        p = self.run_pack(m)
        self.assertEqual(len(p['palette']), 64)
        self.assertEqual(p['sprites']['ship']['indices'], [1] * 9)
        folder = self.folder / 'export'
        manifest = ap.unpack(p, folder)
        again = ap.pack(manifest)
        for key in ('version', 'palette', 'sprites'):
            self.assertEqual(p[key], again[key])
        self.assertEqual(again['asset_metadata']['sprites']['ship']['metrics']['occupied'], 9)

    def test_all_components_are_retained_by_default(self):
        pixels = [(0, 0, 0, 0)] * 9
        pixels[0] = pixels[8] = (40, 80, 90, 255)
        self.png(pixels=pixels)
        m = {'schema': 1, 'sprites': [{'name': 'lights', 'source': 'sprite.png'}]}
        p = self.run_pack(m)
        self.assertEqual(p['asset_metadata']['sprites']['lights']['metrics']['components'], 2)
        self.assertEqual(p['asset_metadata']['sprites']['lights']['removed_component_cells'], 0)
        m['sprites'][0]['components'] = {'mode': 'largest'}
        p = self.run_pack(m)
        self.assertEqual(p['asset_metadata']['sprites']['lights']['removed_component_cells'], 1)

    def test_nonbinary_alpha_requires_explicit_loss_policy(self):
        m = self.manifest()
        self.png(pixels=[(20, 40, 50, 128)] * 9)
        self.assert_rejected(m, 'Nonbinary alpha')
        m['sprites'][0]['alpha'] = {'mode': 'threshold', 'threshold': 128}
        p = self.run_pack(m)
        self.assertEqual(p['asset_metadata']['sprites']['ship']['alpha_changed'], 9)
        self.assertTrue(all((c >> 24 == 255 for c in p['palette'][1:])))

    def test_grid_requires_uniform_explicit_blocks(self):
        m = self.manifest(cell_grid=[2, 2])
        self.png(size=(4, 4))
        p = self.run_pack(m)
        self.assertEqual(p['sprites']['ship']['width'], 2)
        pixels = [(33, 77, 99, 255)] * 16
        pixels[1] = (255, 0, 0, 255)
        self.png(size=(4, 4), pixels=pixels)
        self.assert_rejected(m, 'Nonuniform cell block')

    def test_crop_never_silently_pads_or_fits(self):
        self.assert_rejected(self.manifest(crop=[1, 1, 3, 3]), 'outside source')
        p = self.run_pack(self.manifest(crop=[1, 0, 2, 3]))
        self.assertEqual(p['sprites']['ship']['width'], 2)
        self.assertFalse(p['asset_metadata']['sprites']['ship']['resized'])

    def test_resize_has_explicit_before_after_evidence(self):
        p = self.run_pack(self.manifest(resize={'mode': 'nearest-fit', 'size': [2, 2]}))
        r = p['asset_metadata']['sprites']['ship']
        self.assertTrue(r['resized'])
        self.assertEqual((r['pre_resize_occupied'], r['post_resize_occupied']), (9, 4))

    def test_palette_quantization_is_explicit_and_loss_is_counted(self):
        m = self.manifest()
        self.png(pixels=[(i * 20, 33 + i * 5, 91, 255) for i in range(9)])
        m['palette'] = {'mode': 'exact', 'max_colors': 3}
        self.assert_rejected(m, 'Too many colors')
        m['palette']['mode'] = 'quantize'
        p = self.run_pack(m)
        self.assertEqual(len(p['palette']), 3)
        self.assertGreater(p['asset_metadata']['sprites']['ship']['quantized_color_cells'], 0)

    def test_intentional_holes_are_distinct_from_born_solid(self):
        m = self.manifest(topology='solid')
        pixels = [(33, 77, 99, 255)] * 9
        pixels[4] = (0, 0, 0, 0)
        self.png(pixels=pixels)
        self.assert_rejected(m, 'enclosed transparent holes')
        m['sprites'][0]['topology'] = 'holes-allowed'
        p = self.run_pack(m)
        self.assertEqual(p['asset_metadata']['sprites']['ship']['metrics']['hole_cells'], 1)

    def test_solid_flood_uses_all_boundary_components(self):
        pixels = [(0, 0, 0, 0), (33, 77, 99, 255), (0, 0, 0, 0)] * 3
        m = self.manifest(topology='solid')
        self.png(pixels=pixels)
        p = self.run_pack(m)
        self.assertEqual(p['asset_metadata']['sprites']['ship']['metrics']['hole_cells'], 0)

    def test_transforms_compose_shear_reflection_and_all_cell_corners(self):
        m = self.manifest()
        m['groups'] = [{'name': 'assembly', 'parts': [{'name': 'parent', 'sprite': 'ship', 'transform': {'position': [10, 20], 'basis_x': [-2, 0], 'basis_y': [1, 3]}}, {'name': 'child', 'sprite': 'ship', 'parent': 'parent', 'transform': {'position': [4, 1], 'basis_x': [1, 1], 'basis_y': [0, 0.5]}}]}]
        p = self.run_pack(m)
        metrics = p['asset_metadata']['groups'][0]['metrics']
        self.assertEqual(metrics['occupied_cells'], 18)
        self.assertEqual(metrics['occupied_bounds'], [0.75, 15.5, 14.5, 29.75])

    def test_reused_instances_charge_every_occupied_cell(self):
        m = self.manifest()
        m['budgets'] = {'group_cells': 17}
        m['groups'] = [{'name': 'copies', 'parts': [{'name': 'a', 'sprite': 'ship'}, {'name': 'b', 'sprite': 'ship'}]}]
        self.assert_rejected(m, 'Whole-group')

    def test_invalid_parents_and_composed_extent_are_rejected(self):
        m = self.manifest()
        m['groups'] = [{'name': 'bad', 'parts': [{'name': 'a', 'sprite': 'ship', 'parent': 'later'}]}]
        self.assert_rejected(m, 'earlier part')
        m['groups'][0]['parts'][0] = {'name': 'a', 'sprite': 'ship', 'transform': {'position': [999999, 0]}}
        self.assert_rejected(m, 'corners exceed')

    def test_animation_references_durations_and_frame_budgets(self):
        m = self.manifest()
        m['animations'] = [{'name': 'pulse', 'loop': True, 'frames': [{'kind': 'sprite', 'name': 'ship', 'duration_ms': 100}]}]
        self.assertEqual(len(self.run_pack(m)['asset_metadata']['animations']), 1)
        m['animations'][0]['frames'][0]['duration_ms'] = 0
        self.assert_rejected(m, 'duration_ms')
        m['animations'][0]['frames'][0].update(duration_ms=100, name='unknown')
        self.assert_rejected(m, 'unknown frame')

    def test_invalid_indices_palette_alpha_and_nonfinite_json(self):
        for p in [{'version': 1, 'palette': [0, ap.color((1, 2, 3, 128))], 'sprites': {'s': {'width': 1, 'height': 1, 'indices': [1]}}}, {'version': 1, 'palette': [0, 4294967295], 'sprites': {'s': {'width': 1, 'height': 1, 'indices': [True]}}}]:
            with self.assertRaises(ap.AssetError):
                ap.validate(p)
        path = self.folder / 'bad.json'
        path.write_text('{"x":NaN}')
        with self.assertRaises(ap.AssetError):
            ap.read_json(path)

    def test_duplicate_unused_palette_indices_roundtrip_exactly(self):
        p = {'version': 1, 'palette': [0, 4282668390, 4282668390, 4289449455], 'sprites': {'duplicate': {'width': 3, 'height': 1, 'indices': [1, 2, 0]}}}
        again = ap.pack(ap.unpack(p, self.folder / 'duplicates'))
        self.assertEqual(again['palette'], p['palette'])
        self.assertEqual(again['sprites'], p['sprites'])

    def test_existing_runtime_atlases_keep_all_indices_dimensions_and_palette(self):
        for relative in ['assets/pixels/atlas.json', 'assets/ui/atlas.json', 'assets/pixels/boss-atlas.json']:
            original = ap.read_json(ROOT / relative)
            folder = self.folder / Path(relative).parent.name / Path(relative).stem
            if folder.exists():
                folder = folder.with_name(folder.name + '-next')
            again = ap.pack(ap.unpack(original, folder))
            for key in ('version', 'palette', 'sprites'):
                self.assertEqual(original[key], again[key], relative + ' ' + key)
            if 'rigs' in original:
                self.assertEqual(original['rigs'], again['rigs'])
            for key, s in original['sprites'].items():
                self.assertEqual(sum((i != 0 for i in s['indices'])), sum((i != 0 for i in again['sprites'][key]['indices'])))

    def test_input_png_palette_mismatch_is_rejected(self):
        p = {'version': 1, 'palette': [0, 4279383126], 'sprites': {'x': {'width': 1, 'height': 1, 'indices': [1]}}}
        path = ap.unpack(p, self.folder / 'mismatch')
        m = ap.read_json(path)
        m['palette']['colors'][1] = 4284826401
        ap.write_json(path, m)
        with self.assertRaisesRegex(ap.AssetError, 'palette differs'):
            ap.pack(path)

    def test_scattered_cells_use_bounded_component_scan(self):
        mask = [(x + y) % 2 == 0 for y in range(64) for x in range(64)]
        self.assertEqual(len(ap.components(mask, 64, 64)), 2048)

    def test_unknown_policies_do_not_silently_fall_back(self):
        self.assert_rejected(self.manifest(reszie={'mode': 'nearest-fit', 'size': [2, 2]}), 'Unknown sprite')
        m = self.manifest()
        m['defaults'] = {'drop_components': True}
        self.assert_rejected(m, 'Unknown default')
        m = self.manifest()
        m['palette'] = {'mode': 'provided'}
        self.assert_rejected(m, 'requires explicit colors')

    def test_stale_metrics_and_content_fingerprints_reject_modified_packs(self):
        p = self.run_pack(self.manifest())
        p['asset_metadata']['sprites']['ship']['metrics']['occupied'] = 1
        with self.assertRaisesRegex(ap.AssetError, 'Stale sprite'):
            ap.validate(p)
        p = self.run_pack(self.manifest())
        p['palette'][1] = 4279383126
        with self.assertRaisesRegex(ap.AssetError, 'fingerprint'):
            ap.validate(p)

    def test_bool_version_overflowing_float_and_animation_cap_are_rejected(self):
        m = self.manifest()
        m['schema'] = True
        self.assert_rejected(m, 'schema1')
        path = self.folder / 'overflow.json'
        path.write_text('{"n":1e400}')
        with self.assertRaises(ap.AssetError):
            ap.read_json(path)
        m = self.manifest()
        m['budgets'] = {'animation_frames': 1}
        m['animations'] = [{'name': 'pulse', 'loop': True, 'frames': [{'kind': 'sprite', 'name': 'ship', 'duration_ms': 20}] * 2}]
        self.assert_rejected(m, 'frame budget')

    def test_unpack_refuses_to_overwrite_existing_source_directory(self):
        p = self.run_pack(self.manifest())
        with self.assertRaisesRegex(ap.AssetError, 'absent or empty'):
            ap.unpack(p, self.folder)

    def test_nested_policy_typos_and_malformed_metadata_fail_cleanly(self):
        for policy in [{'alpha': {'mode': 'exact', 'threshhold': 128}}, {'resize': {'mode': 'none', 'szie': [2, 2]}}, {'components': {'mode': 'all', 'drop': True}}]:
            self.assert_rejected(self.manifest(**policy), 'Unknown')
        m = self.manifest()
        m['palette'] = {'mode': 'exact', 'max_colours': 2}
        self.assert_rejected(m, 'palette policy')
        p = self.run_pack(self.manifest())
        p['asset_metadata']['sprites']['ship'] = 'not-an-object'
        with self.assertRaisesRegex(ap.AssetError, 'metadata must be an object'):
            ap.validate(p)
        p = self.run_pack(self.manifest())
        p['asset_metadata']['animations'] = [{'name': 'bad', 'loop': True, 'frames': [None]}]
        with self.assertRaisesRegex(ap.AssetError, 'frame fields'):
            ap.validate(p)

    def test_animated_png_cannot_silently_lose_frames(self):
        frames = [Image.new('RGBA', (2, 2), c) for c in [(255, 0, 0, 255), (0, 255, 0, 255)]]
        frames[0].save(self.folder / 'animated.png', save_all=True, append_images=frames[1:], duration=[100, 100], loop=0)
        self.assert_rejected(self.manifest(source='animated.png'), 'separately declared frames')

    def test_affine_typos_and_nonstring_parents_are_rejected(self):
        m = self.manifest()
        m['groups'] = [{'name': 'group', 'parts': [{'name': 'a', 'sprite': 'ship', 'transform': {'scale': [2, 2]}}]}]
        self.assert_rejected(m, 'Unknown transform')
        m['groups'][0]['parts'][0] = {'name': 'a', 'sprite': 'ship', 'parent': []}
        self.assert_rejected(m, 'part identifier')

    def test_source_work_charges_every_decoded_image_before_loading(self):
        m = self.manifest()
        m['sprites'].append({'name': 'second', 'source': self.png('second.png')})
        m['budgets'] = {'source_pixels': 17}
        self.assert_rejected(m, 'remaining decoded source')
        m['budgets']['source_pixels'] = 18
        self.assertEqual(len(self.run_pack(m)['sprites']), 2)
if __name__ == '__main__':
    unittest.main()
