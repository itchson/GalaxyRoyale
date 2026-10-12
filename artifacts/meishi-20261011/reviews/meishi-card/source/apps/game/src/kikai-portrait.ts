import type { KikaiIdentity } from "./storage";

export type PortraitIdentity = Pick<
  KikaiIdentity,
  "kind" | "corporation" | "sub_corporation"
>;
const kinds = ["dora", "doro", "talo"] as const;
const manufacturers = ["pan", "renko", "hugo"] as const;
const colors = ["GREEN", "RED", "BLUE"] as const;

/** Native profile 9 uses independent type, hair and crystal palette roles. */
export function paintLicensePortrait(
  target: HTMLElement,
  identity: PortraitIdentity,
): boolean {
  const kind = kinds.indexOf(identity.kind);
  const main = manufacturers.indexOf(identity.corporation);
  const sub = manufacturers.indexOf(identity.sub_corporation);
  let changed = false;
  const fields = {
    pixelKind: "portrait",
    pixelValue: "9",
    portraitKind: String(kind),
    portraitMain: String(main),
    portraitSub: String(sub),
    portraitExpression: "0",
    portraitSpeaking: "false",
  };
  for (const [key, value] of Object.entries(fields)) {
    if (target.dataset[key] !== value) {
      target.dataset[key] = value;
      changed = true;
    }
  }
  const label = `${identity.kind.toUpperCase()} profile: ${colors[main]} hair, ${colors[sub]} crystals`;
  if (target.getAttribute("aria-label") !== label) {
    target.setAttribute("aria-label", label);
    changed = true;
  }
  return changed;
}
