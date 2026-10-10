//! Campaign encounter ownership, combat signatures, rewards, and dock inventory.
use crate::{campaign::*, *};

#[derive(Clone, Copy)]
pub(super) struct ArenaEncounter {
    pub index: usize,
    pub tournament: bool,
    pub diagnostic: bool,
    pub center: Vec2,
    pub radius: f32,
    pub opponent: Entity,
}

pub(super) struct ChampionFighter {
    pub index: usize,
    pub age: f32,
    pub timer: f32,
    pub aim: Vec2,
}
pub(super) struct Homing {
    pub turn: f32,
}

impl Game {
    pub(super) fn ship_level(&self) -> u32 {
        let selected = self.profile.campaign.selected_ship as usize;
        campaign::level(self.profile.campaign.ship_xp[selected]).max(if selected == 9 {
            12
        } else {
            1
        })
    }
    pub(super) fn track_run(&mut self, dt: f32) {
        let distance = self.position().length();
        if distance > SAFE_RADIUS {
            let run = self.run.get_or_insert_with(RunSummary::default);
            run.time += dt;
            run.distance_peak = run.distance_peak.max(distance);
        }
    }
    pub(super) fn finish_run(&mut self) {
        if let Some(run) = self.run.take() {
            self.profile.campaign.last_run = Some(run);
            self.changed();
        }
    }
    pub(super) fn bank_cargo(&mut self) {
        let amount = self.profile.campaign.cargo_materials;
        if amount > 0 {
            let space = u32::MAX - self.profile.campaign.stored_materials;
            let banked = amount.min(space);
            self.profile.campaign.stored_materials += banked;
            self.profile.campaign.cargo_materials -= banked;
            if banked > 0 {
                self.changed();
            }
        }
    }
    pub(super) fn award_xp(&mut self, amount: u32) {
        let selected = self.profile.campaign.selected_ship as usize;
        let before = self.profile.campaign.ship_xp[selected];
        self.profile.campaign.ship_xp[selected] = before.saturating_add(amount);
        let gained = self.profile.campaign.ship_xp[selected] - before;
        if gained > 0 {
            if self.position().length() > SAFE_RADIUS {
                self.run.get_or_insert_with(RunSummary::default).xp_gained = self
                    .run
                    .as_ref()
                    .map_or(0, |r| r.xp_gained)
                    .saturating_add(gained);
            }
            self.changed();
        }
    }
    pub(super) fn credit_points(&mut self, category: usize, amount: u32) {
        let before = self.profile.currency[category];
        self.profile.currency[category] = before.saturating_add(amount);
        let gained = self.profile.currency[category] - before;
        if gained > 0 {
            if self.position().length() > SAFE_RADIUS {
                let run = self.run.get_or_insert_with(RunSummary::default);
                run.points_gained[category] = run.points_gained[category].saturating_add(gained);
            }
            self.changed();
        }
    }
    pub(super) fn record_kill(&mut self, kind: Kind) {
        if self.position().length() <= SAFE_RADIUS {
            return;
        }
        let run = self.run.get_or_insert_with(RunSummary::default);
        match kind {
            Kind::Asteroid => run.asteroids = run.asteroids.saturating_add(1),
            Kind::Enemy(_) | Kind::Boss(_) => run.enemies = run.enemies.saturating_add(1),
            Kind::Player => {}
        }
    }
    pub fn start_champion(&mut self, index: usize) -> Result<(), String> {
        if self.arena.is_some() {
            return Err("Finish or retreat from the current trial first.".into());
        }
        let site = campaign::arena(self.profile.seed, index).ok_or("Unknown champion arena.")?;
        if self.position().distance(Vec2::from_array(site.position)) > 180.0 {
            return Err("Fly to this champion's arena to issue a challenge.".into());
        }
        if self.profile.campaign.patches[index] {
            return Err("This champion's patch and blueprint are already earned.".into());
        }
        self.begin_arena(index, false, false);
        Ok(())
    }

    pub fn start_tournament(&mut self) -> Result<(), String> {
        if self.arena.is_some() {
            return Err("A match is already in progress.".into());
        }
        if !self.profile.campaign.patches.iter().all(|v| *v) {
            return Err("Earn all eight champion patches to unlock Galaxy Royale.".into());
        }
        if self.profile.campaign.victory_claimed {
            return Err(
                "You are the reigning Galaxy Royale champion. Your victory record is complete."
                    .into(),
            );
        }
        let center = Vec2::from_array(tournament_site(self.profile.seed).position);
        if self.position().distance(center) > 220.0 {
            return Err("Fly to the Galaxy Royale beacon to enter the next match.".into());
        }
        if self.profile.campaign.complete() {
            self.profile.campaign.royale_results = [0; 8];
            self.profile.campaign.tournament_attempt =
                self.profile.campaign.tournament_attempt.saturating_add(1);
            self.changed();
        }
        let index = self
            .profile
            .campaign
            .royale_results
            .iter()
            .position(|v| *v == 0)
            .ok_or("League already completed.")?;
        self.begin_arena(index, true, false);
        Ok(())
    }

    pub fn retreat_arena(&mut self) -> Result<(), String> {
        if self.arena.is_none() {
            return Err("No active arena encounter.".into());
        }
        self.finish_arena(false, true);
        Ok(())
    }

    pub fn craft_ship(&mut self, index: usize) -> Result<(), String> {
        if self.position().length() > SAFE_RADIUS {
            return Err("Return to Haven dock to construct ships.".into());
        }
        let champion = CHAMPIONS.get(index).ok_or("Unknown blueprint.")?;
        if !self.profile.campaign.blueprints[index] {
            return Err("Defeat this champion to earn its blueprint.".into());
        }
        if self.profile.campaign.crafted[index] {
            return Err("This ship is already in your hangar.".into());
        }
        if self.profile.currency[champion.category] < COST {
            return Err("Construction requires 10 matching category fragments.".into());
        }
        if self.profile.campaign.stored_materials < 6 {
            return Err("Construction also requires 6 materials in Haven storage.".into());
        }
        self.profile.currency[champion.category] -= COST;
        self.profile.campaign.stored_materials -= 6;
        self.profile.campaign.crafted[index] = true;
        self.changed();
        self.event("upgrade", self.position(), 1.0);
        Ok(())
    }

    pub fn switch_ship(&mut self, id: u8) -> Result<(), String> {
        if self.position().length() > SAFE_RADIUS {
            return Err("Return to Haven dock to switch ships.".into());
        }
        if !self.profile.campaign.owned(id) {
            return Err("This ship is not in your hangar.".into());
        }
        if self.profile.campaign.selected_ship != id {
            self.profile.campaign.selected_ship = id;
            self.changed();
            self.enter_dock();
        }
        Ok(())
    }

    pub fn acknowledge_credits(&mut self) {
        if self.profile.campaign.victory_claimed && !self.profile.campaign.credits_seen {
            self.profile.campaign.credits_seen = true;
            self.changed();
        }
    }

    fn clear_arena_combat(&mut self) {
        let enemies: Vec<_> = self
            .world
            .query::<(Entity, &Brain)>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for e in enemies {
            self.remove_enemy(e, false);
        }
        let bullets: Vec<_> = self
            .world
            .query::<(Entity, &Bullet)>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for e in bullets {
            let _ = self.world.despawn(e);
        }
        self.wake.clear();
    }

    fn begin_arena(&mut self, index: usize, tournament: bool, diagnostic: bool) {
        self.stress_mode = false;
        self.clear_arena_combat();
        let site = if tournament {
            tournament_site(self.profile.seed)
        } else {
            campaign::arena(self.profile.seed, index).unwrap()
        };
        let center = Vec2::from_array(site.position);
        let champion = &CHAMPIONS[index];
        let opponent = self.spawn_enemy(champion.family, true, center + Vec2::new(115.0, 0.0), 1);
        let body = campaign::body(index);
        if let Ok(mut actor) = self.world.get::<&mut Actor>(opponent) {
            actor.health = champion.health * if tournament { 1.15 } else { 1.0 };
            actor.max_health = actor.health;
            actor.scale = 1.4;
            actor.radius = pixel_assets::radius(&body, actor.scale);
            actor.damage = 15.0 + index as f32 * 0.8;
        }
        let _ = self.world.insert_one(opponent, body);
        let _ = self.world.insert_one(
            opponent,
            ChampionFighter {
                index,
                age: 0.0,
                timer: 1.4,
                aim: Vec2::new(-1.0, 0.0),
            },
        );
        self.arena = Some(ArenaEncounter {
            index,
            tournament,
            diagnostic,
            center,
            radius: 340.0,
            opponent,
        });
        self.move_player(center - Vec2::new(115.0, 0.0));
        let stats = Stats::from_profile(&self.profile);
        if let Ok(mut actor) = self.world.get::<&mut Actor>(self.player) {
            actor.health = stats.hull;
            actor.max_health = stats.hull;
        }
        let _ = self.world.insert_one(
            self.player,
            campaign::player_body(self.profile.campaign.selected_ship),
        );
        self.shield = stats.shield;
        self.invulnerability = 0.8;
        self.fire_timer = 0.0;
        self.throttle = 0.0;
        self.destination = None;
        self.expedition = 0.0;
        self.since_damage = 0.0;
        self.event("boss", center, 1.0);
        self.purge_arena_rocks();
    }

    pub(super) fn arena_disables(&self, site: &regions::Site) -> bool {
        self.arena.as_ref().is_some_and(|arena| {
            Vec2::from_array(site.position).distance_squared(arena.center) < 1.0
        })
    }

    fn purge_arena_rocks(&mut self) {
        let Some(arena) = self.arena else {
            return;
        };
        let rocks: Vec<_> = self
            .world
            .query::<(Entity, (&Actor, &Motion))>()
            .iter()
            .filter(|(_, (a, m))| {
                a.kind == Kind::Asteroid
                    && m.position.distance(arena.center) < arena.radius + a.radius + 70.0
            })
            .map(|(e, _)| e)
            .collect();
        for rock in rocks {
            let _ = self.world.despawn(rock);
        }
    }

    pub(super) fn finish_arena(&mut self, won: bool, return_to_beacon: bool) {
        let Some(arena) = self.arena.take() else {
            return;
        };
        self.clear_arena_combat();
        if !arena.diagnostic {
            if arena.tournament {
                if self.profile.campaign.royale_results[arena.index] == 0 {
                    self.profile.campaign.royale_results[arena.index] = if won { 1 } else { 2 };
                    self.changed();
                    self.event("royale-round", arena.center, if won { 1.0 } else { 0.0 });
                    if won {
                        self.award_xp(30);
                        let run = self.run.get_or_insert_with(RunSummary::default);
                        run.champions = run.champions.saturating_add(1);
                    }
                }
                if self.profile.campaign.complete() {
                    let (standings, _) = table(self.profile.seed, &self.profile.campaign);
                    if standings[0].entrant == 8 && !self.profile.campaign.victory_claimed {
                        self.profile.campaign.victory_claimed = true;
                        self.profile.campaign.ship_xp[9] =
                            self.profile.campaign.ship_xp[9].max(1320);
                        self.changed();
                        self.event("credits", arena.center, 1.0);
                    }
                }
            } else if won && !self.profile.campaign.patches[arena.index] {
                self.profile.campaign.patches[arena.index] = true;
                self.profile.campaign.blueprints[arena.index] = true;
                let family = CHAMPIONS[arena.index].family;
                let category = CHAMPIONS[arena.index].category;
                self.credit_points(category, 3);
                self.award_xp(40);
                let run = self.run.get_or_insert_with(RunSummary::default);
                run.champions = run.champions.saturating_add(1);
                for quest in &mut self.profile.quests {
                    if quest.event == "bounty" && quest.family == family && !quest.completed {
                        quest.progress = (quest.progress + 1).min(quest.target);
                    }
                }
                self.changed();
                self.event("champion", arena.center, arena.index as f32);
                self.event("upgrade", arena.center, 1.0);
            }
        }
        if return_to_beacon {
            self.move_player(arena.center);
            self.invulnerability = 0.8;
        }
        self.shake = self.shake.max(if won { 3.0 } else { 0.6 });
    }

    pub(super) fn confine_arena(&mut self) {
        let Some(arena) = self.arena else {
            return;
        };
        if self.position().distance(arena.center) > arena.radius + 60.0 {
            self.finish_arena(false, true);
            return;
        }
        for (entity, (motion, actor, segment)) in self
            .world
            .query_mut::<(Entity, (&mut Motion, Option<&Actor>, Option<&Segment>))>()
        {
            if entity != self.player
                && entity != arena.opponent
                && !segment.is_some_and(|s| s.owner == arena.opponent)
            {
                continue;
            }
            if actor.is_none() && segment.is_none() {
                continue;
            }
            let radius = actor.map(|a| a.radius).unwrap_or(12.0);
            let limit = (arena.radius - radius - 4.0).max(1.0);
            let offset = motion.position - arena.center;
            if offset.length_squared() > limit * limit {
                let outward = offset.normalize_or_zero();
                motion.position = arena.center + outward * limit;
                motion.velocity -= outward * motion.velocity.dot(outward).max(0.0);
            }
        }
    }

    pub(super) fn update_campaign(&mut self, dt: f32) {
        self.purge_arena_rocks();
        let Some(arena) = self.arena else {
            return;
        };
        let player = self.position();
        let mut shots = Vec::new();
        let mut wake_hit = false;
        for (_, (motion, actor, champion)) in self
            .world
            .query_mut::<(Entity, (&mut Motion, &Actor, &mut ChampionFighter))>()
        {
            champion.age += dt;
            champion.timer -= dt;
            let delta = player - motion.position;
            let towards = delta.normalize_or_zero();
            let side = Vec2::new(-towards.y, towards.x);
            let phase = champion.age;
            let cycle = phase.rem_euclid(if champion.index == 5 { 4.0 } else { 3.0 });
            if cycle < 1.65 {
                champion.aim = towards;
            }
            let desired = match champion.index {
                0 => {
                    if cycle < 0.65 {
                        towards * 180.0
                    } else {
                        side * (phase * 3.0).sin() * 115.0 - towards * 12.0
                    }
                }
                1 => towards * ((delta.length() - 170.0) * 0.4).clamp(-50.0, 50.0) + side * 35.0,
                2 => towards * 52.0 + side * (phase * 2.4).sin() * 135.0,
                3 => side * (phase * 1.8).sin() * 145.0 + towards * 25.0,
                4 => {
                    if cycle > 1.8 {
                        champion.aim * 210.0
                    } else {
                        side * 80.0 - towards * 15.0
                    }
                }
                5 => {
                    if cycle > 2.3 && cycle < 3.1 {
                        champion.aim * 225.0
                    } else {
                        towards * 15.0
                    }
                }
                6 => side * 125.0 + towards * ((delta.length() - 185.0) * 0.35).clamp(-55.0, 55.0),
                _ => towards * ((delta.length() - 245.0) * 0.6).clamp(-75.0, 55.0),
            };
            motion.velocity += (desired - motion.velocity) * (1.0 - (-6.0 * dt).exp());
            motion.heading = turn_toward(
                motion.heading,
                champion.aim.y.atan2(champion.aim.x),
                PI * dt,
            );
            if champion.index == 4 && cycle > 1.8 && phase > 1.0 {
                wake_hit = wake_hit
                    || segment_circle(
                        motion.previous - forward(motion.heading) * 24.0,
                        motion.position,
                        player,
                        28.0,
                    )
                    .is_some();
            }
            if phase < 1.0 || champion.timer > 0.0 {
                continue;
            }
            let heading = towards.y.atan2(towards.x);
            let damage = actor.damage * 0.55;
            let mut shot = |angle: f32, speed: f32| {
                shots.push((
                    motion.position + forward(angle) * (actor.radius + 4.0),
                    forward(angle) * speed,
                    damage,
                ))
            };
            match champion.index {
                0 => {
                    for n in [-1, 0, 1] {
                        shot(heading + n as f32 * 0.10, 190.0);
                    }
                    champion.timer = if cycle < 0.7 { 0.2 } else { 1.2 };
                }
                1 => {
                    for n in 0..5 {
                        shot(phase * 0.7 + n as f32 * TAU / 5.0, 120.0);
                    }
                    champion.timer = 1.25;
                }
                2 => {
                    for n in 0..7 {
                        shot(phase * 1.4 + n as f32 * TAU / 7.0, 105.0);
                    }
                    champion.timer = 1.6;
                }
                3 => {
                    for n in [-1.0, 1.0] {
                        shot(heading + n * 0.34, 155.0);
                        shot(heading + n * 0.16, 115.0);
                    }
                    champion.timer = 1.1;
                }
                4 => {
                    shot(heading - 0.9, 100.0);
                    shot(heading + 0.9, 100.0);
                    champion.timer = 0.55;
                }
                5 => {
                    if cycle > 3.05 {
                        for n in 0..12 {
                            shot(n as f32 * TAU / 12.0, 140.0);
                        }
                        champion.timer = 2.0;
                    } else {
                        champion.timer = 0.2;
                    }
                }
                6 => {
                    for n in [-1.0, 0.0, 1.0] {
                        shot(heading + phase.sin() * 0.45 + n * 0.24, 145.0);
                    }
                    champion.timer = 0.9;
                }
                _ => {
                    shot(champion.aim.y.atan2(champion.aim.x), 260.0);
                    champion.timer = if phase.rem_euclid(3.0) < 0.6 {
                        0.18
                    } else {
                        1.55
                    };
                }
            }
        }
        for (p, v, d) in shots {
            self.spawn_bullet(p, v, d, false, 1);
        }
        if wake_hit {
            self.hurt_player(7.0, None);
        }
        let position = self
            .world
            .get::<&Motion>(arena.opponent)
            .ok()
            .map(|motion| motion.position);
        if let Some(p) = position {
            let color = CHAMPIONS[arena.index].color;
            if arena.index == 4 || arena.index == 5 {
                self.particle(
                    p,
                    Vec2::ZERO,
                    0.24,
                    rgba(color[0], color[1], color[2], 140),
                    1.5,
                );
            }
        }
    }

    pub(super) fn update_homing(&mut self, dt: f32) {
        if self.world.query::<&Homing>().iter().next().is_none() {
            return;
        }
        let targets: Vec<_> = self
            .world
            .query::<(&Motion, &Actor)>()
            .iter()
            .filter(|(_, a)| matches!(a.kind, Kind::Enemy(_) | Kind::Boss(_)))
            .map(|(m, _)| m.position)
            .collect();
        for (_, (motion, homing)) in self.world.query_mut::<(Entity, (&mut Motion, &Homing))>() {
            if let Some(target) = targets.iter().min_by(|a, b| {
                a.distance_squared(motion.position)
                    .total_cmp(&b.distance_squared(motion.position))
            }) {
                let delta = *target - motion.position;
                motion.heading =
                    turn_toward(motion.heading, delta.y.atan2(delta.x), homing.turn * dt);
                motion.velocity = forward(motion.heading) * motion.velocity.length();
            }
        }
    }

    fn guided_shot(&mut self, motion: Motion, damage: f32, hits: u8, turn: f32) {
        if self.world.query::<&Bullet>().iter().count() >= MAX_BULLETS {
            return;
        }
        let direction = forward(motion.heading);
        self.world.spawn((
            Motion::new(
                motion.position + direction * 20.0,
                direction * 265.0,
                motion.heading,
            ),
            Bullet {
                player: true,
                damage,
                life: 2.5,
                remaining_hits: hits,
                previous_hit: None,
            },
            Homing { turn },
        ));
    }

    pub(super) fn ship_signature(&mut self, motion: Motion, stats: Stats) {
        let ship = self.profile.campaign.selected_ship;
        let shot = self.shot_count;
        let mut fan = |offset: f32, speed: f32, damage: f32, hits: u8| {
            let dir = forward(motion.heading + offset);
            self.spawn_bullet(
                motion.position + dir * 20.0,
                dir * speed,
                stats.damage * damage,
                true,
                hits,
            );
        };
        match ship {
            1 if shot.is_multiple_of(4) => {
                fan(-0.1, 520.0, 0.65, 1);
                fan(0.1, 520.0, 0.65, 1);
            }
            2 if shot.is_multiple_of(6) => {
                fan(FRAC_PI_2, 330.0, 1.0, 1);
                fan(-FRAC_PI_2, 330.0, 1.0, 1);
            }
            3 if shot.is_multiple_of(5) => self.guided_shot(motion, stats.damage * 1.4, 1, 1.9),
            4 if shot.is_multiple_of(3) => {
                fan(-0.28, 370.0, 0.6, 1);
                fan(0.28, 370.0, 0.6, 1);
            }
            7 if shot.is_multiple_of(4) => self.guided_shot(motion, stats.damage, 3, 0.9),
            8 if shot.is_multiple_of(6) => fan(0.0, 780.0, 1.8, 5),
            9 if shot.is_multiple_of(8) => {
                for n in 0..8 {
                    fan(n as f32 * TAU / 8.0, 350.0, 0.5, 1);
                }
                if let Ok(mut actor) = self.world.get::<&mut Actor>(self.player) {
                    actor.health = (actor.health + 4.0).min(actor.max_health);
                }
            }
            _ => {}
        }
    }

    pub(super) fn campaign_hud(&self) -> CampaignHud {
        let p = &self.profile.campaign;
        let champions: Vec<_> = CHAMPIONS
            .iter()
            .enumerate()
            .map(|(index, c)| ChampionHud {
                index,
                id: format!("champion-{index}"),
                name: c.name.into(),
                species: c.species.into(),
                ship: c.ship.into(),
                ability: c.ability.into(),
                silhouette: c.silhouette.into(),
                family: c.family,
                category: c.category,
                position: campaign::arena(self.profile.seed, index).unwrap().position,
                patch: p.patches[index],
                blueprint: p.blueprints[index],
                crafted: p.crafted[index],
            })
            .collect();
        let arena = champions
            .iter()
            .filter_map(|c| {
                let d = self.position().distance(Vec2::from_array(c.position));
                (d <= 320.0).then_some((d, c))
            })
            .min_by(|(a, _), (b, _)| a.total_cmp(b))
            .map(|(_, c)| ArenaHud {
                index: c.index,
                name: c.name.clone(),
                status: if self.arena.is_some() {
                    "active"
                } else if c.patch {
                    "completed"
                } else {
                    "available"
                }
                .into(),
                can_challenge: self.arena.is_none()
                    && !c.patch
                    && self.position().distance(Vec2::from_array(c.position)) <= 180.0,
            });
        let encounter = self.arena.as_ref().and_then(|arena| {
            self.world
                .get::<&Actor>(arena.opponent)
                .ok()
                .map(|a| EncounterHud {
                    index: arena.index,
                    tournament: arena.tournament,
                    health: a.health.max(0.0),
                    max_health: a.max_health,
                })
        });
        let abilities = [
            "Needle burst every 4 shots",
            "Broadside every 6 shots; +20 hull / -8% speed",
            "Guided missile every 5 shots",
            "Fork volley every 3 shots",
            "Automatic fast-flight wake; +10% speed",
            "Damage-triggered nova every 7.5 seconds; +10 hull",
            "Piercing seeker every 4 shots",
            "Piercing spear every 6 shots",
        ];
        let mut ships = vec![ShipHud {
            id: 0,
            name: "Haven Scout".into(),
            ability: "Standard pulse drive".into(),
            owned: true,
            selected: p.selected_ship == 0,
            category: 0,
            craftable: false,
        }];
        for (i, c) in CHAMPIONS.iter().enumerate() {
            ships.push(ShipHud {
                id: (i + 1) as u8,
                name: c.ship.into(),
                ability: abilities[i].into(),
                owned: p.crafted[i],
                selected: p.selected_ship == i as u8 + 1,
                category: c.category,
                craftable: p.blueprints[i] && !p.crafted[i],
            });
        }
        ships.push(ShipHud {id:9,name:"Royale Sovereign".into(),ability:"Intrinsic rank5 offense/defense/passives; prism nova +4 repair; enhanced hull/shield/drive".into(),owned:p.victory_claimed,selected:p.selected_ship==9,category:0,craftable:false});
        let (standings, matches) = table(self.profile.seed, p);
        let tournament_position = tournament_site(self.profile.seed).position;
        CampaignHud {
            champions,
            arena,
            encounter,
            ships,
            tournament_unlocked: p.patches.iter().all(|v| *v),
            tournament_complete: p.complete(),
            tournament_won: p.victory_claimed,
            tournament_position,
            tournament_nearby: self.arena.is_none()
                && self
                    .position()
                    .distance(Vec2::from_array(tournament_position))
                    <= 220.0,
            standings,
            matches,
            credits_pending: p.victory_claimed && !p.credits_seen,
            selected_ship: p.selected_ship,
            tournament_attempt: p.tournament_attempt,
        }
    }

    pub(super) fn render_campaign(&self, frame: &mut FrameBuilder) {
        for (index, c) in CHAMPIONS.iter().enumerate() {
            let position =
                Vec2::from_array(campaign::arena(self.profile.seed, index).unwrap().position)
                    - Vec2::new(0.0, 65.0);
            if !self.visible(position, 40.0)
                || self
                    .arena
                    .is_some_and(|a| a.index == index && !a.tournament)
            {
                continue;
            }
            frame.push_body_colored_material(
                campaign::body_ref(index),
                Transform {
                    position,
                    rotation: 0.0,
                    scale: Vec2::splat(0.8),
                },
                ACTOR_MATERIAL,
                |_, color| color,
            );
            let color = rgba(c.color[0], c.color[1], c.color[2], 210);
            for n in 0..8 {
                frame.push_pixel(
                    position + forward(n as f32 * TAU / 8.0 + self.time * 0.15) * 25.0,
                    1.4,
                    color,
                );
            }
        }
        if self.profile.campaign.patches.iter().all(|v| *v) {
            let position = Vec2::from_array(tournament_site(self.profile.seed).position);
            if self.visible(position, 115.0) {
                frame.push_body_colored_material(
                    campaign::body_ref(8),
                    Transform {
                        position: position - Vec2::new(0.0, 80.0),
                        rotation: 0.0,
                        scale: Vec2::splat(1.5),
                    },
                    ACTOR_MATERIAL,
                    |_, color| color,
                );
                for n in 0..64 {
                    frame.push_pixel(
                        position + forward(n as f32 * TAU / 64.0) * 85.0,
                        1.5,
                        rgba(255, 213, 126, 180),
                    );
                }
            }
        }
        if let Some(arena) = self.arena {
            let color = CHAMPIONS[arena.index].color;
            for n in 0..160 {
                let p = arena.center + forward(n as f32 * TAU / 160.0) * arena.radius;
                if self.visible(p, 5.0) {
                    frame.push_pixel(p, 2.5, rgba(color[0], color[1], color[2], 190));
                }
            }
        }
    }

    pub(super) fn campaign_diagnostic(&mut self, command: &str) -> bool {
        if let Some(index) = command
            .strip_prefix("champion-")
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|v| *v < 8)
        {
            self.reset_to_dock();
            self.move_player(Vec2::from_array(
                campaign::arena(self.profile.seed, index).unwrap().position,
            ));
            self.begin_arena(index, false, true);
            true
        } else if command == "tournament" {
            self.reset_to_dock();
            self.move_player(Vec2::from_array(
                tournament_site(self.profile.seed).position,
            ));
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn challenge(game: &mut Game, index: usize) {
        game.move_player(Vec2::from_array(
            campaign::arena(game.profile.seed, index).unwrap().position,
        ));
        game.start_champion(index).unwrap();
    }
    fn defeat_opponent(game: &mut Game) {
        let arena = game.arena.unwrap();
        game.damage_actor(arena.opponent, 10_000.0, Vec2::ZERO, Vec2::ZERO);
    }
    fn qualified() -> Profile {
        let mut p = Profile::default();
        p.campaign.patches = [true; 8];
        p.campaign.blueprints = [true; 8];
        p
    }
    #[test]
    fn trial_uses_real_pixel_collisions_enables_only_own_sanctuary_and_contains_combat() {
        let mut g = Game::new(Profile::default());
        assert!(g.start_champion(0).is_err());
        challenge(&mut g, 0);
        let arena = g.arena.unwrap();
        assert!(!g.safe());
        g.invulnerability = 0.0;
        let health = g.hud().hull;
        g.hurt_player(10.0, None);
        assert_eq!(g.hud().hull, health - 10.0);
        let motion = *g.world.get::<&Motion>(arena.opponent).unwrap();
        let actor = *g.world.get::<&Actor>(arena.opponent).unwrap();
        let cell = g
            .world
            .get::<&PixelBody>(arena.opponent)
            .unwrap()
            .cells()
            .next()
            .unwrap()
            .0;
        let contact = actor_transform(motion, actor).point(cell);
        g.world.spawn((
            Motion {
                position: contact + Vec2::X * 75.0,
                previous: contact - Vec2::X * 75.0,
                ..Motion::new(contact, Vec2::ZERO, 0.0)
            },
            Bullet {
                player: true,
                damage: 12.0,
                life: 1.0,
                remaining_hits: 1,
                previous_hit: None,
            },
        ));
        g.resolve_collisions(1.0 / 60.0);
        assert!(g.world.get::<&Actor>(arena.opponent).unwrap().health < actor.health);
        let rockpos = arena.center + Vec2::X * 900.0;
        let rock = g.world.spawn((
            Motion::new(rockpos, Vec2::ZERO, 0.0),
            Actor {
                kind: Kind::Asteroid,
                health: 20.0,
                max_health: 20.0,
                radius: 20.0,
                scale: 1.0,
                damage: 8.0,
            },
            asteroid_body(17),
        ));
        g.confine_arena();
        assert_eq!(g.world.get::<&Motion>(rock).unwrap().position, rockpos);
        g.move_player(Vec2::ZERO);
        g.invulnerability = 0.0;
        let hp = g.hud().hull;
        g.hurt_player(50.0, None);
        assert_eq!(g.hud().hull, hp);
        g.reset_to_dock();
        assert!(g.arena.is_none());
        assert!(!g.profile.campaign.patches[0]);
        assert!(g.safe());
    }
    #[test]
    fn champion_reward_is_actual_once_only_and_survives_reload() {
        let mut g = Game::new(Profile::default());
        challenge(&mut g, 3);
        let boss_counts = g.profile.boss_kills;
        let opponent = g.arena.unwrap().opponent;
        defeat_opponent(&mut g);
        assert!(g.profile.campaign.patches[3] && g.profile.campaign.blueprints[3]);
        assert_eq!(g.profile.currency[0], 3);
        assert_eq!(g.profile.campaign.ship_xp[0], 40);
        assert_eq!(g.profile.boss_kills, boss_counts);
        g.kill_actor(opponent, Kind::Boss(0), g.position());
        assert_eq!(g.profile.currency[0], 3);
        assert!(g.start_champion(3).is_err());
        let reload = Game::new(g.profile.clone());
        assert!(reload.profile.campaign.patches[3]);
        assert_eq!(reload.profile.currency[0], 3);
        assert!(reload.arena.is_none());
        g.diagnostic("champion-4");
        defeat_opponent(&mut g);
        assert!(!g.profile.campaign.patches[4]);
        assert_eq!(g.profile.campaign.ship_xp[0], 40);
    }
    #[test]
    fn all_eight_fighters_have_distinct_live_motion_and_weapon_patterns() {
        let mut signatures = std::collections::HashSet::new();
        for index in 0..8 {
            let mut g = Game::new(Profile::default());
            challenge(&mut g, index);
            g.invulnerability = 100.0;
            let mut shots = 0;
            let mut movement = Vec2::ZERO;
            for _ in 0..240 {
                g.tick(&InputFrame::default(), 1.0 / 60.0);
                shots = shots.max(g.world.query::<&Bullet>().iter().count());
                movement += g
                    .world
                    .get::<&Motion>(g.arena.unwrap().opponent)
                    .unwrap()
                    .velocity;
            }
            assert!(shots > 0, "champion {index} must actually fire");
            assert!(
                signatures.insert((
                    shots,
                    (movement.x * 10.0) as i32,
                    (movement.y * 10.0) as i32
                )),
                "duplicate fighter pattern"
            );
        }
    }
    #[test]
    fn round_robin_records_loss_retries_and_awards_sovereign_and_credits_once() {
        let mut locked = Game::new(Profile::default());
        locked.diagnostic("tournament");
        assert!(locked.start_tournament().is_err());
        let mut g = Game::new(qualified());
        assert!(g.start_tournament().is_err());
        g.move_player(Vec2::from_array(tournament_site(g.profile.seed).position));
        g.start_tournament().unwrap();
        g.retreat_arena().unwrap();
        assert_eq!(g.profile.campaign.royale_results[0], 2);
        let saved = g.profile.clone();
        let mut reloaded = Game::new(saved);
        reloaded.move_player(Vec2::from_array(
            tournament_site(reloaded.profile.seed).position,
        ));
        reloaded.start_tournament().unwrap();
        assert_eq!(reloaded.arena.unwrap().index, 1);
        reloaded.reset_to_dock();
        assert_eq!(reloaded.profile.campaign.royale_results[1], 2);
        for i in 2..8 {
            reloaded.move_player(Vec2::from_array(
                tournament_site(reloaded.profile.seed).position,
            ));
            reloaded.start_tournament().unwrap();
            reloaded.retreat_arena().unwrap();
            assert_eq!(reloaded.profile.campaign.royale_results[i], 2);
        }
        let losing_season = reloaded.hud().campaign;
        assert!(losing_season.tournament_complete);
        assert!(!losing_season.tournament_won);
        assert!(!losing_season.credits_pending);
        reloaded.start_tournament().unwrap();
        assert_eq!(reloaded.profile.campaign.tournament_attempt, 1);
        assert_eq!(reloaded.arena.unwrap().index, 0);
        for _ in 0..8 {
            if reloaded.arena.is_none() {
                reloaded.start_tournament().unwrap();
            }
            defeat_opponent(&mut reloaded);
        }
        assert!(reloaded.profile.campaign.victory_claimed);
        assert!(reloaded.hud().campaign.tournament_complete);
        assert!(reloaded.hud().campaign.tournament_won);
        assert_eq!(reloaded.profile.campaign.royale_results, [1; 8]);
        assert!(reloaded.hud().campaign.credits_pending);
        assert_eq!(reloaded.profile.campaign.ship_xp[9], 1320);
        assert!(reloaded.start_tournament().is_err());
        let victory = Game::new(reloaded.profile.clone());
        assert!(victory.profile.campaign.owned(9));
        reloaded.acknowledge_credits();
        let revision = reloaded.profile.revision;
        reloaded.acknowledge_credits();
        assert_eq!(reloaded.profile.revision, revision);
        assert!(!reloaded.hud().campaign.credits_pending);
    }
    #[test]
    fn cargo_capacity_banking_crafting_and_ship_selection_are_atomic() {
        let mut g = Game::new(Profile::default());
        g.move_player(Vec2::new(1800.0, 0.0));
        g.track_run(1.0);
        g.spawn_loot(g.position(), 3, 20);
        g.spawn_loot(g.position(), 0, 7);
        g.update_pickups(g.position(), 1.0 / 60.0);
        assert_eq!(g.profile.campaign.cargo_materials, 16);
        assert_eq!(g.profile.currency[0], 7);
        assert_eq!(
            g.world
                .query::<&Loot>()
                .iter()
                .map(|l| l.amount)
                .sum::<u32>(),
            4
        );
        g.reset_to_dock();
        assert_eq!(g.profile.campaign.cargo_materials, 0);
        assert_eq!(g.profile.campaign.stored_materials, 16);
        assert_eq!(
            g.profile
                .campaign
                .last_run
                .as_ref()
                .unwrap()
                .materials_gained,
            16
        );
        assert_eq!(
            g.profile.campaign.last_run.as_ref().unwrap().points_gained,
            [7, 0, 0]
        );
        g.bank_cargo();
        assert_eq!(g.profile.campaign.stored_materials, 16);
        g.profile.campaign.patches[0] = true;
        g.profile.campaign.blueprints[0] = true;
        g.profile.currency[0] = 10;
        g.profile.campaign.stored_materials = 5;
        assert!(g.craft_ship(0).is_err());
        assert_eq!(g.profile.currency[0], 10);
        assert_eq!(g.profile.campaign.stored_materials, 5);
        g.profile.campaign.stored_materials = 6;
        g.craft_ship(0).unwrap();
        assert_eq!(g.profile.currency[0], 0);
        assert_eq!(g.profile.campaign.stored_materials, 0);
        assert!(g.craft_ship(0).is_err());
        g.switch_ship(1).unwrap();
        assert_eq!(g.world.get::<&PixelBody>(g.player).unwrap().width, 32);
        assert!(g.switch_ship(9).is_err());
    }
    #[test]
    fn real_kills_level_selected_ship_and_gate_global_hardware_purchases() {
        let mut g = Game::new(Profile::default());
        let initial = Stats::from_profile(&g.profile);
        assert!((initial.speed - 100.0).abs() < 0.001);
        assert!((initial.acceleration - 260.0).abs() < 0.001);
        g.profile.currency[0] = 100;
        g.purchase("damage").unwrap();
        assert!(g.purchase("damage").is_err());
        g.move_player(Vec2::new(1800.0, 0.0));
        for _ in 0..12 {
            let e = g.spawn_enemy(0, false, g.position() + Vec2::X * 40.0, 1);
            g.damage_actor(e, 1000.0, Vec2::ZERO, Vec2::ZERO);
        }
        assert_eq!(g.profile.campaign.ship_xp[0], 60);
        assert_eq!(g.ship_level(), 3);
        let level_three = Stats::from_profile(&g.profile);
        assert!((level_three.speed - 105.0).abs() < 0.001);
        assert!((level_three.acceleration - 273.0).abs() < 0.001);
        g.reset_to_dock();
        g.purchase("damage").unwrap();
        assert_eq!(g.profile.ranks[0], 2);
        assert_eq!(g.profile.campaign.last_run.as_ref().unwrap().enemies, 12);
        assert_eq!(g.profile.campaign.last_run.as_ref().unwrap().xp_gained, 60);
        let mut profile = g.profile.clone();
        profile.campaign.patches[0] = true;
        profile.campaign.blueprints[0] = true;
        profile.campaign.crafted[0] = true;
        let mut g = Game::new(profile);
        g.switch_ship(1).unwrap();
        assert_eq!(g.ship_level(), 1);
        assert_eq!(g.profile.ranks[0], 2);
        assert!(Stats::from_profile(&g.profile).damage > 12.0);
        assert!(g.purchase("damage").is_err());
    }
    #[test]
    fn death_retains_banked_points_xp_materials_and_preserves_summary_without_reload_popup() {
        let mut g = Game::new(Profile::default());
        g.move_player(Vec2::new(1800.0, 0.0));
        g.track_run(3.0);
        g.spawn_loot(g.position(), 2, 5);
        g.spawn_loot(g.position(), 3, 2);
        g.update_pickups(g.position(), 0.016);
        let e = g.spawn_enemy(1, false, g.position() + Vec2::X * 35.0, 1);
        g.damage_actor(e, 1000.0, Vec2::ZERO, Vec2::ZERO);
        g.world.get::<&mut Actor>(g.player).unwrap().health = 0.0;
        g.tick(&InputFrame::default(), 1.0 / 60.0);
        assert_eq!(g.position(), Vec2::ZERO);
        assert_eq!(g.profile.currency[2], 5);
        assert_eq!(g.profile.campaign.stored_materials, 2);
        assert_eq!(g.profile.campaign.ship_xp[0], 5);
        let summary = g.profile.campaign.last_run.clone().unwrap();
        assert!(summary.time >= 3.0);
        assert!(summary.distance_peak >= 1800.0);
        assert_eq!(summary.enemies, 1);
        let mut reload = Game::new(g.profile.clone());
        assert_eq!(reload.hud().last_run, Some(summary));
        assert!(reload.take_events().iter().all(|e| e.kind != "death"));
    }
    #[test]
    fn sovereign_has_intrinsic_maximum_upgrades_without_changing_global_progress() {
        let mut p = qualified();
        p.campaign.royale_results = [1; 8];
        p.campaign.victory_claimed = true;
        p.campaign.selected_ship = 9;
        let mut g = Game::new(p);
        assert_eq!(g.profile.ranks, [0; 9]);
        assert_eq!(g.profile.unlocked, [false; 3]);
        assert_eq!(g.ship_level(), 12);
        let stats = Stats::from_profile(&g.profile);
        assert!((stats.hull - 240.0).abs() < 0.001);
        assert!((stats.shield - 100.0).abs() < 0.001);
        assert!((stats.damage - 24.0).abs() < 0.001);
        assert!(g.hud().upgrades.iter().all(|u| u.rank == 5 && u.unlocked));
        assert_eq!(campaign::signature_rank(&g.profile, 6), 5);
        g.move_player(Vec2::new(1800.0, 0.0));
        for _ in 0..8 {
            g.fire(stats);
        }
        assert!(
            g.world
                .query::<&Bullet>()
                .iter()
                .any(|b| b.remaining_hits > 1)
        );
        g.invulnerability = 0.0;
        g.hurt_player(1.0, None);
        assert!(g.repulsor_timer > 0.0);
        g.world.get::<&mut Motion>(g.player).unwrap().velocity = Vec2::X * 200.0;
        g.update_wake(g.position(), 0.2);
        assert!(!g.wake.is_empty());
        assert_eq!(g.profile.ranks, [0; 9]);
    }
    #[test]
    fn invalid_campaign_rewards_and_archives_cannot_own_ships() {
        let mut p = qualified();
        p.campaign.royale_results = [2; 8];
        p.campaign.victory_claimed = true;
        p.campaign.credits_seen = true;
        p.campaign.selected_ship = 9;
        p.campaign.cargo_materials = 999;
        let g = Game::new(p);
        assert!(!g.profile.campaign.victory_claimed);
        assert!(!g.profile.campaign.credits_seen);
        assert_eq!(g.profile.campaign.selected_ship, 0);
        assert_eq!(g.profile.campaign.cargo_materials, 0);
        assert_eq!(g.profile.campaign.stored_materials, 16);
        let legacy: Profile = serde_json::from_str(
            r#"{"schema_version":2,"currency":[7,8,9],"ranks":[5,4,3,2,1,0,0,0,0]}"#,
        )
        .unwrap();
        let g = Game::new(legacy);
        assert_eq!(g.profile.schema_version, 3);
        assert_eq!(g.profile.currency, [7, 8, 9]);
        assert_eq!(g.profile.ranks[..6], [5, 4, 3, 2, 1, 0]);
    }
}
