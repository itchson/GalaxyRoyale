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
  label_rect?: number[];
  control_rect?: number[];
  portrait_expression?: number;
  portrait_speaking?: boolean;
  portrait_kind?: number;
  portrait_main?: number;
  portrait_sub?: number;
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
      // Map pan/zoom moves art without necessarily changing text. Other
      // per-frame world markers already have explicit shell invalidation.
      if (
        records.every(
          ({ attributeName, target }) =>
            attributeName === "style" &&
            target instanceof Element &&
            !target.closest("#map-chart,#flight-status-strip"),
        )
      )
        return;
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
      "droid-screen",
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
            "aria-pressed",
            "data-selected",
            "data-portrait",
            "data-portrait-expression",
            "data-portrait-speaking",
            "data-portrait-kind",
            "data-portrait-main",
            "data-portrait-sub",
            "data-pixel-icon",
            "data-pixel-kind",
            "data-pixel-value",
            "data-context-available",
            "data-context-interactive",
            "style",
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
    for (const name of [
      "input",
      "change",
      "focusin",
      "focusout",
      "scroll",
      "pointerdown",
      "pointerup",
      "pointercancel",
      "keydown",
      "keyup",
    ])
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
      if (["site-button", "dock-button"].includes(element.id))
        return "status-text";
      if (element.id === "menu-close") return "close-icon";
      if (element.id === "save-indicator") return "save-icon";
      const icon = element.dataset.pixelIcon;
      if (
        [
          "close-icon",
          "settings-icon",
          "resume-icon",
          "restart-icon",
          "title-icon",
          "quit-icon",
          "status-icon",
          "map-icon",
          "fullscreen-icon",
        ].includes(icon ?? "")
      )
        return icon!;
      if (element.classList.contains("dock-facility")) return "facility-label";
      if (
        element.closest(".menu-tabs") ||
        element.closest("#collection-navigation") ||
        (element.closest(".collection-filters") && !element.dataset.pixelAction)
      )
        return "nav-tab";
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
      if (element.id === "toast") return "warning-text";
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
      ".launch-card,.menu-card,.console-card,.dialogue-card,.death-card,.droid-card,#game-ui",
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
        regions?: { label: HTMLElement | null; control: HTMLInputElement },
      ): void => {
        if (!visible(element)) return;
        // Facility controls remain accessible and clickable; their pixel label
        // is revealed only by cursor hover or deliberate keyboard focus.
        if (
          kind === "facility-label" &&
          element !== this.hover &&
          !element.contains(document.activeElement)
        )
          return;
        if (element.id === "toast" && !element.classList.contains("visible"))
          return;
        if (
          ["site-button", "dock-button"].includes(element.id) &&
          root
            .querySelector<HTMLElement>("#toast")
            ?.classList.contains("visible")
        )
          return;
        let box = element.getBoundingClientRect();
        const style = getComputedStyle(element);
        if (kind === "status-text" || kind === "warning-text") {
          const flight =
            root.id === "game-ui" &&
            document.body.classList.contains("engine-flight");
          const properties = ["x", "y", "width", "height"].map((axis) =>
            Number.parseFloat(
              style.getPropertyValue?.(`--flight-text-${axis}`) ?? "",
            ),
          );
          if (
            flight &&
            properties.every(Number.isFinite) &&
            properties[2] >= 3 * scale &&
            properties[3] >= 5 * scale
          ) {
            if (element.tagName === "BUTTON") add(element, "status-hit");
            box = {
              x: properties[0],
              y: properties[1],
              width: properties[2],
              height: properties[3],
            } as DOMRect;
          } else if (!flight && kind === "warning-text") {
            // Menu/recovery messages retain the DOM's multiline measured area.
            // Flight warnings use the exact exported single-line footer slot.
            kind = "text";
          } else {
            return;
          }
        }
        const font = Math.max(
          0.25,
          Math.min(3, Number.parseFloat(style.fontSize) / scale / 8),
        );
        const scrollSelector =
          ".menu-tabs,#collection-navigation,#meishi-content,#cargo-content,#map-chart,#map-content,#flight-status,#pause-content,#upgrade-content,#quest-content,#settings-content,#archive-content,#hangar-content,#campaign-content,#console-output,#dialogue-text,#run-summary,.droid-body,.droid-content";
        let clipX = bounds.x,
          clipY = bounds.y,
          clipRight = bounds.x + bounds.width,
          clipBottom = bounds.y + bounds.height;
        // Intersect every enclosing scroll region on both axes. A map chart
        // can itself scroll beyond the content viewport, header or footer.
        for (
          let scroll = element.closest<HTMLElement>(scrollSelector);
          scroll;
          scroll =
            scroll.parentElement?.closest<HTMLElement>(scrollSelector) ?? null
        ) {
          const scrollBounds = scroll.getBoundingClientRect();
          clipX = Math.max(clipX, scrollBounds.x);
          clipY = Math.max(clipY, scrollBounds.y);
          clipRight = Math.min(clipRight, scrollBounds.x + scrollBounds.width);
          clipBottom = Math.min(
            clipBottom,
            scrollBounds.y + scrollBounds.height,
          );
        }
        if (
          clipRight <= clipX ||
          clipBottom <= clipY ||
          box.x + box.width <= clipX ||
          box.x >= clipRight ||
          box.y + box.height <= clipY ||
          box.y >= clipBottom
        )
          return;
        if (kind === "text" && element.id === "dialogue-text") {
          // The paragraph is its own scroll viewport. Keep its original BCR as
          // the clip, while native glyphs move through the full content area.
          const height = Math.max(
            box.height,
            Number.isFinite(element.clientHeight) ? element.clientHeight : 0,
            Number.isFinite(element.scrollHeight) ? element.scrollHeight : 0,
          );
          const scroll = Number.isFinite(element.scrollTop)
            ? Math.max(0, Math.min(element.scrollTop, height - box.height))
            : 0;
          box = {
            x: box.x,
            y: box.y - scroll,
            width: box.width,
            height,
          } as DOMRect;
        }
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
        const regionRect = (region: HTMLElement): number[] => {
          const area = region.getBoundingClientRect();
          return [
            (area.x - bounds.x) / scale,
            (area.y - bounds.y) / scale,
            area.width / scale,
            area.height / scale,
          ];
        };
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
            element.matches(":active") ||
            element.dataset.active === "true" ||
            element.getAttribute("aria-pressed") === "true" ||
            element.getAttribute("aria-selected") === "true",
          disabled: element instanceof HTMLButtonElement && element.disabled,
          ...(kind === "portrait"
            ? {
                portrait_expression: Math.floor(
                  Math.max(
                    0,
                    Math.min(
                      3,
                      Number(element.dataset.portraitExpression) || 0,
                    ),
                  ),
                ),
                portrait_speaking: element.dataset.portraitSpeaking === "true",
                ...(value === 9 &&
                [
                  element.dataset.portraitKind,
                  element.dataset.portraitMain,
                  element.dataset.portraitSub,
                ].every((token) => token !== undefined && /^[0-2]$/.test(token))
                  ? {
                      portrait_kind: Number(element.dataset.portraitKind),
                      portrait_main: Number(element.dataset.portraitMain),
                      portrait_sub: Number(element.dataset.portraitSub),
                    }
                  : {}),
              }
            : {}),
          ...(regions
            ? {
                ...(regions.label
                  ? { label_rect: regionRect(regions.label) }
                  : {}),
                control_rect: regionRect(regions.control),
                disabled: regions.control.disabled,
              }
            : {}),
        });
      };
      if (root.id !== "game-ui")
        add(
          root,
          root.classList.contains("console-card")
            ? "console-panel"
            : root.classList.contains("meishi-surface")
              ? "meishi-panel"
              : "panel",
          "",
          Number(root.dataset.pixelValue ?? 0),
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
          ["status-text", "warning-text"].includes(art.dataset.pixelKind ?? "")
            ? art.innerText
            : "",
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
            ? ".dock-facility,#save-indicator,#site-button,#dock-button,#pause-button,#map-button,#flight-settings,#fullscreen,#quick-settings,#quick-title,#toast,#destination-marker,#toolbar-hint"
            : "#toast"
          : "h2,h3,p,small,.eyebrow,.build-line span,.control-guide span,.upgrade-category span,.upgrade-category b,#quest-content li";
      for (const child of root.querySelectorAll<HTMLElement>(selectors)) {
        if (
          ["status-text", "warning-text"].includes(
            child.dataset.pixelKind ?? "",
          )
        )
          continue;
        add(
          child,
          child.tagName === "BUTTON" ? buttonKind(child) : textKind(child),
          child.id === "pause-button"
            ? child.dataset.pixelIcon === "status-icon"
              ? ""
              : document.body.classList.contains("touch-ui")
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
          child.id === "save-indicator" ||
            child.classList.contains("dock-facility")
            ? Number(child.dataset.pixelValue ?? 0)
            : 0,
        );
      }
      if (root.id !== "game-ui") {
        for (const button of root.querySelectorAll<HTMLButtonElement>(
          "button,.import-label",
        )) {
          // These accessible marker buttons were emitted once by the artwork
          // loop. Their hitboxes do not become generic frames or text objects.
          if (["map-home", "map-site"].includes(button.dataset.pixelKind ?? ""))
            continue;
          add(
            button,
            buttonKind(button),
            button.id === "settings-backup"
              ? "SAVE / BACKUP"
              : (button.textContent ?? ""),
          );
        }
        for (const label of root.querySelectorAll<HTMLLabelElement>(
          ".settings label",
        )) {
          const input = label.querySelector<HTMLInputElement>("input");
          const name = label.querySelector<HTMLElement>(".setting-name");
          const text =
            name?.textContent ?? label.childNodes[0]?.textContent ?? "";
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
              { label: name, control: input },
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
