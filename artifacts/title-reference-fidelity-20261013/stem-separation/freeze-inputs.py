from pathlib import Path
import hashlib
import json
import importlib.metadata as metadata

root = Path(__file__).resolve().parent
locations = [root / 'dependencies', Path('C:/AI/ComfyUI/.venv/Lib/site-packages')]
names = {'demucs', 'julius', 'sphn', 'lameenc', 'torch', 'numpy', 'scipy', 'einops', 'huggingface-hub', 'safetensors', 'pyyaml', 'tqdm'}
packages = []
def pin(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path).replace('\\', '/'), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
for location in locations:
    for distribution in metadata.distributions(path=[str(location)]):
        name = distribution.metadata['Name'].lower().replace('_', '-')
        if name not in names:
            continue
        records = [pin(distribution.locate_file(file)) for file in distribution.files or []
                   if str(file).endswith(('.dist-info/METADATA', '.dist-info/RECORD'))]
        packages.append({'name': name, 'version': distribution.version, 'metadata': records})
assert names == {item['name'] for item in packages}
files = [pin(path) for path in root.rglob('*') if path.is_file() and
         path.suffix in ('.py', '.json', '.log', '.wav') and 'dependencies' not in path.parts and path.name != 'input-pins.json']
result = {'scope': 'Actual installed separation dependencies and retained input/output bytes; source audio/model stay outside shipped game and repository assets.', 'packages': packages, 'files': files}
(root/'input-pins.json').write_text(json.dumps(result, indent=2)+'\n', encoding='utf-8')
print(json.dumps({'packages': len(packages), 'retained_files': len(files)}))
