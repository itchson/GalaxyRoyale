import { spawnSync } from "node:child_process";
import {writePreviewBuild} from './preview-build.mjs';
import {fileURLToPath} from 'node:url';
const commands = [
  ["node", ["scripts/build-wasm.mjs"]],
  ["node", ["node_modules/typescript/bin/tsc", "--noEmit"]],
  [
    "node",
    [
      "node_modules/vite/bin/vite.js",
      "build",
      "--config",
      "apps/game/vite.config.ts",
    ],
  ],
  ["node", ["scripts/pwa.mjs"]],
  [
    "node",
    [
      "node_modules/vite/bin/vite.js",
      "build",
      "--config",
      "apps/studio/vite.config.ts",
    ],
  ],
];
for (const [command, args] of commands) {
  const result = spawnSync(command, args, {
    cwd: new URL("..", import.meta.url),
    stdio: "inherit",
  });
  if (result.status !== 0) process.exit(result.status ?? 1);
}
await writePreviewBuild(fileURLToPath(new URL('..',import.meta.url)));
