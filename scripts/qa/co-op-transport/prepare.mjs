import fs from "node:fs/promises";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";

export const root = path.resolve(
  fileURLToPath(new URL("../../..", import.meta.url)),
);
export const directory = fileURLToPath(new URL(".", import.meta.url));
export const sha = (bytes) =>
  crypto.createHash("sha256").update(bytes).digest("hex");
export const moduleNames = ["wire", "signaling", "transport"];
export const pageNames = ["review.html", "review.js"];
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
export async function prepare(value) {
  const stage = stagePath(value);
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
  for (const name of moduleNames) {
    const source = `apps/game/src/coop/${name}.ts`,
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
        .replaceAll('"./signaling.ts"', '"./signaling.js"');
    const relative = `modules/${name}.js`;
    await fs.writeFile(path.join(stage, relative), output);
    inputs[source] = sha(bytes);
    files[relative] = sha(output);
  }
  for (const name of pageNames) {
    const bytes = await fs.readFile(path.join(directory, name));
    await fs.writeFile(path.join(stage, name), bytes);
    inputs[`scripts/qa/co-op-transport/${name}`] = sha(bytes);
    files[name] = sha(bytes);
  }
  const manifest = {
    version: 1,
    typescript: ts.version,
    scope: "actual browser local RTC loopback; no Game authority",
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
  if (args.length !== 2 || args[0] !== "--stage")
    throw new Error(
      "Usage: node scripts/qa/co-op-transport/prepare.mjs --stage work/co-op-transport/stage",
    );
  const manifest = await prepare(args[1]);
  console.log(
    JSON.stringify({
      prepared: stagePath(args[1]),
      modules: moduleNames.length,
      inputs: Object.keys(manifest.inputs).length,
    }),
  );
}
