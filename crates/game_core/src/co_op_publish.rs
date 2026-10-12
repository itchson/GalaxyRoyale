//! One bounded immutable view of the real host ECS, independent of later ticks.
use super::*;
use co_op::{BodyPage, EntityPage, EntityState, Error, ParticipantId, Snapshot};

pub const MAX_ENTITIES: usize = 512;
pub const MAX_STORAGE_CELLS: usize = 1_048_576;
pub const MAX_BODY_DIMENSION: usize = 512;

/// FNV-1a64 over ASCII `galaxy-body-v1\0`, u32 LE width/height/occupied,
/// then each surviving row-major u32 LE index and RGBA. Deterministic cache
/// identity, not authentication or a collision-free cryptographic assertion.
pub fn body_signature(body: &PixelBody) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut feed = |bytes: &[u8]| {
        for &byte in bytes {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
        }
    };
    feed(b"galaxy-body-v1\0");
    for value in [body.width, body.height, body.occupied_count()] {
        feed(&(value as u32).to_le_bytes());
    }
    for (index, _, color) in body.cells_indexed() {
        feed(&(index as u32).to_le_bytes());
        feed(&color.to_le_bytes());
    }
    format!("{hash:016x}")
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Counts {
    pub entities: usize,
    pub occupied_cells: usize,
    pub storage_cells: usize,
    pub entity_pages: usize,
    pub body_pages: usize,
}
/// Borrowable packed native bodies; no JSON copy is required by native viewers.
pub struct Publication {
    acquisition: u32,
    snapshot: Snapshot,
    rows: Vec<EntityState>,
    bodies: Vec<PixelBody>,
    counts: Counts,
    wake: crate::co_op_wake_fx::WakePage,
}
#[derive(Serialize)]
pub struct Capture<'a> {
    pub acquisition: u32,
    pub snapshot: &'a Snapshot,
    pub counts: Counts,
}
impl Publication {
    pub fn acquisition(&self) -> u32 {
        self.acquisition
    }
    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }
    pub fn entities(&self) -> &[EntityState] {
        &self.rows
    }
    pub fn counts(&self) -> Counts {
        self.counts
    }
    pub fn body(&self, key: &str) -> Option<&PixelBody> {
        self.rows
            .binary_search_by(|row| row.id.as_str().cmp(key))
            .ok()
            .map(|index| &self.bodies[index])
    }
    fn capture(&self) -> Capture<'_> {
        Capture {
            acquisition: self.acquisition,
            snapshot: &self.snapshot,
            counts: self.counts,
        }
    }
    pub fn wake_page(&self) -> crate::co_op_wake_fx::WakePage {
        self.wake.clone()
    }
    pub fn entity_page(&self, offset: usize) -> Result<EntityPage, Error> {
        if offset > self.rows.len() {
            return Err(Error::InvalidOffset);
        }
        let end = offset
            .saturating_add(co_op::ENTITY_PAGE_SIZE)
            .min(self.rows.len());
        Ok(EntityPage {
            session: self.snapshot.session,
            tick: self.snapshot.tick,
            revision: self.snapshot.revision,
            offset,
            next_offset: (end < self.rows.len()).then_some(end),
            entities: self.rows[offset..end].to_vec(),
        })
    }
    pub fn body_page(&self, key: &str, offset: usize) -> Result<BodyPage, Error> {
        let body = self.body(key).ok_or(Error::InvalidEntity)?;
        if offset > body.occupied_count() {
            return Err(Error::InvalidOffset);
        }
        let cells: Vec<_> = body
            .cells_indexed()
            .skip(offset)
            .take(co_op::CELL_PAGE_SIZE)
            .map(|(index, _, color)| [index as u32, color])
            .collect();
        let next = offset + cells.len();
        Ok(BodyPage {
            session: self.snapshot.session,
            tick: self.snapshot.tick,
            revision: self.snapshot.revision,
            id: key.into(),
            width: body.width,
            height: body.height,
            occupied: body.occupied_count(),
            offset,
            next_offset: (next < body.occupied_count()).then_some(next),
            cells,
        })
    }
}
impl Game {
    /// Acquisition is atomic: bounds/identity errors retain the preceding cache.
    pub fn acquire_co_op_publication(
        &mut self,
        expected_session: ParticipantId,
    ) -> Result<Capture<'_>, Error> {
        self.check_co_op_session(expected_session)?;
        let acquisition = self
            .publication_generation
            .checked_add(1)
            .ok_or(Error::Exhausted)?;
        let snapshot = self.co_op_snapshot().ok_or(Error::NoSession)?;
        if snapshot.participants.len() > co_op::MAX_PARTICIPANTS
            || snapshot.projectiles.len() > MAX_BULLETS
        {
            return Err(Error::PublicationBound);
        }
        let wake = crate::co_op_wake_fx::WakePage {
            session: snapshot.session,
            tick: snapshot.tick,
            revision: snapshot.revision,
            points: self.co_op_wake_points(),
        };
        if wake.points.len() > crate::co_op_wake_fx::MAX_POINTS
            || serde_json::to_vec(&wake)
                .map_or(true, |v| v.len() > crate::co_op_wake_fx::MAX_PAGE_BYTES)
        {
            return Err(Error::PublicationBound);
        }
        let mut previous = None;
        let mut member_counts = [0usize; co_op::MAX_PARTICIPANTS];
        for point in &wake.points {
            let member = snapshot
                .participants
                .iter()
                .find(|p| p.slot == point.slot && p.epoch == point.epoch)
                .ok_or(Error::PublicationBound)?;
            let key = (point.slot, point.born_tick);
            member_counts[point.slot as usize] += 1;
            if previous.is_some_and(|p| p >= key)
                || member_counts[point.slot as usize] > crate::co_op_wake_fx::MAX_MEMBER_POINTS
                || point.canonical(member.id, snapshot.tick).is_err()
            {
                return Err(Error::PublicationBound);
            }
            previous = Some(key);
        }
        // Admit scalar bounds before scanning cells for signatures or cloning.
        let mut counts = Counts {
            entities: 0,
            occupied_cells: 0,
            storage_cells: 0,
            entity_pages: 0,
            body_pages: 0,
        };
        for (_, body) in self.world.query::<(&Motion, &PixelBody)>().iter() {
            counts.entities += 1;
            if counts.entities > MAX_ENTITIES
                || body.width == 0
                || body.height == 0
                || body.width > MAX_BODY_DIMENSION
                || body.height > MAX_BODY_DIMENSION
            {
                return Err(Error::PublicationBound);
            }
            counts.storage_cells = counts
                .storage_cells
                .checked_add(
                    body.width
                        .checked_mul(body.height)
                        .ok_or(Error::PublicationBound)?,
                )
                .filter(|&total| total <= MAX_STORAGE_CELLS)
                .ok_or(Error::PublicationBound)?;
            counts.occupied_cells += body.occupied_count();
            counts.body_pages += body.occupied_count().div_ceil(co_op::CELL_PAGE_SIZE);
        }
        counts.entity_pages = counts.entities.div_ceil(co_op::ENTITY_PAGE_SIZE);
        let rows = self.co_op_entity_rows();
        // The world is exclusively borrowed for this synchronous acquisition.
        // All bounds are admitted before the first packed body clone.
        let bodies = rows
            .iter()
            .map(|row| {
                let entity = Entity::from_bits(u64::from_str_radix(&row.id, 16).unwrap()).unwrap();
                (*self.world.get::<&PixelBody>(entity).unwrap()).clone()
            })
            .collect();
        self.publication = Some(Publication {
            acquisition,
            snapshot,
            rows,
            bodies,
            counts,
            wake,
        });
        self.publication_generation = acquisition;
        Ok(self.publication.as_ref().unwrap().capture())
    }
    /// Ticks may advance; membership/end/new acquisition cannot reuse this view.
    pub fn co_op_publication(
        &self,
        expected_session: ParticipantId,
        acquisition: u32,
        revision: u32,
    ) -> Result<&Publication, Error> {
        self.check_co_op_session(expected_session)?;
        let published = self.publication.as_ref().ok_or(Error::NoPublication)?;
        if published.acquisition != acquisition || published.snapshot.revision != revision {
            return Err(Error::StalePublication);
        }
        Ok(published)
    }
    pub fn published_co_op_wake_page(
        &self,
        session: ParticipantId,
        acquisition: u32,
        revision: u32,
    ) -> Result<crate::co_op_wake_fx::WakePage, Error> {
        Ok(self
            .co_op_publication(session, acquisition, revision)?
            .wake_page())
    }
    pub fn published_co_op_entity_page(
        &self,
        session: ParticipantId,
        acquisition: u32,
        revision: u32,
        offset: usize,
    ) -> Result<EntityPage, Error> {
        self.co_op_publication(session, acquisition, revision)?
            .entity_page(offset)
    }
    pub fn published_co_op_body_page(
        &self,
        session: ParticipantId,
        acquisition: u32,
        revision: u32,
        entity: &str,
        offset: usize,
    ) -> Result<BodyPage, Error> {
        self.co_op_publication(session, acquisition, revision)?
            .body_page(entity, offset)
    }
}

#[cfg(test)]
#[path = "co_op_publish_tests.rs"]
mod tests;
