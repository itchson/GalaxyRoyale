import "./debug-console.css";

export interface DebugConsoleOptions {
  ready: () => boolean;
  run: (command: string) => string | Promise<string>;
  onOpen: () => void;
  onClose: () => void;
}

const COMMAND_LIMIT = 256;
const HISTORY_LIMIT = 32;
const OUTPUT_LIMIT = 80;
const TRANSITION_MS = 180;
type ConsolePhase = "closed" | "opening" | "open" | "closing";

export interface ConsoleTransitionDetail {
  phase: ConsolePhase;
  durationMs: number;
}

/** Semantic controls; the pixel UI bridge supplies their in-game artwork. */
export class DebugConsole {
  private readonly screen = document.createElement("section");
  private readonly card = document.createElement("div");
  private readonly output = document.createElement("div");
  private readonly input = document.createElement("input");
  private readonly history: string[] = [];
  private historyIndex = 0;
  private draft = "";
  private previousFocus: HTMLElement | null = null;
  private isOpen = false;
  private phase: ConsolePhase = "closed";
  private transitionTimer: number | null = null;
  private revealFrame: number | null = null;
  private running = false;
  private disposed = false;

  constructor(private readonly options: DebugConsoleOptions) {
    if (document.getElementById("debug-console"))
      throw new Error("A debug console is already mounted.");
    this.screen.id = "debug-console";
    this.screen.className = "console-screen";
    this.screen.hidden = true;
    this.screen.setAttribute("role", "dialog");
    this.screen.setAttribute("aria-modal", "true");
    this.screen.setAttribute("aria-labelledby", "console-title");

    this.card.className = "console-card";
    const heading = document.createElement("h2");
    heading.id = "console-title";
    heading.textContent = "GALAXY ROYALE // CONSOLE";
    const help = document.createElement("p");
    help.className = "console-instructions";
    help.textContent = "HELP / STATS / SCENE <NAME> / CLEAR / ` CLOSE";
    this.output.id = "console-output";
    this.output.setAttribute("role", "log");
    this.output.setAttribute("aria-label", "Console output");
    this.output.setAttribute("aria-live", "polite");
    this.output.setAttribute("aria-relevant", "additions");
    this.input.id = "console-input";
    this.input.type = "text";
    this.input.maxLength = COMMAND_LIMIT;
    this.input.autocomplete = "off";
    this.input.setAttribute("autocapitalize", "off");
    this.input.spellcheck = false;
    this.input.setAttribute("aria-label", "Console command");
    this.input.setAttribute("aria-describedby", "console-title");
    this.input.placeholder = "ENTER COMMAND";
    this.card.append(heading, help, this.output, this.input);
    this.screen.append(this.card);
    document.body.append(this.screen);

    // Capture before the game's bubbling keyboard listeners see console input.
    window.addEventListener("keydown", this.onKeyDown, true);
    window.addEventListener("keyup", this.onKeyUp, true);
    this.screen.addEventListener("pointerdown", this.onPointerDown);
    for (const name of ["pointermove", "pointerup", "pointercancel", "click"])
      this.screen.addEventListener(name, this.stopPointer);
    this.screen.addEventListener("contextmenu", this.onContextMenu);
    this.card.addEventListener("transitionend", this.onTransitionEnd);
  }

  get opened(): boolean {
    return this.isOpen;
  }

  get transitioning(): boolean {
    return this.phase === "opening" || this.phase === "closing";
  }

  open(): void {
    if (
      this.disposed ||
      this.phase === "open" ||
      this.phase === "opening" ||
      !this.options.ready()
    )
      return;
    const freshOpen = !this.isOpen;
    this.cancelTransition();
    if (freshOpen) {
      this.previousFocus =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : null;
      this.isOpen = true;
      this.screen.hidden = false;
      this.options.onOpen();
    }
    const duration = this.reducedMotion() ? 0 : TRANSITION_MS;
    this.screen.classList.remove("console-closing");
    this.screen.classList.add("console-opening");
    this.phase = "opening";
    this.emitTransition(duration);
    const reveal = () => {
      this.revealFrame = null;
      this.screen.classList.add("console-visible");
      if (duration === 0) this.finishOpening();
      else this.scheduleCompletion();
    };
    if (freshOpen && duration > 0) {
      // Commit the off-screen transform before the following frame reveals it.
      this.card.getBoundingClientRect();
      this.revealFrame = requestAnimationFrame(reveal);
    } else reveal();
    this.input.focus({ preventScroll: true });
  }

  close(): void {
    if (!this.isOpen || this.phase === "closing") return;
    this.cancelTransition();
    this.phase = "closing";
    this.screen.classList.remove("console-opening", "console-visible");
    this.screen.classList.add("console-closing");
    const duration = this.reducedMotion() ? 0 : TRANSITION_MS;
    this.emitTransition(duration);
    if (duration === 0) this.finishClose();
    else this.scheduleCompletion();
  }

  private reducedMotion(): boolean {
    return (
      document.body.classList.contains("reduced-motion") ||
      matchMedia("(prefers-reduced-motion: reduce)").matches
    );
  }

  private emitTransition(durationMs: number): void {
    this.screen.dispatchEvent(
      new CustomEvent<ConsoleTransitionDetail>("galaxy-console-transition", {
        bubbles: true,
        detail: { phase: this.phase, durationMs },
      }),
    );
  }

  private cancelTransition(): void {
    if (this.transitionTimer !== null) clearTimeout(this.transitionTimer);
    if (this.revealFrame !== null) cancelAnimationFrame(this.revealFrame);
    this.transitionTimer = null;
    this.revealFrame = null;
  }

  private scheduleCompletion(): void {
    // transitionend is primary; detached tabs or missing transitions need a bound.
    this.transitionTimer = window.setTimeout(() => {
      this.transitionTimer = null;
      if (this.phase === "closing") this.finishClose();
      else if (this.phase === "opening") this.finishOpening();
    }, TRANSITION_MS + 40);
  }

  private finishOpening(): void {
    if (this.phase !== "opening") return;
    this.cancelTransition();
    this.phase = "open";
    this.screen.classList.remove("console-opening");
    this.emitTransition(0);
  }

  private finishClose(): void {
    this.cancelTransition();
    if (!this.isOpen) return;
    this.phase = "closed";
    this.isOpen = false;
    this.screen.hidden = true;
    this.screen.classList.remove(
      "console-opening",
      "console-visible",
      "console-closing",
    );
    this.input.blur();
    this.emitTransition(0);
    this.options.onClose();
    if (this.previousFocus?.isConnected)
      this.previousFocus.focus({ preventScroll: true });
    this.previousFocus = null;
  }

  private readonly onTransitionEnd = (event: TransitionEvent): void => {
    if (event.target !== this.card || event.propertyName !== "transform")
      return;
    if (this.phase === "closing") this.finishClose();
    else if (this.phase === "opening") this.finishOpening();
  };

  /** Each line is one bounded paragraph, never interpreted as HTML. */
  write(message: string): void {
    if (this.disposed) return;
    const lines = message.replace(/\r\n?/g, "\n").split("\n");
    for (const line of lines.slice(-OUTPUT_LIMIT)) {
      const paragraph = document.createElement("p");
      paragraph.textContent = line.slice(0, COMMAND_LIMIT) || " ";
      this.output.append(paragraph);
    }
    while (this.output.childElementCount > OUTPUT_LIMIT)
      this.output.firstElementChild?.remove();
    this.output.scrollTop = this.output.scrollHeight;
  }

  clear(): void {
    this.output.replaceChildren();
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.finishClose();
    window.removeEventListener("keydown", this.onKeyDown, true);
    window.removeEventListener("keyup", this.onKeyUp, true);
    this.screen.remove();
  }

  private readonly onKeyDown = (event: KeyboardEvent): void => {
    if (event.code === "Backquote" || event.key === "`") {
      if (!this.options.ready() || event.isComposing) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      if (!event.repeat)
        this.phase === "closing" || !this.isOpen ? this.open() : this.close();
      return;
    }
    if (!this.isOpen) return;
    event.stopImmediatePropagation();
    if (event.key === "Escape") {
      event.preventDefault();
      this.close();
      return;
    }
    if (this.phase === "closing") {
      event.preventDefault();
      return;
    }
    if (event.key === "Tab") {
      event.preventDefault();
      this.input.focus({ preventScroll: true });
      return;
    }
    if (event.isComposing || document.activeElement !== this.input) return;
    if (event.key === "Enter") {
      event.preventDefault();
      if (!event.repeat) void this.submit();
    } else if (event.key === "ArrowUp" || event.key === "ArrowDown") {
      event.preventDefault();
      this.recall(event.key === "ArrowUp" ? -1 : 1);
    }
  };

  private readonly onKeyUp = (event: KeyboardEvent): void => {
    if (this.isOpen) event.stopImmediatePropagation();
  };

  private readonly stopPointer = (event: Event): void => {
    event.stopPropagation();
  };

  private readonly onPointerDown = (event: PointerEvent): void => {
    event.stopPropagation();
    if (event.target === this.screen) {
      event.preventDefault();
      this.input.focus({ preventScroll: true });
    }
  };

  private readonly onContextMenu = (event: Event): void => {
    event.preventDefault();
    event.stopPropagation();
  };

  private setCommand(command: string): void {
    this.input.value = command;
    this.input.setSelectionRange(command.length, command.length);
    // A value property update is not a DOM mutation: invalidate the pixel bridge.
    this.input.dispatchEvent(new Event("input", { bubbles: true }));
  }

  private recall(direction: -1 | 1): void {
    if (!this.history.length) return;
    if (this.historyIndex === this.history.length)
      this.draft = this.input.value;
    this.historyIndex = Math.max(
      0,
      Math.min(this.history.length, this.historyIndex + direction),
    );
    this.setCommand(
      this.historyIndex === this.history.length
        ? this.draft
        : this.history[this.historyIndex],
    );
  }

  private async submit(): Promise<void> {
    if (this.running) return;
    const command = this.input.value.trim().slice(0, COMMAND_LIMIT);
    if (!command) return;
    this.history.push(command);
    if (this.history.length > HISTORY_LIMIT) this.history.shift();
    this.historyIndex = this.history.length;
    this.draft = "";
    this.setCommand("");
    if (command.toUpperCase() === "CLEAR") {
      this.clear();
      return;
    }
    this.write(`> ${command}`);
    this.running = true;
    this.screen.setAttribute("aria-busy", "true");
    try {
      const result = await this.options.run(command);
      if (result) this.write(result);
    } catch (error) {
      this.write(
        `ERROR: ${error instanceof Error ? error.message : String(error)}`,
      );
    } finally {
      this.running = false;
      this.screen.removeAttribute("aria-busy");
    }
  }
}
