import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative, resolve, sep } from "node:path";
import vm from "node:vm";
import test from "node:test";
import {
  WASM_BACKENDS,
  WASM_FILES,
  createWasmStage,
  publishWasmBuilds,
  readWasmBuilds,
  removeWasmStage,
  wasmContentHash,
} from "../wasm-artifacts.mjs";
import { buildPwa, serviceWorkerSource } from "../pwa.mjs";

function fixture(run) {
  const base = realpathSync(tmpdir());
  const project = mkdtempSync(join(base, "galaxy-wasm-fixture-"));
  const cleanup = () => {
    const delta = relative(base, realpathSync(project));
    assert(
      delta.startsWith("galaxy-wasm-fixture-") &&
        !delta.includes(sep) &&
        !delta.startsWith(".."),
    );
    rmSync(project, { recursive: true });
  };
  try {
    const result = run(project);
    if (result && typeof result.then === "function")
      return result.finally(cleanup);
    cleanup();
    return result;
  } catch (error) {
    cleanup();
    throw error;
  }
}
const rootAt = (project) => join(project, "apps/game/public/wasm");
function binding(directory, revision) {
  mkdirSync(directory, { recursive: true });
  const files = {
    "galaxy.js": `export const revision=${JSON.stringify(revision)};export const binary=new URL('galaxy_bg.wasm',import.meta.url);`,
    "galaxy.d.ts": `export const revision: '${revision}';\n`,
    "galaxy_bg.wasm": Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]),
    "galaxy_bg.wasm.d.ts": `// ${revision}\n`,
  };
  for (const file of WASM_FILES)
    writeFileSync(join(directory, file), files[file]);
}
function pair(project, revision) {
  const stage = createWasmStage(project);
  for (const backend of WASM_BACKENDS)
    binding(join(stage, backend), `${backend}-${revision}`);
  return publishWasmBuilds(project, stage);
}
function manifest(project) {
  return join(rootAt(project), "builds.json");
}
function mappedDir(project, entry) {
  return join(rootAt(project), entry.split("/")[2]);
}

test("a complete pair addresses all four files and removes only stale generated outputs", () =>
  fixture((project) => {
    const first = pair(project, "one");
    const root = rootAt(project);
    for (const backend of WASM_BACKENDS) binding(join(root, backend), "legacy");
    const second = pair(project, "two");
    assert.deepEqual(readWasmBuilds(project), second);
    assert.notEqual(first.webgpu, second.webgpu);
    assert.notEqual(first.webgl2, second.webgl2);
    assert.equal(readdirSync(root).length, 3);
    for (const backend of WASM_BACKENDS) {
      assert.match(
        second[backend],
        new RegExp(`^/wasm/${backend}-[a-f0-9]{64}/galaxy\\.js$`),
      );
      assert(!existsSync(mappedDir(project, first[backend])));
      assert(!existsSync(join(root, backend)));
      assert.deepEqual(
        readdirSync(mappedDir(project, second[backend])).sort(),
        [...WASM_FILES],
      );
      assert.equal(
        wasmContentHash(mappedDir(project, second[backend])),
        second[backend].split("/")[2].slice(backend.length + 1),
      );
      assert.equal(
        new URL("galaxy_bg.wasm", `https://game.test${second[backend]}`)
          .pathname,
        second[backend].replace("galaxy.js", "galaxy_bg.wasm"),
      );
    }
  }));

test("each JS, binary and declaration byte independently participates in the content address", () =>
  fixture((project) => {
    const stage = createWasmStage(project);
    const directory = join(stage, "webgpu");
    binding(directory, "stable");
    const initial = wasmContentHash(directory);
    for (const name of WASM_FILES) {
      const before = readFileSync(join(directory, name));
      writeFileSync(
        join(directory, name),
        Buffer.concat([before, Buffer.from([33])]),
      );
      assert.notEqual(wasmContentHash(directory), initial, name);
      writeFileSync(join(directory, name), before);
      assert.equal(wasmContentHash(directory), initial);
    }
    removeWasmStage(project, stage);
  }));

test("missing, corrupt, incomplete and traversal manifests fail closed with a build instruction", () =>
  fixture((project) => {
    assert.throws(() => readWasmBuilds(project), /pnpm build:wasm/);
    const builds = pair(project, "one");
    const valid = readFileSync(manifest(project));
    for (const value of [
      "{",
      JSON.stringify({ version: 2, builds }),
      JSON.stringify({ version: 1, builds: { webgpu: builds.webgpu } }),
      JSON.stringify({
        version: 1,
        builds: { ...builds, webgpu: "/wasm/../../outside/galaxy.js" },
      }),
    ]) {
      writeFileSync(manifest(project), value);
      assert.throws(() => readWasmBuilds(project));
    }
    writeFileSync(manifest(project), valid);
    writeFileSync(
      join(mappedDir(project, builds.webgpu), "galaxy.js"),
      "tampered",
    );
    assert.throws(() => readWasmBuilds(project), /bytes do not match/);
  }));

test("incomplete staging and unknown public files cannot destroy a prior complete pair", () =>
  fixture((project) => {
    const current = pair(project, "one");
    const prior = readFileSync(manifest(project));
    const stage = createWasmStage(project);
    binding(join(stage, "webgpu"), "incomplete");
    assert.throws(() => publishWasmBuilds(project, stage));
    assert.deepEqual(readFileSync(manifest(project)), prior);
    assert(existsSync(mappedDir(project, current.webgpu)));
    assert(existsSync(mappedDir(project, current.webgl2)));
    removeWasmStage(project, stage);
    writeFileSync(join(rootAt(project), "keep-user-file.txt"), "preserve");
    const next = createWasmStage(project);
    for (const backend of WASM_BACKENDS) binding(join(next, backend), "next");
    assert.throws(
      () => publishWasmBuilds(project, next),
      /Unexpected public WASM entry/,
    );
    assert.deepEqual(readFileSync(manifest(project)), prior);
    assert.equal(
      readFileSync(join(rootAt(project), "keep-user-file.txt"), "utf8"),
      "preserve",
    );
    removeWasmStage(project, next);
  }));

test("the read-only loader rejects a linked manifest without changing external bytes", (t) =>
  fixture((project) => {
    pair(project, "one");
    const external = join(project, "external-manifest.json");
    const original = readFileSync(manifest(project));
    writeFileSync(external, original);
    rmSync(manifest(project));
    try {
      symlinkSync(external, manifest(project), "file");
    } catch (error) {
      if (process.platform === "win32" && error.code === "EPERM") {
        t.skip("Windows file-symlink privilege is unavailable; directory-junction coverage remains active");
        return;
      }
      throw error;
    }
    assert.throws(() => readWasmBuilds(project), /regular nonlinked file/);
    assert.deepEqual(readFileSync(external), original);
  }));

test("the read-only loader rejects a manifest directory junction before parsing", () =>
  fixture((project) => {
    pair(project, "one");
    const external = join(project, "external-directory");
    mkdirSync(external);
    writeFileSync(join(external, "keep.txt"), "preserve");
    rmSync(manifest(project));
    symlinkSync(external, manifest(project), process.platform === "win32" ? "junction" : "dir");
    assert.throws(() => readWasmBuilds(project), /regular nonlinked file/);
    assert.equal(readFileSync(join(external, "keep.txt"), "utf8"), "preserve");
  }));

test("linked managed targets are rejected before cleanup or publication", () =>
  fixture((project) => {
    pair(project, "one");
    const outside = join(project, "outside");
    mkdirSync(outside);
    writeFileSync(join(outside, "keep.txt"), "keep");
    symlinkSync(
      outside,
      join(rootAt(project), "webgpu"),
      process.platform === "win32" ? "junction" : "dir",
    );
    const stage = createWasmStage(project);
    for (const backend of WASM_BACKENDS) binding(join(stage, backend), "next");
    assert.throws(() => publishWasmBuilds(project, stage), /linked|escapes/);
    assert.equal(readFileSync(join(outside, "keep.txt"), "utf8"), "keep");
    removeWasmStage(project, stage);
  }));

function worker(source, own = new Map()) {
  const handlers = new Map();
  const opened = [],
    network = [];
  vm.runInNewContext(source, {
    self: {
      location: { origin: "https://game.test" },
      addEventListener: (kind, listener) => handlers.set(kind, listener),
    },
    URL,
    caches: {
      match() {
        throw new Error("Global cache lookup can mix releases");
      },
      async open(name) {
        opened.push(name);
        return { match: async (path) => own.get(path) };
      },
    },
    async fetch(request) {
      network.push(request.url);
      return "fresh-network";
    },
  });
  return {
    opened,
    network,
    async get(path, mode = "cors") {
      let response;
      handlers.get("fetch")({
        request: { url: `https://game.test${path}`, method: "GET", mode },
        respondWith: (value) => {
          response = value;
        },
      });
      return response === undefined ? undefined : await response;
    },
  };
}

test("new worker navigation and assets read only their own complete release cache", async () => {
  const own = new Map([
    ["/index.html", "own-index"],
    ["/wasm/current/galaxy.js", "own-glue"],
  ]);
  const w = worker(serviceWorkerSource("galaxy-current", [...own.keys()]), own);
  assert.equal(await w.get("/any/route", "navigate"), "own-index");
  assert.equal(await w.get("/wasm/current/galaxy.js"), "own-glue");
  assert.deepEqual(w.network, []);
  assert(w.opened.every((name) => name === "galaxy-current"));
  own.clear();
  assert.equal(await w.get("/any/route", "navigate"), "fresh-network");
  assert.equal(await w.get("/wasm/current/galaxy.js"), "fresh-network");
  assert.equal(await w.get("/api/profile"), undefined);
});

test("legacy stable-path worker cannot return old bindings for a new content-addressed entry or binary", () =>
  fixture(async (project) => {
    const builds = pair(project, "new");
    // Exact legacy fetch-routing algorithm: old workers can remain active until
    // Apply Update, so the new URL must bypass their stable-name ASSETS list.
    const legacy = `const ASSETS=['/wasm/webgpu/galaxy.js','/wasm/webgpu/galaxy_bg.wasm','/wasm/webgl2/galaxy.js','/wasm/webgl2/galaxy_bg.wasm'];self.addEventListener('fetch',event=>{const url=new URL(event.request.url);if(event.request.method!=='GET'||url.origin!==self.location.origin||url.pathname.startsWith('/api/')||url.pathname.startsWith('/studio'))return;if(event.request.mode==='navigate'){event.respondWith(fetch(event.request).catch(()=>caches.match('/index.html')));return;}if(ASSETS.includes(url.pathname))event.respondWith(caches.match(url.pathname).then(cached=>cached||fetch(event.request)));});`;
    const w = worker(legacy);
    assert.equal(await w.get("/", "navigate"), "fresh-network");
    for (const path of Object.values(builds)) {
      assert.equal(
        await w.get(path),
        undefined,
        "legacy worker falls through to network for new glue",
      );
      assert.equal(
        await w.get(path.replace("galaxy.js", "galaxy_bg.wasm")),
        undefined,
        "relative binary has the same new version directory",
      );
    }
    assert.deepEqual(w.opened, []);
  }));

test("actual PWA generation lists exactly the current pairs and derives cache from all payload bytes", () =>
  fixture(async (project) => {
    const builds = pair(project, "release");
    const directory = rootAt(project);
    writeFileSync(
      join(directory, "index.html"),
      `<script>${JSON.stringify(builds)}</script>`,
    );
    const result = await buildPwa(directory);
    assert.equal(result.assets.length, 10);
    assert.equal(new Set(result.assets).size, result.assets.length);
    for (const path of Object.values(builds)) {
      assert(result.assets.includes(path.replace("/wasm/", "/")));
      assert(
        result.assets.includes(
          path.replace("/wasm/", "/").replace("galaxy.js", "galaxy_bg.wasm"),
        ),
      );
    }
    const hash = createHash("sha256");
    for (const name of result.assets)
      hash
        .update(name.slice(1))
        .update(readFileSync(join(directory, name.slice(1))));
    assert.equal(result.cache, `galaxy-${hash.digest("hex").slice(0, 16)}`);
    assert.equal(
      readFileSync(join(directory, "sw.js"), "utf8"),
      serviceWorkerSource(result.cache, result.assets),
    );
  }));
