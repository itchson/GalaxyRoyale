/** Semantic controls provide input and accessibility; Rust draws their pixel cells. */
export interface PixelUiRuntime {
  set_ui_scene(scene: unknown): void;
}
interface Node {
  kind: string;
  rect: number[];
  text: string;
  font: number;
  value: number;
  active: boolean;
  selected: boolean;
  disabled: boolean;
  clip: number[] | null;
}
export class PixelUiBridge {
  private dirty = true;
  private animateUntil = 0;
  private hover: Element | null = null;
  private readonly observer: MutationObserver;
  constructor(
    private runtime: PixelUiRuntime,
    private scale: () => number,
  ) {
    this.observer = new MutationObserver((records) => {
      if (
        records.some(
          ({ target }) =>
            target instanceof Element &&
            target.closest("#debug-console,#dialogue-screen"),
        )
      )
        this.animateUntil = performance.now() + 250;
      this.invalidate();
    });
    for (const id of [
      "launch-screen",
      "menu-screen",
      "game-ui",
      "debug-console",
      "dialogue-screen",
      "death-screen",
    ])
      if (document.getElementById(id))
        this.observer.observe(document.getElementById(id)!, {
          childList: true,
          subtree: true,
          characterData: true,
          attributes: true,
          attributeFilter: [
            "hidden",
            "class",
            "disabled",
            "data-active",
            "aria-selected",
          ],
        });
    document.addEventListener("pointerover", (event) => {
      this.hover = (event.target as Element)?.closest("button,label,select");
      this.invalidate();
    });
    document.addEventListener("pointerout", () => {
      this.hover = null;
      this.invalidate();
    });
    for (const name of ["input", "change", "focusin", "focusout", "scroll"])
      document.addEventListener(name, () => this.invalidate(), true);
    // The native input owns editing; its selection can change without an input event.
    document.addEventListener("selectionchange", () => {
      if (document.activeElement?.id === "console-input") this.invalidate();
    });
    window.addEventListener("resize", () => this.invalidate());
    document.fonts.ready.then(() => this.invalidate());
  }
  invalidate(): void {
    this.dirty = true;
  }
  sync(): void {
    if (!this.dirty && performance.now() >= this.animateUntil) return;
    this.dirty = false;
    const scale = this.scale();
    if (!Number.isFinite(scale) || scale <= 0) return;
    const groups: { position: number[]; nodes: Node[] }[] = [];
    const buttonKind = (element: HTMLElement): string => {
      if (element.id === "menu-close") return "close-icon";
      if (element.id === "save-indicator") return "save-icon";
      if (element.classList.contains("dock-facility")) return "facility-label";
      if (element.closest(".menu-tabs")) return "nav-tab";
      if (element.classList.contains("primary-button")) return "primary-button";
      if (element.classList.contains("restart-button")) return "restart-button";
      if (["return-title", "death-title", "quick-title"].includes(element.id))
        return "exit-button";
      const action = element.dataset.pixelAction;
      return ["offense-button", "defense-button", "passive-button"].includes(
        action ?? "",
      )
        ? action!
        : "button";
    };
    const textKind = (element: HTMLElement): string => {
      const category = element.closest(".upgrade-category");
      for (const name of ["offense", "defense", "passive"])
        if (category?.classList.contains(name)) return `${name}-text`;
      return "text";
    };
    const visible = (element: HTMLElement) =>
      !element.closest("[hidden]") &&
      getComputedStyle(element).display !== "none" &&
      element.getBoundingClientRect().width > 0;
    for (const root of document.querySelectorAll<HTMLElement>(
      ".launch-card,.menu-card,.console-card,.dialogue-card,.death-card,#game-ui",
    )) {
      if (!visible(root)) continue;
      const bounds = root.getBoundingClientRect();
      const position = [bounds.x / scale, bounds.y / scale];
      const nodes: Node[] = [];
      const add = (
        element: HTMLElement,
        kind: string,
        text = "",
        value = 0,
      ) => {
        if (!visible(element)) return;
        if (element.id === "toast" && !element.classList.contains("visible"))
          return;
        const box = element.getBoundingClientRect();
        const font = Math.max(
          1,
          Math.min(
            3,
            Number.parseFloat(getComputedStyle(element).fontSize) / scale / 8,
          ),
        );
        const scroll = element.closest<HTMLElement>(
          ".menu-tabs,#pause-content,#upgrade-content,#quest-content,#settings-content,#archive-content,#hangar-content,#campaign-content,#console-output,#dialogue-text,#run-summary",
        );
        const scrollBounds = scroll ? scroll.getBoundingClientRect() : bounds;
        // Intersect both axes: scrolling never lets engine cells cross a panel.
        const clipX = Math.max(bounds.x, scrollBounds.x);
        const clipY = Math.max(bounds.y, scrollBounds.y);
        const clipRight = Math.min(
          bounds.x + bounds.width,
          scrollBounds.x + scrollBounds.width,
        );
        const clipBottom = Math.min(
          bounds.y + bounds.height,
          scrollBounds.y + scrollBounds.height,
        );
        if (
          clipRight <= clipX ||
          clipBottom <= clipY ||
          box.x + box.width <= clipX ||
          box.x >= clipRight ||
          box.y + box.height <= clipY ||
          box.y >= clipBottom
        )
          return;
        // CSS layout quantizes to subpixels. Preserve the measured center while
        // recovering only the authored campaign icon size within 1/64 UI unit.
        const authored =
          kind === "campaign-patch" ? 24 : kind === "campaign-ship" ? 32 : 0;
        const measuredWidth = box.width / scale,
          measuredHeight = box.height / scale;
        const extent = (size: number) =>
          authored && Math.abs(size - authored) <= 1 / 64 ? authored : size;
        const width = extent(measuredWidth),
          height = extent(measuredHeight);
        nodes.push({
          kind,
          rect: [
            (box.x - bounds.x) / scale + (measuredWidth - width) / 2,
            (box.y - bounds.y) / scale + (measuredHeight - height) / 2,
            width,
            height,
          ],
          text:
            kind === "input"
              ? Array.from(text).slice(0, 256).join("")
              : text.replace(/\s+/g, " ").trim().slice(0, 256),
          font,
          value,
          clip: [
            (clipX - bounds.x) / scale,
            (clipY - bounds.y) / scale,
            (clipRight - clipX) / scale,
            (clipBottom - clipY) / scale,
          ],
          active:
            kind === "campaign-patch"
              ? element.dataset.active === "true"
              : element === this.hover ||
                element.contains(document.activeElement),
          selected:
            element.dataset.active === "true" ||
            element.getAttribute("aria-selected") === "true",
          disabled: element instanceof HTMLButtonElement && element.disabled,
        });
      };
      if (root.id !== "game-ui")
        add(
          root,
          root.classList.contains("console-card") ? "console-panel" : "panel",
        );
      if (root.classList.contains("dialogue-card")) {
        const portrait = root.querySelector<HTMLElement>(".dialogue-portrait");
        if (portrait)
          add(portrait, "portrait", "", Number(portrait.dataset.portrait ?? 0));
      }
      for (const art of root.querySelectorAll<HTMLElement>("[data-pixel-kind]"))
        add(
          art,
          art.dataset.pixelKind ?? "",
          "",
          Number(art.dataset.pixelValue ?? 0),
        );
      if (root.classList.contains("console-card")) {
        const input = root.querySelector<HTMLInputElement>("#console-input");
        if (input)
          add(
            input,
            "input",
            input.value,
            Array.from(
              input.value.slice(0, input.selectionStart ?? input.value.length),
            ).length,
          );
      }
      if (root.classList.contains("launch-card")) {
        add(root.querySelector<HTMLElement>("h1")!, "logo");
        const crest = root.querySelector<HTMLElement>(".title-crest");
        if (crest) add(crest, "seal");
      }
      const selectors =
        root.id === "game-ui"
          ? document.body.classList.contains("engine-flight")
            ? ".dock-facility,#save-indicator,#site-button,#pause-button,#fullscreen,#quick-settings,#quick-title,#toast,#destination-marker"
            : "#toast"
          : "h2,h3,p,small,.eyebrow,.build-line span,.control-guide span,.upgrade-category span,.upgrade-category b,#quest-content li";
      for (const child of root.querySelectorAll<HTMLElement>(selectors))
        add(
          child,
          child.tagName === "BUTTON" ? buttonKind(child) : textKind(child),
          child.id === "pause-button"
            ? document.body.classList.contains("touch-ui")
              ? root.querySelector<HTMLElement>("#quick-menu")?.hidden
                ? "+"
                : "X"
              : "II"
            : child.id === "fullscreen"
              ? "[]"
              : child.id === "destination-marker"
                ? "+"
                : child.id === "save-indicator"
                  ? ""
                  : child.innerText,
          child.id === "save-indicator"
            ? Number(child.dataset.pixelValue ?? 0)
            : 0,
        );
      if (root.id !== "game-ui") {
        for (const button of root.querySelectorAll<HTMLButtonElement>(
          "button,.import-label",
        ))
          add(
            button,
            buttonKind(button),
            button.id === "settings-backup"
              ? "SAVE / BACKUP"
              : (button.textContent ?? ""),
          );
        for (const label of root.querySelectorAll<HTMLLabelElement>(
          ".settings label",
        )) {
          const input = label.querySelector<HTMLInputElement>("input");
          const text = label.childNodes[0]?.textContent ?? "";
          const rangeProgress = () => {
            const min = input!.min === "" ? 0 : Number(input!.min);
            const max = input!.max === "" ? 100 : Number(input!.max);
            const value = Number(input!.value);
            return Number.isFinite(min) &&
              Number.isFinite(max) &&
              Number.isFinite(value) &&
              max > min
              ? Math.max(0, Math.min(1, (value - min) / (max - min)))
              : 0;
          };
          if (input)
            add(
              label,
              input.type === "range" ? "slider" : "toggle",
              text,
              input.type === "range" ? rangeProgress() : input.checked ? 1 : 0,
            );
        }
        for (const rank of root.querySelectorAll<HTMLElement>(".rank-dots"))
          add(rank, "rank", "", rank.querySelectorAll(".earned").length);
      }
      groups.push({ position, nodes: nodes.slice(0, 256) });
    }
    this.runtime.set_ui_scene({ groups });
  }
}
