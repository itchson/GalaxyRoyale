"""Focused authoring/pack tests; does not claim game or browser acceptance."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from PIL import Image

ROOT=Path(__file__).resolve().parent.parent
spec=importlib.util.spec_from_file_location('kikai',ROOT/'scripts/asset-kikai.py')
kikai=importlib.util.module_from_spec(spec);spec.loader.exec_module(kikai)
SOURCE=ROOT/'assets/concepts/kikai/dora-reference-expression-sheet.png'

class KikaiTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory=tempfile.TemporaryDirectory()
        cls.output=Path(cls.directory.name)/'authored'
        cls.atlas=kikai.author(SOURCE,cls.output)
    @classmethod
    def tearDownClass(cls):cls.directory.cleanup()
    def test_exact_authored_pngs_roundtrip_every_index_and_palette(self):
        pack=kikai.asset_pack
        target=Path(self.directory.name)/'unpacked'
        manifest=pack.unpack(self.atlas,target)
        again=pack.pack(manifest)
        for key in ['version','palette','sprites','portrait_roles']:
            self.assertEqual(self.atlas[key],again[key])
    def test_source_loss_is_declared_and_no_component_is_silently_removed(self):
        record=self.atlas['asset_metadata']['provenance']['conversion']
        self.assertEqual(record['source_dimensions'],[1881,836])
        self.assertEqual(record['crop'],[0,0,627,836])
        self.assertEqual(record['output_dimensions'],[96,128])
        self.assertTrue(record['nearest_resize']);self.assertFalse(record['dither'])
        self.assertEqual(record['component_policy'],'all; no component removal')
        self.assertGreater(record['changed_occupied_color_cells'],0)
        for sprite in self.atlas['asset_metadata']['sprites'].values():
            self.assertEqual(sprite['removed_component_cells'],0)
            self.assertFalse(sprite['resized']);self.assertEqual(sprite['alpha_changed'],0)
    def test_portrait_layers_do_not_overlap_or_change_occupied_mask_between_states(self):
        sprites=self.atlas['sprites'];base=sprites['dora_body']['indices']
        expected=None
        for eye in ['neutral','focused','concerned','pleased','blink']:
            for mouth in ['rest','small','open','smile']:
                layers=[base,sprites[f'dora_eyes_{eye}']['indices'],sprites[f'dora_mouth_{mouth}']['indices']]
                mask=[]
                for cells in zip(*layers):
                    self.assertLessEqual(sum(c!=0 for c in cells),1)
                    mask.append(any(cells))
                if expected is None:expected=mask
                self.assertEqual(mask,expected)
        self.assertEqual(sum(expected),8656)
    def test_corporate_hair_and_horn_roles_are_disjoint_and_signals_belong_to_horns(self):
        roles=self.atlas['portrait_roles'];base=self.atlas['sprites']['dora_body']['indices']
        hair,horns,signals=[set(roles[name]) for name in ['hair_cells','horn_cells','signal_cells']]
        self.assertFalse(hair & horns);self.assertTrue(signals <= horns)
        self.assertGreater(len(hair),1000);self.assertGreater(len(horns),100)
        self.assertTrue(all(i//96<38 for i in horns), 'Forehead skin must not inherit crystal colors')
        for i in hair|horns|signals:self.assertTrue(base[i])
        self.assertEqual((roles['default_main'],roles['default_sub']),('PAN','RENKO'))
    def test_ten_controls_are_original_32_cell_silhouettes_with_six_role_palette(self):
        icons=[s for name,s in self.atlas['sprites'].items() if name.startswith('icon_')]
        self.assertEqual(len(icons),10)
        self.assertEqual(len({tuple(c!=0 for c in s['indices']) for s in icons}),10)
        for sprite in icons:
            self.assertEqual((sprite['width'],sprite['height']),(32,32))
            colors={self.atlas['palette'][i] for i in sprite['indices'] if i}
            self.assertTrue(colors <= {kikai.asset_pack.color(c) for c in kikai.ICON_COLORS})
            self.assertGreaterEqual(len(colors),4)
    def test_different_source_shape_refuses_an_implicit_recrop(self):
        wrong=Path(self.directory.name)/'wrong.png';Image.new('RGBA',(3,3)).save(wrong)
        with self.assertRaisesRegex(ValueError,'recrop explicitly'):kikai.prepare_reference(wrong)
    def test_onboarding_dialogue_uses_declared_actual_stages_and_bounded_plain_pages(self):
        data=json.loads((ROOT/'assets/narrative/dora-initial-mission.json').read_text(encoding='utf-8'))
        self.assertEqual([s['stage'] for s in data['stages']],list(range(7)))
        self.assertEqual(data['character']['full_name'],'DORA-RENKO-PAN-1337')
        self.assertEqual(data['character']['speaker'],'ORP#1337')
        seen=set()
        for stage in data['stages']:
            self.assertNotIn('reward',stage)
            for page in stage['pages']:
                self.assertNotIn(page['id'],seen);seen.add(page['id'])
                self.assertGreater(len(page['text']),10);self.assertLessEqual(len(page['text']),256)
                self.assertTrue(all(32<=ord(c)<=126 for c in page['text']))
                self.assertIn(page['expression'],['neutral','focused','concerned','pleased'])
        self.assertEqual(data['stages'][0]['native_action'],'acknowledge_license')
        self.assertEqual(data['stages'][5]['required_flags'],[32,64])

if __name__=='__main__':unittest.main(verbosity=2)
