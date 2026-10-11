import assert from "node:assert/strict";
import { test, before, after } from "node:test";
import fs from "node:fs/promises";
import path from "node:path";
import http from "node:http";
import crypto from "node:crypto";
import { once } from "node:events";
import { execFileSync, spawnSync } from "node:child_process";
import {
  PROJECT_ROOT,
  TOOL_DIRECTORY,
  BASELINE_COMMIT,
  BASELINE_PATH,
  BASELINE_SHA256,
  outputDirectory,
  options,
  MODULE_NAMES,
} from "./config.mjs";
import { prepare } from "./prepare.mjs";
import {
  CAPTURE_LIMIT,
  staticFile,
  capturePath,
  validateOutput,
  createReviewServer,
  serve,
} from "./server.mjs";

const sha = (bytes) => crypto.createHash("sha256").update(bytes).digest("hex");
let output, pins;
before(async () => {
  await fs.mkdir(path.join(PROJECT_ROOT, "work"), { recursive: true });
  output = await fs.mkdtemp(
    path.join(PROJECT_ROOT, "work", "audio-review-helper-test-"),
  );
  pins = await prepare(output);
});
// Test outputs stay in their unique ignored directory; no previous capture is removed.
after(() => console.log(`Helper evidence output: ${output}`));

test("CLI constrains output to work/, ports and bounded autoquit without silently changing an occupied port", () => {
  assert.equal(
    options([]).output,
    path.join(PROJECT_ROOT, "work/audio-review"),
  );
  assert.equal(options([], true).port, 4188);
  assert.equal(options([], true).autoquitSeconds, 1800);
  assert.equal(
    options(["--port", "49123", "--autoquit-seconds", "0"], true).port,
    49123,
  );
  for (const value of ["work", "work/../apps", "../another-checkout/work"])
    assert.throws(() => outputDirectory(value));
  for (const value of ["0", "65536", "1.5", "-1", "4188junk"])
    assert.throws(() => options(["--port", value], true));
  assert.throws(() => options(["--autoquit-seconds", "86401"], true));
  assert.throws(() => options(["--unexpected"]));
});

test("prepare compiles real tracked sources and exact historical Git bytes with locked TypeScript", async () => {
  assert.equal(pins.typescript, "5.9.3");
  const baseline = execFileSync("git", [
    "-C",
    PROJECT_ROOT,
    "show",
    `${BASELINE_COMMIT}:${BASELINE_PATH}`,
  ]);
  assert.equal(sha(baseline), BASELINE_SHA256);
  for (const [source, input] of Object.entries(pins.inputs)) {
    const actual = source.startsWith("git:")
      ? baseline
      : await fs.readFile(path.join(PROJECT_ROOT, source));
    assert.equal(sha(actual), input.sha256);
    assert.equal(actual.length, input.bytes);
    const js = await fs.readFile(path.join(output, "modules", input.module));
    assert.equal(sha(js), input.module_sha256);
    assert(
      js
        .toString()
        .startsWith(`// Actual source: ${source}; SHA256 ${input.sha256}\n`),
    );
    assert(!js.toString().includes('"./audio-state.ts"'));
  }
  const beforeBytes = await fs.readFile(path.join(output, "source-pins.json"));
  await prepare(output);
  assert.deepEqual(
    await fs.readFile(path.join(output, "source-pins.json")),
    beforeBytes,
  );
  await validateOutput(output);
});

test("server refuses an altered prepared module instead of reporting false source pins", async () => {
  const file = path.join(output, "modules/audio.js"),
    original = await fs.readFile(file);
  try {
    await fs.appendFile(file, "\n// declared corruption fixture\n");
    await assert.rejects(validateOutput(output), /does not match source-pins/);
  } finally {
    await fs.writeFile(file, original);
  }
  await validateOutput(output);
});

test("static and capture allowlists cannot address source, arbitrary paths or malformed run IDs", () => {
  assert.equal(
    staticFile("/", output),
    path.join(TOOL_DIRECTORY, "index.html"),
  );
  for (const module of MODULE_NAMES)
    assert.equal(
      staticFile(`/modules/${module}`, output),
      path.join(output, "modules", module),
    );
  for (const url of [
    "/../package.json",
    "/apps/game/src/audio.ts",
    "/captures/report.json",
    "/modules/../audio.js",
    "/%2e%2e/package.json",
    "index.html",
  ])
    assert.equal(staticFile(url, output), null);
  assert(capturePath("/capture/qa-1791686562452-e304e589/report.json", output));
  assert.equal(
    capturePath("/capture/qa-1791686562452-e304e589/current-score.wav", output),
    path.join(
      output,
      "captures",
      "qa-1791686562452-e304e589",
      "current-score.wav",
    ),
  );
  for (const url of [
    "/capture/qa-1791686562452-e304e589/other.json",
    "/capture/../report.json",
    "/capture/qa-1791686562452-e304e589/nested/report.json",
    "/capture/qa-1791686562452-e304e589/current-score-extra.wav",
  ])
    assert.equal(capturePath(url, output), null);
});

async function listeningServer() {
  const server = createReviewServer(output);
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  return { server, origin: `http://127.0.0.1:${server.address().port}` };
}
async function close(server) {
  const stopped = new Promise((resolve) => server.close(resolve));
  server.closeAllConnections();
  await stopped;
}

test("actual loopback HTTP serves pinned modules and preserves captures once, rejecting unrelated Origins and oversized input", async () => {
  const { server, origin } = await listeningServer();
  try {
    const module = await fetch(`${origin}/modules/audio.js`);
    assert.equal(module.status, 200);
    assert.equal(
      sha(Buffer.from(await module.arrayBuffer())),
      Object.values(pins.inputs).find((p) => p.module === "audio.js")
        .module_sha256,
    );
    assert.equal((await fetch(`${origin}/apps/game/src/audio.ts`)).status, 404);
    const route = `${origin}/capture/qa-${Date.now()}-aaaaaaaa/report.json`;
    const body = '{"scope":"HTTP preservation test only; no PCM render"}\n';
    assert.equal(
      (
        await fetch(route, {
          method: "POST",
          body,
          headers: { Origin: "https://other.invalid" },
        })
      ).status,
      403,
    );
    assert.equal(
      (
        await fetch(route, {
          method: "POST",
          body,
          headers: { Origin: origin },
        })
      ).status,
      201,
    );
    assert.equal(
      (
        await fetch(route, {
          method: "POST",
          body: "replacement",
          headers: { Origin: origin },
        })
      ).status,
      409,
    );
    assert.equal(
      await fs.readFile(capturePath(new URL(route).pathname, output), "utf8"),
      body,
    );
    const sizeResult = await new Promise((resolve, reject) => {
      const req = http.request(
        route,
        {
          method: "POST",
          headers: { Origin: origin, "Content-Length": CAPTURE_LIMIT + 1 },
        },
        (res) => {
          res.resume();
          res.on("end", () => resolve(res.statusCode));
        },
      );
      req.on("error", reject);
      req.end("x");
    });
    assert.equal(sizeResult, 413);
  } finally {
    await close(server);
  }
});

test("occupied-port CLI fails clearly and leaves the existing server alive", async () => {
  const { server, origin } = await listeningServer();
  try {
    const result = spawnSync(
      process.execPath,
      [
        path.join(TOOL_DIRECTORY, "server.mjs"),
        "--out",
        output,
        "--port",
        new URL(origin).port,
      ],
      { encoding: "utf8", timeout: 10000 },
    );
    assert.equal(result.status, 1);
    assert.match(result.stderr, /already occupied/);
    assert.equal((await fetch(`${origin}/`)).status, 200);
  } finally {
    await close(server);
  }
});

test("autoquit closes only its actual local server and removes its process signal handlers", async () => {
  const reserve = http.createServer();
  await new Promise((resolve) => reserve.listen(0, "127.0.0.1", resolve));
  const port = reserve.address().port;
  await close(reserve);
  const beforeInt = process.listenerCount("SIGINT"),
    beforeTerm = process.listenerCount("SIGTERM");
  const { server, stop } = await serve({ output, port, autoquitSeconds: 1 });
  await once(server, "close");
  stop(); // Repeat stop is harmless.
  assert.equal(server.listening, false);
  assert.equal(process.listenerCount("SIGINT"), beforeInt);
  assert.equal(process.listenerCount("SIGTERM"), beforeTerm);
});
