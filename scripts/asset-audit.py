"""Compare actual native extraction with declared cells, affine corners and timing."""
import argparse
import hashlib
import importlib.util
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('asset_pack', Path(__file__).with_name('asset-pack.py'))
ap = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ap)

def audit(pack, extraction, historical=False):
    ap.validate(pack)
    metadata = pack['asset_metadata']
    animation = next((a for a in metadata['animations'] if a['name'] == extraction['animation']), None)
    ap.require(animation is not None, 'Unknown extracted animation')
    frames = extraction['frames']
    ap.require(len(frames) == len(animation['frames']), 'Incomplete native timeline')
    results = []
    time = 0
    for step, frame in zip(animation['frames'], frames):
        ap.require(step['kind'] == 'group', 'Native audit supports group frames only')
        group = next((g for g in metadata['groups'] if g['name'] == step['name']))
        ap.require(frame['group'] == group['name'] and frame['duration_ms'] == step['duration_ms'], 'Native frame ordering/duration differs from declaration')
        if not historical:
            ap.require(frame.get('sample_time_ms') == time, 'Native timeline sample time differs from declaration')
        elif 'sample_time_ms' in frame:
            ap.require(frame['sample_time_ms'] == time, 'Historical sample time differs from declaration')
        time += step['duration_ms']
        known = {}
        expected = []
        for part in group['parts']:
            local = ap.affine(part.get('transform', {}))
            parent = part.get('parent')
            transform = ap.compose(known[parent], local) if parent else local
            known[part['name']] = transform
            sprite = pack['sprites'][part['sprite']]
            w, h = (sprite['width'], sprite['height'])
            for i, index in enumerate(sprite['indices']):
                if index:
                    x, y = (i % w - w / 2, i // w - h / 2)
                    corners = [ap.point(transform, [x + dx, y + dy]) for dx, dy in ((0, 0), (1, 0), (1, 1), (0, 1))]
                    expected.append((corners, pack['palette'][index]))
        ap.require(len(frame['cells']) == len(expected) == frame['occupied_cells'], 'Native occupancy differs from packed instances')
        ap.require(frame['transforms'] == len(group['parts']), 'Native transform count differs')
        maximum_error = 0
        for (corners, color), actual in zip(expected, frame['cells']):
            ap.require(actual['color'] == color, 'Native color differs from packed palette')
            ap.require(len(actual['corners']) == 4, 'Native cell needs four corners')
            for want, have in zip(corners, actual['corners']):
                ap.vector(have, 2, 'native corner')
                maximum_error = max(maximum_error, *(abs(want[i] - have[i]) for i in (0, 1)))
        ap.require(maximum_error <= 0.0001, 'Native affine corner mismatch beyond disclosed f32 tolerance')
        ap.vector(frame['bounds'], 4, 'native bounds')
        bound_error = max((abs(a - b) for a, b in zip(frame['bounds'], group['metrics']['occupied_bounds'])))
        ap.require(bound_error <= 0.0001, 'Native occupied bounds mismatch')
        results.append({'group': group['name'], 'sample_time_ms': frame.get('sample_time_ms'), 'duration_ms': frame['duration_ms'], 'occupied_cells': frame['occupied_cells'], 'transforms': frame['transforms'], 'batches': frame['batches'], 'maximum_corner_error_world': maximum_error, 'maximum_bound_error_world': bound_error})
    pending = ['Runtime metadata adapter', 'Both GPU backend presentation', 'Moving human readability', 'Physical phone performance']
    if historical:
        pending.insert(0, 'Final native executable execution after Windows Application Control block')
    return {'scope': 'Actual native PixelBody/FrameBuilder packets compared against authoring cells; offline adapter only, not live Game animation or GPU acceptance', 'historical_extraction': historical, 'timeline_sample_times_verified': not historical, 'frames': results, 'pending': pending}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--extraction', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--historical-extraction', action='store_true', help='Audit preserved packets without claiming final executable/sample-time proof')
    args = parser.parse_args()
    ap.require(args.output.resolve() not in (args.pack.resolve(), args.extraction.resolve()), 'Audit output must not overwrite inputs')
    result = audit(ap.read_json(args.pack), ap.read_json(args.extraction), args.historical_extraction)
    result['recorded_at_utc'] = datetime.now(timezone.utc).isoformat()
    result['input_sha256'] = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in (args.pack, args.extraction)}
    ap.write_json(args.output, result)
    print(json.dumps(result, indent=2))
if __name__ == '__main__':
    main()
