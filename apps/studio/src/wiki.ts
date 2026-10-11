export interface WikiReference {
  path: string;
  label: string;
  role: string;
  line?: number;
}
export interface WikiMedia {
  path: string;
  label: string;
  kind: string;
  provenance?: string;
  caption: string;
}
export interface WikiEntry {
  id: string;
  title: string;
  category: string;
  status: string;
  summary: string;
  tags: string[];
  facts: { label: string; value: string }[];
  sections: { title: string; paragraphs: string[] }[];
  related: string[];
  references: WikiReference[];
  media: WikiMedia[];
}
export interface WikiCatalog {
  schemaVersion: number;
  title: string;
  description: string;
  categories: { id: string; label: string }[];
  entries: WikiEntry[];
}
export const escapeWiki = (value: unknown): string =>
  String(value ?? "").replace(
    /[&<>"']/g,
    (c) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        c
      ]!,
  );
const stateLabels: Record<string, string> = {
  implemented: "Implemented",
  planned: "Planned",
  "verification-pending": "Verification pending",
};
export function wikiFileUrl(kind: "source" | "media", path: string): string {
  if (
    !/^(docs|crates|apps|assets)\/[A-Za-z0-9_./-]+$/.test(path) ||
    path.split("/").some((p) => !p || p.startsWith("."))
  )
    return "";
  return `/api/wiki/${kind}?path=${encodeURIComponent(path)}`;
}
export function searchWiki(
  catalog: WikiCatalog,
  query: string,
  category = "all",
  status = "all",
): WikiEntry[] {
  const words = query
    .toLowerCase()
    .trim()
    .slice(0, 200)
    .split(/\s+/)
    .filter(Boolean);
  return catalog.entries.filter(
    (e) =>
      (category === "all" || e.category === category) &&
      (status === "all" || e.status === status) &&
      words.every((w) =>
        [
          e.title,
          e.summary,
          ...e.tags,
          ...e.facts.flatMap((f) => [f.label, f.value]),
          ...e.sections.flatMap((s) => [s.title, ...s.paragraphs]),
        ]
          .join(" ")
          .toLowerCase()
          .includes(w),
      ),
  );
}
const badge = (status: string) =>
  `<span class="wiki-badge ${escapeWiki(status)}">${escapeWiki(stateLabels[status] || status)}</span>`;
export function renderWikiEntry(
  entry: WikiEntry,
  catalog: WikiCatalog,
): string {
  const e = escapeWiki;
  const category =
    catalog.categories.find((c) => c.id === entry.category)?.label ||
    entry.category;
  return `<article class="wiki-article"><header><div class="wiki-article-top"><span class="eyebrow">${e(category)}</span>${badge(entry.status)}</div><h2>${e(entry.title)}</h2><p class="wiki-lead">${e(entry.summary)}</p></header>${entry.facts.length ? `<dl class="wiki-facts">${entry.facts.map((f) => `<div><dt>${e(f.label)}</dt><dd>${e(f.value)}</dd></div>`).join("")}</dl>` : ""}${entry.sections.map((s) => `<section class="wiki-section"><h3>${e(s.title)}</h3>${s.paragraphs.map((p) => `<p>${e(p)}</p>`).join("")}</section>`).join("")}${entry.media.length ? `<section class="wiki-section"><h3>Art & provenance</h3><div class="wiki-gallery">${entry.media.map((m) => `<figure><a href="${e(wikiFileUrl("media", m.path))}" target="_blank" rel="noopener"><img src="${e(wikiFileUrl("media", m.path))}" alt="${e(m.label)}" loading="lazy"></a><figcaption><span class="wiki-media-kind">${e(m.kind.replaceAll("-", " "))}</span><strong>${e(m.label)}</strong><p>${e(m.caption)}</p>${m.provenance ? `<a href="${e(wikiFileUrl("source", m.provenance))}" target="_blank" rel="noopener">Source provenance ↗</a>` : ""}</figcaption></figure>`).join("")}</div></section>` : ""}<section class="wiki-section"><h3>Authoritative references</h3><p class="wiki-source-note">These local files open as plain text. Source code defines behavior; test and browser acceptance are separate evidence.</p><ul class="wiki-sources">${entry.references.map((r) => `<li><a href="${e(wikiFileUrl("source", r.path))}" target="_blank" rel="noopener">${e(r.label)} ↗</a><span>${e(r.role)}${r.line ? ` · line ${r.line}` : ""}</span><code>${e(r.path)}</code></li>`).join("")}</ul></section>${
    entry.related.length
      ? `<section class="wiki-section"><h3>Continue reading</h3><div class="wiki-related">${entry.related
          .map((id) => {
            const target = catalog.entries.find((v) => v.id === id);
            return target
              ? `<button data-wiki-entry="${e(id)}">${e(target.title)} <span>→</span></button>`
              : "";
          })
          .join("")}</div></section>`
      : ""
  }</article>`;
}
export class WikiView {
  catalog: WikiCatalog | null = null;
  loading = false;
  error = "";
  query = "";
  category = "all";
  status = "all";
  selected = "";
  async load(onChange: () => void) {
    if (this.loading || this.catalog) return;
    this.loading = true;
    this.error = "";
    try {
      const response = await fetch("/api/wiki", { cache: "no-store" });
      if (!response.ok)
        throw new Error(
          "DevBible unavailable. Check the local API and the versioned catalog.",
        );
      this.catalog = (await response.json()) as WikiCatalog;
    } catch (error) {
      this.error =
        error instanceof Error ? error.message : "Could not read the DevBible.";
    } finally {
      this.loading = false;
      onChange();
    }
  }
  render(): string {
    const e = escapeWiki;
    if (!this.catalog)
      return `<section class="wiki-empty"><span class="eyebrow">DEVBIBLE / SOURCE REFERENCE</span><h1>${this.error ? "Reference unavailable" : "Loading the DevBible"}</h1><p>${e(this.error || "Reading the actual versioned catalog. No demonstration data is substituted.")}</p>${this.error ? '<button class="primary" data-wiki-retry>Retry catalog</button>' : ""}</section>`;
    const c = this.catalog,
      entries = searchWiki(c, this.query, this.category, this.status),
      selected = c.entries.find((entry) => entry.id === this.selected);
    return `<div class="wiki"><header class="wiki-intro"><div><span class="eyebrow">GALAXY ROYALE / SOURCE REFERENCE</span><h1>DevBible<span>.</span></h1><p>${e(c.description)}</p></div><div class="wiki-count"><strong>${c.entries.length}</strong><span>curated entries<br>catalog v${c.schemaVersion}</span></div></header><div class="wiki-tools"><label class="wiki-search"><span aria-hidden="true">⌕</span><input id="wiki-search" type="search" maxlength="200" placeholder="Find characters, ships, rules or source facts" aria-label="Search DevBible" value="${e(this.query)}"></label><select id="wiki-status" aria-label="Filter implementation status"><option value="all">Every status</option>${Object.entries(
      stateLabels,
    )
      .map(
        ([key, label]) =>
          `<option value="${key}" ${key === this.status ? "selected" : ""}>${label}</option>`,
      )
      .join(
        "",
      )}</select></div><nav class="wiki-categories" aria-label="DevBible categories"><button data-wiki-category="all" aria-pressed="${this.category === "all"}">All entries</button>${c.categories.map((v) => `<button data-wiki-category="${e(v.id)}" aria-pressed="${this.category === v.id}">${e(v.label)}</button>`).join("")}</nav>${selected ? `<div class="wiki-reading-heading"><button class="secondary" data-wiki-back>← Browse entries</button><span>Source-backed reference · ${badge(selected.status)}</span></div><div class="wiki-layout"><aside class="wiki-index" aria-label="Matching references"><h2>${entries.length} ${this.query ? "matches" : "entries"}</h2>${entries.map((v) => `<button data-wiki-entry="${e(v.id)}" class="${selected.id === v.id ? "selected" : ""}">${e(v.title)}<small>${e(c.categories.find((x) => x.id === v.category)?.label)}</small></button>`).join("")}</aside>${renderWikiEntry(selected, c)}</div>` : `<div class="wiki-results-heading"><h2>${this.query ? "Search results" : "Explore the universe"}</h2><span>${entries.length} ${entries.length === 1 ? "entry" : "entries"}</span></div><section class="wiki-grid">${entries.map((v) => `<button class="wiki-card" data-wiki-entry="${e(v.id)}"><div><span class="eyebrow">${e(c.categories.find((x) => x.id === v.category)?.label)}</span>${badge(v.status)}</div><h2>${e(v.title)}</h2><p>${e(v.summary)}</p><footer><span>${v.references.length} source references</span><b aria-hidden="true">↗</b></footer></button>`).join("") || '<p class="wiki-no-results">No entries match. Try fewer words or another category.</p>'}</section>`}</div>`;
  }
  bind(root: HTMLElement, onChange: () => void) {
    const select = (id: string) => {
      this.selected = id;
      history.replaceState(null, "", id ? `#wiki/${id}` : "#wiki");
      onChange();
    };
    root
      .querySelectorAll<HTMLButtonElement>("[data-wiki-entry]")
      .forEach((b) => (b.onclick = () => select(b.dataset.wikiEntry!)));
    root
      .querySelector("[data-wiki-back]")
      ?.addEventListener("click", () => select(""));
    root
      .querySelector("[data-wiki-retry]")
      ?.addEventListener("click", () => void this.load(onChange));
    root.querySelectorAll<HTMLButtonElement>("[data-wiki-category]").forEach(
      (b) =>
        (b.onclick = () => {
          this.category = b.dataset.wikiCategory!;
          this.selected = "";
          history.replaceState(null, "", "#wiki");
          onChange();
        }),
    );
    root
      .querySelector<HTMLSelectElement>("#wiki-status")
      ?.addEventListener("change", (event) => {
        this.status = (event.target as HTMLSelectElement).value;
        onChange();
      });
    root
      .querySelector<HTMLInputElement>("#wiki-search")
      ?.addEventListener("input", (event) => {
        this.query = (event.target as HTMLInputElement).value;
        onChange();
        root.querySelector<HTMLInputElement>("#wiki-search")?.focus();
      });
  }
}
