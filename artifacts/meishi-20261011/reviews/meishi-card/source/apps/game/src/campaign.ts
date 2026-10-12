import "./campaign.css";
import type { LoadoutShipHud } from "./loadout";
import type { KikaiIdentity } from "./storage";

export interface ChampionIdentity extends KikaiIdentity {
  designation: string;
  short_designation: string;
}

export interface Champion {
  index: number;
  id: string;
  name: string;
  species: string;
  ship: string;
  ability: string;
  silhouette: string;
  family: number;
  category: number;
  position: [number, number];
  patch: boolean;
  blueprint: boolean;
  crafted: boolean;
  discovered?: boolean;
  identity: ChampionIdentity | null;
  sponsor: KikaiIdentity["corporation"] | null;
  display_model: string;
  legacy_model: string;
  legacy_species: string;
  introduction: string;
}
export interface CampaignHud {
  champions: Champion[];
  arena: {
    index: number;
    name: string;
    status: string;
    can_challenge: boolean;
  } | null;
  encounter: {
    index: number;
    tournament: boolean;
    health: number;
    max_health: number;
  } | null;
  ships: LoadoutShipHud[];
  tournament_unlocked: boolean;
  tournament_complete: boolean;
  tournament_won: boolean;
  tournament_position: [number, number];
  tournament_nearby: boolean;
  standings: {
    entrant: number;
    name: string;
    played: number;
    wins: number;
    losses: number;
    points: number;
    tie_break: number;
    rank: number;
  }[];
  matches: { a: number; b: number; winner: number | null; source: string }[];
  credits_pending: boolean;
  selected_ship: number;
  tournament_attempt: number;
}
export interface CampaignActions {
  navigate: (position: [number, number]) => void;
  challenge: (index: number) => void;
  tournament: () => void;
  craft: (index: number) => void;
  select: (id: number) => void;
  credits: () => void;
}
function text(tag: "h3" | "p", value: string): HTMLElement {
  const node = document.createElement(tag);
  node.textContent = value;
  return node;
}
function icon(kind: string, value: number, active: boolean): HTMLElement {
  const node = document.createElement("div");
  node.className = `campaign-icon ${kind}`;
  node.dataset.pixelKind = kind;
  node.dataset.pixelValue = String(value);
  node.dataset.active = String(active);
  node.setAttribute("aria-hidden", "true");
  return node;
}
function button(
  label: string,
  callback: () => void,
  disabled = false,
): HTMLButtonElement {
  const node = document.createElement("button");
  node.textContent = label;
  node.disabled = disabled;
  node.addEventListener("click", callback);
  return node;
}

/** View-only campaign pages. All rewards, costs and encounter gates live in Rust. */
export function paintCampaign(
  target: HTMLElement,
  hud: CampaignHud,
  actions: CampaignActions,
): void {
  target.replaceChildren();
  const collected = hud.champions.filter((champion) => champion.patch).length;
  target.append(
    text(
      "p",
      `${collected}/8 ARENA PATCHES / defeat champions to earn their ship blueprints.`,
    ),
  );
  const roster = document.createElement("div");
  roster.className = "campaign-roster";
  const knownChampions = hud.champions.filter(
    (champion) => champion.discovered ?? champion.blueprint,
  );
  if (!knownChampions.length)
    roster.append(
      text(
        "p",
        "Explore the universe to discover arena champions. Their ships and abilities appear when found.",
      ),
    );
  for (const champion of knownChampions) {
    const card = document.createElement("article");
    card.className = "campaign-card";
    card.append(icon("campaign-patch", champion.index, champion.patch));
    const content = document.createElement("div");
    content.append(text("h3", `${champion.name} / ${champion.species}`));
    if (champion.identity)
      content.append(
        text(
          "p",
          `${champion.identity.short_designation} / ${champion.sponsor?.toUpperCase() ?? "INDEPENDENT"} SPONSORED`,
        ),
      );
    content.append(text("p", `${champion.ship} / ${champion.ability}`));
    content.append(
      text(
        "p",
        champion.patch
          ? "PATCH + BLUEPRINT SECURED"
          : `${Math.round(champion.position[0])}, ${Math.round(champion.position[1])} / undefeated`,
      ),
    );
    const controls = document.createElement("div");
    controls.className = "campaign-controls";
    controls.append(
      button(
        "SET COURSE",
        () => actions.navigate(champion.position),
        !!hud.encounter,
      ),
    );
    if (hud.arena?.index === champion.index)
      controls.append(
        button(
          champion.patch ? "DEFEATED" : "CHALLENGE",
          () => actions.challenge(champion.index),
          !hud.arena.can_challenge || !!hud.encounter,
        ),
      );
    content.append(controls);
    card.append(content);
    roster.append(card);
  }
  target.append(roster);
  const league = document.createElement("section");
  league.className = "campaign-league";
  if (hud.tournament_won) league.append(icon("campaign-patch", 8, true));
  league.append(text("h3", "GALAXY ROYALE"));
  if (!hud.tournament_unlocked) {
    league.append(
      text(
        "p",
        "Collect all eight patches to reveal the tournament gate in your universe.",
      ),
    );
  } else {
    const played = hud.matches.filter(
      (match) => (match.a === 8 || match.b === 8) && match.winner !== null,
    ).length;
    league.append(
      text(
        "p",
        hud.tournament_won
          ? "LEAGUE WON / Sovereign ship unlocked. Your galaxy remains open."
          : hud.tournament_complete
            ? "SEASON COMPLETE / return to the gate to begin a new season."
            : `SEASON ${hud.tournament_attempt + 1} / ${played}/8 duels complete. Place first to claim the Sovereign.`,
      ),
    );
    league.append(
      text(
        "p",
        "3 points per win. Equal scores use the seeded league tie break. NPC duels are simulated.",
      ),
    );
    const controls = document.createElement("div");
    controls.className = "campaign-controls";
    controls.append(
      button(
        "GATE COURSE",
        () => actions.navigate(hud.tournament_position),
        !!hud.encounter,
      ),
    );
    controls.append(
      button(
        played === 8 ? "NEW SEASON" : "ENTER NEXT DUEL",
        actions.tournament,
        !hud.tournament_nearby || !!hud.encounter || hud.tournament_won,
      ),
    );
    if (hud.tournament_won)
      controls.append(button("END CREDITS", actions.credits));
    league.append(controls);
    const table = document.createElement("div");
    table.className = "standings";
    table.setAttribute("role", "table");
    table.setAttribute("aria-label", "Round robin standings");
    table.append(text("p", "RANK / ENTRANT / WINS / LOSSES / POINTS"));
    for (const entrant of [...hud.standings].sort((a, b) => a.rank - b.rank)) {
      const row = text(
        "p",
        `${entrant.rank}. ${entrant.name} / ${entrant.wins}W ${entrant.losses}L / ${entrant.points} PTS`,
      );
      row.dataset.active = String(entrant.entrant === 8);
      table.append(row);
    }
    league.append(table);
    const ledger = document.createElement("div");
    ledger.hidden = true;
    const toggle = button("MATCH LEDGER", () => {
      ledger.hidden = !ledger.hidden;
    });
    for (const match of hud.matches) {
      const name = (id: number) =>
        id === 8 ? "YOU" : (hud.champions[id]?.name ?? "UNKNOWN");
      ledger.append(
        text(
          "p",
          `${name(match.a)} / ${name(match.b)} : ${match.winner === null ? "PENDING" : name(match.winner) + " WINS"} [${match.source.toUpperCase()}]`,
        ),
      );
    }
    league.append(toggle, ledger);
  }
  target.append(league);
}

export function paintCredits(
  target: HTMLElement,
  actions: CampaignActions,
): void {
  target.replaceChildren();
  const card = document.createElement("section");
  card.className = "ending-credits";
  card.append(icon("campaign-ship", 8, true));
  card.append(text("h3", "GALAXY ROYALE / CHAMPION"));
  card.append(
    text(
      "p",
      "Eight arenas. A league of rival universes. One star still standing.",
    ),
  );
  card.append(text("p", "YOUR SOVEREIGN AWAITS AT HAVEN"));
  card.append(text("h3", "CREATED BY ITCHSON"));
  card.append(
    text(
      "p",
      "Production, engine, gameplay and audiovisual development / Itchson + Codex",
    ),
  );
  card.append(
    text(
      "p",
      "Original pixel ships, portraits and insignia / Itchson + Codex. Concept references / OpenAI image generation.",
    ),
  );
  card.append(
    text("p", "Procedural score and sound / Galaxy Royale audio engine"),
  );
  card.append(
    text(
      "p",
      "Built with Rust, hecs, wgpu and Web Audio. Tiny5 Project Authors / SIL Open Font License.",
    ),
  );
  card.append(text("p", "THANK YOU FOR FLYING / your adventure continues"));
  card.append(button("OPEN HANGAR", actions.credits));
  target.append(card);
}
