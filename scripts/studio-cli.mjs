import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";

export function command(resource, action, args = []) {
  if (resource === "snapshot") return { url: "/api/project", method: "GET" };
  if (resource === "wiki") return { url: "/api/wiki", method: "GET" };
  const flag = args.indexOf("--file");
  if (flag < 0 || !args[flag + 1])
    throw new Error(
      "Provide an explicit UTF-8 JSON input using --file <path>.",
    );
  const file = args[flag + 1];
  if (resource === "project" && action === "update")
    return { url: "/api/project", method: "PATCH", file };
  if (["task", "feedback"].includes(resource) && action === "update") {
    const id = args[0];
    if (!id || id.startsWith("--")) throw new Error("Update requires an ID.");
    return {
      url: `/api/${resource === "task" ? "tasks" : "feedback"}/${encodeURIComponent(id)}`,
      method: "PATCH",
      file,
    };
  }
  if (
    action === "create" &&
    ["task", "review", "decision", "build", "feedback", "run"].includes(
      resource,
    )
  )
    return {
      url: `/api/${resource === "feedback" ? "feedback" : resource + "s"}`,
      method: "POST",
      file,
    };
  throw new Error(
    "Usage: snapshot | wiki | project update --file <json> | task/feedback update <id> --file <json> | task/review/decision/build/feedback/run create --file <json>",
  );
}
if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  try {
    const [resource, action, ...args] = process.argv.slice(2),
      request = command(resource, action, args);
    const options = { method: request.method };
    if (request.file) {
      options.headers = { "Content-Type": "application/json" };
      options.body = JSON.stringify(
        JSON.parse(await readFile(request.file, "utf8")),
      );
    }
    const response = await fetch(
        "http://127.0.0.1:4174" + request.url,
        options,
      ),
      result = await response.json();
    if (!response.ok)
      throw new Error(`HTTP ${response.status}: ${result.error}`);
    console.log(JSON.stringify(result, null, 2));
  } catch (error) {
    console.error(
      error.cause?.code === "ECONNREFUSED"
        ? "Production API is offline. Start npm run dev; the CLI never writes the store directly."
        : error.message,
    );
    process.exitCode = 1;
  }
}
