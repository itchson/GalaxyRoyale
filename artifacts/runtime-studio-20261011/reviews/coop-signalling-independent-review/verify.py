from pathlib import Path
import hashlib,json,subprocess
b=Path(__file__).resolve().parent;r=b.parents[1]
m=json.loads((b/"manifest.json").read_text(encoding="utf-8"))
for x in m["files"]:
 d=(b/x["path"]).read_bytes();assert len(d)==x["bytes"] and hashlib.sha256(d).hexdigest()==x["sha256"],x["path"]
s=json.loads((b/"peer-verification.json").read_text(encoding="utf-8"))
for x in s["inputs"]:
 d=subprocess.check_output(["git","show",s["commit"]+":"+x["path"]],cwd=r);assert len(d)==x["bytes"] and hashlib.sha256(d).hexdigest()==x["sha256"],x["path"]
print(f"VERIFIED {len(m['files'])} frozen raw files and {len(s['inputs'])} exact committed inputs; no tests rerun")
