"""Reproduce the Haven service example from the real accepted dock cell source."""
import argparse, hashlib, importlib.util, json, sys
from pathlib import Path
from PIL import Image
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('asset_pack', Path(__file__).with_name('asset-pack.py'))
ap = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ap)
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--atlas', type=Path, required=True)
parser.add_argument('--output-dir', type=Path, required=True)
args = parser.parse_args()
atlas = ap.read_json(args.atlas)
ap.validate(atlas)
folder = args.output_dir
folder.mkdir(parents=True, exist_ok=True)
palette = atlas['palette']

def write(name, w, h, indices):
    image = Image.new('P', (w, h))
    image.putpalette([v for c in palette for v in ap.rgba(c)[:3]])
    image.info['transparency'] = bytes((ap.rgba(c)[3] for c in palette))
    image.putdata(indices)
    image.save(folder / name, bits=8, optimize=False)
dock = atlas['sprites']['dock']
write('dock.png', dock['width'], dock['height'], dock['indices'])

def nearest(rgb):
    return min(range(1, len(palette)), key=lambda i: sum(((ap.rgba(palette[i])[j] - rgb[j]) ** 2 for j in range(3))))
metal = nearest((79, 109, 128))
lit = nearest((88, 222, 239))
dim = nearest((27, 52, 68))
gold = nearest((224, 185, 104))
rows = ['..##....##..', '.#aa#..#bb#.', '.#aa#..#bb#.', '..##....##..', '............', '..##....##..', '..##....##..', '............']
for frame in (0, 1):
    colors = {'#': metal, 'a': lit if frame == 0 else dim, 'b': dim if frame == 0 else gold, '.': 0}
    write(f'service-{frame}.png', 12, 8, [colors[c] for row in rows for c in row])

def group(frame):
    return {'name': f'haven-{frame}', 'parts': [{'name': 'dock', 'sprite': 'dock', 'transform': {'position': [0, 0], 'basis_x': [0.9, 0.12], 'basis_y': [-0.08, 0.75]}}, {'name': 'service-left', 'sprite': f'service-{frame}', 'parent': 'dock', 'transform': {'position': [-22, 8], 'basis_x': [0.75, 0.18], 'basis_y': [0, 0.9]}}, {'name': 'service-reflected', 'sprite': f'service-{frame}', 'parent': 'service-left', 'transform': {'position': [24, -12], 'basis_x': [-0.8, 0], 'basis_y': [0.25, 0.8]}}]}
manifest = {'schema': 1, 'palette': {'mode': 'provided', 'max_colors': len(palette), 'colors': palette}, 'sprites': [{'name': 'dock', 'source': 'dock.png', 'encoding': 'indices', 'topology': 'holes-allowed'}, {'name': 'service_0', 'source': 'service-0.png', 'encoding': 'indices', 'topology': 'holes-allowed'}, {'name': 'service_1', 'source': 'service-1.png', 'encoding': 'indices', 'topology': 'holes-allowed'}], 'groups': [group(0), group(1)], 'animations': [{'name': 'service-pulse', 'loop': True, 'frames': [{'kind': 'group', 'name': f'haven-{i}', 'duration_ms': 600} for i in (0, 1)]}], 'budgets': {'group_cells': 2000, 'group_transforms': 16}, 'provenance': {'source_type': 'accepted-cell-source-plus-original-authored-service-lights', 'dock_source': 'assets/pixels/atlas.json#dock', 'dock_source_sha256': hashlib.sha256(args.atlas.read_bytes()).hexdigest(), 'generator': 'scripts/asset-example.py', 'note': "Dock retains the accepted generated atlas's exact palette/indices. Lamps are original authored cells. Animation is an offline preview contract, not automatic Game integration."}}
for g in manifest['groups']:
    for part in g['parts']:
        if part['sprite'].startswith('service-'):
            part['sprite'] = part['sprite'].replace('-', '_')
ap.write_json(folder / 'manifest.json', manifest)
packed = ap.pack(folder / 'manifest.json')
ap.write_json(folder / 'pack.json', packed)
ap.preview(packed, folder / 'cells-preview.png', 4)
print(json.dumps({'output': str(folder), 'sprites': ap.validate(packed), 'groups': [g['metrics'] for g in packed['asset_metadata']['groups']]}, indent=2))
