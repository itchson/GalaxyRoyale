//! Bounded seeded asteroid admission. Co-op interest order is spatial, not roster order.
use super::*;

const MAX_ASTEROIDS: usize = 112;
const CANDIDATES: u8 = 12;
const MAX_INTEREST_CHUNKS: usize = co_op::MAX_PARTICIPANTS * 9;

enum Candidate {
    Skipped,
    Spawned(Entity),
    Full,
}

impl Game {
    fn asteroid_count(&self) -> usize {
        self.world
            .query::<&Actor>()
            .iter()
            .filter(|a| a.kind == Kind::Asteroid)
            .count()
    }

    fn chunk_candidate(&mut self, key: (i32, i32), index: u8, count: usize) -> Candidate {
        let hash = hash3(self.profile.seed, key.0, key.1, index as u32);
        let p = Vec2::new(
            key.0 as f32 * CHUNK + unit(hash) * CHUNK,
            key.1 as f32 * CHUNK + unit(mix(hash)) * CHUNK,
        );
        if p.length() < SAFE_RADIUS + 100.0
            || self
                .arena
                .as_ref()
                .is_some_and(|arena| p.distance(arena.center) < arena.radius + 70.0)
            || regions::protected(self.profile.seed, p).is_some()
        {
            return Candidate::Skipped;
        }
        let mut birth = AsteroidBirth::at(self.profile.seed, p);
        birth.variety = pixel_assets::asteroid_variety(hash);
        if birth.source == AsteroidSource::Outer && index > 2 {
            return Candidate::Skipped;
        }
        if count >= MAX_ASTEROIDS {
            return Candidate::Full;
        }
        let belt = birth.source != AsteroidSource::Outer;
        let size = asteroid_art::size(hash, birth.material_tier, belt);
        let scale = size * asteroid_art::CELL_SCALE;
        let body = pixel_assets::asteroid_instance(hash, birth.material_tier);
        if let Some(orbit) = &mut birth.orbit {
            orbit.speed = asteroid_art::speed(hash, birth.material_tier, true);
        }
        let body_radius = pixel_assets::radius(&body, scale);
        if regions::sites_near(self.profile.seed, p)
            .iter()
            .any(|site| {
                p.distance(Vec2::from_array(site.position)) < site.radius + body_radius + 2.0
            })
        {
            return Candidate::Skipped;
        }
        let heading = unit(mix(hash ^ 71)) * TAU;
        let velocity = if let Some(orbit) = birth.orbit {
            orbit.tangent(p) * orbit.speed
        } else {
            forward(heading) * asteroid_art::speed(hash, birth.material_tier, false)
        };
        let hp = 20.0 + size * 30.0;
        Candidate::Spawned(self.world.spawn((
            Motion::new(p, velocity, heading),
            Actor {
                kind: Kind::Asteroid,
                health: hp,
                max_health: hp,
                radius: body_radius,
                scale,
                damage: 8.0,
            },
            body,
            birth,
        )))
    }

    fn unload_chunks(&mut self, centers: &[(i32, i32)]) {
        let stale: Vec<_> = self
            .chunks
            .keys()
            .copied()
            .filter(|&(x, y)| {
                !centers
                    .iter()
                    .any(|&(cx, cy)| (x - cx).abs() <= 2 && (y - cy).abs() <= 2)
            })
            .collect();
        for key in stale {
            if let Some(entities) = self.chunks.remove(&key) {
                for entity in entities {
                    let _ = self.world.despawn(entity);
                }
            }
            self.co_op_chunk_progress.remove(&key);
        }
    }

    pub(super) fn update_chunks(&mut self, position: Vec2) {
        if self.co_op.is_some() {
            self.update_co_op_chunks();
            return;
        }
        // Preserve solo traversal, candidate order and cap-stop completion exactly.
        let center = (position / CHUNK).floor().as_ivec2();
        self.unload_chunks(&[(center.x, center.y)]);
        let mut count = self.asteroid_count();
        for dy in -1..=1 {
            for dx in -1..=1 {
                let key = (center.x + dx, center.y + dy);
                if self.chunks.contains_key(&key) {
                    continue;
                }
                let mut spawned = Vec::new();
                for index in 0..CANDIDATES {
                    match self.chunk_candidate(key, index, count) {
                        Candidate::Skipped => {}
                        Candidate::Full => break,
                        Candidate::Spawned(entity) => {
                            spawned.push(entity);
                            count += 1;
                        }
                    }
                }
                self.chunks.insert(key, spawned);
            }
        }
    }

    pub(super) fn update_co_op_chunks(&mut self) {
        let positions = self.co_op_positions();
        let mut centers = [(0, 0); co_op::MAX_PARTICIPANTS];
        let mut center_count = 0;
        for position in positions.into_iter().flatten() {
            let p = (position / CHUNK).floor().as_ivec2();
            centers[center_count] = (p.x, p.y);
            center_count += 1;
        }
        self.unload_chunks(&centers[..center_count]);
        // Also permits explicit native diagnostic resets of the entity-list map.
        self.co_op_chunk_progress
            .retain(|key, _| self.chunks.contains_key(key));
        let mut keys = [(0, 0); MAX_INTEREST_CHUNKS];
        let mut len = 0;
        for &(cx, cy) in &centers[..center_count] {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let key = (cx + dx, cy + dy);
                    if keys[..len].contains(&key) {
                        continue;
                    }
                    keys[len] = key;
                    len += 1;
                }
            }
        }
        // Seeded tie-breaks keep the same candidate identity at each world key,
        // independent of pilot slot/admission order or randomized HashMap layout.
        keys[..len].sort_unstable_by_key(|&(x, y)| (hash3(self.profile.seed, x, y, 12), x, y));
        for &key in &keys[..len] {
            if let std::collections::hash_map::Entry::Vacant(entry) = self.chunks.entry(key) {
                entry.insert(Vec::new());
                self.co_op_chunk_progress.insert(key, 0);
            }
        }
        let mut count = self.asteroid_count();
        // One candidate per chunk per round. Budget-deferred cursors remain live;
        // destroyed/admitted/skipped candidates advance and are never regenerated.
        for index in 0..CANDIDATES {
            for &key in &keys[..len] {
                if self.co_op_chunk_progress.get(&key) != Some(&index) {
                    continue;
                }
                match self.chunk_candidate(key, index, count) {
                    Candidate::Full => return,
                    Candidate::Skipped => {}
                    Candidate::Spawned(entity) => {
                        self.chunks.get_mut(&key).unwrap().push(entity);
                        count += 1;
                    }
                }
                if index + 1 == CANDIDATES {
                    self.co_op_chunk_progress.remove(&key);
                } else {
                    self.co_op_chunk_progress.insert(key, index + 1);
                }
            }
        }
    }
}
