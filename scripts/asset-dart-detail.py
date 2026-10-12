"""Author Dart32 material clusters on its frozen426-cell mask; exact indexed pack.

No resizing/quantization/new cells/palette edits. Only --apply writes the dart
indices substring in the legacy atlas after verifying all retained bytes.
"""
from __future__ import annotations
import argparse,hashlib,importlib.util,json,re,sys
from pathlib import Path
sys.dont_write_bytecode=True
from PIL import Image,__version__ as pillow_version
ROOT=Path(__file__).resolve().parents[1]
DEFAULT=ROOT/'assets/pixels/dart-detail'
SIGNALS=frozenset((4,6,33))
PATTERN=re.compile(rb'("dart":\{"width":32,"height":32,"indices":\[)([0-9,]+)(\]\})')
spec=importlib.util.spec_from_file_location('dart_asset_pack',ROOT/'scripts/asset-pack.py')
AP=importlib.util.module_from_spec(spec);spec.loader.exec_module(AP)

def digest(raw):return hashlib.sha256(raw).hexdigest()
def write_json(path,value):
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes((json.dumps(value,indent=2,allow_nan=False)+'\n').encode('utf-8'))

def authored_indices(baseline):
    old = baseline['sprites']['dart']['indices']
    AP.require(len(old) == 1024 and sum(bool(i) for i in old) == 426, 'Frozen Dart grid/mask changed')
    result = []
    for i, original in enumerate(old):
        if not original or original in SIGNALS:
            result.append(original)
            continue
        x, y = i % 32, i // 32
        dx = abs(x - 15)
        color = original
        # Existing shaded red wings remain the class material. Compact diagonal
        # plates get a lit rim, alloy plane and dark seam instead of broad bands.
        diagonal = dx - (21 - y)
        if 13 <= y <= 18 and 4 <= dx <= 9 and 0 <= diagonal <= 2:
            color = (25, 7, 5)[diagonal] if x < 15 else (31, 16, 11)[diagonal]
        # A framed blue canopy and inset keel organize the centre independently
        # of the retained emissive sites. These colours never opt into pulsing.
        if 8 <= y <= 12 and dx <= 2:
            color = 16 if dx == 2 else 26
            if y == 8:
                color = 5 if x <= 15 else 16
            elif x == 15 and y in (9, 10):
                color = 8
        elif 17 <= y <= 23 and dx <= 1:
            color = 31
            if x == 15 and y in (19, 20):
                color = 3
        elif 16 <= y <= 23 and 2 <= dx <= 3:
            # The two rails frame a recessed module; left/up-facing surfaces
            # carry one bright cell, while right/down-facing seams stay dark.
            color = (11 if dx == 3 else 25) if x < 15 else (31 if dx == 3 else 16)
            if y == 16:
                color = 5 if x < 15 else 11
        elif 20 <= y <= 23 and 4 <= dx <= 5:
            color = 49 if y % 2 == 0 else (16 if x < 15 else 25)
        # Tiny rim/inner-shadow steps give the aft red plates depth without
        # repainting the broad source gradient or brightening every perimeter.
        if y == 17 and 10 <= dx <= 12:
            color = 17 if x < 15 else 34
        elif y == 22 and 11 <= dx <= 13:
            color = 38
        AP.require(color not in SIGNALS, 'New material must not add a signal site')
        result.append(color)
    AP.require([bool(i) for i in result] == [bool(i) for i in old], 'Dart occupancy changed')
    AP.require(all(a == b for a, b in zip(old, result) if a in SIGNALS or b in SIGNALS), 'Dart signal sites changed')
    return result

def surgical_atlas(raw,baseline,candidate):
    matches=list(PATTERN.finditer(raw));AP.require(len(matches)==1,'Exact32 Dart span required')
    found=matches[0];old=baseline['sprites']['dart']['indices']
    current=[int(i) for i in found.group(2).split(b',')]
    AP.require(current in (old,candidate),'Unexpected Dart art; refusing overwrite')
    encoded=lambda values:json.dumps(values,separators=(',',':')).encode()[1:-1]
    restored=raw[:found.start(2)]+encoded(old)+raw[found.end(2):]
    AP.require(digest(restored.replace(b'\r\n',b'\n'))==baseline['source_atlas_sha256_lf'],'Non-Dart atlas bytes changed; review new baseline first')
    changed=raw[:found.start(2)]+encoded(candidate)+raw[found.end(2):]
    before=json.loads(raw);after=json.loads(changed)
    AP.require(after['palette']==baseline['palette']==before['palette'],'Palette changed')
    AP.require({k:v for k,v in after['sprites'].items() if k!='dart'}=={k:v for k,v in before['sprites'].items() if k!='dart'},'Other sprites changed')
    return changed

def author(output=DEFAULT):
    baseline=AP.read_json(DEFAULT/'baseline.json');AP.validate({k:baseline[k] for k in ('version','palette','sprites')})
    indices=authored_indices(baseline);palette=baseline['palette'];output.mkdir(parents=True,exist_ok=True)
    im=Image.new('P',(32,32));im.putpalette([c for p in palette for c in AP.rgba(p)[:3]])
    im.info['transparency']=bytes(AP.rgba(p)[3] for p in palette);im.putdata(indices)
    im.save(output/'source.png',optimize=False,bits=8)
    groups=[{'name':name,'parts':[{'name':'hull','sprite':'dart','transform':{'basis_x':[scale,0],'basis_y':[0,scale]}}]} for name,scale in [('dart-native',1),('dart-game',.75)]]
    manifest={'schema':1,'palette':{'mode':'provided','max_colors':64,'colors':palette},'sprites':[{'name':'dart','source':'source.png','encoding':'indices','topology':'holes-allowed'}],'groups':groups,'animations':[{'name':'dart-review','loop':False,'frames':[{'kind':'group','name':g['name'],'duration_ms':500} for g in groups]}],'budgets':{'sprite_cells':426,'group_cells':426,'group_transforms':1,'extent':64},'provenance':{'source_type':'original-authored32-grid-material-clusters-on-frozen-mask','reference_commit':baseline['reference_commit'],'baseline_sha256_lf':digest((DEFAULT/'baseline.json').read_bytes().replace(b'\r\n',b'\n')),'animation_policy':'Preserve exact baseline sites/indices4,6,33; all new material indices are non-pulsing'}}
    write_json(output/'manifest.json',manifest);packed=AP.pack(output/'manifest.json');write_json(output/'pack.json',packed)
    AP.require(packed['sprites']['dart']['indices']==indices and packed['palette']==palette,'Exact indexed conversion changed candidate')
    old=baseline['sprites']['dart']['indices']
    provenance={'source_grid':[32,32],'occupied':426,'palette_entries':64,'changed_color_cells':sum(a!=b for a,b in zip(old,indices)),'preserved_signal_cells':sum(a in SIGNALS for a in old),'mask_sha256':digest(bytes(bool(i) for i in old)),'baseline_indices_sha256':digest(bytes(old)),'candidate_indices_sha256':digest(bytes(indices)),'generation_model':None,'authorship':'Original shaded32-grid canopy/recess/rails/beveled plate clusters; source wing gradients retained; no generated concept','text_fingerprint_policy':'UTF-8 text hashes canonicalize CRLF to LF; PNG/indices/mask hashes are exact binary' ,'pillow':pillow_version,'source_sha256':digest((output/'source.png').read_bytes()),'manifest_sha256_lf':digest((output/'manifest.json').read_bytes().replace(b'\r\n',b'\n')),'pack_sha256_lf':digest((output/'pack.json').read_bytes().replace(b'\r\n',b'\n')),'author_script_sha256_lf':digest(Path(__file__).read_bytes().replace(b'\r\n',b'\n')),'converter_sha256_lf':digest((ROOT/'scripts/asset-pack.py').read_bytes().replace(b'\r\n',b'\n'))}
    write_json(output/'provenance.json',provenance);return packed,provenance

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output-dir',type=Path,default=DEFAULT);parser.add_argument('--apply',action='store_true');args=parser.parse_args()
    packed,provenance=author(args.output_dir)
    if args.apply:
        baseline=AP.read_json(DEFAULT/'baseline.json');path=ROOT/'assets/pixels/atlas.json';path.write_bytes(surgical_atlas(path.read_bytes(),baseline,packed['sprites']['dart']['indices']))
    print(json.dumps({'output':str(args.output_dir),'applied':args.apply,**provenance},indent=2))
if __name__=='__main__':main()
