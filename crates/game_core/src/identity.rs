//! Persistent Kikai identity, explicit fresh registration and real onboarding facts.
//! Existing hull0 remains the starter's XP/loadout history; legacy pilots keep
//! their original Scout and inventory. No constructor repeats a starter grant.
use crate::*;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum KikaiKind {
    Dora,
    Doro,
    Talo,
}
impl KikaiKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Dora => "DORA",
            Self::Doro => "DORO",
            Self::Talo => "TALO",
        }
    }
    /// Display-only callsign prefix; persisted kind and full designation stay intact.
    pub fn callsign_prefix(self) -> char {
        match self {
            Self::Dora => 'A',
            Self::Doro => 'O',
            Self::Talo => 'T',
        }
    }
    pub fn gender(self) -> &'static str {
        match self {
            Self::Dora => "Female",
            Self::Doro => "Male",
            Self::Talo => "Neutral",
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Corporation {
    Pan,
    Renko,
    Hugo,
}
impl Corporation {
    pub fn name(self) -> &'static str {
        match self {
            Self::Pan => "PAN",
            Self::Renko => "RENKO",
            Self::Hugo => "HUGO",
        }
    }
    pub fn role(self) -> &'static str {
        match self {
            Self::Pan => "Passive",
            Self::Renko => "Offensive",
            Self::Hugo => "Defensive",
        }
    }
    pub fn starter_name(self) -> &'static str {
        match self {
            Self::Pan => "PAN HIKARI",
            Self::Renko => "RENKO HIBANA",
            Self::Hugo => "HUGO MAMORI",
        }
    }
    pub fn ability(self) -> u8 {
        match self {
            Self::Pan => 2,
            Self::Renko => 0,
            Self::Hugo => 1,
        }
    }
    pub fn color(self) -> [u8; 3] {
        match self {
            Self::Pan => [109, 225, 133],
            Self::Renko => [244, 105, 108],
            Self::Hugo => [104, 180, 244],
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct KikaiIdentity {
    pub kind: KikaiKind,
    pub corporation: Corporation,
    pub sub_corporation: Corporation,
    pub hex_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct OnboardingProgress {
    #[serde(default = "legacy_onboarding_version")]
    pub version: u8,
    pub stage: u8,
    pub flags: u8,
    pub asteroids_destroyed: u8,
    pub materials_collected: u8,
    pub legacy_complete: bool,
}
fn legacy_onboarding_version() -> u8 {
    1
}
impl Default for OnboardingProgress {
    fn default() -> Self {
        Self {
            version: 2,
            stage: 0,
            flags: 0,
            asteroids_destroyed: 0,
            materials_collected: 0,
            legacy_complete: false,
        }
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct IdentityHud {
    pub kind: KikaiKind,
    pub kind_name: String,
    pub gender: String,
    pub corporation: Corporation,
    pub corporation_role: String,
    pub sub_corporation: Corporation,
    pub sub_corporation_name: String,
    pub hex_id: String,
    pub designation: String,
    pub short_designation: String,
    pub starter_ship: String,
    pub starter_ability: String,
    pub license: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct OnboardingHud {
    pub stage: u8,
    pub flags: u8,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub version: u8,
    pub asteroids_destroyed: u8,
    pub materials_collected: u8,
    pub target: u8,
    pub material_id: String,
    pub material_name: String,
    pub asteroid_name: String,
    pub starter_id: String,
    pub starter_name: String,
    pub starter_slot: String,
    pub carried_matching: u32,
    pub banked_matching: u32,
    pub can_upgrade: bool,
    pub upgrade_reason: String,
}
pub(super) const LICENSE: u8 = 1;
pub(super) const DEPARTED: u8 = 2;
pub(super) const BELT_DESTROYED: u8 = 4;
pub(super) const T1_COLLECTED: u8 = 8;
pub(super) const T1_BANKED: u8 = 16;
pub(super) const ADDITIONAL_CRAFTED: u8 = 32;
pub(super) const ADDITIONAL_EQUIPPED: u8 = 64;

pub fn stage(flags: u8) -> u8 {
    for (index, mask) in [
        LICENSE,
        DEPARTED,
        BELT_DESTROYED,
        T1_COLLECTED,
        T1_BANKED,
        ADDITIONAL_CRAFTED | ADDITIONAL_EQUIPPED,
    ]
    .into_iter()
    .enumerate()
    {
        if flags & mask != mask {
            return index as u8;
        }
    }
    6
}
fn canonical_hex(value: &str) -> Option<String> {
    (value.len() == 16 && value.bytes().all(|b| b.is_ascii_hexdigit()))
        .then(|| value.to_ascii_lowercase())
}
/// The shell supplies8 cryptographically random bytes, encoded as16 hex digits.
/// Hashing that identifier only derives a fresh universe seed; it is not an RNG.
pub fn create_profile(
    kind: &str,
    corporation: &str,
    sub_corporation: &str,
    hex_id: &str,
) -> Result<Profile, String> {
    let kind = match kind {
        "dora" => KikaiKind::Dora,
        "doro" => KikaiKind::Doro,
        "talo" => KikaiKind::Talo,
        _ => return Err("Choose DORA, DORO or TALO.".into()),
    };
    let corporation = match corporation {
        "pan" => Corporation::Pan,
        "renko" => Corporation::Renko,
        "hugo" => Corporation::Hugo,
        _ => return Err("Choose PAN, RENKO or HUGO.".into()),
    };
    let sub_corporation = match sub_corporation {
        "pan" => Corporation::Pan,
        "renko" => Corporation::Renko,
        "hugo" => Corporation::Hugo,
        _ => return Err("Choose a PAN, RENKO or HUGO sub-manufacturer.".into()),
    };
    let hex_id = canonical_hex(hex_id).ok_or("Identity requires exactly 16 hexadecimal digits.")?;
    let mut seed = 0x811c9dc5u32;
    for byte in hex_id.bytes() {
        seed = (seed ^ byte as u32).wrapping_mul(0x01000193);
    }
    let mut profile = Profile {
        seed: seed.max(1),
        identity: Some(KikaiIdentity {
            kind,
            corporation,
            sub_corporation,
            hex_id,
        }),
        onboarding: Some(OnboardingProgress::default()),
        ..Profile::default()
    };
    for state in &mut profile.campaign.abilities[..3] {
        *state = campaign::AbilityState::default();
    }
    let id = corporation.ability();
    profile.campaign.abilities[id as usize] = campaign::AbilityState {
        blueprint: true,
        rank: 1,
    };
    profile.campaign.loadouts[0].slots[abilities::get(id).unwrap().category] = Some(id);
    Ok(profile)
}
pub(super) fn sanitize(profile: &mut Profile) -> bool {
    let before = (profile.identity.clone(), profile.onboarding.clone());
    if profile.schema_version < 5 {
        profile.identity = None;
        profile.onboarding = None;
    } else if let Some(identity) = &mut profile.identity {
        if let Some(hex) = canonical_hex(&identity.hex_id) {
            identity.hex_id = hex;
        } else {
            profile.identity = None;
            profile.onboarding = None;
        }
    }
    if profile.identity.is_none() {
        profile.onboarding = None;
    }
    crate::training::sanitize_progress(profile);
    (profile.identity.clone(), profile.onboarding.clone()) != before
}
pub fn hud(profile: &Profile) -> Option<IdentityHud> {
    let identity = profile.identity.as_ref()?;
    let corp = identity.corporation;
    let designation = format!(
        "{}-{}-{}-{}",
        identity.kind.name(),
        corp.name(),
        identity.sub_corporation.name(),
        identity.hex_id.to_ascii_uppercase()
    );
    let short_designation = format!(
        "{}{}{}#{}",
        identity.kind.callsign_prefix(),
        &corp.name()[..1],
        &identity.sub_corporation.name()[..1],
        identity.hex_id.to_ascii_uppercase()
    );
    Some(IdentityHud {
        kind: identity.kind,
        kind_name: identity.kind.name().into(),
        gender: identity.kind.gender().into(),
        corporation: corp,
        corporation_role: corp.role().into(),
        sub_corporation: identity.sub_corporation,
        sub_corporation_name: identity.sub_corporation.name().into(),
        hex_id: identity.hex_id.clone(),
        designation,
        short_designation,
        starter_ship: corp.starter_name().into(),
        starter_ability: abilities::get(corp.ability()).unwrap().name.into(),
        license: format!(
            "{} MEISHI CARD: {} systems. Your {} RAICHO includes an equipped rank 1 {}. Keep collected points; bank carried materials physically at Haven before death or abandonment.",
            corp.name(),
            corp.role().to_lowercase(),
            corp.starter_name(),
            abilities::get(corp.ability()).unwrap().name
        ),
    })
}
pub fn onboarding_hud(game: &Game) -> Option<OnboardingHud> {
    let profile = &game.profile;
    let p = profile.onboarding.as_ref()?;
    let corp = profile.identity.as_ref()?.corporation;
    let material = crate::training::material_for_corporation(corp);
    let def = abilities::get(corp.ability()).unwrap();
    let carried = profile.campaign.cargo_training_materials[material.index];
    let banked = profile.campaign.stored_training_materials[material.index];
    let slot = ["offense", "defense", "passive"][def.category];
    let stage = p.stage.min(6);
    let title = [
        "Accept your MEISHI CARD",
        "Leave Haven",
        "Destroy 10 asteroids",
        "Collect 10 matching materials",
        "Bank at Haven",
        "Upgrade your equipped starter",
        "Flight training complete",
    ][stage as usize];
    let description = match stage {
        0 => "Review your corporation's starter RAICHO and equipment, then acknowledge your MEISHI CARD.".into(),
        1 => "Fly beyond Haven's protected 240-unit radius. The dock shield keeps asteroids outside.".into(),
        2 => format!(
            "Asteroids destroyed: {}/10. Destroy real asteroids of any variety. Haven's tutorial belt awards no XP for your RAICHO; outer asteroids retain their normal rewards.",
            p.asteroids_destroyed
        ),
        3 => format!(
            "Collected {}/10 {}. Assigned source: {}. Collect matching tier 1 drops after the asteroid mission. Materials share cargo space and are lost on death or abandonment.",
            p.materials_collected, material.name, material.asteroid
        ),
        4 => format!(
            "Carried: {carried} {}; banked: {banked}/10. Return physically to Haven to bank 10. Collected progress is {}/10; replace any material lost or spent. Reload never banks cargo.",
            material.name, p.materials_collected
        ),
        5 if profile.campaign.abilities[corp.ability() as usize].rank >= 2 => format!(
            "At Haven, equip {} in the {slot} slot on your {} RAICHO. Banked: {banked}/10 {}. Your already-owned upgrade is recognized without another material cost.",
            def.name, corp.starter_name(), material.name
        ),
        5 => format!(
            "At Haven, equip {} in the {slot} slot on your {} RAICHO. Banked: {banked}/10 {}. The first upgrade costs 10 {} and no points, and works at RAICHO level 1.",
            def.name, corp.starter_name(), material.name, material.name
        ),
        _ => "Your corporation's starter RAICHO is ready for the outer reaches. Each RAICHO earns its own XP; discovered blueprints and dock inventory persist.".into(),
    };
    Some(OnboardingHud {
        stage: p.stage,
        flags: p.flags,
        title: title.into(),
        description,
        completed: p.stage == 6,
        version: p.version,
        asteroids_destroyed: p.asteroids_destroyed,
        materials_collected: p.materials_collected,
        target: 10,
        material_id: material.id.into(),
        material_name: material.name.into(),
        asteroid_name: material.asteroid.into(),
        starter_id: def.key.into(),
        starter_name: def.name.into(),
        starter_slot: slot.into(),
        carried_matching: carried,
        banked_matching: banked,
        can_upgrade: game.training_upgrade_check(corp.ability()).is_ok(),
        upgrade_reason: game
            .training_upgrade_check(corp.ability())
            .err()
            .unwrap_or_default(),
    })
}

// Original final-resolution48×48 cells. Corporate frames have different
// occupancy/silhouettes as well as palette; they are never raster textures.
static STARTERS: LazyLock<[PixelBody; 3]> = LazyLock::new(|| {
    std::array::from_fn(|variant| {
        let corp = [Corporation::Pan, Corporation::Renko, Corporation::Hugo][variant];
        let accent = corp.color();
        let mut mask = [[false; 48]; 48];
        for (y, row) in mask.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                let dx = (x as f32 - 23.5).abs();
                *cell = match variant {
                    0 => {
                        (3..=37).contains(&y)
                            && dx
                                <= if y < 18 {
                                    2.0 + (y as f32 - 3.0) * 0.25
                                } else {
                                    9.5 - (y as f32 - 25.0).abs() * 0.30
                                }
                            || (24..=40).contains(&y) && (9.0..=14.0).contains(&dx)
                            || (29..=32).contains(&y) && dx <= 15.0
                            || (38..=44).contains(&y) && (3.0..=6.0).contains(&dx)
                    }
                    1 => {
                        (2..=43).contains(&y)
                            && dx
                                <= if y < 25 {
                                    1.5 + y as f32 * 0.17
                                } else {
                                    6.5 - (y as f32 - 25.0) * 0.20
                                }
                            || (18..=37).contains(&y)
                                && dx > 7.0
                                && dx <= 16.0 - (y as f32 - 27.0).abs() * 0.65
                            || (12..=32).contains(&y) && (11.0..=13.0).contains(&dx)
                            || (28..=30).contains(&y) && dx <= 16.0
                            || (36..=43).contains(&y) && (3.0..=7.0).contains(&dx)
                    }
                    _ => {
                        (5..=39).contains(&y)
                            && dx
                                <= if y < 16 {
                                    3.5 + (y as f32 - 5.0) * 0.5
                                } else {
                                    12.5
                                }
                            || (16..=36).contains(&y) && (14.0..=19.0).contains(&dx)
                            || (22..=27).contains(&y) && dx <= 20.5
                            || (38..=43).contains(&y) && (5.0..=9.0).contains(&dx)
                    }
                };
            }
        }
        let mut rows = Vec::with_capacity(48);
        for (y, row) in mask.iter().enumerate() {
            let mut result = String::with_capacity(48);
            for (x, &occupied) in row.iter().enumerate() {
                let dx = (x as f32 - 23.5).abs();
                let at = |xx: isize, yy: isize| {
                    xx >= 0 && yy >= 0 && xx < 48 && yy < 48 && mask[yy as usize][xx as usize]
                };
                let color = if !occupied {
                    ' '
                } else if (39..=41).contains(&y)
                    && dx > 3.0
                    && dx < 9.0
                    && at(x as isize - 1, y as isize)
                    && at(x as isize + 1, y as isize)
                {
                    'e'
                } else if !at(x as isize, y as isize - 1) || !at(x as isize - 1, y as isize) {
                    'h'
                } else if !at(x as isize, y as isize + 1) || !at(x as isize + 1, y as isize) {
                    's'
                } else if (13..=20).contains(&y) && dx < 2.5 {
                    // Recessed cockpit frame, dark glass and a one-cell glare.
                    if y == 13 || y == 20 {
                        'v'
                    } else if x == 22 && y < 18 {
                        'w'
                    } else if y < 17 {
                        'g'
                    } else {
                        'q'
                    }
                } else if (39..=42).contains(&y) && dx > 3.0 && dx < 9.5 {
                    'e'
                } else if (y == 23 || y == 31 || y == 35) && [3.5, 7.5, 11.5, 17.5].contains(&dx) {
                    'b'
                } else if variant == 1 && dx > 10.0 && y < 26 {
                    'm' // Lancer's two forward metallic weapon rails.
                } else if (25..=30).contains(&y) && dx < 1.5 {
                    if y == 25 || y == 30 { 'v' } else { 'r' } // Flush reactor window.
                } else if (21..=35).contains(&y) && dx < 1.0 {
                    'w' // Raised longitudinal spine.
                } else if (y == 24 || y == 32) && dx > 4.0 {
                    'v'
                } else if (27..=35).contains(&y) && (3.5..=6.5).contains(&dx) {
                    if y.is_multiple_of(2) { 'v' } else { 'm' } // Paired cooling vents.
                } else if dx > if variant == 2 { 13.0 } else { 9.0 } {
                    // Outer nacelles: narrow corporate strips on plated roofs.
                    if (variant == 2 && dx >= 17.0) || (variant != 2 && dx >= 12.0) {
                        'a'
                    } else if (y / 3).is_multiple_of(2) {
                        'm'
                    } else {
                        'w'
                    }
                } else if y < 12 || (2.5..=3.5).contains(&dx) && y > 20 {
                    'a'
                } else if y > 20 && dx > 4.5 {
                    if y < 29 { 'm' } else { 'd' }
                } else if x < 24 {
                    'm'
                } else {
                    'd'
                };
                result.push(color);
            }
            rows.push(result);
        }
        let borrowed: Vec<_> = rows.iter().map(String::as_str).collect();
        PixelBody::from_ascii(
            &borrowed,
            &[
                ('h', rgba(219, 237, 227, 255)),
                ('s', rgba(57, 77, 91, 255)),
                ('a', rgba(accent[0], accent[1], accent[2], 255)),
                ('d', rgba(accent[0] / 2, accent[1] / 2, accent[2] / 2, 255)),
                ('v', rgba(23, 33, 44, 255)),
                ('g', rgba(121, 222, 225, 255)),
                ('q', rgba(39, 100, 127, 255)),
                ('m', rgba(83, 107, 119, 255)),
                ('w', rgba(152, 175, 180, 255)),
                ('r', rgba(230, 225, 176, 255)),
                ('e', rgba(238, 218, 139, 255)),
                ('b', rgba(168, 177, 151, 255)),
            ],
        )
    })
});
pub fn starter_body(corporation: Corporation) -> &'static PixelBody {
    &STARTERS[match corporation {
        Corporation::Pan => 0,
        Corporation::Renko => 1,
        Corporation::Hugo => 2,
    }]
}
pub(super) fn player_body(profile: &Profile) -> PixelBody {
    if profile.campaign.selected_ship == 0
        && let Some(identity) = &profile.identity
    {
        starter_body(identity.corporation).clone()
    } else {
        campaign::player_body(profile.campaign.selected_ship)
    }
}

pub(super) fn main_corporation(profile: &Profile) -> Option<Corporation> {
    profile.identity.as_ref().map(|id| id.corporation)
}
pub(super) const EMITTER: u32 = u32::from_le_bytes([238, 218, 139, 255]);
pub(super) fn exhaust_color(profile: &Profile, alpha: u8) -> u32 {
    let rgb = if profile.campaign.selected_ship == 0 {
        main_corporation(profile).map(Corporation::color)
    } else {
        None
    };
    let [r, g, b] = rgb.unwrap_or([90, 230, 255]);
    rgba(r, g, b, alpha)
}
pub(super) fn exhaust_light(profile: &Profile) -> [f32; 3] {
    if profile.campaign.selected_ship == 0
        && let Some(corp) = main_corporation(profile)
    {
        corp.color().map(|v| v as f32 / 255.0)
    } else {
        [0.2, 0.78, 1.0]
    }
}
pub(super) fn jet_glow_color(profile: &Profile, alpha: u8) -> u32 {
    if profile.campaign.selected_ship == 0 && profile.identity.is_some() {
        exhaust_color(profile, alpha)
    } else {
        rgba(120, 225, 255, alpha)
    }
}
/// Select only authored signal/trim colors; steel, glass and fasteners survive.
pub(super) fn dock_color(profile: &Profile, color: u32) -> u32 {
    let Some(corp) = main_corporation(profile) else {
        return color;
    };
    let rgb = color.to_le_bytes();
    let signal = color == pixel_assets::color(6) || color == pixel_assets::color(4);
    let trim = rgb[..3] == [81, 166, 184] || rgb[..3] == [102, 186, 196];
    if signal || trim {
        let c = corp.color();
        let factor = if signal { 1.0 } else { 0.72 };
        rgba(
            (c[0] as f32 * factor) as u8,
            (c[1] as f32 * factor) as u8,
            (c[2] as f32 * factor) as u8,
            rgb[3],
        )
    } else {
        color
    }
}
static CRESTS: LazyLock<[PixelBody; 3]> = LazyLock::new(|| {
    let art = [
        [
            "       xx       ",
            "      xhhx      ",
            "     xhaahx     ",
            "    xhaaaahx    ",
            "    haa  aah    ",
            "   xhaa  aahx   ",
            "   ha   b  ah   ",
            "  xha bbb  ahx  ",
            "  xhaa  b aahx  ",
            "   xhaa  aahx   ",
            "    xhaaahx     ",
            "     xhahx      ",
            "      xhx       ",
            "       x        ",
        ],
        [
            "      xx       ",
            "      hh       ",
            "     xhhx      ",
            "     haah      ",
            "  xx haah xx   ",
            "  hh habh hh   ",
            " xahxhabhxahx  ",
            " haaahbbhaaah  ",
            "  haaahhaahh   ",
            "   haahhah     ",
            "   xhahhax     ",
            "    hh  hh     ",
            "    xx  xx     ",
            "               ",
        ],
        [
            "    xxxxxx     ",
            "   xhhhhhhex   ",
            "  xhaaaaaahx   ",
            " xhaa    aahx  ",
            " haa  bb  aah  ",
            " haa bbbb aah  ",
            " haa  bb  aah  ",
            " xhaa bb aahx  ",
            "  xhaa  aahx   ",
            "   xhaa aah    ",
            "    xhaaah     ",
            "     xhhh      ",
            "      xx       ",
            "               ",
        ],
    ];
    std::array::from_fn(|i| {
        let c = [Corporation::Pan, Corporation::Renko, Corporation::Hugo][i].color();
        PixelBody::from_ascii(
            &art[i],
            &[
                ('x', rgba(23, 33, 44, 255)),
                ('h', rgba(152, 177, 175, 255)),
                ('a', rgba(c[0], c[1], c[2], 255)),
                ('b', rgba(225, 230, 205, 255)),
                ('e', rgba(c[0] / 2, c[1] / 2, c[2] / 2, 255)),
            ],
        )
    })
});
pub(super) fn render_crest(
    frame: &mut FrameBuilder,
    profile: &Profile,
    camera: Vec2,
    viewport: Vec2,
    time: f32,
) {
    let Some(corp) = main_corporation(profile) else {
        return;
    };
    let position = Vec2::new(0.0, -64.0);
    if ((position - camera).abs() - viewport * 0.5).max_element() > 12.0 {
        return;
    }
    let index = match corp {
        Corporation::Pan => 0,
        Corporation::Renko => 1,
        Corporation::Hugo => 2,
    };
    let accent = corp.color();
    let lit = rgba(accent[0], accent[1], accent[2], 255);
    frame.push_body_colored_material(
        &CRESTS[index],
        Transform {
            position,
            scale: Vec2::splat(1.0),
            ..Default::default()
        },
        ACTOR_MATERIAL,
        |_, color| {
            if color == lit {
                pulse_color(color, 0.94 + (time * 1.4).sin() * 0.06)
            } else {
                color
            }
        },
    );
}

impl Game {
    pub fn can_register_droid(&self) -> bool {
        let p = &self.profile;
        let c = &p.campaign;
        p.identity.is_none()
            && p.onboarding.is_none()
            && p.currency == [0; 3]
            && p.ranks == [0; 9]
            && p.unlocked == [false; 3]
            && p.kills == [0; 3]
            && p.boss_kills == [0; 3]
            && p.boss_generations == [[0; 5]; 3]
            && p.pity == [0; 3]
            && p.pending == [false; 3]
            && p.discoveries.is_empty()
            && p.quests.is_empty()
            && p.relics.is_empty()
            && c.patches == [false; 8]
            && c.blueprints == [false; 8]
            && c.crafted == [false; 8]
            && c.selected_ship == 0
            && c.royale_results == [0; 8]
            && c.tournament_attempt == 0
            && !c.victory_claimed
            && !c.credits_seen
            && c.ship_xp == [0; 10]
            && c.cargo_materials == [0; 5]
            && c.stored_materials == [0; 5]
            && c.components == [0; crate::components::COUNT]
            && c.last_run.is_none()
            && c.loadouts == [campaign::ShipLoadout::default(); 10]
            && c.abilities
                .iter()
                .enumerate()
                .all(|(i, a)| a.rank == 0 && a.blueprint == (i < 3))
            && self.arena.is_none()
            && self.run.is_none()
            && self.expedition == 0.0
            && self.position().length() <= SAFE_RADIUS
            && self.progression_enabled()
            && self.material_loss == [0; 5]
    }
    pub fn acknowledge_license(&mut self) -> Result<(), String> {
        if self.profile.identity.is_none() || self.profile.onboarding.is_none() {
            return Err("This pilot has no new MEISHI training mission.".into());
        }
        self.onboarding_fact(LICENSE);
        Ok(())
    }
    pub(super) fn onboarding_fact(&mut self, flag: u8) {
        if !self.progression_enabled() {
            return;
        }
        // The counted v2 objectives are recorded only by their actual event paths.
        if flag & !(LICENSE | DEPARTED) != 0 {
            return;
        }
        if let Some(progress) = &mut self.profile.onboarding
            && progress.flags & flag != flag
        {
            progress.flags |= flag;
            progress.stage = stage(progress.flags);
            self.changed();
        }
    }
    pub(super) fn discover_basic(&mut self, category: usize) {
        if self.profile.identity.is_none() || category >= 3 || !self.progression_enabled() {
            return;
        }
        if let Some((index, _)) = abilities::ABILITIES[..3]
            .iter()
            .enumerate()
            .find(|(_, def)| def.category == category)
            && !self.profile.campaign.abilities[index].blueprint
        {
            self.profile.campaign.abilities[index].blueprint = true;
            self.changed();
        }
    }
    pub(super) fn additional_basic(&self, id: u8) -> bool {
        id < 3
            && self
                .profile
                .identity
                .as_ref()
                .is_some_and(|identity| identity.corporation.ability() != id)
    }
}

#[cfg(test)]
#[path = "identity_tests.rs"]
mod tests;
