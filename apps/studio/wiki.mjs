import { readFile, lstat, realpath } from "node:fs/promises";
import { resolve, relative, sep, extname } from "node:path";
import { ApiError } from "./store.mjs";

const states = ["implemented", "planned", "verification-pending"];
const mediaKinds = [
  "runtime-art",
  "converted-concept",
  "concept",
  "runtime-screenshot",
];
const imageTypes = {
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".webp": "image/webp",
};
const sourceTypes = [".md", ".rs", ".ts", ".wgsl", ".json", ".py", ".mjs"];
const fail = (message) => {
  throw new ApiError(400, message);
};
const text = (value, name, max = 5000) => {
  if (typeof value !== "string" || !value.trim() || value.length > max)
    fail(`Invalid Wiki ${name}.`);
  return value;
};
const list = (value, name, max) => {
  if (!Array.isArray(value) || value.length > max)
    fail(`Invalid Wiki ${name}.`);
  return value;
};
export function canonicalWikiPath(value) {
  if (
    typeof value !== "string" ||
    value.length > 600 ||
    !/^(docs|crates|apps|assets|artifacts)\/[A-Za-z0-9_./-]+$/.test(value) ||
    value.split("/").some((part) => !part || part.startsWith("."))
  )
    fail("Invalid Wiki file path.");
  return value;
}
function validateData(value, depth = 0) {
  if (depth > 6) fail("Wiki data nesting exceeds limit.");
  if (value === null || typeof value === "boolean") return;
  if (typeof value === "number") {
    if (!Number.isFinite(value)) fail("Invalid Wiki numeric data.");
    return;
  }
  if (typeof value === "string") {
    if (value.length > 2000) fail("Wiki data text exceeds limit.");
    return;
  }
  if (Array.isArray(value)) {
    if (value.length > 100) fail("Wiki data array exceeds limit.");
    value.forEach((item) => validateData(item, depth + 1));
    return;
  }
  if (value && typeof value === "object") {
    if (Object.keys(value).length > 40) fail("Wiki data object exceeds limit.");
    Object.values(value).forEach((item) => validateData(item, depth + 1));
    return;
  }
  fail("Invalid Wiki data.");
}
export function validateCatalog(input) {
  if (!input || input.schemaVersion !== 1)
    fail("Unsupported Wiki catalog version.");
  text(input.title, "title", 200);
  text(input.description, "description", 2000);
  const categories = list(input.categories, "categories", 20);
  const categoryIds = new Set();
  for (const category of categories) {
    if (!category || typeof category !== "object")
      fail("Invalid Wiki category.");
    if (!/^[a-z0-9-]{1,80}$/.test(category.id) || categoryIds.has(category.id))
      fail("Invalid or duplicate Wiki category.");
    categoryIds.add(category.id);
    text(category.label, "category label", 100);
  }
  const entries = list(input.entries, "entries", 200),
    ids = new Set();
  for (const item of entries) {
    if (!item || typeof item !== "object") fail("Invalid Wiki entry.");
    if (!/^[a-z0-9-]{1,80}$/.test(item.id) || ids.has(item.id))
      fail("Invalid or duplicate Wiki entry.");
    ids.add(item.id);
    text(item.title, "entry title", 200);
    text(item.summary, "summary", 2000);
    if (!categoryIds.has(item.category) || !states.includes(item.status))
      fail("Invalid Wiki category or status.");
    list(item.tags, "tags", 30).forEach((tag) => text(tag, "tag", 80));
    list(item.facts, "facts", 40).forEach((fact) => {
      if (!fact || typeof fact !== "object") fail("Invalid Wiki fact.");
      text(fact.label, "fact label", 200);
      text(fact.value, "fact value", 2000);
    });
    list(item.sections, "sections", 20).forEach((section) => {
      if (!section || typeof section !== "object")
        fail("Invalid Wiki section.");
      text(section.title, "section title", 200);
      list(section.paragraphs, "paragraphs", 20).forEach((p) =>
        text(p, "paragraph", 12000),
      );
    });
    list(item.references, "references", 40).forEach((ref) => {
      if (!ref || typeof ref !== "object") fail("Invalid Wiki reference.");
      canonicalWikiPath(ref.path);
      if (!sourceTypes.includes(extname(ref.path)))
        fail("Wiki source type is not allowed.");
      text(ref.label, "reference label", 200);
      text(ref.role, "reference role", 100);
      if (
        ref.line !== undefined &&
        (!Number.isSafeInteger(ref.line) || ref.line < 1)
      )
        fail("Invalid source line.");
    });
    list(item.media, "media", 10).forEach((media) => {
      if (!media || typeof media !== "object") fail("Invalid Wiki media.");
      canonicalWikiPath(media.path);
      if (!imageTypes[extname(media.path)] || !mediaKinds.includes(media.kind))
        fail("Invalid Wiki media type or classification.");
      text(media.label, "media label", 200);
      text(media.caption, "media caption", 2000);
      if (media.provenance) {
        canonicalWikiPath(media.provenance);
        if (!sourceTypes.includes(extname(media.provenance)))
          fail("Invalid media provenance type.");
      }
    });
    list(item.related, "related entries", 30).forEach((id) =>
      text(id, "related entry", 80),
    );
    if (item.data !== undefined) validateData(item.data);
  }
  for (const item of entries)
    if (item.related.some((id) => !ids.has(id)))
      fail("Wiki related entry does not exist.");
  return input;
}
export function filterEntries(
  catalog,
  { q = "", category = "all", status = "all" } = {},
) {
  if (typeof q !== "string" || q.length > 200)
    fail("Wiki search is limited to 200 characters.");
  if (category !== "all" && !catalog.categories.some((c) => c.id === category))
    fail("Unknown Wiki category.");
  if (status !== "all" && !states.includes(status))
    fail("Unknown Wiki status.");
  const words = q.toLowerCase().trim().split(/\s+/).filter(Boolean);
  return catalog.entries.filter((entry) => {
    if (
      (category !== "all" && entry.category !== category) ||
      (status !== "all" && entry.status !== status)
    )
      return false;
    const haystack = [
      entry.title,
      entry.summary,
      ...entry.tags,
      ...entry.facts.flatMap((f) => [f.label, f.value]),
      ...entry.sections.flatMap((s) => [s.title, ...s.paragraphs]),
    ]
      .join(" ")
      .toLowerCase();
    return words.every((word) => haystack.includes(word));
  });
}
// Exact catalog allowlisting and no-link ancestors prevent the read routes becoming a filesystem browser.
async function regularFile(root, path, max) {
  root = await realpath(root);
  const relativePath = canonicalWikiPath(path),
    absolute = resolve(root, relativePath);
  const rel = relative(root, absolute);
  if (!rel || rel.startsWith(`..${sep}`) || rel === "..")
    fail("Wiki file leaves workspace.");
  let cursor = root;
  try {
    for (const [index, part] of relativePath.split("/").entries()) {
      cursor = resolve(cursor, part);
      const info = await lstat(cursor);
      if (
        info.isSymbolicLink() ||
        (index < relativePath.split("/").length - 1
          ? !info.isDirectory()
          : !info.isFile())
      )
        fail("Wiki files must be regular files without links.");
    }
    const actual = await realpath(absolute),
      expected = absolute;
    if (
      process.platform === "win32"
        ? actual.toLowerCase() !== expected.toLowerCase()
        : actual !== expected
    )
      fail("Wiki file resolves to another location.");
    if ((await lstat(absolute)).size > max)
      throw new ApiError(413, "Wiki file exceeds size limit.");
    return absolute;
  } catch (error) {
    if (error instanceof ApiError) throw error;
    throw new ApiError(404, "Wiki file does not exist.");
  }
}
export class WikiStore {
  constructor(root) {
    this.root = resolve(root);
  }
  async load() {
    const path = await regularFile(
      this.root,
      "docs/wiki/catalog.json",
      1024 * 1024,
    );
    try {
      return validateCatalog(JSON.parse(await readFile(path, "utf8")));
    } catch (error) {
      if (error instanceof ApiError) throw error;
      throw new ApiError(400, "Invalid Wiki catalog JSON.");
    }
  }
  async catalog(query) {
    const catalog = await this.load();
    return { ...catalog, entries: filterEntries(catalog, query) };
  }
  async entry(id) {
    const entry = (await this.load()).entries.find((e) => e.id === id);
    if (!entry) throw new ApiError(404, "Wiki entry not found.");
    return entry;
  }
  async file(kind, requested) {
    if (!["media", "source"].includes(kind)) fail("Invalid Wiki file kind.");
    const path = canonicalWikiPath(requested),
      catalog = await this.load();
    const allowed = new Set(
      catalog.entries.flatMap((entry) =>
        kind === "media"
          ? entry.media.map((m) => m.path)
          : [
              ...entry.references.map((r) => r.path),
              ...entry.media.flatMap((m) =>
                m.provenance ? [m.provenance] : [],
              ),
            ],
      ),
    );
    if (!allowed.has(path))
      throw new ApiError(404, "Wiki file is not listed in the catalog.");
    const absolute = await regularFile(
      this.root,
      path,
      kind === "media" ? 12 * 1024 * 1024 : 2 * 1024 * 1024,
    );
    const bytes = await readFile(absolute);
    if (kind === "media") {
      const extension = extname(path);
      const valid =
        extension === ".png"
          ? bytes
              .subarray(0, 8)
              .equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]))
          : [".jpg", ".jpeg"].includes(extension)
            ? bytes[0] === 255 && bytes[1] === 216 && bytes[2] === 255
            : bytes.subarray(0, 4).toString() === "RIFF" &&
              bytes.subarray(8, 12).toString() === "WEBP";
      if (!valid) fail("Wiki image header does not match its type.");
      return { bytes, type: imageTypes[extension] };
    }
    return { bytes, type: "text/plain; charset=utf-8" };
  }
}
