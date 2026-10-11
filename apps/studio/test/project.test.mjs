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
import { ProjectStore } from "../store.mjs";
import { makeServer } from "../server.mjs";
import { command } from "../../../scripts/studio-cli.mjs";
const initial = () => ({
  schemaVersion: 1,
  revision: 7,
  name: "Galaxy Royale",
  phase: "Foundation",
  updatedAt: new Date().toISOString(),
  tasks: [],
  reviews: [{ id: "prior-review", title: "Actual prior review" }],
  decisions: [],
  builds: [],
  feedback: [],
  activity: [],
  runRecords: [],
});
const payload = (patch, expectedRevision = 7) => ({
  author: "Producer",
  source: "Explicit source-backed phase review",
  expectedRevision,
  patch,
});
async function fixture(t) {
  const root = await mkdtemp(join(tmpdir(), "galaxy-phase-"));
  assert.equal(resolve(dirname(root)), resolve(tmpdir()));
  assert.ok(basename(root).startsWith("galaxy-phase-"));
  const file = join(root, "project.json");
  await writeFile(file, JSON.stringify(initial()));
  const store = await new ProjectStore(file, root).load();
  t.after(() => rm(root, { recursive: true, force: true }));
  return { root, file, store };
}
test("phase update records metadata and activity only; restart and backup retain previous complete document", async (t) => {
  const { root, file, store } = await fixture(t);
  let observed = 0;
  store.listeners.add(() => observed++);
  const result = await store.mutate(
    "project.update",
    payload({
      phase: "Campaign and equipment",
      phaseDescription: "Implemented core; phone acceptance pending.",
    }),
  );
  assert.equal(result.revision, 8);
  assert.equal(result.item.id, "project");
  assert.equal(store.project.phase, "Campaign and equipment");
  assert.equal(
    store.project.phaseDescription,
    "Implemented core; phone acceptance pending.",
  );
  assert.equal(observed, 1);
  assert.deepEqual(store.project.reviews, initial().reviews);
  assert.deepEqual(store.project.tasks, []);
  assert.deepEqual(store.project.runRecords, []);
  assert.equal(store.project.activity[0].kind, "project.update");
  assert.equal(store.project.activity[0].author, "Producer");
  assert.equal(
    JSON.parse(await readFile(file + ".backup", "utf8")).phase,
    "Foundation",
  );
  assert.deepEqual(
    (await new ProjectStore(file, root).load()).snapshot(),
    store.snapshot(),
  );
});
test("queued project metadata edits use the project revision, accept one and atomically reject stale second", async (t) => {
  const { file, store } = await fixture(t);
  const results = await Promise.allSettled([
    store.mutate("project.update", payload({ phase: "Accepted" })),
    store.mutate("project.update", payload({ phase: "Stale" })),
  ]);
  assert.equal(results[0].status, "fulfilled");
  assert.equal(results[1].reason.status, 409);
  assert.equal(store.project.phase, "Accepted");
  assert.equal(store.project.activity.length, 1);
  assert.equal(JSON.parse(await readFile(file, "utf8")).revision, 8);
});
test("invalid phase patches reject full-project rewrites without revision, disk, activity or notification mutation", async (t) => {
  const { file, store } = await fixture(t);
  const before = await readFile(file, "utf8");
  let observed = 0;
  store.listeners.add(() => observed++);
  for (const patch of [
    {},
    { phase: "" },
    { phase: "x".repeat(201) },
    { phaseDescription: 1 },
    { phaseDescription: "x".repeat(2001) },
    { tasks: [] },
    { revision: 999 },
    [],
    null,
  ])
    await assert.rejects(
      store.mutate("project.update", payload(patch)),
      (error) => error.status === 400,
    );
  assert.equal(await readFile(file, "utf8"), before);
  assert.equal(store.project.revision, 7);
  assert.equal(store.project.activity.length, 0);
  assert.equal(observed, 0);
  await store.mutate("project.update", payload({ phaseDescription: "" }));
  assert.equal(store.project.phase, "Foundation");
  assert.equal(store.project.phaseDescription, "");
});
test("HTTP PATCH project accepts revision-checked metadata and preserves origin/content-type barriers", async (t) => {
  const { store } = await fixture(t),
    server = makeServer(store);
  await new Promise((r) => server.listen(0, "127.0.0.1", r));
  t.after(() => new Promise((r) => server.close(r)));
  const base = `http://127.0.0.1:${server.address().port}`;
  const options = {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(
      payload({ phase: "Implemented / verification pending" }),
    ),
  };
  assert.equal((await fetch(base + "/api/project", options)).status, 200);
  assert.equal((await fetch(base + "/api/project", options)).status, 409);
  assert.equal(
    (
      await fetch(base + "/api/project", {
        ...options,
        headers: { ...options.headers, Origin: "https://foreign.example" },
      })
    ).status,
    403,
  );
  assert.equal(
    (
      await fetch(base + "/api/project", {
        ...options,
        headers: { "Content-Type": "text/plain" },
      })
    ).status,
    415,
  );
  assert.equal(
    (await (await fetch(base + "/api/project")).json()).phase,
    "Implemented / verification pending",
  );
});
test("CLI project update requires explicit file and routes to PATCH without direct store access", () => {
  assert.deepEqual(
    command("project", "update", ["--file", "work/phase.json"]),
    { url: "/api/project", method: "PATCH", file: "work/phase.json" },
  );
  assert.deepEqual(command("snapshot"), { url: "/api/project", method: "GET" });
  assert.deepEqual(command("wiki"), { url: "/api/wiki", method: "GET" });
  assert.throws(() => command("project", "create", ["--file", "x"]), /Usage/);
  assert.throws(() => command("project", "update", []), /explicit/);
  assert.equal(
    command("task", "update", ["a/b", "--file", "x"]).url,
    "/api/tasks/a%2Fb",
  );
});
