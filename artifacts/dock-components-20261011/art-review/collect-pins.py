from pathlib import Path
import hashlib, json, subprocess

OUT=Path(__file__).resolve().parent
ROOT=Path('C:/Users/mitch/Documents/ChatGPT/galaxy-royale')
CORE=Path('C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale')
ART=Path('C:/Users/mitch/.codex/worktrees/galaxy-pixel-art/galaxy-royale')

def pin(root, rel, commit=None):
    raw=(root/rel).read_bytes()
    result={'path':str(root/rel),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),
            'canonicalLfSha256':hashlib.sha256(raw.replace(b'\r\n',b'\n')).hexdigest()}
    if commit:
        committed=subprocess.check_output(['git','show',commit+':'+rel],cwd=root)
        result['committedCanonicalMatch']=raw.replace(b'\r\n',b'\n')==committed.replace(b'\r\n',b'\n')
        assert result['committedCanonicalMatch'],rel
    return result

data={'scope':'Independent source/native transaction and shell VM review; no browser or balance acceptance',
      'coreCommit':'373a1fd16ce02b7879cf95122087f555973d4558',
      'componentArtCommit':'2d2bcabe18bf9eb4155419fad64f0d5341548ac3',
      'rootHeadAtFinalRead':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
      'core':[pin(CORE,p,'373a1fd16ce02b7879cf95122087f555973d4558') for p in [
        'crates/game_core/src/components.rs','crates/game_core/src/component_tests.rs',
        'crates/game_core/src/campaign.rs','crates/game_core/src/ship_progression.rs',
        'crates/game_core/src/lib.rs','docs/ABILITY_COMPONENTS.md']],
      'shell':[pin(ROOT,p) for p in ['apps/game/src/loadout.ts','apps/game/src/loadout.css',
        'apps/game/src/main.ts','scripts/test/loadout.test.mjs','scripts/test/forge-integration.test.mjs']],
      'art':[pin(ART,'crates/game_web/src/component_art.rs','2d2bcabe18bf9eb4155419fad64f0d5341548ac3')],
      'evidence':[pin(OUT,p) for p in ['native-tests.log','shell-tests.log']]}
(OUT/'pins.json').write_text(json.dumps(data,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps({'root':data['rootHeadAtFinalRead'],'corePins':len(data['core']),
  'shellPins':len(data['shell']),'allCommittedCoreMatched':True}))
