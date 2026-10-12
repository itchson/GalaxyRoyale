import fs from "node:fs/promises";
import path from "node:path";
import { createHash } from "node:crypto";
import { parseArgs } from "node:util";
import { fileURLToPath } from "node:url";

const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const extensions = new Set([".exe", ".dll", ".pdb", ".rlib", ".wasm"]);

// Freezes bytes only. Compiler/source provenance must be recorded separately.
export async function freezeNativeProof({ workspace, source, output }) {
  const base = await fs.realpath(workspace);
  const input = path.resolve(base, source);
  const destination = path.resolve(base, output);
  const relative = path.relative(base, destination);
  if (
    path.isAbsolute(relative) ||
    relative.startsWith("..") ||
    !["work", "artifacts"].includes(relative.split(path.sep)[0])
  )
    throw Error("Proof output must be inside workspace work/ or artifacts/.");
  // Reject redirects before creating directories; never alter a source target.
  let current = base;
  for (const segment of relative.split(path.sep)) {
    current = path.join(current, segment);
    try {
      if ((await fs.lstat(current)).isSymbolicLink())
        throw Error("Proof output cannot traverse a symbolic link.");
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
    }
  }
  const extension = path.extname(input).toLowerCase();
  const stat = await fs.stat(input);
  if (
    !extensions.has(extension) ||
    !stat.isFile() ||
    stat.size > 256 * 1024 * 1024
  )
    throw Error("Expected a compiled artifact no larger than 256 MiB.");
  const bytes = await fs.readFile(input);
  const digest = sha(bytes);
  if (sha(await fs.readFile(input)) !== digest)
    throw Error("Native target changed during proof capture.");
  await fs.mkdir(destination, { recursive: true });
  const file = path.join(destination, digest + extension);
  try {
    await fs.writeFile(file, bytes, { flag: "wx" });
  } catch (error) {
    if (error.code !== "EEXIST") throw error;
  }
  if (sha(await fs.readFile(file)) !== digest)
    throw Error("Existing frozen proof is corrupt; it was not replaced.");
  await fs.chmod(file, 0o444);
  return {
    source: input,
    path: file,
    bytes: bytes.length,
    sha256: digest,
    scope:
      "Immutable byte snapshot only; compiler/source/run provenance is separate.",
  };
}

if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  const { values } = parseArgs({
    options: { source: { type: "string" }, out: { type: "string" } },
  });
  if (!values.source || !values.out)
    throw Error(
      "Use --source <artifact> --out <workspace work/proof-directory>.",
    );
  console.log(
    JSON.stringify(
      await freezeNativeProof({
        workspace: process.cwd(),
        source: values.source,
        output: values.out,
      }),
      null,
      2,
    ),
  );
}
