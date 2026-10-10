//! Eight real champion trials and a nine-entrant solo round-robin league.
use crate::{PixelBody, pixel_assets, regions};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CampaignProgress {
    pub patches: [bool; 8],
    pub blueprints: [bool; 8],
    pub crafted: [bool; 8],
    pub selected_ship: u8,
    /// 0: unplayed; 1: player win; 2: player loss/forfeit.
    pub royale_results: [u8; 8],
    pub tournament_attempt: u32,
    pub victory_claimed: bool,
    pub credits_seen: bool,
    pub ship_xp: [u32; 10],
    pub cargo_materials: u32,
    pub stored_materials: u32,
    pub last_run: Option<RunSummary>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct RunSummary {
    pub time: f32,
    pub distance_peak: f32,
    pub enemies: u32,
    pub asteroids: u32,
    pub champions: u32,
    pub points_gained: [u32; 3],
    pub xp_gained: u32,
    pub materials_gained: u32,
}
pub fn level(xp: u32) -> u32 {
    let mut level = 1;
    while level < 100 && (level as u64 * (level as u64 + 1) * 10) <= xp as u64 {
        level += 1;
    }
    level
}
pub fn next_xp(level: u32) -> u32 {
    level.min(100) * level.min(100).saturating_add(1) * 10
}
pub fn required_level(rank: u8) -> u32 {
    [1, 3, 5, 8, 12][rank.saturating_sub(1).min(4) as usize]
}
pub fn rank_cap(level: u32) -> u8 {
    [1, 3, 5, 8, 12]
        .iter()
        .filter(|gate| level >= **gate)
        .count() as u8
}
pub fn signature_rank(p: &crate::Profile, index: usize) -> u8 {
    if p.campaign.selected_ship == 9 {
        5
    } else {
        p.ranks[index].min(rank_cap(level(
            p.campaign.ship_xp[p.campaign.selected_ship as usize],
        )))
    }
}

impl CampaignProgress {
    pub fn sanitize(&mut self) -> bool {
        let before = self.clone();
        self.blueprints = self.patches;
        for i in 0..8 {
            self.crafted[i] &= self.blueprints[i];
            if self.royale_results[i] > 2 {
                self.royale_results[i] = 0;
            }
        }
        if !self.patches.iter().all(|v| *v) {
            self.royale_results = [0; 8];
        }
        self.victory_claimed &= self.patches.iter().all(|v| *v) && self.complete();
        self.credits_seen &= self.victory_claimed;
        self.cargo_materials = self.cargo_materials.min(16);
        if let Some(run) = &mut self.last_run {
            if !run.time.is_finite() || !run.distance_peak.is_finite() {
                self.last_run = None;
            } else {
                run.time = run.time.clamp(0.0, 2_678_400.0);
                run.distance_peak = run.distance_peak.clamp(0.0, 1_414_214.0);
            }
        }
        if !self.owned(self.selected_ship) {
            self.selected_ship = 0;
        }
        *self != before
    }
    pub fn owned(&self, id: u8) -> bool {
        match id {
            0 => true,
            1..=8 => self.crafted[id as usize - 1],
            9 => self.victory_claimed,
            _ => false,
        }
    }
    pub fn complete(&self) -> bool {
        self.royale_results.iter().all(|v| *v != 0)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ChampionHud {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub species: String,
    pub ship: String,
    pub ability: String,
    pub silhouette: String,
    pub family: usize,
    pub category: usize,
    pub position: [f32; 2],
    pub patch: bool,
    pub blueprint: bool,
    pub crafted: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct ArenaHud {
    pub index: usize,
    pub name: String,
    pub status: String,
    pub can_challenge: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct EncounterHud {
    pub index: usize,
    pub tournament: bool,
    pub health: f32,
    pub max_health: f32,
}
#[derive(Clone, Debug, Serialize)]
pub struct ShipHud {
    pub id: u8,
    pub name: String,
    pub ability: String,
    pub owned: bool,
    pub selected: bool,
    pub category: usize,
    pub craftable: bool,
}
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct StandingHud {
    pub entrant: usize,
    pub name: String,
    pub played: u8,
    pub wins: u8,
    pub losses: u8,
    pub points: u8,
    pub tie_break: u32,
    pub rank: u8,
}
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct MatchHud {
    pub a: usize,
    pub b: usize,
    pub winner: Option<usize>,
    pub source: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct CampaignHud {
    pub champions: Vec<ChampionHud>,
    pub arena: Option<ArenaHud>,
    pub encounter: Option<EncounterHud>,
    pub ships: Vec<ShipHud>,
    pub tournament_unlocked: bool,
    pub tournament_complete: bool,
    pub tournament_won: bool,
    pub tournament_position: [f32; 2],
    pub tournament_nearby: bool,
    pub standings: Vec<StandingHud>,
    pub matches: Vec<MatchHud>,
    pub credits_pending: bool,
    pub selected_ship: u8,
    pub tournament_attempt: u32,
}

pub struct Champion {
    pub name: &'static str,
    pub species: &'static str,
    pub ship: &'static str,
    pub ability: &'static str,
    pub silhouette: &'static str,
    pub family: usize,
    pub category: usize,
    pub grid: [i32; 2],
    pub color: [u8; 3],
    pub health: f32,
}
pub const CHAMPIONS: [Champion; 8] = [
    Champion {
        name: "Lyra Voss",
        species: "Human",
        ship: "Dawnneedle",
        ability: "Stutter dash / needle burst",
        silhouette: "needle",
        family: 0,
        category: 0,
        grid: [-1, -1],
        color: [255, 195, 88],
        health: 260.0,
    },
    Champion {
        name: "Korr-9",
        species: "Automaton",
        ship: "Bulwark",
        ability: "Rotating broadside",
        silhouette: "shoulder-armor",
        family: 1,
        category: 1,
        grid: [0, -1],
        color: [255, 104, 102],
        health: 390.0,
    },
    Champion {
        name: "Sable Myrr",
        species: "Vesper",
        ship: "Nightrail",
        ability: "Serpent spiral / guided missiles",
        silhouette: "forked-serpent",
        family: 2,
        category: 2,
        grid: [1, -1],
        color: [188, 123, 255],
        health: 350.0,
    },
    Champion {
        name: "Xil of Aurora",
        species: "Crystalline",
        ship: "Prism",
        ability: "Offset fork volleys",
        silhouette: "split-wings",
        family: 0,
        category: 0,
        grid: [1, 0],
        color: [104, 230, 255],
        health: 300.0,
    },
    Champion {
        name: "Maera Quill",
        species: "Human",
        ship: "Windrunner",
        ability: "Charge / damaging ion wake",
        silhouette: "swept-wings",
        family: 0,
        category: 2,
        grid: [1, 1],
        color: [113, 244, 153],
        health: 290.0,
    },
    Champion {
        name: "Orun Vale",
        species: "Golem",
        ship: "Ironwake",
        ability: "Telegraphed ram / impact nova",
        silhouette: "ram-prow",
        family: 1,
        category: 1,
        grid: [0, 1],
        color: [255, 157, 88],
        health: 440.0,
    },
    Champion {
        name: "Vexa Null",
        species: "Voidborn",
        ship: "Voidcoil",
        ability: "Circling helix / piercing seekers",
        silhouette: "ring-fins",
        family: 2,
        category: 2,
        grid: [-1, 1],
        color: [239, 116, 228],
        health: 370.0,
    },
    Champion {
        name: "Atlas-0",
        species: "Ancient automaton",
        ship: "Sunspear",
        ability: "Sniper burst / piercing spear",
        silhouette: "spear-prow",
        family: 1,
        category: 0,
        grid: [-1, 0],
        color: [255, 233, 158],
        health: 340.0,
    },
];

pub fn arena(seed: u32, index: usize) -> Option<regions::Site> {
    let c = CHAMPIONS.get(index)?;
    regions::site_by_id(seed, &format!("{}:{}", c.grid[0], c.grid[1]))
}
pub fn tournament_site(seed: u32) -> regions::Site {
    regions::site_by_id(seed, "0:2").expect("bounded remote tournament grid")
}
fn mix(mut n: u32) -> u32 {
    n ^= n >> 16;
    n = n.wrapping_mul(0x7feb352d);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846ca68b);
    n ^ (n >> 16)
}

/// NPC-NPC matches are seeded simulation, never passed off as player combat.
pub fn table(seed: u32, p: &CampaignProgress) -> (Vec<StandingHud>, Vec<MatchHud>) {
    let mut standings: Vec<_> = (0..9)
        .map(|entrant| StandingHud {
            entrant,
            name: if entrant == 8 {
                "YOU".into()
            } else {
                CHAMPIONS[entrant].name.into()
            },
            played: 0,
            wins: 0,
            losses: 0,
            points: 0,
            tie_break: mix(seed ^ (entrant as u32).wrapping_mul(0x9e3779b9) ^ 0x726f796c),
            rank: 0,
        })
        .collect();
    let mut matches = Vec::with_capacity(36);
    for a in 0..9 {
        for b in a + 1..9 {
            let (winner, source) = if b == 8 {
                (
                    match p.royale_results[a] {
                        1 => Some(8),
                        2 => Some(a),
                        _ => None,
                    },
                    if p.royale_results[a] == 0 {
                        "pending"
                    } else {
                        "played"
                    },
                )
            } else {
                (
                    Some(
                        if mix(seed ^ (a as u32 * 7919) ^ (b as u32 * 104729) ^ 0x6e706373) & 1 == 0
                        {
                            a
                        } else {
                            b
                        },
                    ),
                    "simulated",
                )
            };
            if let Some(winner) = winner {
                standings[a].played += 1;
                standings[b].played += 1;
                standings[winner].wins += 1;
                standings[winner].points += 3;
                standings[if winner == a { b } else { a }].losses += 1;
            }
            matches.push(MatchHud {
                a,
                b,
                winner,
                source: source.into(),
            });
        }
    }
    standings.sort_by_key(|s| (std::cmp::Reverse(s.points), s.tie_break, s.entrant));
    for (i, s) in standings.iter_mut().enumerate() {
        s.rank = (i + 1) as u8;
    }
    (standings, matches)
}

#[derive(Deserialize)]
struct Atlas {
    version: u32,
    palette: Vec<u32>,
    sprites: std::collections::HashMap<String, Sprite>,
}
#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
}
static BODIES: LazyLock<[PixelBody; 9]> = LazyLock::new(|| {
    let atlas: Atlas = serde_json::from_str(include_str!("../../../assets/ui/campaign-atlas.json"))
        .expect("compiled authored campaign atlas");
    assert_eq!(atlas.version, 1);
    assert_eq!(atlas.palette.len(), 64);
    assert_eq!(atlas.palette[0], 0);
    let key = |i: usize| char::from_u32(0xe000 + i as u32).unwrap();
    let palette: Vec<_> = atlas
        .palette
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &c)| (key(i), c))
        .collect();
    std::array::from_fn(|i| {
        let sprite = &atlas.sprites[&format!("campaign_ship_{i}")];
        assert_eq!((sprite.width, sprite.height), (32, 32));
        assert_eq!(sprite.indices.len(), 1024);
        assert!(sprite.indices.iter().all(|i| (*i as usize) < 64));
        let rows: Vec<String> = sprite
            .indices
            .chunks_exact(32)
            .map(|row| {
                row.iter()
                    .map(|&i| if i == 0 { ' ' } else { key(i as usize) })
                    .collect()
            })
            .collect();
        let borrowed: Vec<_> = rows.iter().map(String::as_str).collect();
        PixelBody::from_ascii(&borrowed, &palette)
    })
});
pub fn body(index: usize) -> PixelBody {
    BODIES[index.min(8)].clone()
}
pub fn body_ref(index: usize) -> &'static PixelBody {
    &BODIES[index.min(8)]
}
pub fn player_body(id: u8) -> PixelBody {
    if id == 0 {
        pixel_assets::body("player")
    } else {
        body((id - 1) as usize)
    }
}
pub fn player_scale(id: u8) -> f32 {
    if id == 0 { 0.7 } else { 1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vec2;
    #[test]
    fn seeded_grid_arenas_and_authored_outlines_are_unique() {
        for seed in [1, 17, 0x47524f59, u32::MAX] {
            let arenas: Vec<_> = (0..8).map(|i| arena(seed, i).unwrap()).collect();
            for (i, a) in arenas.iter().enumerate() {
                assert!(Vec2::from_array(a.position).length() > 1400.0);
                for b in &arenas[i + 1..] {
                    assert_ne!(a.id, b.id);
                    assert!(
                        Vec2::from_array(a.position).distance(Vec2::from_array(b.position)) > 800.0
                    );
                }
            }
            assert_eq!(tournament_site(seed).id, "0:2");
        }
        let bodies: Vec<_> = (0..9).map(body).collect();
        for (i, a) in bodies.iter().enumerate() {
            assert_eq!((a.width, a.height), (32, 32));
            assert!(a.occupied_count() > 100);
            for b in &bodies[i + 1..] {
                assert_ne!(a.cells().collect::<Vec<_>>(), b.cells().collect::<Vec<_>>());
            }
        }
    }
    #[test]
    fn round_robin_is_complete_seeded_and_player_results_are_real() {
        let mut p = CampaignProgress::default();
        let (a, m) = table(17, &p);
        assert_eq!(m.len(), 36);
        assert_eq!(m.iter().filter(|m| m.source == "simulated").count(), 28);
        assert_eq!(m.iter().filter(|m| m.source == "pending").count(), 8);
        assert_eq!(a.iter().map(|s| s.wins as usize).sum::<usize>(), 28);
        assert_eq!(table(17, &p), (a, m));
        p.royale_results = [1; 8];
        let (s, m) = table(17, &p);
        assert_eq!(s[0].entrant, 8);
        assert_eq!(s[0].wins, 8);
        assert_eq!(s[0].points, 24);
        assert!(s.iter().all(|s| s.played == 8));
        assert_eq!(m.iter().filter(|m| m.source == "played").count(), 8);
    }
}
