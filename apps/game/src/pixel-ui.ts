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
          attributeFilter: ["hidden", "class", "disabled"],
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
        // The scroll area clips semantic elements before they reach the engine.
        if (box.bottom < bounds.top || box.top > bounds.bottom) return;
        const font = Math.max(
          1,
          Math.min(
            3,
            Number.parseFloat(getComputedStyle(element).fontSize) / scale / 8,
          ),
        );
        const scroll = element.closest<HTMLElement>(
          "#upgrade-content,#quest-content,#settings-content,#hangar-content,#campaign-content,#console-output,#dialogue-text",
        );
        const clip = scroll ? scroll.getBoundingClientRect() : bounds;
        nodes.push({
          kind,
          rect: [
            (box.x - bounds.x) / scale,
            (box.y - bounds.y) / scale,
            box.width / scale,
            box.height / scale,
          ],
          text: text.replace(/\s+/g, " ").trim().slice(0, 256),
          font,
          value,
          clip: [
            (clip.x - bounds.x) / scale,
            (clip.y - bounds.y) / scale,
            clip.width / scale,
            clip.height / scale,
          ],
          active:
            element === this.hover ||
            element.contains(document.activeElement) ||
            element.dataset.active === "true",
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
        add(art, art.dataset.pixelKind ?? "", "", Number(art.dataset.pixelValue ?? 0));
      if (root.classList.contains("console-card")) {
        const input = root.querySelector<HTMLInputElement>("#console-input");
        if (input) add(input, "input", input.value);
      }
      if (root.classList.contains("launch-card")) {
        add(root.querySelector<HTMLElement>("h1")!, "logo");
        const crest = root.querySelector<HTMLElement>(".title-crest");
        if (crest) add(crest, "seal");
      }
      const selectors =
        root.id === "game-ui"
          ? document.body.classList.contains("engine-flight")
            ? "#dock-button,#site-button,#pause-button,#fullscreen,#quick-settings,#quick-title,#toast,#destination-marker"
            : "#toast"
          : "h2,h3,p,small,.eyebrow,.build-line span,.control-guide span,.upgrade-category span,.upgrade-category b,#quest-content li";
      for (const child of root.querySelectorAll<HTMLElement>(selectors))
        add(
          child,
          child.tagName === "BUTTON" ? "button" : "text",
          child.id === "pause-button"
            ? document.body.classList.contains("touch-ui") ? root.querySelector<HTMLElement>("#quick-menu")?.hidden ? "+" : "X" : "II"
            : child.id === "fullscreen"
              ? "[]"
              : child.id === "destination-marker"
                ? "+"
                : child.innerText,
        );
      if (root.id !== "game-ui") {
        for (const button of root.querySelectorAll<HTMLButtonElement>(
          "button,.import-label",
        ))
          add(button, "button", button.textContent ?? "");
        for (const label of root.querySelectorAll<HTMLLabelElement>(
          ".settings label",
        )) {
          const input = label.querySelector<HTMLInputElement>("input");
          const text = label.childNodes[0]?.textContent ?? "";
          const rangeProgress = () => {
            const min = input!.min === "" ? 0 : Number(input!.min);
            const max = input!.max === "" ? 100 : Number(input!.max);
            const value = Number(input!.value);
            return Number.isFinite(min) && Number.isFinite(max) && Number.isFinite(value) && max > min
              ? Math.max(0, Math.min(1, (value - min) / (max - min)))
              : 0;
          };
          if (input)
            add(
              label,
              input.type === "range" ? "slider" : "toggle",
              text,
              input.type === "range"
                ? rangeProgress()
                : input.checked
                  ? 1
                  : 0,
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
