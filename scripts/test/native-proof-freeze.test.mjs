import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { freezeNativeProof } from "../qa/freeze-native-proof.mjs";

async function fixture(t) {
  const workspace = await fs.mkdtemp(
    path.join(os.tmpdir(), "galaxy-native-proof-"),
  );
  await fs.mkdir(path.join(workspace, "work"));
  const source = path.join(workspace, "mutable.exe");
  await fs.writeFile(
    source,
    "MZ labelled test bytes, not a compiled executable",
  );
  const options = { workspace, source, output: "work/frozen" };
  t.after(async () => {
    for (const name of await fs
      .readdir(path.join(workspace, "work/frozen"))
      .catch(() => []))
      await fs.chmod(path.join(workspace, "work/frozen", name), 0o666);
    assert.equal(path.dirname(workspace), os.tmpdir());
    await fs.rm(workspace, { recursive: true, force: true });
  });
  return options;
}
test("reusing a mutable native target cannot overwrite its previous proof copy", async (t) => {
  const options = await fixture(t);
  const old = await freezeNativeProof(options);
  const bytes = await fs.readFile(old.path);
  await fs.writeFile(options.source, "MZ different labelled test bytes");
  const next = await freezeNativeProof(options);
  assert.notEqual(next.sha256, old.sha256);
  assert.deepEqual(await fs.readFile(old.path), bytes);
  assert.deepEqual(await freezeNativeProof(options), next);
});
test("a corrupted immutable proof is rejected without repairing or relabelling it", async (t) => {
  const options = await fixture(t);
  const saved = await freezeNativeProof(options);
  await fs.chmod(saved.path, 0o666);
  await fs.writeFile(saved.path, "deliberate test corruption");
  await assert.rejects(freezeNativeProof(options), /corrupt/);
  assert.equal(
    await fs.readFile(saved.path, "utf8"),
    "deliberate test corruption",
  );
});
test("proof output cannot escape the intended workspace", async (t) => {
  const options = await fixture(t);
  await assert.rejects(
    freezeNativeProof({ ...options, output: "../outside" }),
    /inside workspace/,
  );
});
