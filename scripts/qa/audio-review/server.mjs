import http from "node:http";
import fs from "node:fs/promises";
import path from "node:path";
import crypto from "node:crypto";
import {
  MODULE_NAMES,
  HELPER_FILES,
  TYPESCRIPT_VERSION,
  BASELINE_SHA256,
  TOOL_DIRECTORY,
  options,
  isMain,
} from "./config.mjs";

export const CAPTURE_LIMIT = 12 * 1024 * 1024;
const assets = new Set(["index.html", "review.js", "metrics.js"]);
const mime = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json",
};
const sha = (bytes) => crypto.createHash("sha256").update(bytes).digest("hex");

export function staticFile(urlPath, output) {
  const name = urlPath === "/" ? "index.html" : urlPath.slice(1);
  if (!urlPath.startsWith("/")) return null;
  if (assets.has(name)) return path.join(TOOL_DIRECTORY, name);
  if (name === "source-pins.json") return path.join(output, name);
  if (MODULE_NAMES.some((module) => name === `modules/${module}`))
    return path.join(output, name);
  return null;
}

export function capturePath(urlPath, output) {
  const match =
    /^\/capture\/(qa-[0-9]{13}-[a-f0-9]{8})\/(prior-engine\.wav|warm-engine\.wav|representative-mix\.wav|max-bus-burst\.wav|report\.json)$/.exec(
      urlPath,
    );
  return match ? path.join(output, "captures", match[1], match[2]) : null;
}

export async function validateOutput(output) {
  let pins;
  try {
    pins = JSON.parse(
      await fs.readFile(path.join(output, "source-pins.json"), "utf8"),
    );
  } catch {
    throw new Error(
      `Prepared source-pins.json is unavailable at ${output}. Run prepare.mjs first.`,
    );
  }
  if (
    pins.schema_version !== 1 ||
    pins.typescript !== TYPESCRIPT_VERSION ||
    !Object.values(pins.inputs ?? {}).some(
      (p) => p.module === "baseline-audio.js" && p.sha256 === BASELINE_SHA256,
    )
  )
    throw new Error(
      "Prepared source pins have an unsupported schema, compiler or historical baseline.",
    );
  for (const module of MODULE_NAMES) {
    const input = Object.values(pins.inputs ?? {}).find(
      (entry) => entry.module === module,
    );
    if (
      !input ||
      sha(await fs.readFile(path.join(output, "modules", module))) !==
        input.module_sha256
    )
      throw new Error(
        `Prepared module ${module} does not match source-pins.json. Prepare a fresh output directory.`,
      );
  }
  for (const name of HELPER_FILES) {
    if (
      sha(await fs.readFile(path.join(TOOL_DIRECTORY, name))) !==
      pins.helpers?.[name]
    )
      throw new Error(
        `Tracked helper ${name} changed after preparation. Prepare a fresh output directory.`,
      );
  }
}

function readBody(request) {
  if (Number(request.headers["content-length"]) > CAPTURE_LIMIT) {
    request.resume();
    return Promise.reject(
      Object.assign(new Error("Capture size limit exceeded"), { status: 413 }),
    );
  }
  return new Promise((resolve, reject) => {
    const chunks = [];
    let count = 0,
      failed = false;
    request.on("data", (chunk) => {
      count += chunk.length;
      if (failed) return;
      if (count > CAPTURE_LIMIT) {
        failed = true;
        chunks.length = 0;
        reject(
          Object.assign(new Error("Capture size limit exceeded"), {
            status: 413,
          }),
        );
      } else chunks.push(chunk);
    });
    request.on("end", () => {
      if (!failed) resolve(Buffer.concat(chunks));
    });
    request.on("error", reject);
  });
}

export function createReviewServer(output, origin) {
  const server = http.createServer(async (request, response) => {
    response.setHeader("Cache-Control", "no-store");
    response.setHeader("X-Content-Type-Options", "nosniff");
    try {
      const allowedOrigin =
        origin ?? `http://127.0.0.1:${server.address().port}`;
      const url = new URL(request.url, allowedOrigin);
      if (request.method === "POST") {
        const outputFile = capturePath(url.pathname, output);
        if (!outputFile || request.headers.origin !== allowedOrigin) {
          request.resume();
          response.writeHead(403);
          response.end("Rejected capture path or Origin");
          return;
        }
        const body = await readBody(request);
        await fs.mkdir(path.dirname(outputFile), { recursive: true });
        await fs.writeFile(outputFile, body, { flag: "wx" });
        response.writeHead(201, { "Content-Type": "application/json" });
        response.end(
          JSON.stringify({
            saved:
              path.basename(path.dirname(outputFile)) +
              "/" +
              path.basename(outputFile),
            bytes: body.length,
          }),
        );
        return;
      }
      const file = request.method === "GET" && staticFile(url.pathname, output);
      if (!file) {
        response.writeHead(404);
        response.end("Not found");
        return;
      }
      const bytes = await fs.readFile(file);
      response.writeHead(200, {
        "Content-Type": mime[path.extname(file)] ?? "application/octet-stream",
      });
      response.end(bytes);
    } catch (error) {
      response.writeHead(
        error.code === "EEXIST"
          ? 409
          : error.code === "ENOENT"
            ? 404
            : (error.status ?? 500),
      );
      response.end(error instanceof Error ? error.message : String(error));
    }
  });
  server.requestTimeout = 30000;
  server.headersTimeout = 10000;
  server.keepAliveTimeout = 5000;
  return server;
}

export async function serve({ output, port, autoquitSeconds }) {
  await validateOutput(output);
  const origin = `http://127.0.0.1:${port}`;
  const server = createReviewServer(output, origin);
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(port, "127.0.0.1", resolve);
  });
  let closing = false,
    deadline,
    autoquit;
  const stop = () => {
    if (closing) return;
    closing = true;
    clearTimeout(autoquit);
    console.log("Closing isolated audio review server.");
    server.close(() => {
      clearTimeout(deadline);
      process.off("SIGINT", stop);
      process.off("SIGTERM", stop);
    });
    server.closeIdleConnections();
    deadline = setTimeout(() => server.closeAllConnections(), 5000);
    deadline.unref();
  };
  process.once("SIGINT", stop);
  process.once("SIGTERM", stop);
  if (autoquitSeconds) {
    autoquit = setTimeout(stop, autoquitSeconds * 1000);
    autoquit.unref();
  }
  console.log(
    `Isolated native DSP review ${origin}/; output ${output}; autoquit ${autoquitSeconds || "disabled"} seconds`,
  );
  return { server, stop };
}

if (isMain(import.meta.url)) {
  try {
    const args = options(process.argv.slice(2), true);
    if (args.help)
      console.log(
        "node scripts/qa/audio-review/server.mjs [--out work/audio-review] [--port 4188] [--autoquit-seconds 1800] (0 disables autoquit)",
      );
    else await serve(args);
  } catch (error) {
    console.error(
      error.code === "EADDRINUSE"
        ? "Audio review port is already occupied. Stop its verified owner or choose another explicit --port; no existing process was stopped."
        : error instanceof Error
          ? error.message
          : String(error),
    );
    process.exitCode = 1;
  }
}
