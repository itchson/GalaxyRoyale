import { spawnSync } from "node:child_process";
import { readFileSync, mkdirSync, existsSync } from "node:fs";
import { resolve } from "node:path";
const root = resolve(import.meta.dirname, "..");
function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: root,
    stdio: "inherit",
    shell: process.platform === "win32" && command.endsWith(".cmd"),
  });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
if (!existsSync(resolve(root, "Cargo.lock")))
  run("cargo", ["generate-lockfile"]);
const lock = readFileSync(resolve(root, "Cargo.lock"), "utf8");
const version = lock.match(
  /name = "wasm-bindgen"\r?\nversion = "([^"]+)"/,
)?.[1];
if (!version)
  throw new Error("Cannot resolve wasm-bindgen version from Cargo.lock");
const installed = spawnSync("wasm-bindgen", ["--version"], {
  encoding: "utf8",
});
if (installed.status !== 0 || !installed.stdout.includes(version)) {
  run("cargo", [
    "install",
    "wasm-bindgen-cli",
    "--version",
    version,
    "--locked",
  ]);
}
for (const [feature, name] of [
  ["webgpu", "webgpu"],
  ["webgl", "webgl2"],
]) {
  run("cargo", [
    "build",
    "--locked",
    "--release",
    "--target",
    "wasm32-unknown-unknown",
    "-p",
    "game_web",
    "--no-default-features",
    "--features",
    feature,
  ]);
  const out = resolve(root, "apps/game/public/wasm", name);
  mkdirSync(out, { recursive: true });
  run("wasm-bindgen", [
    resolve(root, "target/wasm32-unknown-unknown/release/game_web.wasm"),
    "--target",
    "web",
    "--out-dir",
    out,
    "--out-name",
    "galaxy",
  ]);
}
