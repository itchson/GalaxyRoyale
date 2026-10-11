import "./universe-map.css";

export interface UniverseMapSite {
  id: string;
  name: string;
  position: [number, number];
  radius: number;
  event: string;
  status: string;
  biome: { kind: number; name: string };
  progress: number | null;
  target: number | null;
  belt_inner: number | null;
  belt_outer: number | null;
  material_tier: number | null;
  recommended_level: number | null;
}
export interface UniverseMapData {
  position: [number, number];
  heading: number;
  biome: { kind: number; name: string };
  home_radius: number;
  belt_inner: number;
  belt_outer: number;
  sites: UniverseMapSite[];
}
interface MapView {
  data: UniverseMapData;
  navigate: (position: [number, number]) => void;
  center: [number, number];
  span: number;
  fitted: boolean;
  selected: string | null;
  field: HTMLElement;
  caption: HTMLElement;
  detail: HTMLElement[];
  course: HTMLButtonElement;
  home: HTMLElement;
  player: HTMLElement;
  rings: HTMLElement[];
  markers: Map<string, HTMLButtonElement>;
  drag: { id: number; x: number; y: number } | null;
}
const views = new WeakMap<HTMLElement, MapView>();
const MAX_MARKERS = 64;
const point = (value: number[]): value is [number, number] =>
  value.length === 2 && value.every(Number.isFinite);
function node(tag: string, className = "", label = ""): HTMLElement {
  const result = document.createElement(tag);
  result.className = className;
  if (label) result.textContent = label;
  return result;
}
function art(kind: string, tag = "section"): HTMLElement {
  const result = node(tag, `map-art ${kind}`);
  result.dataset.pixelKind = kind;
  if (tag !== "button") result.setAttribute("aria-hidden", "true");
  return result;
}
function write(target: HTMLElement, value: string): boolean {
  if (target.textContent === value) return false;
  target.textContent = value;
  return true;
}
function attribute(target: HTMLElement, key: string, value: string): boolean {
  if (target.getAttribute(key) === value) return false;
  target.setAttribute(key, value);
  return true;
}
function style(
  target: HTMLElement,
  key: "left" | "top" | "width" | "height",
  value: number,
): boolean {
  const next = `${value.toFixed(3)}px`;
  if (target.style[key] === next) return false;
  target.style[key] = next;
  return true;
}
function control(
  label: string,
  accessible: string,
  action: () => void,
): HTMLButtonElement {
  const result = node("button", "map-control", label) as HTMLButtonElement;
  result.type = "button";
  result.setAttribute("aria-label", accessible);
  result.title = accessible;
  result.addEventListener("click", action);
  return result;
}
function fit(view: MapView, all: boolean): void {
  const { data } = view;
  const points = [
    [0, 0],
    data.position,
    ...(all ? data.sites.map((site) => site.position) : []),
  ].filter(point);
  const outer = Number.isFinite(data.belt_outer)
    ? Math.max(0, data.belt_outer)
    : 0;
  const minX = Math.min(-outer, ...points.map((p) => p[0]));
  const maxX = Math.max(outer, ...points.map((p) => p[0]));
  const minY = Math.min(-outer, ...points.map((p) => p[1]));
  const maxY = Math.max(outer, ...points.map((p) => p[1]));
  const bounds = view.field.getBoundingClientRect();
  const aspect =
    bounds.width > 0 && bounds.height > 0 ? bounds.width / bounds.height : 1;
  view.center = [(minX + maxX) / 2, (minY + maxY) / 2];
  view.span = Math.max(
    600,
    Math.min(4_000_000, Math.max(maxX - minX, (maxY - minY) * aspect) * 1.18),
  );
  view.fitted = true;
}
function createView(
  target: HTMLElement,
  data: UniverseMapData,
  navigate: MapView["navigate"],
): MapView {
  const field = art("map-field");
  field.id = "map-chart";
  field.className = "universe-map-chart";
  field.removeAttribute("aria-hidden");
  field.tabIndex = 0;
  field.setAttribute("role", "group");
  field.setAttribute(
    "aria-label",
    "Charted universe. Drag to pan, arrow keys to pan, plus or minus to zoom.",
  );
  const caption = node("p", "map-caption");
  const detail = [
    node("h3"),
    node("p"),
    node("p"),
    node("p", "map-safety-note"),
  ];
  const home = art("map-home", "button");
  home.setAttribute("aria-label", "Select Haven");
  home.title = "Haven / home dock";
  const player = art("map-player");
  const rings = Array.from({ length: 6 }, () => art("map-ring"));
  field.append(...rings, home, player);
  const course = control(
    "SET COURSE",
    "Set course to selected location",
    () => {
      const site = view.data.sites.find((entry) => entry.id === view.selected);
      const destination: [number, number] = site ? [...site.position] : [0, 0];
      if (point(destination)) view.navigate(destination);
    },
  );
  const view: MapView = {
    data,
    navigate,
    center: [0, 0],
    span: 4000,
    fitted: false,
    selected: null,
    field,
    caption,
    detail,
    course,
    home,
    player,
    rings,
    markers: new Map(),
    drag: null,
  };
  const zoom = (factor: number): void => {
    view.span = Math.max(600, Math.min(4_000_000, view.span * factor));
    render(view);
  };
  const controls = node("nav", "map-controls");
  controls.setAttribute("aria-label", "Map view controls");
  controls.append(
    control("HOME", "Center map on Haven", () => {
      view.center = [0, 0];
      render(view);
    }),
    control("SHIP", "Center map on your ship", () => {
      if (point(view.data.position)) view.center = [...view.data.position];
      render(view);
    }),
    control("ALL", "Fit all charted locations", () => {
      fit(view, true);
      render(view);
    }),
    control("-", "Zoom map out", () => zoom(1.5)),
    control("+", "Zoom map in", () => zoom(1 / 1.5)),
  );
  const details = node("section", "map-details");
  details.setAttribute("aria-label", "Selected location");
  details.append(...detail, course);
  target.replaceChildren(controls, field, caption, details);
  home.addEventListener("click", () => {
    view.selected = null;
    render(view);
  });
  field.addEventListener("pointerdown", (event) => {
    const pointer = event as PointerEvent;
    if (pointer.button !== 0 || (pointer.target as Element)?.closest("button"))
      return;
    view.drag = {
      id: pointer.pointerId,
      x: pointer.clientX,
      y: pointer.clientY,
    };
    field.setPointerCapture(pointer.pointerId);
    field.classList.add("map-dragging");
  });
  field.addEventListener("pointermove", (event) => {
    const pointer = event as PointerEvent;
    const drag = view.drag;
    const width = field.getBoundingClientRect().width;
    if (!drag || drag.id !== pointer.pointerId || width <= 0) return;
    const units = view.span / width;
    const x = view.center[0] - (pointer.clientX - drag.x) * units;
    const y = view.center[1] - (pointer.clientY - drag.y) * units;
    if (Number.isFinite(x) && Number.isFinite(y)) view.center = [x, y];
    view.drag = {
      id: pointer.pointerId,
      x: pointer.clientX,
      y: pointer.clientY,
    };
    render(view);
  });
  const end = (): void => {
    view.drag = null;
    field.classList.remove("map-dragging");
  };
  for (const name of ["pointerup", "pointercancel", "lostpointercapture"])
    field.addEventListener(name, end);
  field.addEventListener(
    "wheel",
    (event) => {
      const wheel = event as WheelEvent;
      if (!Number.isFinite(wheel.deltaY) || wheel.deltaY === 0) return;
      wheel.preventDefault();
      zoom(wheel.deltaY > 0 ? 1.25 : 0.8);
    },
    { passive: false },
  );
  field.addEventListener("keydown", (event) => {
    const key = event as KeyboardEvent;
    const shift = view.span * 0.12;
    if (key.key === "ArrowLeft") view.center[0] -= shift;
    else if (key.key === "ArrowRight") view.center[0] += shift;
    else if (key.key === "ArrowUp") view.center[1] -= shift;
    else if (key.key === "ArrowDown") view.center[1] += shift;
    else if (key.key === "+" || key.key === "=") {
      zoom(1 / 1.5);
      key.preventDefault();
      return;
    } else if (key.key === "-") {
      zoom(1.5);
      key.preventDefault();
      return;
    } else return;
    key.preventDefault();
    render(view);
  });
  views.set(target, view);
  return view;
}
function render(view: MapView): boolean {
  const bounds = view.field.getBoundingClientRect();
  if (bounds.width <= 0 || bounds.height <= 0) return false;
  if (!view.fitted) fit(view, false);
  let changed = false;
  const scale = bounds.width / view.span;
  const sites = view.data.sites.filter((site) => point(site.position));
  const selected = sites.find((site) => site.id === view.selected);
  if (!selected) view.selected = null;
  const project = (position: [number, number]): [number, number] => [
    (position[0] - view.center[0]) * scale + bounds.width / 2,
    (position[1] - view.center[1]) * scale + bounds.height / 2,
  ];
  const place = (target: HTMLElement, position: [number, number]): void => {
    const [x, y] = project(position);
    changed = style(target, "left", x) || changed;
    changed = style(target, "top", y) || changed;
  };
  const ring = (
    index: number,
    center: [number, number],
    radius: number | null,
    kind: number,
  ): void => {
    const target = view.rings[index];
    const hidden = radius === null || !Number.isFinite(radius) || radius <= 0;
    if (target.hidden !== hidden) {
      target.hidden = hidden;
      changed = true;
    }
    if (hidden) return;
    place(target, center);
    changed = style(target, "width", radius! * 2 * scale) || changed;
    changed = style(target, "height", radius! * 2 * scale) || changed;
    changed = attribute(target, "data-pixel-value", String(kind)) || changed;
  };
  ring(0, [0, 0], view.data.home_radius, 0);
  ring(1, [0, 0], view.data.belt_inner, 1);
  ring(2, [0, 0], view.data.belt_outer, 1);
  place(view.home, [0, 0]);
  changed =
    attribute(view.home, "data-active", String(view.selected === null)) ||
    changed;
  const playerHidden = !point(view.data.position);
  if (view.player.hidden !== playerHidden) {
    view.player.hidden = playerHidden;
    changed = true;
  }
  if (!playerHidden) place(view.player, view.data.position);
  changed =
    attribute(
      view.player,
      "data-pixel-value",
      String(Number.isFinite(view.data.heading) ? view.data.heading : 0),
    ) || changed;
  const visible = sites
    .filter((site) => {
      const [x, y] = project(site.position);
      return x >= 0 && x <= bounds.width && y >= 0 && y <= bounds.height;
    })
    .sort((a, b) => {
      if (a.id === b.id) return 0;
      if (a.id === view.selected) return -1;
      if (b.id === view.selected) return 1;
      const distance = (site: UniverseMapSite): number =>
        (site.position[0] - view.center[0]) ** 2 +
        (site.position[1] - view.center[1]) ** 2;
      return distance(a) - distance(b) || a.id.localeCompare(b.id);
    });
  const shown = visible.slice(0, MAX_MARKERS);
  const ids = new Set(shown.map((site) => site.id));
  for (const [id, marker] of view.markers)
    if (!ids.has(id)) {
      marker.remove();
      view.markers.delete(id);
      changed = true;
    }
  for (const site of shown) {
    let marker = view.markers.get(site.id);
    if (!marker) {
      marker = art("map-site", "button") as HTMLButtonElement;
      marker.type = "button";
      marker.addEventListener("click", () => {
        view.selected = site.id;
        render(view);
      });
      view.markers.set(site.id, marker);
      view.field.append(marker);
      changed = true;
    }
    place(marker, site.position);
    changed =
      attribute(
        marker,
        "aria-label",
        `Select ${site.name} / ${site.biome.name}`,
      ) || changed;
    changed =
      attribute(marker, "data-pixel-value", String(site.biome.kind)) || changed;
    changed =
      attribute(marker, "data-active", String(view.selected === site.id)) ||
      changed;
  }
  ring(3, selected?.position ?? [0, 0], selected?.radius ?? null, 2);
  ring(4, selected?.position ?? [0, 0], selected?.belt_inner ?? null, 1);
  ring(5, selected?.position ?? [0, 0], selected?.belt_outer ?? null, 1);
  changed =
    write(
      view.caption,
      `CENTER ${Math.round(view.center[0])}, ${Math.round(view.center[1])} / SPAN ${Math.round(view.span)} / ${shown.length}/${visible.length} VISIBLE BEACONS / ${sites.length} CHARTED`,
    ) || changed;
  changed =
    write(view.detail[0], selected?.name ?? "HAVEN / HOME DOCK") || changed;
  changed =
    write(
      view.detail[1],
      selected
        ? `${selected.biome.name} / ${selected.event.toUpperCase()} / ${selected.status.toUpperCase()}${selected.progress !== null && selected.target !== null ? ` / ${selected.progress}/${selected.target}` : ""}`
        : `SAFE RADIUS ${Math.round(view.data.home_radius)} / ASTEROID BELT ${Math.round(view.data.belt_inner)}-${Math.round(view.data.belt_outer)}`,
    ) || changed;
  changed =
    write(
      view.detail[2],
      selected
        ? `COORD ${Math.round(selected.position[0])}, ${Math.round(selected.position[1])}${selected.material_tier !== null ? ` / T${selected.material_tier} MATERIALS` : ""}${selected.recommended_level !== null ? ` / RECOMMENDED RAICHO LV ${selected.recommended_level}` : ""}`
        : `CURRENT BIOME / ${view.data.biome.name}${sites.length ? "" : " / REACH REMOTE SANCTUARIES TO CHART BEACONS"}`,
    ) || changed;
  changed =
    write(
      view.detail[3],
      selected && (selected.belt_inner === null || selected.belt_outer === null)
        ? "SHIELD BLOCKS ASTEROIDS"
        : "SHIELD BLOCKS ASTEROIDS / BELT ORBITS OUTSIDE",
    ) || changed;
  return changed;
}

/** Uses charted positions supplied by Rust. Navigation sets a course, never teleports. */
export function paintUniverseMap(
  container: HTMLElement,
  data: UniverseMapData,
  navigate: MapView["navigate"],
): boolean {
  const previous = views.get(container);
  const view = previous ?? createView(container, data, navigate);
  view.data = data;
  view.navigate = navigate;
  return render(view) || !previous;
}
