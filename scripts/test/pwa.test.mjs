import test from "node:test";
import assert from "node:assert/strict";
import {
  mkdtemp,
  mkdir,
  readFile,
  writeFile,
  rename,
  rm,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve, dirname, basename, join } from "node:path";
import { spawnSync } from "node:child_process";
import { runInNewContext } from "node:vm";

async function generated(t) {
  const root = await mkdtemp(join(tmpdir(), "galaxy-pwa-"));
  assert.equal(resolve(dirname(root)), resolve(tmpdir()));
  assert.ok(basename(root).startsWith("galaxy-pwa-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  await mkdir(join(root, "scripts"));
  await mkdir(join(root, "dist", "assets"), { recursive: true });
  await mkdir(join(root, "dist", "wasm", "webgl"), { recursive: true });
  await writeFile(
    join(root, "scripts", "pwa.mjs"),
    await readFile(new URL("../pwa.mjs", import.meta.url), "utf8"),
  );
  await writeFile(
    join(root, "dist", "index.html"),
    "<html>Test game build</html>",
  );
  await writeFile(
    join(root, "dist", "assets", "game.js"),
    'console.log("fixture")',
  );
  await writeFile(
    join(root, "dist", "wasm", "webgl", "galaxy.wasm"),
    new Uint8Array([0, 97, 115, 109]),
  );
  await writeFile(join(root, "dist", "manifest.webmanifest"), "{}");
  const generate = async () => {
    const result = spawnSync(
      process.execPath,
      [join(root, "scripts", "pwa.mjs")],
      { encoding: "utf8" },
    );
    assert.equal(result.status, 0, result.stderr);
    return readFile(join(root, "dist", "sw.js"), "utf8");
  };
  return { root, generate, source: await generate() };
}

function worker(source) {
  const events = new Map();
  const cacheData = new Map();
  const deleted = [];
  const network = [];
  let online = true;
  let skipWaiting = 0;
  const cacheObject = (name) => {
    if (!cacheData.has(name)) cacheData.set(name, new Map());
    const entries = cacheData.get(name);
    return {
      addAll: async (assets) => {
        for (const path of assets)
          entries.set(path, new Response(`cached:${path}`));
      },
      match: async (path) => entries.get(path)?.clone(),
    };
  };
  const caches = {
    open: async (name) => cacheObject(name),
    keys: async () => [...cacheData.keys()],
    delete: async (name) => {
      deleted.push(name);
      return cacheData.delete(name);
    },
    match: async (path) => {
      for (const entries of cacheData.values())
        if (entries.has(path)) return entries.get(path).clone();
    },
  };
  const self = {
    location: { origin: "https://galaxy.example" },
    addEventListener: (name, listener) => events.set(name, listener),
    skipWaiting: () => {
      skipWaiting++;
      return Promise.resolve();
    },
  };
  const fetch = async (request) => {
    network.push(request.url);
    if (!online) throw new Error("Offline fixture");
    return new Response(`network:${new URL(request.url).pathname}`);
  };
  const data = runInNewContext(source + "\n;({cache:CACHE,assets:ASSETS});", {
    self,
    caches,
    URL,
    fetch,
    Response,
  });
  const dispatch = async (name, extra = {}) => {
    const promises = [];
    let response;
    let intercepted = false;
    events.get(name)?.({
      ...extra,
      waitUntil: (p) => promises.push(p),
      respondWith: (p) => {
        intercepted = true;
        response = p;
      },
    });
    await Promise.all(promises);
    return { intercepted, response: response ? await response : undefined };
  };
  const fetchRequest = (
    path,
    { method = "GET", mode = "cors", origin = self.location.origin } = {},
  ) => dispatch("fetch", { request: { url: origin + path, method, mode } });
  return {
    data,
    cacheData,
    deleted,
    network,
    dispatch,
    fetchRequest,
    setOnline: (value) => (online = value),
    skips: () => skipWaiting,
  };
}

test("PWA generation is deterministic, excludes its worker, and changes version when asset content or path changes", async (t) => {
  const { root, source, generate } = await generated(t);
  const first = worker(source).data;
  assert.equal(await generate(), source);
  assert.equal(first.assets.includes("/sw.js"), false);
  assert.deepEqual(
    [...first.assets],
    [
      "/assets/game.js",
      "/index.html",
      "/manifest.webmanifest",
      "/wasm/webgl/galaxy.wasm",
    ],
  );
  await rename(
    join(root, "dist", "assets", "game.js"),
    join(root, "dist", "assets", "renamed.js"),
  );
  const renamed = worker(await generate()).data;
  assert.notEqual(
    renamed.cache,
    first.cache,
    "file paths participate in cache version",
  );
  await writeFile(
    join(root, "dist", "assets", "renamed.js"),
    'console.log("changed")',
  );
  assert.notEqual(worker(await generate()).data.cache, renamed.cache);
});

test("installed game assets and entry navigation work offline; unknown, API, studio, foreign and non-GET requests pass through", async (t) => {
  const { source } = await generated(t);
  const w = worker(source);
  await w.dispatch("install");
  w.setOnline(false);
  const navigation = await w.fetchRequest("/", { mode: "navigate" });
  assert.equal(navigation.intercepted, true);
  assert.equal(await navigation.response.text(), "cached:/index.html");
  const asset = await w.fetchRequest("/wasm/webgl/galaxy.wasm");
  assert.equal(asset.intercepted, true);
  assert.equal(await asset.response.text(), "cached:/wasm/webgl/galaxy.wasm");
  const before = w.network.length;
  for (const [path, options] of [
    ["/api/project", { mode: "navigate" }],
    ["/studio", { mode: "navigate" }],
    ["/studio/index.html", {}],
    ["/unlisted-resource.js", {}],
    ["/assets/game.js", { method: "POST" }],
    ["/assets/game.js", { origin: "https://other.example" }],
  ]) {
    assert.equal(
      (await w.fetchRequest(path, options)).intercepted,
      false,
      `${path} should pass through`,
    );
  }
  assert.equal(
    w.network.length,
    before,
    "excluded requests are not fetched by the worker",
  );
});

test("navigation and listed assets use their coherent installed cache with network only for missing entries", async (t) => {
  const { source } = await generated(t);
  const w = worker(source);
  await w.dispatch("install");
  assert.equal(
    await (await w.fetchRequest("/", { mode: "navigate" })).response.text(),
    "cached:/index.html",
  );
  const count = w.network.length;
  assert.equal(
    await (await w.fetchRequest("/assets/game.js")).response.text(),
    "cached:/assets/game.js",
  );
  assert.equal(w.network.length, count);
  w.cacheData.get(w.data.cache).delete("/index.html");
  assert.equal(
    await (await w.fetchRequest("/", { mode: "navigate" })).response.text(),
    "network:/",
  );
  w.cacheData.get(w.data.cache).delete("/assets/game.js");
  assert.equal(
    await (await w.fetchRequest("/assets/game.js")).response.text(),
    "network:/assets/game.js",
  );
});

test("updates wait until an explicit dock activation request, then remove only obsolete game caches", async (t) => {
  const { source } = await generated(t);
  const w = worker(source);
  await w.dispatch("install");
  assert.equal(w.skips(), 0, "installation must not interrupt an active run");
  await w.dispatch("message", { data: "unrelated" });
  assert.equal(w.skips(), 0);
  await w.dispatch("message", { data: "ACTIVATE_AT_DOCK" });
  assert.equal(w.skips(), 1);
  w.cacheData.set("galaxy-old-version", new Map());
  w.cacheData.set("unrelated-cache", new Map());
  await w.dispatch("activate");
  assert.deepEqual(w.deleted, ["galaxy-old-version"]);
  assert.equal(w.cacheData.has(w.data.cache), true);
  assert.equal(w.cacheData.has("unrelated-cache"), true);
});
