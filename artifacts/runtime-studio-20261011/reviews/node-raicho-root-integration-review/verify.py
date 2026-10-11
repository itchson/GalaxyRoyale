from pathlib import Path
import json,hashlib,subprocess
base=Path(__file__).resolve().parent
repo=base.parents[1]
m=json.loads((base/"manifest.json").read_text(encoding="utf-8"))
for x in m["files"]:
 d=(base/x["path"]).read_bytes();assert len(d)==x["bytes"] and hashlib.sha256(d).hexdigest()==x["sha256"],x["path"]
s=json.loads((base/"source-checks.json").read_text(encoding="utf-8"))
def blob(c,p):return subprocess.check_output(["git","show",c+":"+p],cwd=repo)
for x in s["inputs"]:
 d=blob(s["commits"]["review_head"],x["path"]);assert len(d)==x["bytes"] and hashlib.sha256(d).hexdigest()==x["sha256"],x["path"]
for x in s["candidate_comparisons"]:assert blob(x["original_commit"],x["path"])==blob(x["integrated_commit"],x["path"]),x["path"]
print(f"VERIFIED {len(m['files'])} frozen raw files, {len(s['inputs'])} exact current Git blobs, {len(s['candidate_comparisons'])} exact candidate comparisons; no build/test/browser")
