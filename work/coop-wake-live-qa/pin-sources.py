import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path.cwd().resolve()
work = root / 'work/coop-wake-live-qa'
base = '8b0896d7ccd2c4dece1dc7a0d2cae6c4986505ab'
fixture = '87d92f2cbf10c24dda585b47f05de2fad6fe178f'
def git(*args):
    return subprocess.check_output(['git', *args], cwd=root)
def pin(data):
    return {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
def raw_pin(name):
    return pin((root / name).read_bytes())
changed = git('diff', '--cached', '--name-only').decode().splitlines()
assert len(changed) == 14
assert all(n.startswith('scripts/qa/co-op-transport/') or n in [
    'scripts/test/coop-wake-live-qa.test.mjs', 'scripts/test/coop-transport-fifth-qa.test.mjs',
    'scripts/test/coop-transport-wasm-qa.test.mjs'] for n in changed)
sources = []
for name in changed:
    raw = (root / name).read_bytes()
    canonical = git('show', ':' + name)
    assert raw.replace(b'\r\n', b'\n') == canonical.replace(b'\r\n', b'\n')
    sources.append({'path': name, 'raw': pin(raw), 'index_canonical': pin(canonical)})
unchanged = [
    'apps/game/src/storage.ts', 'apps/game/src/main.ts',
    *['apps/game/src/coop/' + n + '.ts' for n in ['controller', 'wire', 'wasm-ports', 'transport', 'signaling', 'room-code']],
    'apps/coop/server.mjs', 'apps/coop/store.mjs',
    *['crates/game_core/src/' + n + '.rs' for n in ['co_op', 'co_op_control', 'co_op_loadout', 'co_op_wake', 'co_op_wake_fx', 'co_op_publish', 'co_op_replica', 'co_op_fx', 'co_op_world_bounds', 'identity', 'lib']],
    *['crates/game_web/src/' + n + '.rs' for n in ['browser', 'co_op_bridge', 'co_op_replica_bridge', 'co_op_bridge_wake_tests']],
    'scripts/qa/co-op-transport/fifth-shot.js', 'scripts/qa/co-op-transport/client.html',
    'scripts/test/coop-private-room-provenance-qa.test.mjs', 'tsconfig.json', 'package.json',
]
inputs = []
for name in unchanged:
    canonical = git('show', base + ':' + name)
    assert git('show', 'HEAD:' + name) == canonical
    assert (root / name).read_bytes().replace(b'\r\n', b'\n') == canonical.replace(b'\r\n', b'\n')
    inputs.append({'path': name, 'raw': raw_pin(name), 'git_base_canonical': pin(canonical)})
original = git('ls-tree', '-r', '--name-only', fixture, 'work/coop-wake-live-fixtures/native-fixtures').decode().splitlines()
original += ['crates/game_web/src/co_op_bridge_wake_tests.rs', 'work/coop-wake-live-fixtures/HANDOFF.md', 'work/coop-wake-live-fixtures/MANIFEST.json']
original_pins = [{'path': n, **pin(git('show', fixture + ':' + n))} for n in original]
for role in ['host', 'guest']:
    assert (root / f'scripts/qa/co-op-transport/fixtures/configured-wake/{role}-profile.json').read_bytes() == git('show', fixture + f':work/coop-wake-live-fixtures/native-fixtures/{role}-profile.json')
refs = {}
for branch in ['coop-wake-live-fixtures', 'coop-wake-browser', 'coop-wake-presentation', 'coop-wake-sanctuary-order']:
    refs['codex/' + branch] = git('rev-parse', 'codex/' + branch).decode().strip()
out = {'version': 1, 'observed_at': datetime.now(timezone.utc).isoformat(),
       'scope': 'Actual raw working files and staged canonical blobs; no compiled/WASM/RTC/live input assertion.',
       'base': base, 'branch': git('branch', '--show-current').decode().strip(),
       'branch_creation_reflog': git('reflog', 'show', 'codex/coop-wake-live-qa', '--format=%H %gs', '-n', '2').decode().splitlines(),
       'preserved_refs': refs, 'changed_sources': sources, 'unchanged_inputs': inputs,
       'original_fixture_commit': fixture, 'original_fixture_canonical_inputs': original_pins}
(work / 'source-pins.json').write_text(json.dumps(out, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps({'changed_sources': len(sources), 'unchanged_inputs': len(inputs), 'original_fixture_inputs': len(original_pins), 'original_profiles_byte_exact': True}))
