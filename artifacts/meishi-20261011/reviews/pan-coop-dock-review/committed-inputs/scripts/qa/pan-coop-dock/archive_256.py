"""Lossless archival of the three actual native proof exports, not runtime data.

Zero timestamp/empty filename make the gzip header stable. Compressed bytes
are pinned evidence, not promised equal across different zlib versions.
"""
import gzip
import hashlib
import importlib.util
import io
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location("dock256", Path(__file__).with_name("convert_256.py"))
N = importlib.util.module_from_spec(spec)
spec.loader.exec_module(N)
NAMES = ("native-extraction.json", "rig-extraction.json", "zoom-extraction.json")
MAX_BYTES = 32 * 1024 * 1024


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encode(raw):
    N.M.AP.require(len(raw) <= MAX_BYTES, "Native evidence exceeds bounded decode limit")
    buffer = io.BytesIO()
    with gzip.GzipFile(fileobj=buffer, filename="", mode="wb", mtime=0, compresslevel=9) as out:
        out.write(raw)
    return buffer.getvalue()


def decode(encoded, record):
    N.M.AP.require(digest(encoded) == record["gzip_sha256"], "Encoded evidence hash differs")
    N.M.AP.require(len(encoded) == record["gzip_bytes"], "Encoded evidence size differs")
    with gzip.GzipFile(fileobj=io.BytesIO(encoded), mode="rb") as source:
        raw = source.read(MAX_BYTES + 1)
    N.M.AP.require(len(raw) <= MAX_BYTES, "Native evidence exceeds bounded decode limit")
    N.M.AP.require(len(raw) == record["decoded_bytes"], "Decoded evidence size differs")
    N.M.AP.require(digest(raw) == record["decoded_sha256"], "Decoded evidence hash differs")
    return raw


def read(folder, name):
    record = N.M.AP.read_json(folder / "archive.json")["files"][name]
    return json.loads(decode((folder / (name + ".gz")).read_bytes(), record))


def archive(folder=N.DEFAULT):
    records = {}
    for name in NAMES:
        raw = (folder / name).read_bytes()
        encoded = encode(raw)
        record = {"decoded_bytes": len(raw), "decoded_sha256": digest(raw),
                  "gzip_bytes": len(encoded), "gzip_sha256": digest(encoded)}
        N.M.AP.require(decode(encoded, record) == raw, "Evidence archive changed bytes")
        (folder / (name + ".gz")).write_bytes(encoded)
        records[name] = record
    N.M.write_json(folder / "archive.json", {
        "format": 1, "codec": "gzip", "timestamp": 0, "max_decoded_bytes_per_file": MAX_BYTES,
        "scope": "Exact native proof bytes; no loss, runtime pack or geometry changes",
        "files": records,
    })
    print(json.dumps(records, indent=2))


if __name__ == "__main__":
    archive()
