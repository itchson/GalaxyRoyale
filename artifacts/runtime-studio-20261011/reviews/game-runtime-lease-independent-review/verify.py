from pathlib import Path
import json,hashlib,subprocess
b=Path(__file__).resolve().parent;r=b.parents[1]
m=json.loads((b/"manifest.json").read_text(encoding="utf-8"))
for x in m["files"]:
 d=(b/x["path"]).read_bytes();assert len(d)==x["bytes"] and hashlib.sha256(d).hexdigest()==x["sha256"],x["path"]
s=json.loads((b/"source-pins.json").read_text(encoding="utf-8"))
for x in s["inputs"]:
 d=subprocess.check_output(["git","show",x["commit"]+":"+x["path"]],cwd=r);assert len(d)==x["bytes"] and hashlib.sha256(d).hexdigest()==x["sha256"],x["path"]
e=json.loads((b/"execution-inputs.json").read_text(encoding="utf-8"))
d=subprocess.check_output(["git","show",e["permanent_test_commit"]+":"+e["permanent_test_path"]],cwd=r);assert hashlib.sha256(d).hexdigest()==e["permanent_test_sha256"]
print(f"VERIFIED {len(m['files'])} frozen raw files, {len(s['inputs'])} original/repair Git inputs and exact permanent-test commit; no test rerun")
