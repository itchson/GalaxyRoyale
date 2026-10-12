/** Real provenance/QA preparation checks. No WASM execution, RTC, HTTP listener or saved profile. */
import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { execFileSync } from "node:child_process";
import ts from "typescript";
import { buildIdentity, treeHash } from "../preview-build.mjs";
import {
  root,
  ownedSources,
  moduleNames,
  serviceSources,
  sha,
  prepare,
  preparedNames,
  verifyOwnedSource,
  readServiceInputs,
  verifyServiceInputs,
} from "../qa/co-op-transport/prepare.mjs";

test("historical transport manifests keep their frozen files and current manifests require their pinned room-code module", () => {
  const historical = preparedNames({ version: 1 });
  assert.equal(historical.length, 5);
  assert(!historical.includes("modules/room-code.js"));
  const files = Object.fromEntries(
    historical.map((name) => [name, "a".repeat(64)]),
  );
  assert.deepEqual(preparedNames({ version: 1, files }), historical);
  const current = preparedNames({
    version: 1,
    files: { ...files, "modules/room-code.js": "b".repeat(64) },
  });
  assert.equal(current.length, 6);
  assert(current.includes("modules/room-code.js"));
  assert.deepEqual(
    current.filter((name) => name !== "modules/room-code.js"),
    historical,
  );
});

test("historical factory and owned WASM manifests preserve eighteen or nineteen files while current stages serve the pinned extra module", () => {
  const builds = {
    webgpu: `/wasm/webgpu-${"a".repeat(64)}/galaxy.js`,
    webgl2: `/wasm/webgl2-${"b".repeat(64)}/galaxy.js`,
  };
  for (const version of [2, 3]) {
    for (const fifth_shot of [false, true]) {
      const mode = {
        version,
        mode: "wasm",
        builds,
        fifth_shot,
        ...(version === 3 ? { profile_mode: "owned" } : {}),
      };
      const historical = preparedNames(mode);
      assert.equal(historical.length, fifth_shot ? 19 : 18);
      assert(!historical.includes("modules/room-code.js"));
      const files = Object.fromEntries(
        historical.map((name) => [name, "c".repeat(64)]),
      );
      assert.deepEqual(preparedNames({ ...mode, files }), historical);
      const current = preparedNames({
        ...mode,
        files: { ...files, "modules/room-code.js": "d".repeat(64) },
      });
      assert.equal(current.length, fifth_shot ? 20 : 19);
      assert(current.includes("modules/room-code.js"));
      assert.deepEqual(
        current.filter((name) => name !== "modules/room-code.js"),
        historical,
      );
      assert.equal(current.includes("fifth-shot.js"), fifth_shot);
      assert(current.includes("modules/storage.js"));
    }
  }
});

test("room-code admission uses an own prepared-file marker and never admits arbitrary module or inherited fields", () => {
  const files = Object.assign(
    Object.create({ "modules/room-code.js": "a".repeat(64) }),
    { "modules/unshipped-secret.js": "b".repeat(64) },
  );
  const historical = preparedNames({ version: 1, files });
  assert(!historical.includes("modules/room-code.js"));
  assert(!historical.includes("modules/unshipped-secret.js"));
  files["modules/room-code.js"] = "c".repeat(64);
  const current = preparedNames({ version: 1, files });
  assert(current.includes("modules/room-code.js"));
  assert(!current.includes("modules/unshipped-secret.js"));
});

const evidenceRoot = path.join(
  root,
  "work/private-room-live-plan/provenance-fixtures",
);
async function fixture(carried = true) {
  await fs.mkdir(evidenceRoot, { recursive: true });
  const source = await fs.mkdtemp(path.join(evidenceRoot, "declared-"));
  const run = (...args) =>
    execFileSync(
      "git",
      [
        "-c",
        "user.name=Codex declared QA fixture",
        "-c",
        "user.email=codex@openai.com",
        ...args,
      ],
      { cwd: source, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
    ).trim();
  for (const directory of ["crates", "assets", "dist", "work"])
    await fs.mkdir(path.join(source, directory), { recursive: true });
  for (const relative of [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "package.json",
    "pnpm-lock.yaml",
    "tsconfig.json",
    "scripts/build.mjs",
    "scripts/build-wasm.mjs",
    "scripts/pwa.mjs",
    "scripts/wasm-artifacts.mjs",
    "scripts/preview-build.mjs",
    "scripts/game-runtime-lease.mjs",
    ...ownedSources,
    ...serviceSources,
  ]) {
    await fs.mkdir(path.dirname(path.join(source, relative)), {
      recursive: true,
    });
    await fs.writeFile(
      path.join(source, relative),
      "// declared provenance input; never executed\n",
    );
  }
  await fs.writeFile(
    path.join(source, "dist/index.html"),
    "declared package bytes; not a build",
  );
  run("init", "-b", "codex/testing");
  run("config", "core.autocrlf", "false");
  run("add", ".");
  run("commit", "-m", "Declared compiled-revision fixture; no build performed");
  const compiled = await buildIdentity(source);
  if (carried) {
    await fs.mkdir(path.join(source, "docs"));
    await fs.writeFile(
      path.join(source, "docs/record.md"),
      "declared record-only revision",
    );
    run("add", "docs/record.md");
    run("commit", "-m", "Declared record-only carry fixture");
  }
  const current = await buildIdentity(source);
  const stamp = {
    schemaVersion: 1,
    ...current,
    packageHash: await treeHash(source, ["dist"]),
    builtAt: "2026-10-12T00:00:00.000Z",
    ...(carried
      ? {
          compiledFromHead: compiled.head,
          carriedAt: "2026-10-12T00:01:00.000Z",
          carryReason: "Declared record-only carry; no new build.",
        }
      : {}),
  };
  const save = (value = stamp) =>
    fs.writeFile(
      path.join(source, "work/preview-build.json"),
      JSON.stringify(value),
    );
  await save();
  return {
    source,
    run,
    compiled,
    current,
    stamp,
    save,
    options: {
      sourceRoot: source,
      sourceCommit: current.head,
      dist: path.join(source, "dist"),
    },
  };
}

test("actual owned verification accepts original and fully validated record-only carry schemas", async () => {
  for (const carried of [false, true]) {
    const f = await fixture(carried);
    const accepted = await verifyOwnedSource(f.options);
    assert.equal(accepted.compiledSourceCommit, f.compiled.head);
    assert.equal(accepted.blobs.size, ownedSources.length);
    assert.deepEqual(accepted.gitInputs, accepted.compiledGitInputs);
    assert.deepEqual(accepted.buildStamp, f.stamp);
  }
});

test("carry provenance fails closed on unknown, partial, malformed or temporally inconsistent fields", async () => {
  const f = await fixture();
  const cases = [
    { ...f.stamp, extra: "unknown" },
    { ...f.stamp, compiledFromHead: "not-a-commit" },
    { ...f.stamp, compiledFromHead: f.current.head },
    { ...f.stamp, carriedAt: "2026-10-11T23:59:59.000Z" },
    { ...f.stamp, carriedAt: "2026-10-12" },
    { ...f.stamp, carryReason: " " },
    { ...f.stamp, carryReason: "a".repeat(513) },
    { ...f.stamp, carryReason: "control\ncharacter" },
    { ...f.stamp, carryReason: 17 },
  ];
  const partial = { ...f.stamp };
  delete partial.carriedAt;
  cases.push(partial);
  for (const stamp of cases) {
    await f.save(stamp);
    await assert.rejects(verifyOwnedSource(f.options), /normal build record/);
  }
});

test("carry cannot claim an unrelated Git revision or altered compiled runtime even with restamped live hashes", async () => {
  const f = await fixture();
  const unrelated = f.run(
    "commit-tree",
    f.run("rev-parse", "HEAD^{tree}"),
    "-m",
    "Declared unrelated revision",
  );
  await f.save({ ...f.stamp, compiledFromHead: unrelated });
  await assert.rejects(verifyOwnedSource(f.options), /not an ancestor/);
  await fs.writeFile(
    path.join(f.source, "crates/changed.rs"),
    "// changed compiled dependency\n",
  );
  f.run("add", "crates/changed.rs");
  f.run("commit", "-m", "Declared runtime mismatch");
  const changed = await buildIdentity(f.source);
  await f.save({
    ...f.stamp,
    head: changed.head,
    sourceHash: changed.sourceHash,
  });
  await assert.rejects(
    verifyOwnedSource({ ...f.options, sourceCommit: changed.head }),
    /compiled runtime dependency changed/,
  );
});

test("current live adapter, package and checkout mismatches remain rejected", async () => {
  const f = await fixture();
  await fs.appendFile(
    path.join(f.source, ownedSources[0]),
    "// dirty dependency\n",
  );
  await assert.rejects(verifyOwnedSource(f.options), /changed since/);
  await f.save({
    ...f.stamp,
    sourceHash: (await buildIdentity(f.source)).sourceHash,
  });
  await assert.rejects(verifyOwnedSource(f.options), /exact compiled source/);
  const original = f.run("show", `${f.current.head}:${ownedSources[0]}`) + "\n";
  await fs.writeFile(path.join(f.source, ownedSources[0]), original);
  await f.save();
  await fs.appendFile(
    path.join(f.source, "dist/index.html"),
    "changed package",
  );
  await assert.rejects(verifyOwnedSource(f.options), /changed since/);
  await fs.writeFile(
    path.join(f.source, "dist/index.html"),
    "declared package bytes; not a build",
  );
  await fs.writeFile(
    path.join(f.source, "docs/next.md"),
    "another declared record-only revision",
  );
  f.run("add", "docs/next.md");
  f.run("commit", "-m", "Declared current HEAD mismatch");
  await assert.rejects(verifyOwnedSource(f.options), /changed since/);
  const next = await buildIdentity(f.source);
  await f.save({ ...f.stamp, head: next.head });
  f.run("checkout", "-b", "codex/other");
  await assert.rejects(
    verifyOwnedSource({ ...f.options, sourceCommit: next.head }),
    /changed since/,
  );
});

test("actual QA service source pins detect server/store change and reject unknown pin fields", async () => {
  const f = await fixture(false);
  const pins = await readServiceInputs(f.source);
  assert.deepEqual(Object.keys(pins).sort(), [...serviceSources].sort());
  await verifyServiceInputs(pins, f.source);
  await assert.rejects(
    verifyServiceInputs({ ...pins, extra: "a".repeat(64) }, f.source),
    /Invalid actual QA service/,
  );
  await assert.rejects(
    verifyServiceInputs({ ...pins, [serviceSources[0]]: 17 }, f.source),
    /Invalid actual QA service/,
  );
  for (const relative of serviceSources) {
    const file = path.join(f.source, relative),
      original = await fs.readFile(file);
    await fs.appendFile(file, "// changed service\n");
    await assert.rejects(
      verifyServiceInputs(pins, f.source),
      /service source changed/,
    );
    await fs.writeFile(file, original);
  }
});

test("actual transport-only preparation closes room-code JS imports and pins the Root service used by QA", async () => {
  await fs.mkdir(evidenceRoot, { recursive: true });
  const stage = await fs.mkdtemp(
    path.join(evidenceRoot, "actual-transport-stage-"),
  );
  const manifest = await prepare(stage);
  assert(moduleNames.includes("room-code"));
  assert.deepEqual(
    preparedNames(manifest).sort(),
    Object.keys(manifest.files).sort(),
  );
  for (const name of moduleNames) {
    const file = path.join(stage, "modules", `${name}.js`);
    const source = await fs.readFile(file, "utf8");
    const ast = ts.createSourceFile(
      file,
      source,
      ts.ScriptTarget.ES2022,
      true,
      ts.ScriptKind.JS,
    );
    for (const statement of ast.statements) {
      if (
        !ts.isImportDeclaration(statement) ||
        !ts.isStringLiteral(statement.moduleSpecifier)
      )
        continue;
      const specifier = statement.moduleSpecifier.text;
      if (!specifier.startsWith(".")) continue;
      assert(
        specifier.endsWith(".js"),
        `${name} has an unserved import ${specifier}`,
      );
      assert(
        (await fs.stat(path.resolve(path.dirname(file), specifier))).isFile(),
      );
    }
  }
  const { CoopTransport } = await import(
    pathToFileURL(path.join(stage, "modules/transport.js")).href
  );
  assert.equal(typeof CoopTransport, "function");
  const actualPins = await readServiceInputs();
  assert.deepEqual(manifest.service_inputs, actualPins);
  for (const relative of serviceSources) {
    assert.equal(
      manifest.inputs[relative],
      sha(await fs.readFile(path.join(root, relative))),
    );
    assert.equal(manifest.inputs[relative], manifest.service_inputs[relative]);
  }
});
