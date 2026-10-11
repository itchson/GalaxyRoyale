import fs from "node:fs/promises";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import { wasmContentHash, WASM_FILES } from "../../wasm-artifacts.mjs";

export const root = path.resolve(
  fileURLToPath(new URL("../../..", import.meta.url)),
);
export const directory = fileURLToPath(new URL(".", import.meta.url));
export const sha = (bytes) =>
  crypto.createHash("sha256").update(bytes).digest("hex");
export const moduleNames = ["wire", "signaling", "transport"];
export const pageNames = ["review.html", "review.js"];
export const wasmModuleNames = [...moduleNames, "controller", "wasm-ports"];
export const wasmPageNames = [
  "wasm-review.html",
  "wasm-review.js",
  "client.html",
  "client.js",
];
export const fileLimit = (name) =>
  name.endsWith(".wasm")
    ? 32 * 1024 * 1024
    : name.endsWith(".d.ts")
      ? 100_000
      : 250_000;
export function preparedNames(manifest) {
  if (manifest.version === 1 && manifest.mode === undefined)
    return [...pageNames, ...moduleNames.map((n) => `modules/${n}.js`)];
  if (
    manifest.version !== 2 ||
    manifest.mode !== "wasm" ||
    !manifest.builds ||
    Object.keys(manifest.builds).sort().join() !== "webgl2,webgpu"
  )
    throw new Error("Invalid prepared mode.");
  const names = [
    ...wasmPageNames,
    ...wasmModuleNames.map((n) => `modules/${n}.js`),
    "modules/storage.js",
  ];
  for (const backend of ["webgpu", "webgl2"]) {
    const match = manifest.builds[backend]?.match(
      /^\/wasm\/(webgpu|webgl2)-([a-f0-9]{64})\/galaxy\.js$/,
    );
    if (!match || match[1] !== backend)
      throw new Error("Invalid pinned backend.");
    names.push(...WASM_FILES.map((n) => `wasm/${backend}-${match[2]}/${n}`));
  }
  return names;
}
async function regular(pathname, limit) {
  const stat = await fs.lstat(pathname);
  if (!stat.isFile() || stat.isSymbolicLink() || stat.size > limit)
    throw new Error("Invalid bounded input file.");
  return fs.readFile(pathname);
}
export function stagePath(value) {
  if (typeof value !== "string" || !value)
    throw new Error("Provide an explicit work stage path.");
  const target = path.resolve(value),
    work = path.join(root, "work");
  if (!target.startsWith(work + path.sep))
    throw new Error(
      "Stage must be a subdirectory of this checkout's work directory.",
    );
  return target;
}
export async function prepare(value, { dist } = {}) {
  const stage = stagePath(value);
  if (await fs.stat(path.join(stage, "manifest.json")).catch(() => null))
    throw new Error(
      "Choose a fresh stage; frozen manifests/captures are not overwritten.",
    );
  const ts = createRequire(path.join(root, "package.json"))("typescript");
  const pkg = JSON.parse(
    await fs.readFile(path.join(root, "package.json"), "utf8"),
  );
  if (ts.version !== "5.9.3" || pkg.devDependencies.typescript !== "5.9.3")
    throw new Error(
      "Install lockfile-pinned TypeScript5.9.3 before preparing.",
    );
  await fs.mkdir(path.join(stage, "modules"), { recursive: true });
  const inputs = {},
    files = {};
  for (const name of dist ? [...wasmModuleNames, "storage"] : moduleNames) {
    const source =
        name === "storage"
          ? "apps/game/src/storage.ts"
          : `apps/game/src/coop/${name}.ts`,
      bytes = await fs.readFile(path.join(root, source));
    const result = ts.transpileModule(bytes.toString("utf8"), {
      compilerOptions: {
        target: ts.ScriptTarget.ES2022,
        module: ts.ModuleKind.ES2022,
      },
      fileName: source,
      reportDiagnostics: true,
    });
    if (
      result.diagnostics?.some(
        (d) => d.category === ts.DiagnosticCategory.Error,
      )
    )
      throw new Error(
        "Transport source transpilation failed; run typecheck for diagnostics.",
      );
    const output =
      `// Actual source SHA256 ${sha(bytes)}\n` +
      result.outputText
        .replaceAll('"./wire.ts"', '"./wire.js"')
        .replaceAll('"./signaling.ts"', '"./signaling.js"')
        .replaceAll('"./transport.ts"', '"./transport.js"')
        .replaceAll('"./controller.ts"', '"./controller.js"');
    const relative = `modules/${name}.js`;
    await fs.writeFile(path.join(stage, relative), output);
    inputs[source] = sha(bytes);
    files[relative] = sha(output);
  }
  for (const name of dist ? wasmPageNames : pageNames) {
    const bytes = await fs.readFile(path.join(directory, name));
    await fs.writeFile(path.join(stage, name), bytes);
    inputs[`scripts/qa/co-op-transport/${name}`] = sha(bytes);
    files[name] = sha(bytes);
  }
  let builds, artifacts;
  if (dist) {
    if (!path.isAbsolute(dist) || path.basename(dist) !== "dist")
      throw new Error("Use an explicit absolute normal dist directory.");
    const wasmRoot = path.join(dist, "wasm");
    const mapping = JSON.parse(
      (await regular(path.join(wasmRoot, "builds.json"), 4096)).toString(
        "utf8",
      ),
    );
    if (
      mapping.version !== 1 ||
      Object.keys(mapping).sort().join() !== "builds,version" ||
      Object.keys(mapping.builds ?? {})
        .sort()
        .join() !== "webgl2,webgpu"
    )
      throw new Error("Invalid normal backend map.");
    builds = mapping.builds;
    artifacts = [];
    for (const backend of ["webgpu", "webgl2"]) {
      const entry = builds[backend],
        match = entry?.match(
          /^\/wasm\/(webgpu|webgl2)-([a-f0-9]{64})\/galaxy\.js$/,
        );
      if (!match || match[1] !== backend)
        throw new Error("Invalid normal backend URL.");
      const sourceDir = path.join(dist, "wasm", `${backend}-${match[2]}`);
      if (wasmContentHash(sourceDir) !== match[2])
        throw new Error("Normal backend content hash mismatch.");
      for (const name of WASM_FILES) {
        const bytes = await regular(
          path.join(sourceDir, name),
          fileLimit(name),
        );
        if (
          name.endsWith(".wasm") &&
          !bytes
            .subarray(0, 8)
            .equals(Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]))
        )
          throw new Error("Not a normal WASM binary.");
        const relative = `wasm/${backend}-${match[2]}/${name}`;
        await fs.mkdir(path.dirname(path.join(stage, relative)), {
          recursive: true,
        });
        await fs.writeFile(path.join(stage, relative), bytes);
        files[relative] = sha(bytes);
        artifacts.push({
          path: relative,
          bytes: bytes.length,
          sha256: sha(bytes),
        });
      }
    }
    // Re-read every source after copying; a concurrent normal rebuild must fail visibly.
    for (const item of artifacts)
      if (sha(await fs.readFile(path.join(dist, item.path))) !== item.sha256)
        throw new Error("Normal dist changed during preparation.");
  }
  const manifest = {
    version: dist ? 2 : 1,
    ...(dist
      ? { mode: "wasm", builds, artifacts, dist: path.resolve(dist) }
      : {}),
    typescript: ts.version,
    scope: dist
      ? "prepared real two-Galaxy WASM/RTC QA; browser execution pending; factory guest v1, no shared progression/NAT acceptance"
      : "actual browser local RTC loopback; no Game authority",
    inputs,
    files,
  };
  await fs.writeFile(
    path.join(stage, "manifest.json"),
    JSON.stringify(manifest, null, 2) + "\n",
  );
  return manifest;
}
if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  const args = process.argv.slice(2);
  if (
    ![2, 4].includes(args.length) ||
    args[0] !== "--stage" ||
    (args.length === 4 && args[2] !== "--dist")
  )
    throw new Error(
      "Usage: node scripts/qa/co-op-transport/prepare.mjs --stage work/co-op-transport/stage [--dist ABSOLUTE_ROOT_DIST]",
    );
  const manifest = await prepare(args[1], { dist: args[3] });
  console.log(
    JSON.stringify({
      prepared: stagePath(args[1]),
      modules:
        manifest.version === 2
          ? wasmModuleNames.length + 1
          : moduleNames.length,
      inputs: Object.keys(manifest.inputs).length,
    }),
  );
}
