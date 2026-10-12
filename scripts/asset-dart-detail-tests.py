"""Focused exact-mask/palette/pipeline/atlas-span proofs for authored Dart32."""
from pathlib import Path
from unittest.mock import patch
import importlib.util,json,subprocess,sys,tempfile,unittest
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('dart_detail_author',ROOT/'scripts/asset-dart-detail.py');D=importlib.util.module_from_spec(spec);spec.loader.exec_module(D)

class DartDetailProof(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.baseline=D.AP.read_json(D.DEFAULT/'baseline.json')
        cls.old=cls.baseline['sprites']['dart']['indices']
        cls.new=D.authored_indices(cls.baseline)
        cls.raw=subprocess.check_output(['git','show',f'{cls.baseline["reference_commit"]}:assets/pixels/atlas.json'],cwd=ROOT)

    def test_baseline_is_exact_recorded_git_sprite_and_palette(self):
        reference=json.loads(self.raw)
        self.assertEqual(self.baseline['version'],reference['version'])
        self.assertEqual(self.baseline['palette'],reference['palette'])
        self.assertEqual(self.baseline['sprites'],{'dart':reference['sprites']['dart']})
        self.assertEqual(D.digest(self.raw.replace(b'\r\n',b'\n')),self.baseline['source_atlas_sha256_lf'])

    def test_exact426mask_grid_palette_and_original_signal_sites(self):
        packed=D.AP.read_json(D.DEFAULT/'pack.json');s=packed['sprites']['dart']
        self.assertEqual((s['width'],s['height']),(32,32));self.assertEqual(s['indices'],self.new)
        self.assertEqual(packed['palette'],self.baseline['palette'])
        self.assertEqual(sum(bool(i) for i in self.new),426)
        self.assertEqual([bool(i) for i in self.old],[bool(i) for i in self.new])
        for signal in (4,6,33):
            self.assertEqual([i for i,c in enumerate(self.old) if c==signal],[i for i,c in enumerate(self.new) if c==signal])
        self.assertTrue(any(a!=b for a,b in zip(self.old,self.new)))

    def test_surgical_update_retains_all_other_actual_atlas_bytes(self):
        match=D.PATTERN.search(self.raw);self.assertIsNotNone(match)
        changed=D.surgical_atlas(self.raw,self.baseline,self.new);after=D.PATTERN.search(changed)
        self.assertEqual(self.raw[:match.start(2)],changed[:after.start(2)])
        self.assertEqual(self.raw[match.end(2):],changed[after.end(2):])
        self.assertEqual(D.surgical_atlas(changed,self.baseline,self.new),changed)
        original=json.loads(self.raw);candidate=json.loads(changed)
        self.assertEqual(original['palette'],candidate['palette'])
        for name in original['sprites']:
            if name!='dart':self.assertEqual(original['sprites'][name],candidate['sprites'][name])

    def test_unreviewed_other_atlas_or_dart_changes_refuse_overwrite(self):
        with self.assertRaisesRegex(D.AP.AssetError,'Non-Dart atlas bytes changed'):
            D.surgical_atlas(self.raw.replace(b'"version":1',b'"version":2',1),self.baseline,self.new)
        match=D.PATTERN.search(self.raw);old=self.old.copy();old[0]=1
        changed=self.raw[:match.start(2)]+json.dumps(old,separators=(',',':')).encode()[1:-1]+self.raw[match.end(2):]
        with self.assertRaisesRegex(D.AP.AssetError,'Unexpected Dart art'):
            D.surgical_atlas(changed,self.baseline,self.new)

    def test_actual_indexed_png_and_exact_pack_reproduce_all_owned_outputs(self):
        owned=ROOT/'work/enemy-art-quality-review/dart-test-temp';owned.mkdir(parents=True,exist_ok=True)
        with tempfile.TemporaryDirectory(dir=owned) as folder:
            output=Path(folder);self.assertTrue(output.resolve().is_relative_to(owned.resolve()))
            rebuilt,_=D.author(output)
            self.assertEqual(rebuilt['sprites']['dart']['indices'],self.new)
            for name in ('source.png','manifest.json','pack.json','provenance.json'):
                actual=(output/name).read_bytes();expected=(D.DEFAULT/name).read_bytes()
                if name.endswith('.json'):actual=actual.replace(b'\r\n',b'\n');expected=expected.replace(b'\r\n',b'\n')
                self.assertEqual(actual,expected,name)

    def test_crlf_reference_reproduces_same_canonical_outputs(self):
        owned=ROOT/'work/enemy-art-quality-review/dart-test-temp';owned.mkdir(parents=True,exist_ok=True)
        with tempfile.TemporaryDirectory(dir=owned) as folder:
            task=Path(folder);self.assertTrue(task.resolve().is_relative_to(owned.resolve()))
            reference=task/'reference';reference.mkdir();original=D.DEFAULT
            raw=(original/'baseline.json').read_bytes().replace(b'\r\n',b'\n')
            (reference/'baseline.json').write_bytes(raw.replace(b'\n',b'\r\n'))
            with patch.object(D,'DEFAULT',reference):D.author(task/'output')
            for name in ('source.png','manifest.json','pack.json','provenance.json'):
                actual=(task/'output'/name).read_bytes();expected=(original/name).read_bytes()
                if name.endswith('.json'):expected=expected.replace(b'\r\n',b'\n')
                self.assertEqual(actual,expected,name)

    def test_shipped_atlas_is_exact_frozen_candidate_with_original_other_bytes(self):
        actual=(ROOT/'assets/pixels/atlas.json').read_bytes()
        expected=D.surgical_atlas(self.raw,self.baseline,self.new)
        self.assertEqual(actual.replace(b'\r\n',b'\n'),expected)

if __name__=='__main__':unittest.main()
