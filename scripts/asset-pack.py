"""Audited offline PNG <-> version-1 engine cell packs. See docs/ASSET_PIPELINE.md."""
from __future__ import annotations
import argparse
import hashlib
import json
import math
import re
from collections import deque
from pathlib import Path
from PIL import Image, ImageDraw, __version__ as pillow_version
TOOL_VERSION = '1.0'
NAME = re.compile('^[a-z][a-z0-9_-]{0,63}$')
MAX_SOURCE_PIXELS = 16777216
LIMITS = {'source_pixels': MAX_SOURCE_PIXELS, 'sprite_cells': 65536, 'group_cells': 60000, 'group_transforms': 256, 'animation_frames': 256, 'extent': 1000000}

class AssetError(ValueError):
    pass

def require(condition, message):
    if not condition:
        raise AssetError(message)

def integer(value, low, high, label):
    require(type(value) is int and low <= value <= high, f'{label} must be an integer in {low}..{high}')
    return value

def name(value):
    require(isinstance(value, str) and NAME.fullmatch(value), f'Invalid asset identifier: {value!r}')
    return value

def vector(value, size, label):
    require(isinstance(value, list) and len(value) == size, f'{label} must have {size} values')
    require(all((type(n) in (float, int) and math.isfinite(n) and (abs(n) <= 1000000) for n in value)), f'{label} must be finite and bounded')
    return value

def read_json(path):
    require(path.stat().st_size <= 32000000, 'JSON input exceeds 32 MB')

    def finite(s):
        value = float(s)
        require(math.isfinite(value), 'Nonfinite JSON number')
        return value
    return json.loads(path.read_text(encoding='utf-8-sig'), parse_float=finite, parse_constant=lambda s: (_ for _ in ()).throw(AssetError(f'Nonfinite JSON: {s}')))

def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False, allow_nan=False) + '\n', encoding='utf-8')

def color(rgba):
    return sum((v << 8 * i for i, v in enumerate(rgba)))

def rgba(c):
    return tuple((c >> s & 255 for s in (0, 8, 16, 24)))

def components(mask, width, height, diagonal=False):
    remaining = {i for i, filled in enumerate(mask) if filled}
    found = []
    offsets = [(-1, 0), (1, 0), (0, -1), (0, 1)]
    if diagonal:
        offsets += [(-1, -1), (1, -1), (-1, 1), (1, 1)]
    for first in range(len(mask)):
        if first not in remaining:
            continue
        remaining.remove(first)
        group, todo = ([first], deque([first]))
        while todo:
            i = todo.popleft()
            x, y = (i % width, i // width)
            for dx, dy in offsets:
                nx, ny = (x + dx, y + dy)
                ni = ny * width + nx
                if 0 <= nx < width and 0 <= ny < height and (ni in remaining):
                    remaining.remove(ni)
                    group.append(ni)
                    todo.append(ni)
        found.append(group)
    return found

def topology(indices, width, height):
    """Four-neighbor solids; empty cells touching any image edge are exterior."""
    mask = [i != 0 for i in indices]
    groups = components(mask, width, height)
    empty = [not m for m in mask]
    holes = [g for g in components(empty, width, height) if not any((i % width in (0, width - 1) or i // width in (0, height - 1) for i in g))]
    filled = [i for i, m in enumerate(mask) if m]
    if not filled:
        return {'occupied': 0, 'components': 0, 'hole_cells': 0, 'bounds': None}
    xs, ys = ([i % width for i in filled], [i // width for i in filled])
    return {'occupied': len(filled), 'components': len(groups), 'hole_cells': sum(map(len, holes)), 'bounds': [min(xs), min(ys), max(xs) + 1, max(ys) + 1]}

def policy(spec, defaults):
    p = {'encoding': 'rgba', 'cell_grid': [1, 1], 'alpha': {'mode': 'exact'}, 'components': {'mode': 'all'}, 'resize': {'mode': 'none'}, 'topology': 'holes-allowed'}
    require(set(defaults) <= set(p), 'Unknown default conversion policy')
    require(set(spec) <= set(p) | {'name', 'source', 'crop', 'anchor'}, 'Unknown sprite field/policy')
    p.update(defaults)
    p.update({k: v for k, v in spec.items() if k in p})
    require(p['encoding'] in ('rgba', 'indices'), 'encoding must be rgba or indices')
    vector(p['cell_grid'], 2, 'cell_grid')
    for n in p['cell_grid']:
        integer(n, 1, 64, 'cell_grid')
    require(isinstance(p['alpha'], dict) and p['alpha'].get('mode') in ('exact', 'threshold'), 'alpha policy must be exact or threshold')
    require(set(p['alpha']) <= ({'mode'} if p['alpha']['mode'] == 'exact' else {'mode', 'threshold'}), 'Unknown alpha policy field')
    if p['alpha']['mode'] == 'threshold':
        integer(p['alpha'].get('threshold'), 1, 255, 'alpha threshold')
    require(isinstance(p['components'], dict) and p['components'].get('mode') in ('all', 'largest'), 'component policy must be all or largest')
    require(set(p['components']) == {'mode'}, 'Unknown component policy field')
    require(isinstance(p['resize'], dict) and p['resize'].get('mode') in ('none', 'nearest-fit'), 'resize policy must be none or nearest-fit')
    require(set(p['resize']) <= ({'mode'} if p['resize']['mode'] == 'none' else {'mode', 'size'}), 'Unknown resize policy field')
    if p['resize']['mode'] == 'nearest-fit':
        vector(p['resize'].get('size'), 2, 'resize size')
        for n in p['resize']['size']:
            integer(n, 1, 256, 'resize side')
    require(p['topology'] in ('solid', 'holes-allowed'), 'topology must be solid or holes-allowed')
    return p

def import_png(path, spec, p, palette, source_pixels_remaining=MAX_SOURCE_PIXELS):
    """Exact blocks by default; indexed input validates effective PNG tRNS RGBA."""
    with Image.open(path) as image:
        require(image.format == 'PNG', 'Asset sources must be PNG files')
        require(getattr(image, 'n_frames', 1) == 1, 'Animated PNG needs separately declared frames; implicit first-frame import is forbidden')
        require(image.width * image.height <= source_pixels_remaining, 'PNG exceeds remaining decoded source pixel-work budget')
        image.load()
        source_size = list(image.size)
        crop = spec.get('crop', [0, 0, image.width, image.height])
        require(isinstance(crop, list) and len(crop) == 4, 'crop is [x,y,width,height]')
        x, y, w, h = [integer(n, 0 if i < 2 else 1, MAX_SOURCE_PIXELS, 'crop') for i, n in enumerate(crop)]
        require(x + w <= image.width and y + h <= image.height, 'Crop is outside source; padding is never inferred')
        if p['encoding'] == 'indices':
            require(image.mode == 'P' and palette is not None, 'indices encoding needs indexed PNG and provided palette')
            png_palette = image.getpalette('RGBA')
            require(png_palette is not None, 'Indexed PNG has no palette')
            check = Image.new('P', (len(palette), 1))
            check.putpalette(image.getpalette())
            if 'transparency' in image.info:
                check.info['transparency'] = image.info['transparency']
            check.putdata(list(range(len(palette))))
            require([color(c) for c in check.convert('RGBA').get_flattened_data()] == palette, 'Indexed PNG palette differs from manifest; ambiguous RGB remapping is forbidden')
            require(p['cell_grid'] == [1, 1] and p['alpha']['mode'] == 'exact' and (p['components']['mode'] == 'all') and (p['resize']['mode'] == 'none'), 'Indexed exact mode forbids lossy policies')
            cropped = image.crop((x, y, x + w, y + h))
            values = list(cropped.get_flattened_data())
            require(all((i < len(palette) for i in values)), 'Indexed PNG references undeclared palette entries')
            return ((w, h, values), {'source_dimensions': source_size, 'crop': crop, 'alpha_changed': 0, 'removed_component_cells': 0, 'resized': False, 'source_cell_count': w * h})
        pixels = list(image.convert('RGBA').crop((x, y, x + w, y + h)).get_flattened_data())
    gx, gy = p['cell_grid']
    require(w % gx == 0 and h % gy == 0, 'Crop dimensions must divide exactly by cell_grid')
    sampled = []
    for cy in range(0, h, gy):
        for cx in range(0, w, gx):
            block = [pixels[yy * w + xx] for yy in range(cy, cy + gy) for xx in range(cx, cx + gx)]
            require(all((c == block[0] for c in block)), 'Nonuniform cell block: grid sampling would discard detail')
            sampled.append(block[0])
    width, height = (w // gx, h // gy)
    changed = 0
    hidden_rgb = 0
    clean = []
    for c in sampled:
        alpha = c[3]
        if p['alpha']['mode'] == 'exact':
            require(alpha in (0, 255), 'Nonbinary alpha needs an explicit threshold policy')
        else:
            a = 255 if alpha >= p['alpha']['threshold'] else 0
            changed += a != alpha
            alpha = a
        hidden_rgb += alpha == 0 and c[:3] != (0, 0, 0)
        clean.append((*c[:3], 255) if alpha else (0, 0, 0, 0))
    groups = components([c[3] != 0 for c in clean], width, height, diagonal=True)
    removed = 0
    if p['components']['mode'] == 'largest' and groups:
        keep = set(max(groups, key=lambda g: (len(g), -min(g))))
        removed = sum((c[3] != 0 and i not in keep for i, c in enumerate(clean)))
        clean = [c if i in keep else (0, 0, 0, 0) for i, c in enumerate(clean)]
    resized = p['resize']['mode'] != 'none'
    before_occupied = sum((c[3] != 0 for c in clean))
    if resized:
        target = p['resize']['size']
        im = Image.new('RGBA', (width, height))
        im.putdata(clean)
        factor = min(target[0] / width, target[1] / height)
        fit = (max(1, round(width * factor)), max(1, round(height * factor)))
        scaled = im.resize(fit, Image.Resampling.NEAREST)
        canvas = Image.new('RGBA', tuple(target))
        canvas.paste(scaled, ((target[0] - fit[0]) // 2, (target[1] - fit[1]) // 2))
        clean = list(canvas.get_flattened_data())
        width, height = target
    return ((width, height, clean), {'source_dimensions': source_size, 'crop': crop, 'source_cell_count': len(sampled), 'alpha_changed': changed, 'transparent_rgb_canonicalized': hidden_rgb, 'source_components_8_connected': len(groups), 'removed_component_cells': removed, 'resized': resized, 'pre_resize_occupied': before_occupied, 'post_resize_occupied': sum((c[3] != 0 for c in clean))})

def affine(transform):
    require(isinstance(transform, dict), 'transform must be an object')
    require(set(transform) <= {'position', 'basis_x', 'basis_y'}, 'Unknown transform field')
    return [vector(transform.get('position', [0, 0]), 2, 'position'), vector(transform.get('basis_x', [1, 0]), 2, 'basis_x'), vector(transform.get('basis_y', [0, 1]), 2, 'basis_y')]

def point(t, p):
    return [t[0][i] + t[1][i] * p[0] + t[2][i] * p[1] for i in range(2)]

def compose(parent, child):
    pos = point(parent, child[0])
    bx = [parent[1][i] * child[1][0] + parent[2][i] * child[1][1] for i in range(2)]
    by = [parent[1][i] * child[2][0] + parent[2][i] * child[2][1] for i in range(2)]
    return [pos, bx, by]

def group_metrics(group, sprites, limits):
    """Full affine/all-cell-corner bounds, charging each reused instance once."""
    require(isinstance(group, dict) and set(group) <= {'name', 'parts', 'metrics'}, 'Invalid group fields')
    name(group.get('name'))
    parts = group.get('parts')
    require(isinstance(parts, list) and 1 <= len(parts) <= limits['group_transforms'], 'Group transform budget exceeded or group empty')
    known = {}
    cells = 0
    coords = []
    for part in parts:
        require(isinstance(part, dict) and set(part) <= {'name', 'sprite', 'parent', 'transform'}, 'Invalid group part fields')
        label = name(part.get('name'))
        require(label not in known, 'Duplicate part name')
        sprite = sprites.get(name(part.get('sprite')))
        require(sprite is not None, 'Unknown group sprite')
        local = affine(part.get('transform', {}))
        parent = part.get('parent')
        require(parent is None or isinstance(parent, str), 'Parent must be a part identifier')
        require(parent is None or parent in known, 'Parent must be an earlier part; cycles/forward references are forbidden')
        t = compose(known[parent], local) if parent else local
        require(all((math.isfinite(v) and abs(v) <= limits['extent'] for vec in t for v in vec)), 'Composed transform is nonfinite/out of bounds')
        known[label] = t
        cells += sum((i != 0 for i in sprite['indices']))
        require(cells <= limits['group_cells'], 'Whole-group occupied-cell budget exceeded')
        w, h = (sprite['width'], sprite['height'])
        for i, index in enumerate(sprite['indices']):
            if index:
                x, y = (i % w - w / 2, i // w - h / 2)
                coords.extend((point(t, [x + dx, y + dy]) for dx, dy in ((0, 0), (1, 0), (0, 1), (1, 1))))
    require(all((math.isfinite(v) and abs(v) <= limits['extent'] for p in coords for v in p)), 'Transformed occupied-cell corners exceed extent')
    return {'occupied_cells': cells, 'transforms': len(parts), 'occupied_bounds': [min((p[0] for p in coords)), min((p[1] for p in coords)), max((p[0] for p in coords)), max((p[1] for p in coords))]}

def validate(atlas):
    require(isinstance(atlas, dict) and type(atlas.get('version')) is int and (atlas.get('version') == 1), 'Expected engine pack version1')
    palette = atlas.get('palette')
    require(isinstance(palette, list) and 2 <= len(palette) <= 256 and (palette[0] == 0), 'Palette needs 2..256 entries and transparent index0')
    for i, c in enumerate(palette):
        integer(c, 0, 4294967295, 'RGBA color')
        require(i == 0 or c >> 24 == 255, 'Nonzero palette entries must be opaque')
    sprites = atlas.get('sprites')
    require(isinstance(sprites, dict) and 1 <= len(sprites) <= 256, 'Pack needs 1..256 sprites')
    summary = {}
    total = 0
    for key, s in sprites.items():
        name(key)
        require(isinstance(s, dict), 'Sprite must be an object')
        w = integer(s.get('width'), 1, 256, 'width')
        h = integer(s.get('height'), 1, 256, 'height')
        values = s.get('indices')
        require(isinstance(values, list) and len(values) == w * h, 'Indices must exactly match row-major dimensions')
        for i in values:
            integer(i, 0, len(palette) - 1, 'palette index')
        t = topology(values, w, h)
        require(t['occupied'] > 0, 'Empty sprite is not an engine object')
        total += w * h
        require(total <= 1048576, 'Total pack cell-storage work budget exceeded')
        summary[key] = t
    metadata = atlas.get('asset_metadata')
    if metadata is not None:
        require(isinstance(metadata, dict) and type(metadata.get('schema')) is int and (metadata.get('schema') == 1), 'Unknown asset metadata schema')
        limits = budget(metadata.get('budgets', {}))
        records = metadata.get('sprites', {})
        require(isinstance(records, dict) and set(records) == set(sprites), 'Metadata must describe exactly the packed sprites')
        for key, t in summary.items():
            require(isinstance(records[key], dict), 'Sprite metadata must be an object')
            require(t['occupied'] <= limits['sprite_cells'], 'Sprite occupied-cell budget exceeded')
            require(records[key].get('topology') in ('solid', 'holes-allowed'), 'Invalid metadata topology')
            if 'metrics' in records[key]:
                require(records[key]['metrics'] == t, 'Stale sprite metrics')
            vector(records[key].get('anchor', [0, 0]), 2, 'metadata anchor')
            if records[key].get('topology') == 'solid':
                require(t['components'] == 1 and t['hole_cells'] == 0, 'Solid sprite has disconnected cells or an enclosed hole')
        groups = metadata.get('groups', [])
        require(isinstance(groups, list) and len(groups) <= 256, 'Too many groups')
        group_names = set()
        for g in groups:
            require(isinstance(g, dict), 'Group must be an object')
            name(g.get('name'))
            require(g['name'] not in group_names, 'Duplicate group name')
            group_names.add(g['name'])
            actual = group_metrics(g, sprites, limits)
            if 'metrics' in g:
                require(actual == g['metrics'], 'Stale group metrics')
        animations = metadata.get('animations', [])
        require(isinstance(animations, list) and len(animations) <= 256, 'Too many animations')
        seen = set()
        for a in animations:
            require(isinstance(a, dict) and set(a) <= {'name', 'loop', 'frames'}, 'Invalid animation fields')
            require(name(a.get('name')) not in seen, 'Duplicate animation name')
            seen.add(a['name'])
            require(type(a.get('loop')) is bool, 'Animation loop must be boolean')
            require(isinstance(a.get('frames'), list) and 1 <= len(a['frames']) <= limits['animation_frames'], 'Animation frame budget exceeded')
            for f in a['frames']:
                require(isinstance(f, dict) and set(f) <= {'kind', 'name', 'duration_ms'}, 'Invalid animation frame fields')
                require(f.get('kind') in ('sprite', 'group'), 'Animation frame kind must be sprite/group')
                name(f.get('name'))
                require(f.get('name') in (sprites if f['kind'] == 'sprite' else group_names), 'Animation references unknown frame')
                integer(f.get('duration_ms'), 1, 60000, 'frame duration_ms')
        if 'cell_content_sha256' in metadata:
            core = {k: atlas[k] for k in ('version', 'palette', 'sprites')}
            actual = hashlib.sha256(json.dumps(core, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
            require(actual == metadata['cell_content_sha256'], 'Stale cell content fingerprint')
    return summary

def budget(given):
    require(isinstance(given, dict) and set(given) <= set(LIMITS), 'Unknown budget field')
    result = {**LIMITS, **given}
    for k, v in result.items():
        integer(v, 1, LIMITS[k], f'budget {k}')
    return result

def pack(manifest_path):
    manifest = read_json(manifest_path)
    require(isinstance(manifest, dict), 'Manifest must be an object')
    require(type(manifest.get('schema')) is int and manifest.get('schema') == 1, 'Expected authoring manifest schema1')
    entries = manifest.get('sprites')
    require(isinstance(entries, list) and 1 <= len(entries) <= 256, 'Manifest needs 1..256 sprites')
    defaults = manifest.get('defaults', {})
    require(isinstance(defaults, dict), 'defaults must be an object')
    limits = budget(manifest.get('budgets', {}))
    config = manifest.get('palette', {'mode': 'exact', 'max_colors': 64})
    require(isinstance(config, dict) and config.get('mode') in ('exact', 'provided', 'quantize'), 'palette mode must be exact/provided/quantize')
    require(set(config) <= {'mode', 'max_colors', 'colors'} and (config['mode'] == 'provided' or 'colors' not in config), 'Unknown or inapplicable palette policy field')
    count = integer(config.get('max_colors', 64), 2, 256, 'max_colors')
    palette = config.get('colors') if config['mode'] == 'provided' else None
    require(config['mode'] != 'provided' or palette is not None, 'Provided palette requires explicit colors')
    if palette is not None:
        validate({'version': 1, 'palette': palette, 'sprites': {'check': {'width': 1, 'height': 1, 'indices': [1]}}})
        require(len(palette) <= count, 'Provided palette exceeds max_colors')
    raw = {}
    records = {}
    policies = {}
    decoded_source_pixels = 0
    stored_cells = 0
    for s in entries:
        require(isinstance(s, dict), 'Manifest sprite must be an object')
        key = name(s.get('name'))
        require(key not in raw, 'Duplicate sprite name')
        require(isinstance(s.get('source'), str) and s['source'], 'Sprite source must be a PNG path')
        source = manifest_path.parent / s['source']
        p = policy(s, defaults)
        frame, record = import_png(source, s, p, palette, limits['source_pixels'] - decoded_source_pixels)
        w, h, _ = frame
        decoded_source_pixels += record['source_dimensions'][0] * record['source_dimensions'][1]
        stored_cells += w * h
        require(decoded_source_pixels <= limits['source_pixels'], 'Total decoded source pixel-work budget exceeded')
        require(stored_cells <= 1048576, 'Total pack cell-storage work budget exceeded')
        require(1 <= w <= 256 and 1 <= h <= 256, 'Sprite dimensions exceed 256 cells; resizing must be explicit')
        raw[key] = frame
        policies[key] = p
        require(sum((w * h for w, h, _ in raw.values())) <= 1048576, 'Total imported cell-storage work budget exceeded')
        records[key] = {'source': s['source'], 'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(), 'policies': p, 'topology': p['topology'], **record}
    quantized = {}
    if palette is None:
        colors = sorted({color(c) for key, (_, _, values) in raw.items() for c in values if c[3]})
        if config['mode'] == 'exact':
            require(len(colors) < count, 'Too many colors for exact palette; quantization must be explicit')
            palette = [0] + colors
            if colors:
                palette += [colors[0]] * (count - len(palette))
        else:
            require(colors, 'No occupied colors')
            samples = [c[:3] for _, _, values in raw.values() for c in values if c[3]]
            im = Image.new('RGB', (len(samples), 1))
            im.putdata(samples)
            q = im.quantize(colors=count - 1, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
            qp = q.getpalette()
            palette = [0] + [color((*qp[i * 3:i * 3 + 3], 255)) for i in range(count - 1)]
            values = iter(q.get_flattened_data())
            for key, (_, _, pixels) in raw.items():
                quantized[key] = [next(values) + 1 if c[3] else 0 for c in pixels]
    lookup = {c: i for i, c in reversed(list(enumerate(palette)))}
    sprites = {}
    for key, (w, h, values) in raw.items():
        if policies[key]['encoding'] == 'indices':
            indices = values
        elif key in quantized:
            indices = quantized[key]
        else:
            require(all((color(c) in lookup for c in values)), 'Source color absent from exact provided palette')
            indices = [lookup[color(c)] for c in values]
        t = topology(indices, w, h)
        require(t['occupied'] <= limits['sprite_cells'], 'Sprite work budget exceeded')
        if policies[key]['topology'] == 'solid':
            require(t['components'] == 1 and t['hole_cells'] == 0, 'Born-solid sprite is disconnected or has enclosed transparent holes')
        changed = sum((color(c) != palette[i] for c, i in zip(values, indices))) if policies[key]['encoding'] != 'indices' else 0
        sprites[key] = {'width': w, 'height': h, 'indices': indices}
        records[key].update({'metrics': t, 'quantized_color_cells': changed, 'anchor': vector(next((s for s in entries if s['name'] == key)).get('anchor', [0, 0]), 2, 'anchor')})
    groups = manifest.get('groups', [])
    require(isinstance(groups, list) and len(groups) <= 256, 'Too many groups')
    for g in groups:
        g['metrics'] = group_metrics(g, sprites, limits)
    result = {'version': 1, 'palette': palette, 'sprites': sprites, 'asset_metadata': {'schema': 1, 'tool': {'name': 'asset-pack', 'version': TOOL_VERSION, 'pillow': pillow_version}, 'manifest_sha256': hashlib.sha256(manifest_path.read_bytes()).hexdigest(), 'provenance': manifest.get('provenance', {}), 'palette_policy': config, 'budgets': limits, 'sprites': records, 'groups': groups, 'animations': manifest.get('animations', [])}}
    extensions = manifest.get('extensions', {})
    require(isinstance(extensions, dict) and (not set(extensions) & set(result)), 'Opaque extension fields cannot replace the cell ABI/metadata')
    result.update(extensions)
    result['asset_metadata']['opaque_extension_fields'] = sorted(extensions)
    validate(result)
    core = {k: result[k] for k in ('version', 'palette', 'sprites')}
    result['asset_metadata']['cell_content_sha256'] = hashlib.sha256(json.dumps(core, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    return result

def unpack(atlas, folder):
    validate(atlas)
    require(not folder.exists() or not any(folder.iterdir()), 'Unpack output directory must be absent or empty')
    folder.mkdir(parents=True, exist_ok=True)
    palette = atlas['palette']
    rgb = [v for c in palette for v in rgba(c)[:3]]
    rows = []
    for key, s in atlas['sprites'].items():
        im = Image.new('P', (s['width'], s['height']))
        im.putpalette(rgb)
        im.info['transparency'] = bytes((rgba(c)[3] for c in palette))
        im.putdata(s['indices'])
        im.save(folder / f'{key}.png', optimize=False, bits=8)
        rows.append({'name': key, 'source': f'{key}.png', 'encoding': 'indices', 'topology': 'holes-allowed'})
    old = atlas.get('asset_metadata', {})
    manifest = {'schema': 1, 'palette': {'mode': 'provided', 'max_colors': len(palette), 'colors': palette}, 'sprites': rows, 'groups': old.get('groups', []), 'animations': old.get('animations', []), 'provenance': {'source_type': 'indexed-pack-roundtrip'}, 'extensions': {k: v for k, v in atlas.items() if k not in ('version', 'palette', 'sprites', 'asset_metadata')}}
    if 'budgets' in old:
        manifest['budgets'] = old['budgets']
    for s in rows:
        prior = old.get('sprites', {}).get(s['name'], {})
        s['topology'] = prior.get('topology', 'holes-allowed')
        s['anchor'] = prior.get('anchor', [0, 0])
    write_json(folder / 'manifest.json', manifest)
    return folder / 'manifest.json'

def preview(atlas, path, zoom):
    summary = validate(atlas)
    integer(zoom, 1, 16, 'preview zoom')
    side = max((max(s['width'], s['height']) for s in atlas['sprites'].values())) * zoom
    cols = min(4, len(summary))
    tile = side + 32
    height = side + 52
    require(cols * tile * math.ceil(len(summary) / cols) * height <= MAX_SOURCE_PIXELS, 'Preview exceeds pixel budget; use lower zoom or split pack')
    image = Image.new('RGB', (cols * tile, math.ceil(len(summary) / cols) * height), (6, 12, 20))
    draw = ImageDraw.Draw(image)
    for n, (key, s) in enumerate(atlas['sprites'].items()):
        x = n % cols * tile + 16
        y = n // cols * height + 32
        im = Image.new('RGBA', (s['width'], s['height']))
        im.putdata([rgba(atlas['palette'][i]) for i in s['indices']])
        im = im.resize((im.width * zoom, im.height * zoom), Image.Resampling.NEAREST)
        image.paste(im, (x, y), im)
        draw.text((x, y - 20), f"{key} / {summary[key]['occupied']} cells", fill=(173, 208, 214))
    path.parent.mkdir(parents=True, exist_ok=True)
    image.save(path)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    p = commands.add_parser('pack')
    p.add_argument('--manifest', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p = commands.add_parser('unpack')
    p.add_argument('--input', type=Path, required=True)
    p.add_argument('--output-dir', type=Path, required=True)
    p = commands.add_parser('validate')
    p.add_argument('--input', type=Path, required=True)
    p = commands.add_parser('preview')
    p.add_argument('--input', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--zoom', type=int, default=4)
    args = parser.parse_args()
    try:
        if args.command == 'pack':
            require(args.manifest.resolve() != args.output.resolve(), 'Input and output must differ')
            require(all(((args.manifest.parent / s['source']).resolve() != args.output.resolve() for s in read_json(args.manifest)['sprites'])), 'Output must not overwrite a source PNG')
            atlas = pack(args.manifest)
            write_json(args.output, atlas)
            print(json.dumps({'pack': str(args.output), 'sprites': validate(atlas)}))
        elif args.command == 'unpack':
            print(unpack(read_json(args.input), args.output_dir))
        elif args.command == 'validate':
            print(json.dumps(validate(read_json(args.input)), indent=2))
        else:
            require(args.input.resolve() != args.output.resolve(), 'Input and output must differ')
            preview(read_json(args.input), args.output, args.zoom)
            print(args.output)
    except (AssetError, OSError, KeyError, TypeError, json.JSONDecodeError) as e:
        parser.exit(1, f'Asset operation failed: {e}\n')
if __name__ == '__main__':
    main()
