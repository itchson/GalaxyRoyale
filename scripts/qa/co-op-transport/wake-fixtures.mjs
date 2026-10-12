import fs from "node:fs/promises";
import path from "node:path";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";

export const WAKE_SCENARIO = "configured-ion-wake-r1-e1";
export const WAKE_FIXTURE_COMMIT = "87d92f2cbf10c24dda585b47f05de2fad6fe178f";
export const wakePreparedFiles = Object.freeze([
  "wake-observation.js",
  "fixtures/configured-wake/provenance.json",
  "fixtures/configured-wake/host-profile.json",
  "fixtures/configured-wake/guest-profile.json",
]);
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
export async function readWakeFixtures(directory, sourceRoot) {
  const raw = await fs.readFile(path.join(directory, wakePreparedFiles[1]));
  const metadata = JSON.parse(raw);
  if (
    Object.keys(metadata).sort().join() !==
      "fixture_source_commit,profiles,provenance,scenario,version" ||
    metadata.version !== 1 ||
    metadata.scenario !== WAKE_SCENARIO ||
    metadata.provenance !==
      "configured native fixture; NOT earned progression" ||
    metadata.fixture_source_commit !== WAKE_FIXTURE_COMMIT ||
    Object.keys(metadata.profiles ?? {})
      .sort()
      .join() !== "guest,host"
  )
    throw new Error("Invalid configured native fixture provenance.");
  const files = new Map([[wakePreparedFiles[1], raw]]);
  for (const role of ["host", "guest"]) {
    const expected = metadata.profiles[role],
      relative = `fixtures/configured-wake/${role}-profile.json`;
    const bytes = await fs.readFile(path.join(directory, relative));
    const original = execFileSync(
      "git",
      [
        "show",
        `${WAKE_FIXTURE_COMMIT}:work/coop-wake-live-fixtures/native-fixtures/${role}-profile.json`,
      ],
      { cwd: sourceRoot, maxBuffer: 65_536, stdio: ["ignore", "pipe", "pipe"] },
    );
    const response = JSON.parse(
      execFileSync(
        "git",
        [
          "show",
          `${WAKE_FIXTURE_COMMIT}:work/coop-wake-live-fixtures/native-fixtures/${role}-native-export-response.json`,
        ],
        {
          cwd: sourceRoot,
          maxBuffer: 65_536,
          stdio: ["ignore", "pipe", "pipe"],
        },
      ),
    );
    if (
      Object.keys(expected).sort().join() !==
        "bytes,native_export,path,schema_version,seed,sha256" ||
      expected.path !== relative ||
      expected.bytes !== original.length ||
      expected.sha256 !== sha(original) ||
      bytes.length > 65_536 ||
      !bytes.equals(original) ||
      JSON.stringify(expected.native_export) !==
        JSON.stringify(response.results[0].value)
    )
      throw new Error(
        "Configured profile differs from actual frozen native output.",
      );
    const profile = JSON.parse(bytes);
    if (
      expected.schema_version !== 7 ||
      expected.seed !== profile.seed ||
      !profile.seed
    )
      throw new Error("Invalid configured profile scalar provenance.");
    files.set(relative, bytes);
  }
  if (metadata.profiles.host.seed === metadata.profiles.guest.seed)
    throw new Error("Configured native role seeds must remain distinct.");
  return { metadata, files };
}
export async function verifyWakeStage(manifest, directory, sourceRoot) {
  const fixture = await readWakeFixtures(directory, sourceRoot);
  if (
    manifest.version !== 3 ||
    manifest.mode !== "wasm" ||
    manifest.profile_mode !== "owned" ||
    manifest.configured_wake !== true ||
    manifest.fifth_shot !== true ||
    JSON.stringify(manifest.wake_fixture) !== JSON.stringify(fixture.metadata)
  )
    throw new Error("Configured native fixture provenance changed.");
  const pages = [
    "wasm-review.html",
    "wasm-review.js",
    "client.html",
    "client.js",
    "fifth-shot.js",
    ...wakePreparedFiles,
  ];
  for (const name of [
    ...pages,
    "wake-fixtures.mjs",
    "server.mjs",
    "prepare.mjs",
  ]) {
    const bytes =
      fixture.files.get(name) ??
      (await fs.readFile(path.join(directory, name)));
    const expected = sha(bytes);
    if (
      manifest.inputs?.[`scripts/qa/co-op-transport/${name}`] !== expected ||
      (pages.includes(name) && manifest.files?.[name] !== expected)
    )
      throw new Error(
        "Configured Wake helper/fixture source pins are incomplete or changed.",
      );
  }
  return fixture;
}
