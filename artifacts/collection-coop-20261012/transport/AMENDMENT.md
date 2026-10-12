# Native metadata and actual browser-origin correction

Frozen follow-up `38742784f6000135a68354750bdf0d3204334422` atop `4f89b53d86711699acb6f912fb3d64f7720aac3c`; clean tracked checkout. Five owned files only. Original HANDOFF and first evidence remain historical and unchanged.

Engine's independent actual public-Game probe found a legitimate page absent from the original 64-row fixture: naturally spawned Coil segments have surviving hull15 and unknown maximum0. Exact whole page SHA256 `2f98830878dc86104acca83267414d4afc889f590020e7a295799378478e33bb`, tick/revision1460, four segments. This follow-up permits hull>max only for kind=`segment` and max_hull=`0`. All finite/dimension/type bounds remain. The first actual row is committed into the regression; negative enemy,nonzero insufficient maximum,negative/nonfinite health cases still reject. The complete peer-exported page passes exact encode/decode at18,518 framed bytes; copied byte-identically into native-fixtures. This is wire interoperability,not our own native simulation execution.

Root reported the first actual same-page browser run failed before ready: service correctly rejects Origin-less GET polling,while a same-origin browser GET does not send Origin. The tool now prints page `http://localhost:4189` and explicitly uses service `http://127.0.0.1:4189` on the **same single listener/process**. Allowed Origin is exactly localhost; Host guard permits only localhost and127.0.0.1 at the configured port. Config carries explicit page/service origins;CSP connect allows only self +that service. Service security policy is unchanged. Do not open the page through127.0.0.1;page check visibly fails instead of inventing readiness. The actual HTTP helper test proves credentialed polling without Origin remains403 and with the exact page Origin succeeds200.

Executed current focused suite **27/27**; TypeScript and diff check pass. All test listeners close. `amendment-final-test.log`, `amendment-typecheck.log`, `segment-compatibility.json` are owner execution; native page provenance belongs to Engine's public probe. Browser source freeze is ready for Root to run again. No successful native RTC execution claimed yet, no NAT/separate-network/shared-gameplay/save integration claim.

After integrating and stopping the old verified QA service, prepare a NEW stage so first failed captures are untouched:

```powershell
node scripts/qa/co-op-transport/prepare.mjs --stage work/co-op-transport/stage-amendment
node scripts/qa/co-op-transport/server.mjs --stage work/co-op-transport/stage-amendment --port 4189 --max-ms 1200000
```

Root CUA opens `http://localhost:4189`; Connect→Send join/input fixture→Host end→Rejoin→Dispose. Declared admission fixture still leaves simulation waiting. Root owns browser/listener lifecycle; this task leaves no running instance. SIGINT/SIGTERM/20-minute auto-stop remain. The local Spark worker returned no substantive audit; no peer approval is claimed from that worker.
