from pathlib import Path
import hashlib,json
work=Path('work/coop-wake-presentation-review')
author=Path('C:/Users/mitch/.codex/worktrees/galaxy-pixel-art/galaxy-royale/work/coop-wake-presentation-design')
pins=json.loads((author/'ADDENDUM-v2-pins.json').read_text(encoding='utf-8-sig'))
for name,expected in [('DESIGN.md',pins['original_design_sha256']),('source-pins.json',pins['original_source_pins_sha256']),('ADDENDUM-v2.md',pins['addendum']['sha256'])]:
    data=(author/name).read_bytes()
    assert hashlib.sha256(data).hexdigest()==expected,name
    if name=='ADDENDUM-v2.md':
        assert len(data)==pins['addendum']['bytes']
        (work/'author-design'/name).write_bytes(data)
    else:
        assert data==(work/'author-design'/name).read_bytes(),name
(work/'author-design/ADDENDUM-v2-pins.json').write_bytes((author/'ADDENDUM-v2-pins.json').read_bytes())
draft=(work/'REVIEW-DRAFT.md').read_text(encoding='utf-8')
s=draft.replace('## Three contract refinements before implementation', '## Original findings, resolved by the pinned version-two addendum')
s=s.replace('All three findings were sent directly to the author and Root. An explicit versioned resolution should be accepted before coding; preserve the frozen proposal input bytes.', '''All three findings were sent directly to the author and Root. The author preserved the frozen original bytes and added ADDENDUM-v2.md (8,287 bytes; SHA-256 76790eca3f4560c5081c2154776c33e95b20c5e08b6c513e04a7fc937e148280) plus ADDENDUM-v2-pins.json (SHA-256 98d77e3a875651e2fa093f7b1de2e53f6bdddfd9071b164809d89119c7d29ea4). This final review independently checked those exact inputs and the unchanged original pins.

The refinements are now resolved as a contract: raw finite f64 positive/max life plus positive finite canonical f32 conversion, mirrored by JS raw bounds and Math.fround; new point coordinates alone receive the exact 1,000,014 raw/canonical bound; surviving ID+epoch same-tick complete sets compare validated canonical f32 captured positions/life and stable birth keys, distinct from receiver-aged deadlines. Same-tick empty/legacy/positive set mutation rejects before Scene/cache moves. Retired memberships/new epochs clear independently; strictly newer successful ticks may clear or revalidate actual remaining state. No unbounded tombstone history or gameplay sample clamp is added.

**Final verdict: no remaining design blocker for this bounded persistent-dot implementation when the original proposal is read together with ADDENDUM-v2.** This approves a source-backed implementation direction, not implemented/runtime behavior. Root owns authorization, strict JS/controller work and later integration; future code must preserve Root332ddd5's semantic-equivalent hit_owner.replace(owner) amendment.''')
s=s.replace('No runtime approval is recorded in this draft. Final design acceptance depends on the three explicit refinements;', 'No runtime approval is recorded in this review. The three refinements are resolved by the verified addendum;')
s=s.replace('The author DESIGN has SHA-256', 'The original author DESIGN has SHA-256')
s=s.replace('The proposed direction is appropriate for a bounded first implementation:', 'The final design direction, with version-two refinements, is appropriate for a bounded first implementation:')
(work/'REVIEW.md').write_text(s,encoding='utf-8')
manifest={'version':1,'scope':'Independent source and design/addendum review; no runtime implementation/test/build/browser/RTC execution','native_source_base':'87de1f7afad1a53183fcca91e146c9a59ee012cd','author_design_sha256':pins['original_design_sha256'],'author_source_pins_sha256':pins['original_source_pins_sha256'],'author_addendum_sha256':pins['addendum']['sha256'],'raw_and_canonical_semantics':'source snapshots are exact canonical Git blob bytes, raw author hashes are separately verified-source.json; all evidence hashes below are stored raw bytes','files':[]}
for p in sorted(work.rglob('*')):
    if not p.is_file() or p.name=='MANIFEST.json':continue
    data=p.read_bytes()
    manifest['files'].append({'path':p.relative_to(work).as_posix(),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
(work/'MANIFEST.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
for name in ['REVIEW.md','MANIFEST.json','verified-source.json']:
    print(name,hashlib.sha256((work/name).read_bytes()).hexdigest())
print('PIN_COUNT',len(manifest['files']))
