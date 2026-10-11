import "./kikai.css";
import type { KikaiIdentity } from "./storage";

export interface IdentityHud extends KikaiIdentity {
  kind_name: string;
  gender: string;
  corporation_role: string;
  designation: string;
  short_designation: string;
  sub_corporation_name: string;
  starter_ship: string;
  starter_ability: string;
  license: string;
}
export interface OnboardingHud {
  stage: number;
  flags: number;
  title: string;
  description: string;
  completed: boolean;
}
export const corporations = {
  pan: {
    name: "PAN DYNAMICS",
    role: "PASSIVE",
    hair: "GREEN",
    ship: "PAN PATHFINDER",
    slot: "PASSIVE",
  },
  renko: {
    name: "RENKO CYBERNETICS",
    role: "OFFENSIVE",
    hair: "RED",
    ship: "RENKO LANCER",
    slot: "OFFENSIVE",
  },
  hugo: {
    name: "HUGO CORP",
    role: "DEFENSIVE",
    hair: "BLUE",
    ship: "HUGO BULWARK",
    slot: "DEFENSIVE",
  },
} as const;

/** Eight random bytes, never a timestamp or Math.random identity. */
export function newDroidHash(
  random: Pick<Crypto, "getRandomValues"> = crypto,
): string {
  return Array.from(random.getRandomValues(new Uint8Array(8)), (byte) =>
    byte.toString(16).padStart(2, "0"),
  ).join("");
}

export class DroidCreation {
  private readonly screen = document.createElement("section");
  private readonly detail = document.createElement("p");
  private readonly submit = document.createElement("button");
  private readonly closeButton = document.createElement("button");
  private readonly choices: HTMLButtonElement[] = [];
  private kind: KikaiIdentity["kind"] = "dora";
  private corporation: KikaiIdentity["corporation"] = "pan";
  private subCorporation: KikaiIdentity["sub_corporation"] = "renko";
  private busy = false;
  private readonly onKey = (event: KeyboardEvent) => {
    if (this.screen.hidden) return;
    event.stopImmediatePropagation();
    if (event.key === "Escape" && !this.busy) {
      event.preventDefault();
      this.close();
    }
    if (event.key === "Tab") {
      event.preventDefault();
      const buttons = [...this.choices, this.submit, this.closeButton].filter(
        (button) => !button.disabled,
      );
      const current = buttons.indexOf(
        document.activeElement as HTMLButtonElement,
      );
      const next =
        current < 0
          ? event.shiftKey
            ? buttons.length - 1
            : 0
          : (current + (event.shiftKey ? -1 : 1) + buttons.length) %
            buttons.length;
      buttons[next]?.focus();
    }
  };
  constructor(
    private options: {
      create: (identity: KikaiIdentity) => Promise<void>;
      onOpen: () => void;
      onClose: () => void;
    },
  ) {
    this.screen.id = "droid-screen";
    this.screen.className = "screen-overlay droid-screen";
    this.screen.hidden = true;
    this.screen.setAttribute("role", "dialog");
    this.screen.setAttribute("aria-modal", "true");
    this.screen.setAttribute("aria-labelledby", "droid-title");
    const card = document.createElement("div");
    card.className = "droid-card";
    this.closeButton.type = "button";
    this.closeButton.className = "icon-button droid-close";
    this.closeButton.dataset.pixelIcon = "close-icon";
    this.closeButton.setAttribute("aria-label", "Cancel droid creation");
    this.closeButton.textContent = "X";
    this.closeButton.onclick = () => this.close();
    const title = document.createElement("h2");
    title.id = "droid-title";
    title.textContent = "YOUR KIKAI LICENSE";
    const intro = document.createElement("p");
    intro.textContent =
      "Choose your droid and manufacturer. Each license includes a level 1 ship and one equipped ability.";
    const types = document.createElement("section");
    types.className = "droid-options";
    types.setAttribute("aria-label", "Droid type");
    const typeHeading = document.createElement("h3");
    typeHeading.textContent = "DROID TYPE";
    for (const [kind, gender] of [
      ["dora", "Female"],
      ["doro", "Male"],
      ["talo", "Neutral"],
    ] as const) {
      const button = this.choice(`${kind.toUpperCase()} / ${gender}`, () => {
        this.kind = kind;
        this.paint();
      });
      button.dataset.droidKind = kind;
      types.append(button);
    }
    const corps = document.createElement("section");
    corps.className = "droid-options";
    corps.setAttribute("aria-label", "Manufacturer");
    const corpHeading = document.createElement("h3");
    corpHeading.textContent = "MANUFACTURER";
    for (const [id, value] of Object.entries(corporations)) {
      const corporation = id as KikaiIdentity["corporation"];
      const button = this.choice(value.name, () => {
        this.corporation = corporation;
        this.paint();
      });
      button.dataset.corporation = id;
      button.dataset.pixelAction = `${corporation === "pan" ? "passive" : corporation === "renko" ? "offense" : "defense"}-button`;
      corps.append(button);
    }
    const secondaryHeading = document.createElement("h3");
    secondaryHeading.textContent = "CRYSTAL HORNS / SUB-MANUFACTURER";
    const secondary = document.createElement("section");
    secondary.className = "droid-options";
    secondary.setAttribute("aria-label", "Sub-manufacturer");
    for (const [id, color] of [
      ["pan", "GREEN"],
      ["renko", "RED"],
      ["hugo", "BLUE"],
    ] as const) {
      const button = this.choice(`${id.toUpperCase()} / ${color}`, () => {
        this.subCorporation = id;
        this.paint();
      });
      button.dataset.subCorporation = id;
      button.dataset.pixelAction = `${id === "pan" ? "passive" : id === "renko" ? "offense" : "defense"}-button`;
      secondary.append(button);
    }
    this.detail.className = "droid-license-summary";
    this.detail.setAttribute("aria-live", "polite");
    this.submit.type = "button";
    this.submit.className = "primary-button";
    this.submit.textContent = "ISSUE LICENSE";
    this.submit.onclick = () => void this.confirm();
    const content = document.createElement("section");
    content.className = "droid-content";
    content.append(
      intro,
      typeHeading,
      types,
      corpHeading,
      corps,
      secondaryHeading,
      secondary,
      this.detail,
    );
    card.append(this.closeButton, title, content, this.submit);
    this.screen.append(card);
    document.body.append(this.screen);
    window.addEventListener("keydown", this.onKey, true);
    this.screen.addEventListener("contextmenu", (event) =>
      event.preventDefault(),
    );
    this.paint();
  }
  get opened(): boolean {
    return !this.screen.hidden;
  }
  open(): void {
    this.screen.hidden = false;
    this.paint();
    this.options.onOpen();
    this.choices[0].focus();
  }
  close(): void {
    if (this.busy || this.screen.hidden) return;
    this.screen.hidden = true;
    this.options.onClose();
  }
  private choice(label: string, action: () => void): HTMLButtonElement {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = label;
    button.onclick = action;
    this.choices.push(button);
    return button;
  }
  private paint(): void {
    for (const button of this.choices) {
      const selected =
        button.dataset.droidKind === this.kind ||
        button.dataset.corporation === this.corporation ||
        button.dataset.subCorporation === this.subCorporation;
      button.setAttribute("aria-pressed", String(selected));
      button.dataset.selected = String(selected);
    }
    const corp = corporations[this.corporation];
    this.detail.textContent = `${this.kind.toUpperCase()}-${this.corporation.toUpperCase()}-${this.subCorporation.toUpperCase()} / ${corp.hair} HAIR / ${corporations[this.subCorporation].hair} HORNS. ${corp.ship} / LEVEL 1 / ${corp.slot} ABILITY EQUIPPED. YOUR HASH IS ISSUED ON CONFIRMATION.`;
  }
  private async confirm(): Promise<void> {
    if (this.busy) return;
    this.busy = true;
    for (const button of [...this.choices, this.submit, this.closeButton])
      button.disabled = true;
    try {
      await this.options.create({
        kind: this.kind,
        corporation: this.corporation,
        sub_corporation: this.subCorporation,
        hex_id: newDroidHash(),
      });
      this.screen.hidden = true;
      this.options.onClose();
    } catch (error) {
      this.detail.textContent =
        error instanceof Error ? error.message : String(error);
    } finally {
      this.busy = false;
      for (const button of [...this.choices, this.submit, this.closeButton])
        button.disabled = false;
    }
  }
}
