// One content-addressed bindgen pair shared by normal builds and Vite.
import assert from "node:assert/strict";
import { createHash, randomUUID } from "node:crypto";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { isAbsolute, join, relative, resolve, sep } from "node:path";

export const WASM_BACKENDS = Object.freeze(["webgpu", "webgl2"]);
export const WASM_FILES = Object.freeze([
  "galaxy.d.ts",
  "galaxy.js",
  "galaxy_bg.wasm",
  "galaxy_bg.wasm.d.ts",
]);
const managed = /^(?:webgpu|webgl2)(?:-[a-f0-9]{64})?$|^\.stage-[a-f0-9-]{36}$/;

function publicRoot(projectRoot, create = false) {
  assert(
    isAbsolute(projectRoot),
    "WASM helpers require an absolute project root",
  );
  const project = realpathSync(projectRoot);
  let path = project;
  for (const part of ["apps", "game", "public", "wasm"]) {
    path = join(path, part);
    if (existsSync(path)) {
      assert(
        !lstatSync(path).isSymbolicLink(),
        `Unsafe WASM directory link: ${path}`,
      );
      assert(
        lstatSync(path).isDirectory(),
        `Expected a WASM directory: ${path}`,
      );
    } else if (create) mkdirSync(path);
  }
  return path;
}
function inside(root, path) {
  const delta = relative(root, realpathSync(path));
  assert(
    delta &&
      delta !== ".." &&
      !delta.startsWith(`..${sep}`) &&
      !isAbsolute(delta),
    "WASM target escapes its public root",
  );
}
function safeGeneratedTree(path, stage = false) {
  assert(!lstatSync(path).isSymbolicLink(), "Refusing a linked WASM target");
  assert(lstatSync(path).isDirectory(), "Expected a generated WASM directory");
  for (const item of readdirSync(path, { withFileTypes: true })) {
    const child = join(path, item.name);
    assert(
      !lstatSync(child).isSymbolicLink(),
      "Refusing linked generated contents",
    );
    if (stage && item.isDirectory() && WASM_BACKENDS.includes(item.name))
      safeGeneratedTree(child);
    else
      assert(
        item.isFile() && WASM_FILES.includes(item.name),
        `Unexpected generated WASM content: ${item.name}`,
      );
  }
}
export function wasmContentHash(directory) {
  safeGeneratedTree(directory);
  assert.deepEqual(
    readdirSync(directory).sort(),
    [...WASM_FILES],
    "A backend requires exactly four bindgen files",
  );
  const hash = createHash("sha256").update("galaxy-wasm-v1\0");
  for (const name of WASM_FILES) {
    const data = readFileSync(join(directory, name));
    const length = Buffer.alloc(8);
    length.writeBigUInt64LE(BigInt(data.length));
    hash.update(name).update("\0").update(length).update(data);
  }
  return hash.digest("hex");
}
export function readWasmBuilds(projectRoot) {
  const root = publicRoot(projectRoot);
  const manifest = join(root, "builds.json");
  if (!existsSync(manifest))
    throw new Error(
      "WASM build manifest is missing. Run pnpm build:wasm, then restart the dev server.",
    );
  const manifestStat = lstatSync(manifest);
  assert(
    manifestStat.isFile() && !manifestStat.isSymbolicLink(),
    "WASM build manifest must be a regular nonlinked file",
  );
  let data;
  try {
    data = JSON.parse(readFileSync(manifest, "utf8"));
  } catch {
    throw new Error(
      "WASM build manifest is invalid JSON. Run pnpm build:wasm.",
    );
  }
  assert(
    data && typeof data === "object" && data.version === 1,
    "Unsupported WASM build manifest",
  );
  assert.deepEqual(
    Object.keys(data).sort(),
    ["builds", "version"],
    "Unexpected WASM manifest fields",
  );
  assert(
    data.builds &&
      typeof data.builds === "object" &&
      !Array.isArray(data.builds),
    "Invalid WASM build mapping",
  );
  assert.deepEqual(
    Object.keys(data.builds).sort(),
    [...WASM_BACKENDS].sort(),
    "Both WASM backends are required",
  );
  const builds = {};
  const directories = [];
  for (const backend of WASM_BACKENDS) {
    const path = data.builds[backend];
    assert(typeof path === "string", "Invalid WASM entry URL");
    const match = path.match(
      /^\/wasm\/(webgpu|webgl2)-([a-f0-9]{64})\/galaxy\.js$/,
    );
    assert(
      match && match[1] === backend,
      `Invalid content-addressed ${backend} URL`,
    );
    const name = `${backend}-${match[2]}`;
    const directory = join(root, name);
    inside(root, directory);
    assert.equal(
      wasmContentHash(directory),
      match[2],
      `WASM bytes do not match ${backend} URL; rebuild both backends`,
    );
    builds[backend] = path;
    directories.push(name);
  }
  assert.deepEqual(
    readdirSync(root).sort(),
    ["builds.json", ...directories].sort(),
    "Stale or unexpected WASM outputs; rebuild before serving",
  );
  return Object.freeze(builds);
}
export function createWasmStage(projectRoot) {
  const root = publicRoot(projectRoot, true);
  const stage = join(root, `.stage-${randomUUID()}`);
  mkdirSync(stage);
  return stage;
}
export function removeWasmStage(projectRoot, stage) {
  const root = publicRoot(projectRoot);
  assert(
    /^\.stage-[a-f0-9-]{36}$/.test(relative(root, resolve(stage))),
    "Not a managed WASM staging directory",
  );
  if (!existsSync(stage)) return;
  inside(root, stage);
  safeGeneratedTree(stage, true);
  rmSync(stage, { recursive: true });
}
export function publishWasmBuilds(projectRoot, stage) {
  const root = publicRoot(projectRoot);
  inside(root, stage);
  assert(
    /^\.stage-[a-f0-9-]{36}$/.test(relative(root, resolve(stage))),
    "Not a managed WASM stage",
  );
  safeGeneratedTree(stage, true);
  // Preflight every existing cleanup target before any publication/deletion.
  for (const item of readdirSync(root, { withFileTypes: true })) {
    assert(
      !lstatSync(join(root, item.name)).isSymbolicLink(),
      "Refusing a linked public WASM entry",
    );
    if (item.name === "builds.json") {
      assert(
        item.isFile() && !lstatSync(join(root, item.name)).isSymbolicLink(),
        "Invalid WASM manifest file",
      );
      continue;
    }
    assert(
      item.isDirectory() && managed.test(item.name),
      `Unexpected public WASM entry: ${item.name}`,
    );
    inside(root, join(root, item.name));
    safeGeneratedTree(join(root, item.name), item.name.startsWith(".stage-"));
  }
  const planned = WASM_BACKENDS.map((backend) => {
    const source = join(stage, backend);
    const hash = wasmContentHash(source);
    return { backend, source, name: `${backend}-${hash}`, hash };
  });
  for (const entry of planned) {
    const destination = join(root, entry.name);
    if (existsSync(destination))
      assert.equal(
        wasmContentHash(destination),
        entry.hash,
        "Existing hashed WASM target is corrupt",
      );
  }
  for (const entry of planned) {
    const destination = join(root, entry.name);
    if (!existsSync(destination)) renameSync(entry.source, destination);
  }
  const builds = Object.fromEntries(
    planned.map(({ backend, name }) => [backend, `/wasm/${name}/galaxy.js`]),
  );
  const temporary = join(root, `.builds-${randomUUID()}.json`);
  writeFileSync(
    temporary,
    JSON.stringify({ version: 1, builds }, null, 2) + "\n",
  );
  renameSync(temporary, join(root, "builds.json"));
  const keep = new Set(planned.map(({ name }) => name));
  for (const item of readdirSync(root, { withFileTypes: true })) {
    if (!item.isDirectory() || keep.has(item.name)) continue;
    const target = join(root, item.name);
    assert(
      managed.test(item.name),
      "Refusing an unmanaged WASM cleanup target",
    );
    inside(root, target);
    safeGeneratedTree(target, item.name.startsWith(".stage-"));
    rmSync(target, { recursive: true });
  }
  return readWasmBuilds(projectRoot);
}
