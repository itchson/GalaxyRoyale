import { spawn } from "node:child_process";
const children = [
  ["apps/studio/server.mjs"],
  ["node_modules/vite/bin/vite.js", "--config", "apps/studio/vite.config.ts"],
  ["node_modules/vite/bin/vite.js", "--config", "apps/game/vite.config.ts"],
].map((args) =>
  spawn(process.execPath, args, {
    cwd: new URL("..", import.meta.url),
    stdio: "inherit",
  }),
);
function stop() {
  for (const child of children) child.kill();
}
process.on("SIGINT", () => {
  stop();
  process.exit();
});
process.on("SIGTERM", () => {
  stop();
  process.exit();
});
for (const child of children)
  child.on("exit", (code) => {
    if (code && code !== 0) {
      stop();
      process.exitCode = code;
    }
  });
