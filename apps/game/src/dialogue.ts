import "./dialogue.css";

export interface DialogueContent {
  speaker: string;
  text: string;
  portrait?: number;
  actionLabel?: string;
  onAction?: () => void | Promise<void>;
  onNext?: () => void;
  onPrevious?: () => void;
  onReplay?: () => void;
  page?: number;
  pageCount?: number;
}

export interface DialogueOptions {
  onOpen: () => void;
  onClose: () => void;
}

/** Semantic conversation controls; the pixel bridge draws their artwork. */
export class DialoguePanel {
  private readonly screen = document.createElement("section");
  private readonly portrait = document.createElement("div");
  private readonly speaker = document.createElement("h2");
  private readonly text = document.createElement("p");
  private readonly action = document.createElement("button");
  private readonly closeButton = document.createElement("button");
  private readonly previous = document.createElement("button");
  private readonly next = document.createElement("button");
  private readonly replay = document.createElement("button");
  private previousFocus: HTMLElement | null = null;
  private onAction: DialogueContent["onAction"];
  private onNext: DialogueContent["onNext"];
  private onPrevious: DialogueContent["onPrevious"];
  private onReplay: DialogueContent["onReplay"];
  private isOpen = false;
  private busy = false;
  private disposed = false;
  private revision = 0;

  constructor(private readonly options: DialogueOptions) {
    if (document.getElementById("dialogue-screen"))
      throw new Error("A dialogue panel is already mounted.");
    this.screen.id = "dialogue-screen";
    this.screen.className = "dialogue-screen";
    this.screen.hidden = true;
    this.screen.setAttribute("role", "dialog");
    this.screen.setAttribute("aria-modal", "true");
    this.screen.setAttribute("aria-labelledby", "dialogue-title");
    this.screen.setAttribute("aria-describedby", "dialogue-text");
    const card = document.createElement("div");
    card.className = "dialogue-card";
    this.portrait.className = "dialogue-portrait";
    this.portrait.setAttribute("aria-hidden", "true");
    const content = document.createElement("div");
    content.className = "dialogue-content";
    this.speaker.id = "dialogue-title";
    this.text.id = "dialogue-text";
    this.text.setAttribute("aria-live", "polite");
    const actions = document.createElement("div");
    actions.className = "dialogue-actions";
    this.action.id = "dialogue-action";
    this.action.type = "button";
    this.closeButton.id = "dialogue-close";
    this.closeButton.type = "button";
    this.closeButton.textContent = "CLOSE";
    this.previous.id = "dialogue-previous";
    this.previous.type = "button";
    this.previous.textContent = "BACK";
    this.next.id = "dialogue-next";
    this.next.type = "button";
    this.replay.id = "dialogue-replay";
    this.replay.type = "button";
    this.replay.textContent = "REPLAY VOICE";
    actions.append(this.previous, this.next, this.action, this.replay, this.closeButton);
    content.append(this.speaker, this.text, actions);
    card.append(this.portrait, content);
    this.screen.append(card);
    document.body.append(this.screen);
    this.action.addEventListener("click", this.act);
    this.closeButton.addEventListener("click", this.close);
    this.next.addEventListener("click", () => this.onNext?.());
    this.previous.addEventListener("click", () => this.onPrevious?.());
    this.replay.addEventListener("click", () => this.onReplay?.());
    window.addEventListener("keydown", this.onKeyDown, true);
    window.addEventListener("keyup", this.onKeyUp, true);
    for (const name of ["pointerdown", "pointermove", "pointerup", "click"])
      this.screen.addEventListener(name, this.stopPointer);
    this.screen.addEventListener("contextmenu", this.contextMenu);
  }

  get opened(): boolean {
    return this.isOpen;
  }

  show(content: DialogueContent): void {
    if (this.disposed) return;
    this.revision++;
    this.busy = false;
    this.screen.removeAttribute("aria-busy");
    this.speaker.textContent = content.speaker.trim().slice(0, 64);
    this.text.textContent = content.text.trim().slice(0, 256);
    const portrait = content.portrait ?? 0;
    this.portrait.dataset.portrait = String(
      Number.isFinite(portrait)
        ? Math.max(0, Math.min(7, Math.floor(portrait)))
        : 0,
    );
    this.onAction = content.onAction;
    this.onNext = content.onNext;
    this.onPrevious = content.onPrevious;
    this.onReplay = content.onReplay;
    this.next.hidden = !content.onNext;
    this.previous.hidden = !content.onPrevious;
    this.replay.hidden = !content.onReplay;
    this.next.textContent = `NEXT ${Math.max(1, (content.page ?? 0) + 1)}/${Math.max(1, content.pageCount ?? 1)}`;
    this.action.textContent = (content.actionLabel ?? "ACCEPT")
      .trim()
      .slice(0, 40);
    this.action.hidden = !content.onAction;
    this.action.disabled = false;
    if (!this.isOpen) {
      this.previousFocus =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : null;
      this.isOpen = true;
      this.screen.hidden = false;
      this.options.onOpen();
    }
    if (!this.isOpen || this.disposed) return;
    // Refreshes and actions cannot accidentally move focus back into flight.
    (this.next.hidden ? this.action.hidden ? this.closeButton : this.action : this.next).focus({
      preventScroll: true,
    });
  }

  readonly close = (): void => {
    if (!this.isOpen) return;
    this.revision++;
    this.isOpen = false;
    this.busy = false;
    this.onAction = undefined;
    this.onNext = undefined;
    this.onPrevious = undefined;
    this.onReplay = undefined;
    this.screen.hidden = true;
    this.screen.removeAttribute("aria-busy");
    const previousFocus = this.previousFocus;
    this.previousFocus = null;
    this.options.onClose();
    if (!this.isOpen && previousFocus?.isConnected)
      previousFocus.focus({ preventScroll: true });
  };

  dispose(): void {
    if (this.disposed) return;
    this.close();
    this.disposed = true;
    window.removeEventListener("keydown", this.onKeyDown, true);
    window.removeEventListener("keyup", this.onKeyUp, true);
    this.screen.remove();
  }

  private readonly act = async (): Promise<void> => {
    if (!this.isOpen || this.busy || !this.onAction) return;
    const revision = this.revision;
    const callback = this.onAction;
    this.busy = true;
    this.action.disabled = true;
    this.screen.setAttribute("aria-busy", "true");
    try {
      await callback();
    } catch (error) {
      if (this.isOpen && revision === this.revision)
        this.text.textContent = (
          error instanceof Error ? error.message : String(error)
        ).slice(0, 256);
    } finally {
      // A late action result cannot modify a closed or newer conversation.
      if (this.isOpen && revision === this.revision) {
        this.busy = false;
        this.action.disabled = false;
        this.screen.removeAttribute("aria-busy");
      }
    }
  };

  private readonly onKeyDown = (event: KeyboardEvent): void => {
    if (!this.isOpen) return;
    event.stopImmediatePropagation();
    if (event.key === "Escape") {
      event.preventDefault();
      if (!event.isComposing) this.close();
    } else if (event.key === "Tab") {
      event.preventDefault();
      const available = [this.previous, this.next, this.action, this.replay, this.closeButton]
        .filter(button => !button.hidden && !button.disabled);
      const index = available.indexOf(
        document.activeElement as HTMLButtonElement,
      );
      const next =
        index < 0
          ? event.shiftKey
            ? available.length - 1
            : 0
          : (index + (event.shiftKey ? -1 : 1) + available.length) %
            available.length;
      available[next]!.focus();
    }
  };
  private readonly onKeyUp = (event: KeyboardEvent): void => {
    if (this.isOpen) event.stopImmediatePropagation();
  };
  private readonly stopPointer = (event: Event): void =>
    event.stopPropagation();
  private readonly contextMenu = (event: Event): void => {
    event.preventDefault();
    event.stopPropagation();
  };
}
