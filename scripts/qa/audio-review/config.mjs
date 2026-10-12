import path from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

export const TOOL_DIRECTORY = path.dirname(fileURLToPath(import.meta.url));
export const PROJECT_ROOT = path.resolve(TOOL_DIRECTORY, "../../..");
export const TYPESCRIPT_VERSION = "5.9.3";
export const BASELINE_COMMIT = "3bf0c02b5e9fc0011c0ce6921bfaf0fbe664c65d";
export const BASELINE_PATH = "apps/game/src/audio.ts";
export const BASELINE_SHA256 =
  "6e06ef04f40367ab8519adfd445258b0c284d6b1c7ac3d955ea212962c629f64";
export const MODULE_NAMES = [
  "audio.js", "audio-state.js", "title-theme.js", "music-clock.js", "music-score.js", "baseline-audio.js",
];
export const HELPER_FILES = [
  "config.mjs",
  "prepare.mjs",
  "server.mjs",
  "review.js",
  "metrics.js",
  "index.html",
];

export function outputDirectory(value, root = PROJECT_ROOT) {
  const work = path.join(root, "work");
  const output = path.resolve(root, value ?? "work/audio-review");
  const relative = path.relative(work, output);
  if (
    !relative ||
    relative === ".." ||
    relative.startsWith(`..${path.sep}`) ||
    path.isAbsolute(relative)
  )
    throw new Error(
      "--out must name a directory inside this checkout's work/ directory.",
    );
  return output;
}

export function options(args, server = false) {
  const { values } = parseArgs({
    args,
    strict: true,
    allowPositionals: false,
    options: {
      out: { type: "string" },
      help: { type: "boolean" },
      ...(server
        ? {
            port: { type: "string", default: "4188" },
            "autoquit-seconds": { type: "string", default: "1800" },
          }
        : {}),
    },
  });
  const result = {
    output: outputDirectory(values.out),
    help: values.help ?? false,
  };
  if (server) {
    const integer = (name, min, max) => {
      const value = values[name];
      if (!/^\d+$/.test(value) || Number(value) < min || Number(value) > max)
        throw new Error(`--${name} must be an integer in ${min}..${max}.`);
      return Number(value);
    };
    result.port = integer("port", 1, 65535);
    result.autoquitSeconds = integer("autoquit-seconds", 0, 86400);
  }
  return result;
}

export function isMain(url) {
  return (
    process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(url)
  );
}
