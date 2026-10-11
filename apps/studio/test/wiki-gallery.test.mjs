import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { WikiStore, canonicalWikiPath } from "../wiki.mjs";
import { renderWikiEntry, wikiFileUrl } from "../src/wiki.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const wiki = new WikiStore(root);
const catalog = await wiki.load();
const prefix = "artifacts/presentation-refinement-20261011/";
// Exact tracked bytes copied from cac1f03; no source-generation or Git-history dependency.
const pictures = [
  [
    "galaxy-royale",
    "webgpu-haven.jpg",
    "runtime-screenshot",
    "830a20d6dfd40905090a168918d1a571fd545c02d9c4eceefb430118e6d5d334",
  ],
  [
    "sanctuaries",
    "webgpu-belt.jpg",
    "runtime-screenshot",
    "42514e6cce99cca0af72e4b0d6c13209c4da0f9a6888f24b283cbe27437aaf85",
  ],
  [
    "pixel-engine",
    "native-art-contact-sheet.png",
    "runtime-art",
    "10063d573057be4dfdba591880d1f971a7e7d3bc0e1280d92f6927b262f16a8c",
  ],
];

test("gallery serves the three exact committed native/browser artifacts with distinct classifications and readable provenance", async () => {
  for (const [id, name, kind, hash] of pictures) {
    const entry = catalog.entries.find((entry) => entry.id === id);
    const media = entry.media.find((media) => media.path === prefix + name);
    assert.ok(media);
    assert.equal(media.kind, kind);
    const file = await wiki.file("media", media.path);
    assert.equal(file.type, name.endsWith(".jpg") ? "image/jpeg" : "image/png");
    assert.equal(createHash("sha256").update(file.bytes).digest("hex"), hash);
    const source = await wiki.file("source", media.provenance);
    assert.equal(source.type, "text/plain; charset=utf-8");
    assert.ok(source.bytes.length > 0);
    const html = renderWikiEntry(entry, catalog);
    assert.ok(html.includes(wikiFileUrl("media", media.path)));
    assert.ok(html.includes(wikiFileUrl("source", media.provenance)));
  }
});

test("gallery captions preserve observed desktop/QA scope and do not turn a native raster into GPU acceptance", () => {
  const get = (id) => catalog.entries.find((e) => e.id === id).media.at(-1);
  assert.match(get("galaxy-royale").caption, /1280×720/);
  assert.match(get("galaxy-royale").caption, /Historical[\s\S]*no mobile/);
  assert.match(get("sanctuaries").caption, /QA console, not natural travel/);
  assert.match(get("pixel-engine").caption, /software raster/);
  assert.match(get("pixel-engine").caption, /not a browser screenshot/);
});

test("artifact namespace retains exact catalog allowlists and excludes arbitrary records, hidden paths and audio", async () => {
  assert.equal(
    canonicalWikiPath(prefix + "webgpu-haven.jpg"),
    prefix + "webgpu-haven.jpg",
  );
  for (const path of [
    "artifacts/../production/project.json",
    "artifacts/.env",
    "artifacts//capture.png",
  ])
    assert.throws(() => canonicalWikiPath(path));
  await assert.rejects(
    wiki.file("source", prefix + "board-audit.json"),
    (e) => e.status === 404,
  );
  await assert.rejects(
    wiki.file("media", prefix + "warm-engine.wav"),
    (e) => e.status === 404,
  );
  const docs = await wiki.file("source", prefix + "HUD_HANDOFF.md");
  assert.match(docs.bytes.toString(), /native|presentation/i);
  // The copy retained no production ledger: this feature never imported that commit's project.json delta.
  const readme = await readFile(
    new URL("../../../docs/DEV_BIBLE.md", import.meta.url),
    "utf8",
  );
  assert.match(
    readme,
    /did not import or edit that commit's production ledger/,
  );
});
