import fs from "node:fs/promises";
import path from "node:path";
import crypto from "node:crypto";
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import {
  BASELINE_COMMIT,
  BASELINE_PATH,
  BASELINE_SHA256,
  TYPESCRIPT_VERSION,
  PROJECT_ROOT,
  TOOL_DIRECTORY,
  HELPER_FILES,
  options,
  isMain,
} from "./config.mjs";

const sha = (bytes) => crypto.createHash("sha256").update(bytes).digest("hex");

export async function prepare(output) {
  // Resolve only the existing, lockfile-pinned dependency in this checkout.
  const ts = createRequire(path.join(PROJECT_ROOT, "package.json"))(
    "typescript",
  );
  const pkg = JSON.parse(
    await fs.readFile(path.join(PROJECT_ROOT, "package.json"), "utf8"),
  );
  if (
    ts.version !== TYPESCRIPT_VERSION ||
    pkg.devDependencies.typescript !== TYPESCRIPT_VERSION
  )
    throw new Error(
      `This review requires locked TypeScript ${TYPESCRIPT_VERSION}; run pnpm install --frozen-lockfile.`,
    );
  let baseline;
  try {
    baseline = execFileSync(
      "git",
      ["-C", PROJECT_ROOT, "show", `${BASELINE_COMMIT}:${BASELINE_PATH}`],
      { maxBuffer: 1024 * 1024, stdio: ["ignore", "pipe", "pipe"] },
    );
  } catch {
    throw new Error(
      `Historical source ${BASELINE_COMMIT}:${BASELINE_PATH} is unavailable. Fetch the repository history (git fetch --unshallow for a shallow clone) before preparing; no snapshot fallback is used.`,
    );
  }
  if (sha(baseline) !== BASELINE_SHA256)
    throw new Error(
      "Historical Git audio source differs from its accepted SHA256.",
    );

  const sources = [
    [
      "title-theme.js",
      "apps/game/src/title-theme.ts",
      await fs.readFile(
        path.join(PROJECT_ROOT, "apps/game/src/title-theme.ts"),
      ),
    ],
    [
      "audio.js",
      "apps/game/src/audio.ts",
      await fs.readFile(path.join(PROJECT_ROOT, "apps/game/src/audio.ts")),
    ],
    [
      "audio-state.js",
      "apps/game/src/audio-state.ts",
      await fs.readFile(
        path.join(PROJECT_ROOT, "apps/game/src/audio-state.ts"),
      ),
    ],
    ["baseline-audio.js", `git:${BASELINE_COMMIT}:${BASELINE_PATH}`, baseline],
  ];
  const inputs = {},
    modules = [];
  for (const [name, source, bytes] of sources) {
    const result = ts.transpileModule(bytes.toString("utf8"), {
      compilerOptions: {
        target: ts.ScriptTarget.ES2022,
        module: ts.ModuleKind.ES2022,
      },
      fileName: BASELINE_PATH,
      reportDiagnostics: true,
    });
    const errors = (result.diagnostics ?? []).filter(
      (d) => d.category === ts.DiagnosticCategory.Error,
    );
    if (errors.length)
      throw new Error(
        ts.formatDiagnosticsWithColorAndContext(errors, {
          getCurrentDirectory: () => PROJECT_ROOT,
          getCanonicalFileName: (f) => f,
          getNewLine: () => "\n",
        }),
      );
    const js =
      `// Actual source: ${source}; SHA256 ${sha(bytes)}\n` +
      result.outputText
        .replaceAll('"./audio-state.ts"', '"./audio-state.js"')
        .replaceAll('"./title-theme.ts"', '"./title-theme.js"');
    modules.push([name, js]);
    inputs[source] = {
      bytes: bytes.length,
      sha256: sha(bytes),
      module: name,
      module_sha256: sha(js),
    };
  }
  const helpers = {};
  for (const name of HELPER_FILES)
    helpers[name] = sha(await fs.readFile(path.join(TOOL_DIRECTORY, name)));
  const pins = {
    schema_version: 1,
    purpose: "isolated native OfflineAudioContext review; not shipped",
    typescript: ts.version,
    lockfile_sha256: sha(
      await fs.readFile(path.join(PROJECT_ROOT, "pnpm-lock.yaml")),
    ),
    baseline_shared_state:
      "Historical AudioEngine uses the current shared state mixer, matching the declared engine-only comparison; this is not a historical score snapshot.",
    inputs,
    helpers,
  };
  // Generated modules/captures stay ignored. No tracked runtime source is copied or edited.
  await fs.mkdir(path.join(output, "modules"), { recursive: true });
  for (const [name, js] of modules)
    await fs.writeFile(path.join(output, "modules", name), js);
  await fs.writeFile(
    path.join(output, "source-pins.json"),
    JSON.stringify(pins, null, 2) + "\n",
  );
  return pins;
}

if (isMain(import.meta.url)) {
  try {
    const args = options(process.argv.slice(2));
    if (args.help)
      console.log(
        "node scripts/qa/audio-review/prepare.mjs [--out work/audio-review]",
      );
    else {
      const pins = await prepare(args.output);
      console.log(
        JSON.stringify({
          output: args.output,
          actual_sources: Object.keys(pins.inputs).length,
          typescript: pins.typescript,
        }),
      );
    }
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
