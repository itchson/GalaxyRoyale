import {readFileSync,writeFileSync} from "node:fs";
const path="C:/Users/mitch/.codex/worktrees/galaxy-pixel-art/galaxy-royale/scripts/test/fixtures/co-op-nova-fx/maximum-header.json";
const text=readFileSync(path,"utf8"), h=JSON.parse(text);
const frame={v:1,session:h.snapshot.session,seq:1,kind:"publication",payload:h};
const observed={scope:"Read/JSON re-encode existing native-exported fixture only; no tests, native execution, transport, or candidate encoding",fixture:path,pretty_file_bytes:Buffer.byteLength(text),compact_header_bytes:Buffer.byteLength(JSON.stringify(h)),compact_publication_frame_bytes:Buffer.byteLength(JSON.stringify(frame)),frame_limit_bytes:65536,remaining_frame_bytes:65536-Buffer.byteLength(JSON.stringify(frame)),participants:h.snapshot.participants.length,projectiles:h.snapshot.projectiles.length,nova_records:h.snapshot.nova_fx.records.length};
writeFileSync(new URL("./fixture-size-observation.json",import.meta.url),JSON.stringify(observed,null,2)+"\n");
console.log(JSON.stringify(observed));