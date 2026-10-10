export interface InputFrame {
  joystick: [number, number] | null;
  destination: [number, number] | null;
  fire: boolean;
}
export class FlightInput {
  private movePointer: number | null = null;
  private firePointer: number | null = null;
  private joystick: [number, number] | null = null;
  private destination: [number, number] | null = null;
  private mouseFire = false;
  private mouseMove = false;
  private keys = new Set<string>();
  enabled = false;
  camera: [number, number] = [0, 0];
  viewport: [number, number] = [800, 360];
  constructor(
    private canvas: HTMLCanvasElement,
    private stickArea: HTMLElement,
    private stick: HTMLElement,
    private fireButton: HTMLButtonElement,
    private onDestination: (world: [number, number]) => void,
    private onFireStart: () => void = () => {},
  ) {
    canvas.addEventListener("contextmenu", (e) => e.preventDefault());
    canvas.addEventListener("pointerdown", (e) => {
      if (!this.enabled || e.pointerType === "touch") return;
      canvas.setPointerCapture(e.pointerId);
      if (e.button === 0) {
        this.mouseMove = true;
        this.target(e);
      }
      this.mouseFire = (e.buttons & 2) !== 0;
      if (e.button === 2 && this.mouseFire) this.onFireStart();
    });
    canvas.addEventListener("pointermove", (e) => {
      if (!this.enabled || e.pointerType === "touch") return;
      this.mouseFire = (e.buttons & 2) !== 0;
      this.mouseMove = (e.buttons & 1) !== 0;
      if (this.mouseMove) this.target(e);
    });
    canvas.addEventListener("pointerup", (e) => {
      this.mouseFire = (e.buttons & 2) !== 0;
      this.mouseMove = (e.buttons & 1) !== 0;
    });
    canvas.addEventListener("pointercancel", () => this.reset());
    canvas.addEventListener("lostpointercapture", () => {
      this.mouseFire = false;
      this.mouseMove = false;
    });
    stickArea.addEventListener("pointerdown", (e) => {
      if (!this.enabled || this.movePointer !== null) return;
      e.preventDefault();
      this.movePointer = e.pointerId;
      stickArea.setPointerCapture(e.pointerId);
      this.moveStick(e);
    });
    stickArea.addEventListener("pointermove", (e) => {
      if (e.pointerId === this.movePointer && this.enabled) this.moveStick(e);
    });
    const releaseMove = (e: PointerEvent) => {
      if (e.pointerId === this.movePointer) {
        this.movePointer = null;
        this.joystick = null;
        stick.style.transform = "";
      }
    };
    stickArea.addEventListener("pointerup", releaseMove);
    stickArea.addEventListener("pointercancel", releaseMove);
    stickArea.addEventListener("lostpointercapture", releaseMove);
    fireButton.addEventListener("pointerdown", (e) => {
      if (!this.enabled || this.firePointer !== null) return;
      e.preventDefault();
      this.firePointer = e.pointerId;
      fireButton.setPointerCapture(e.pointerId);
      fireButton.classList.add("firing");
      this.onFireStart();
    });
    const releaseFire = (e: PointerEvent) => {
      if (e.pointerId === this.firePointer) {
        this.firePointer = null;
        fireButton.classList.remove("firing");
      }
    };
    fireButton.addEventListener("pointerup", releaseFire);
    fireButton.addEventListener("pointercancel", releaseFire);
    fireButton.addEventListener("lostpointercapture", releaseFire);
    window.addEventListener("keydown", (e) => {
      if (
        this.enabled &&
        [
          "w",
          "a",
          "s",
          "d",
          "ArrowUp",
          "ArrowDown",
          "ArrowLeft",
          "ArrowRight",
        ].includes(e.key)
      ) {
        this.keys.add(e.key.toLowerCase());
        e.preventDefault();
      }
    });
    window.addEventListener("keyup", (e) =>
      this.keys.delete(e.key.toLowerCase()),
    );
    window.addEventListener("blur", () => this.reset());
  }
  private target(e: PointerEvent): void {
    const r = this.canvas.getBoundingClientRect();
    this.destination = [
      this.camera[0] +
        ((e.clientX - r.left - r.width / 2) * this.viewport[0]) / r.width,
      this.camera[1] +
        ((e.clientY - r.top - r.height / 2) * this.viewport[1]) / r.height,
    ];
    this.onDestination(this.destination);
  }
  /** Same one-shot course command used by a mouse click, exposed for navigation. */
  setCourse(world: [number, number]): void {
    if (!this.enabled || !world.every(Number.isFinite)) return;
    this.destination = [...world];
    this.onDestination(this.destination);
  }
  private moveStick(e: PointerEvent): void {
    const r = this.stickArea.getBoundingClientRect();
    const max = r.width * 0.34;
    let x = (e.clientX - r.left - r.width / 2) / max;
    let y = (e.clientY - r.top - r.height / 2) / max;
    const length = Math.hypot(x, y);
    if (length > 1) {
      x /= length;
      y /= length;
    }
    this.joystick = [x, y];
    this.stick.style.transform = `translate(${x * max}px, ${y * max}px)`;
  }
  sample(): InputFrame {
    if (!this.enabled)
      return { joystick: null, destination: null, fire: false };
    let joystick = this.joystick;
    const x =
      Number(this.keys.has("d") || this.keys.has("arrowright")) -
      Number(this.keys.has("a") || this.keys.has("arrowleft"));
    const y =
      Number(this.keys.has("s") || this.keys.has("arrowdown")) -
      Number(this.keys.has("w") || this.keys.has("arrowup"));
    if (x || y) {
      const l = Math.hypot(x, y);
      joystick = [x / l, y / l];
    }
    const frame: InputFrame = {
      joystick,
      destination: this.destination,
      fire: this.mouseFire || this.firePointer !== null,
    };
    this.destination = null;
    return frame;
  }
  reset(): void {
    this.joystick = null;
    this.destination = null;
    this.movePointer = null;
    this.firePointer = null;
    this.mouseFire = false;
    this.mouseMove = false;
    this.keys.clear();
    this.stick.style.transform = "";
    this.fireButton.classList.remove("firing");
  }
}
