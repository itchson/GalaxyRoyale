import test from "node:test";
import assert from "node:assert/strict";
import {
  readFile,
  writeFile,
  mkdtemp,
  mkdir,
  rm,
  symlink,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve, dirname, basename } from "node:path";
import { fileURLToPath } from "node:url";
import {
  WikiStore,
  validateCatalog,
  filterEntries,
  canonicalWikiPath,
} from "../wiki.mjs";
import { ProjectStore } from "../store.mjs";
import { makeServer } from "../server.mjs";
import {
  escapeWiki,
  searchWiki,
  renderWikiEntry,
  wikiFileUrl,
} from "../src/wiki.ts";
const workspace = fileURLToPath(new URL("../../../", import.meta.url));
const catalog = JSON.parse(
  await readFile(join(workspace, "docs/wiki/catalog.json"), "utf8"),
);
const small = () => ({
  schemaVersion: 1,
  title: "Actual fixture",
  description: "Typed source-backed reference",
  categories: [{ id: "ships", label: "Ships" }],
  entries: [
    {
      id: "scout",
      title: "Scout",
      category: "ships",
      status: "implemented",
      summary: "Starter hull",
      tags: ["pilot"],
      facts: [{ label: "Drive", value: "100" }],
      sections: [
        { title: "Rules", paragraphs: ["Shared ranks; active ship cap."] },
      ],
      related: [],
      references: [
        {
          path: "docs/wiki/scout.md",
          label: "scout.md",
          role: "implementation",
        },
      ],
      media: [
        {
          path: "assets/scout.png",
          label: "Scout cells",
          kind: "runtime-art",
          provenance: "docs/wiki/scout.md",
          caption: "Authored contact sheet; no browser claim.",
        },
      ],
    },
  ],
});
async function fixture(t) {
  const root = await mkdtemp(join(tmpdir(), "galaxy-wiki-"));
  assert.equal(resolve(dirname(root)), resolve(tmpdir()));
  assert.ok(basename(root).startsWith("galaxy-wiki-"));
  await mkdir(join(root, "docs/wiki"), { recursive: true });
  await mkdir(join(root, "assets"));
  await writeFile(
    join(root, "docs/wiki/catalog.json"),
    JSON.stringify(small()),
  );
  await writeFile(join(root, "docs/wiki/scout.md"), "# Scout\nActual cells.\n");
  await writeFile(
    join(root, "assets/scout.png"),
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
  );
  await writeFile(join(root, ".env"), "fixture secret not exposed");
  const wiki = new WikiStore(root);
  t.after(() => rm(root, { recursive: true, force: true }));
  return { root, wiki };
}
test("actual catalog categories, references and classified media all validate and resolve", async () => {
  assert.equal(validateCatalog(catalog), catalog);
  assert.equal(catalog.entries.length, 40);
  const wiki = new WikiStore(workspace);
  for (const entry of catalog.entries) {
    assert.ok(entry.references.length > 0, entry.id);
    for (const ref of entry.references)
      assert.ok((await wiki.file("source", ref.path)).bytes.length > 0);
    for (const media of entry.media) {
      const file = await wiki.file("media", media.path);
      assert.ok(file.type.startsWith("image/"));
      assert.ok((await wiki.file("source", media.provenance)).bytes.length > 0);
    }
  }
});
test("server and actual UI search cover names, source facts, categories and verification boundaries", () => {
  for (const [q, expected] of [
    ["Korr-9", ["champion-1"]],
    ["Bastion 390", ["champion-1"]],
    ["712", ["radiation-mote"]],
    ["reload cargo", ["cargo"]],
    ["17", ["abilities"]],
  ]) {
    const server = filterEntries(catalog, { q }).map((v) => v.id),
      client = searchWiki(catalog, q).map((v) => v.id);
    assert.deepEqual(client, server);
    for (const id of expected) assert.ok(server.includes(id), q);
  }
  assert.equal(filterEntries(catalog, { category: "champions" }).length, 11);
  assert.deepEqual(
    filterEntries(catalog, { status: "planned" }).map((e) => e.id),
    ["future"],
  );
  assert.equal(
    filterEntries(catalog, { q: "not-present-in-catalog" }).length,
    0,
  );
  assert.throws(() => filterEntries(catalog, { q: "x".repeat(201) }), /200/);
  assert.throws(
    () => filterEntries(catalog, { category: "unknown" }),
    /category/,
  );
});
test("rendering escapes every prose/attribute surface and constructs only local allowlisted link shapes", () => {
  const malicious = small(),
    entry = malicious.entries[0];
  entry.title = "<img src=x onerror=alert(1)>";
  entry.summary = "<script>alert(1)</script>";
  entry.facts[0].value = '" onclick="alert(1)';
  entry.references[0].label = "<svg onload=alert(1)>";
  entry.media[0].caption = "<iframe src=javascript:alert(1)>";
  const html = renderWikiEntry(entry, malicious);
  assert.ok(!html.includes("<script>"));
  assert.ok(!html.includes("<svg "));
  assert.ok(!html.includes("<iframe "));
  assert.ok(html.includes("&lt;script&gt;"));
  assert.ok(html.includes("&quot; onclick=&quot;"));
  assert.equal(wikiFileUrl("source", "javascript:alert(1)"), "");
  assert.equal(wikiFileUrl("media", "assets/../.env"), "");
  assert.equal(escapeWiki("<>&\"'"), "&lt;&gt;&amp;&quot;&#39;");
  assert.ok(html.includes("/api/wiki/source?path=docs%2Fwiki%2Fscout.md"));
});
test("catalog validates stable ids, related links, status and bounded structured content", () => {
  for (const mutate of [
    (c) => c.entries.push(structuredClone(c.entries[0])),
    (c) => c.entries[0].related.push("missing"),
    (c) => (c.entries[0].status = "shipping-perfect"),
    (c) => (c.entries[0].references[0].path = ".env"),
    (c) => (c.entries[0].media[0].path = "assets/icon.svg"),
    (c) => (c.entries[0].media[0].kind = "browser-proof"),
    (c) => (c.entries[0].summary = "x".repeat(2001)),
  ]) {
    const c = small();
    mutate(c);
    assert.throws(() => validateCatalog(c));
  }
});
test("source/media read paths reject traversal, secrets, unlisted and wrong-type content", async (t) => {
  const { root, wiki } = await fixture(t);
  assert.match(
    (await wiki.file("source", "docs/wiki/scout.md")).bytes.toString(),
    /Scout/,
  );
  for (const path of [
    ".env",
    "docs/wiki/../../.env",
    "assets\\scout.png",
    "C:/outside.png",
    "assets/scout.png\0",
    "assets//scout.png",
  ])
    await assert.rejects(
      wiki.file("source", path),
      (error) => error.status === 400,
    );
  await assert.rejects(
    wiki.file("source", "docs/wiki/catalog.json"),
    (error) => error.status === 404,
  );
  await assert.rejects(
    wiki.file("media", "docs/wiki/scout.md"),
    (error) => error.status === 404,
  );
  await writeFile(
    join(root, "assets/scout.png"),
    "<script>not an image</script>",
  );
  await assert.rejects(wiki.file("media", "assets/scout.png"), /header/);
  assert.throws(() => canonicalWikiPath("https://example.com/x.png"), /path/);
});
test("catalog itself and listed ancestor links fail closed rather than reading redirected content", async (t) => {
  const { root, wiki } = await fixture(t),
    external = await mkdtemp(join(tmpdir(), "galaxy-wiki-link-"));
  assert.equal(resolve(dirname(external)), resolve(tmpdir()));
  assert.ok(basename(external).startsWith("galaxy-wiki-link-"));
  t.after(() => rm(external, { recursive: true, force: true }));
  await writeFile(join(external, "scout.md"), "external fixture");
  await mkdir(join(root, "docs/real"));
  await writeFile(join(root, "docs/real/scout.md"), "internal fixture");
  const edited = small();
  edited.entries[0].references[0].path = "docs/linked/scout.md";
  await writeFile(join(root, "docs/wiki/catalog.json"), JSON.stringify(edited));
  try {
    await symlink(
      external,
      join(root, "docs/linked"),
      process.platform === "win32" ? "junction" : "dir",
    );
  } catch (error) {
    if (["EPERM", "EACCES", "ENOTSUP"].includes(error.code)) {
      t.skip("OS cannot create test links");
      return;
    }
    throw error;
  }
  await assert.rejects(
    wiki.file("source", "docs/linked/scout.md"),
    /without links/,
  );
  // A directory junction around catalog.json needs no privileged file-symlink operation on Windows.
  await mkdir(join(root, "copy"));
  await writeFile(join(root, "copy/catalog.json"), JSON.stringify(small()));
  await rm(join(root, "docs/wiki"), { recursive: true });
  await symlink(
    join(root, "copy"),
    join(root, "docs/wiki"),
    process.platform === "win32" ? "junction" : "dir",
  );
  await assert.rejects(wiki.load(), /without links/);
});
test("HTTP Wiki routes return actual listed bytes, filtered catalog and controlled lookup errors", async (t) => {
  const { root, wiki } = await fixture(t),
    file = join(root, "project.json");
  await writeFile(
    file,
    JSON.stringify({
      schemaVersion: 1,
      revision: 1,
      tasks: [],
      reviews: [],
      decisions: [],
      builds: [],
      feedback: [],
      activity: [],
    }),
  );
  const store = await new ProjectStore(file, root).load();
  const server = makeServer(store, wiki);
  await new Promise((r) => server.listen(0, "127.0.0.1", r));
  t.after(() => new Promise((r) => server.close(r)));
  const base = `http://127.0.0.1:${server.address().port}`;
  const entry = await fetch(base + "/api/wiki/scout");
  assert.equal(entry.status, 200);
  assert.equal((await entry.json()).title, "Scout");
  assert.equal((await fetch(base + "/api/wiki/missing")).status, 404);
  assert.equal(
    (await (await fetch(base + "/api/wiki?q=100")).json()).entries.length,
    1,
  );
  assert.equal(
    (await (await fetch(base + "/api/wiki?q=miss")).json()).entries.length,
    0,
  );
  const source = await fetch(
    base + "/api/wiki/source?path=docs%2Fwiki%2Fscout.md",
  );
  assert.equal(source.headers.get("content-type"), "text/plain; charset=utf-8");
  assert.equal(source.headers.get("x-content-type-options"), "nosniff");
  assert.match(source.headers.get("content-security-policy"), /sandbox/);
  assert.match(await source.text(), /Actual cells/);
  const media = await fetch(base + "/api/wiki/media?path=assets%2Fscout.png");
  assert.equal(media.headers.get("content-type"), "image/png");
  assert.equal((await media.arrayBuffer()).byteLength, 8);
  assert.equal((await fetch(base + "/api/wiki/source?path=.env")).status, 400);
  assert.equal(
    (await fetch(base + "/api/wiki/source?path=docs%2Fwiki%2Fcatalog.json"))
      .status,
    404,
  );
  assert.equal((await fetch(base + "/api/wiki?status=live")).status, 400);
});
