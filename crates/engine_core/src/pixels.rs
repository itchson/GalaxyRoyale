use crate::{AffineTransform, Vec2};

const EDGE_EPSILON: f32 = 0.00001;

/// An object-local unit-cell field. Colors are packed little-endian RGBA;
/// occupancy is a separate bitset, so clearing a cell never changes its neighbors.
#[derive(Debug, Clone)]
pub struct PixelBody {
    pub width: usize,
    pub height: usize,
    colors: Box<[u32]>,
    occupancy: Box<[u64]>,
    occupied: usize,
}

impl PixelBody {
    /// Imports a validated packed field without palette searches or changing its
    /// original anchor. Missing indexes remain real holes, even at alpha zero.
    /// `maximum_cells` is the caller's allocation budget, not a render-size floor.
    pub fn from_indexed_rgba(
        width: usize,
        height: usize,
        cells: &[[u32; 2]],
        maximum_cells: usize,
    ) -> Result<Self, &'static str> {
        let length = width
            .checked_mul(height)
            .ok_or("pixel dimensions overflow")?;
        if length > maximum_cells || cells.len() > length {
            return Err("pixel allocation budget exceeded");
        }
        // Reject bad input before allocating its full packed field. Native pages
        // are row-major, so strict order also rejects duplicates without a set.
        let mut previous = None;
        for &[index, _] in cells {
            let index = index as usize;
            if index >= length || previous.is_some_and(|old| index <= old) {
                return Err("pixel indexes must be unique, ordered and in bounds");
            }
            previous = Some(index);
        }
        let mut body = Self {
            width,
            height,
            colors: vec![0; length].into_boxed_slice(),
            occupancy: vec![0; length.div_ceil(64)].into_boxed_slice(),
            occupied: cells.len(),
        };
        for &[index, color] in cells {
            let index = index as usize;
            body.colors[index] = color;
            body.occupancy[index / 64] |= 1 << (index % 64);
        }
        Ok(body)
    }

    /// Unknown characters are empty. Ragged rows are padded with empty cells.
    /// Cell centers are relative to the complete body's center, including holes.
    pub fn from_ascii(rows: &[&str], palette: &[(char, u32)]) -> Self {
        let width = rows
            .iter()
            .map(|row| row.chars().count())
            .max()
            .unwrap_or(0);
        let height = rows.len();
        let length = width
            .checked_mul(height)
            .expect("pixel body dimensions overflow");
        let mut body = Self {
            width,
            height,
            colors: vec![0; length].into_boxed_slice(),
            occupancy: vec![0; length.div_ceil(64)].into_boxed_slice(),
            occupied: 0,
        };
        for (y, row) in rows.iter().enumerate() {
            for (x, character) in row.chars().enumerate() {
                if let Some(&(_, color)) = palette.iter().rev().find(|&&(key, _)| key == character)
                {
                    let index = y * width + x;
                    body.colors[index] = color;
                    body.occupancy[index / 64] |= 1 << (index % 64);
                    body.occupied += 1;
                }
            }
        }
        body
    }

    pub fn occupied_count(&self) -> usize {
        self.occupied
    }

    pub fn is_occupied(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height && self.occupied_index(y * self.width + x)
    }

    pub fn color_at(&self, x: usize, y: usize) -> Option<u32> {
        self.is_occupied(x, y)
            .then(|| self.colors[y * self.width + x])
    }

    pub fn cells(&self) -> impl Iterator<Item = (Vec2, u32)> + '_ {
        self.cells_indexed()
            .map(|(_, position, color)| (position, color))
    }

    /// Stable row-major indices allow role masks to partition surviving cells.
    pub fn cells_indexed(&self) -> impl Iterator<Item = (usize, Vec2, u32)> + '_ {
        (0..self.colors.len())
            .filter(|&index| self.occupied_index(index))
            .map(|index| (index, self.center(index), self.colors[index]))
    }

    /// Earliest segment contact with an occupied unit square. Uses clipped grid
    /// traversal, including both neighboring cells at exact edges/corners. This
    /// prevents fast projectiles tunneling and lets them pass through pixel holes.
    pub fn hit_segment(&self, from_local: Vec2, to_local: Vec2) -> Option<Vec2> {
        if self.occupied == 0 || !from_local.is_finite() || !to_local.is_finite() {
            return None;
        }
        let half = Vec2::new(self.width as f32, self.height as f32) * 0.5;
        let from = from_local + half;
        let delta = to_local - from_local;
        if !from.is_finite() || !delta.is_finite() {
            return None;
        }
        let (enter, exit) = clip_segment(from, delta, Vec2::ZERO, half * 2.0)?;
        let mut t = enter;
        let entry = from + delta * t;
        if self.occupied_at_contact(entry) {
            return Some(from_local + delta * t);
        }
        if delta == Vec2::ZERO {
            return None;
        }

        let step_x = delta.x.signum() as i32;
        let step_y = delta.y.signum() as i32;
        let mut x = directed_cell(entry.x, delta.x);
        let mut y = directed_cell(entry.y, delta.y);
        let increment_x = if delta.x == 0.0 {
            f32::INFINITY
        } else {
            1.0 / delta.x.abs()
        };
        let increment_y = if delta.y == 0.0 {
            f32::INFINITY
        } else {
            1.0 / delta.y.abs()
        };
        let boundary_x = if step_x > 0 { (x + 1) as f32 } else { x as f32 };
        let boundary_y = if step_y > 0 { (y + 1) as f32 } else { y as f32 };
        let mut next_x = if delta.x == 0.0 {
            f32::INFINITY
        } else {
            (boundary_x - from.x) / delta.x
        };
        let mut next_y = if delta.y == 0.0 {
            f32::INFINITY
        } else {
            (boundary_y - from.y) / delta.y
        };

        // A clipped straight segment crosses at most width + height grid edges.
        for _ in 0..self.width + self.height + 4 {
            if self.occupied_grid(x, y) {
                return Some(from_local + delta * t);
            }
            let next = next_x.min(next_y);
            if !next.is_finite() || next > exit + EDGE_EPSILON {
                return None;
            }
            t = next.max(enter).min(exit);
            if self.occupied_at_contact(from + delta * t) {
                return Some(from_local + delta * t);
            }
            // Compare crossing times relative to their magnitude. Advancing both
            // axes only at the same crossing avoids missing narrow diagonal cells.
            let tolerance = f32::EPSILON * 4.0 * next.abs().max(1.0);
            if next_x <= next + tolerance {
                x += step_x;
                next_x += increment_x;
            }
            if next_y <= next + tolerance {
                y += step_y;
                next_y += increment_y;
            }
        }
        None
    }

    pub fn hit_segment_world(
        &self,
        from: Vec2,
        to: Vec2,
        transform: AffineTransform,
    ) -> Option<Vec2> {
        let a = transform.try_inverse_point(from)?;
        let b = transform.try_inverse_point(to)?;
        self.hit_segment(a, b).map(|point| transform.point(point))
    }

    /// Removes occupied squares touched by a local-space circle and returns their
    /// centers/colors for debris. Radius zero removes cells containing the point.
    /// Exact boundary contacts include both neighbors, matching segment collision.
    pub fn damage(&mut self, local: Vec2, radius: f32) -> Vec<(Vec2, u32)> {
        if self.occupied == 0 || !local.is_finite() || !radius.is_finite() || radius < 0.0 {
            return Vec::new();
        }
        let half = Vec2::new(self.width as f32, self.height as f32) * 0.5;
        let point = local + half;
        let min_x = ((point.x - radius).ceil() as i64).saturating_sub(1).max(0);
        let min_y = ((point.y - radius).ceil() as i64).saturating_sub(1).max(0);
        let max_x = ((point.x + radius).floor() as i64).min(self.width as i64 - 1);
        let max_y = ((point.y + radius).floor() as i64).min(self.height as i64 - 1);
        let mut removed = Vec::new();
        let radius_squared = radius * radius;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let index = y as usize * self.width + x as usize;
                if !self.occupied_index(index) {
                    continue;
                }
                let minimum = Vec2::new(x as f32, y as f32);
                let nearest = point.clamp(minimum, minimum + Vec2::ONE);
                if (point - nearest).length_squared() <= radius_squared {
                    self.occupancy[index / 64] &= !(1 << (index % 64));
                    self.occupied -= 1;
                    removed.push((self.center(index), self.colors[index]));
                }
            }
        }
        removed
    }

    /// Four-connected fragments, retaining positions relative to this body's
    /// origin. Call after damage rather than every frame; callers choose whether
    /// a component becomes a physical fragment or cosmetic debris.
    pub fn connected_components(&self) -> Vec<Vec<(Vec2, u32)>> {
        let mut seen = vec![false; self.colors.len()];
        let mut components = Vec::new();
        let mut pending = Vec::new();
        for start in 0..self.colors.len() {
            if seen[start] || !self.occupied_index(start) {
                continue;
            }
            let mut component = Vec::new();
            pending.push(start);
            seen[start] = true;
            while let Some(index) = pending.pop() {
                component.push((self.center(index), self.colors[index]));
                let x = index % self.width;
                let y = index / self.width;
                let neighbors = [
                    (x > 0).then(|| index - 1),
                    (x + 1 < self.width).then(|| index + 1),
                    (y > 0).then(|| index - self.width),
                    (y + 1 < self.height).then(|| index + self.width),
                ];
                for neighbor in neighbors.into_iter().flatten() {
                    if !seen[neighbor] && self.occupied_index(neighbor) {
                        seen[neighbor] = true;
                        pending.push(neighbor);
                    }
                }
            }
            components.push(component);
        }
        components
    }

    fn occupied_index(&self, index: usize) -> bool {
        self.occupancy[index / 64] & (1 << (index % 64)) != 0
    }

    fn occupied_grid(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && self.is_occupied(x as usize, y as usize)
    }

    fn occupied_at_contact(&self, point: Vec2) -> bool {
        let x = point.x.floor() as i32;
        let y = point.y.floor() as i32;
        let edge_x = (point.x - point.x.round()).abs() <= EDGE_EPSILON;
        let edge_y = (point.y - point.y.round()).abs() <= EDGE_EPSILON;
        self.occupied_grid(x, y)
            || (edge_x && self.occupied_grid(point.x.round() as i32 - 1, y))
            || (edge_y && self.occupied_grid(x, point.y.round() as i32 - 1))
            || (edge_x
                && edge_y
                && self.occupied_grid(point.x.round() as i32 - 1, point.y.round() as i32 - 1))
    }

    fn center(&self, index: usize) -> Vec2 {
        Vec2::new(
            (index % self.width) as f32 + 0.5 - self.width as f32 * 0.5,
            (index / self.width) as f32 + 0.5 - self.height as f32 * 0.5,
        )
    }
}

fn directed_cell(value: f32, direction: f32) -> i32 {
    if direction < 0.0 && (value - value.round()).abs() <= EDGE_EPSILON {
        value.round() as i32 - 1
    } else {
        value.floor() as i32
    }
}

fn clip_segment(from: Vec2, delta: Vec2, minimum: Vec2, maximum: Vec2) -> Option<(f32, f32)> {
    let mut enter = 0.0_f32;
    let mut exit = 1.0_f32;
    for axis in 0..2 {
        if delta[axis] == 0.0 {
            if from[axis] < minimum[axis] || from[axis] > maximum[axis] {
                return None;
            }
        } else {
            let a = (minimum[axis] - from[axis]) / delta[axis];
            let b = (maximum[axis] - from[axis]) / delta[axis];
            enter = enter.max(a.min(b));
            exit = exit.min(a.max(b));
            if enter > exit {
                return None;
            }
        }
    }
    Some((enter, exit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_import_preserves_holes_anchor_and_transparent_occupancy() {
        let source = PixelBody::from_ascii(
            &["A B", " C "],
            &[('A', 0xff00_00ff), ('B', 0x0000_0000), ('C', 0xff00_ff00)],
        );
        let cells: Vec<_> = source
            .cells_indexed()
            .map(|(i, _, c)| [i as u32, c])
            .collect();
        let imported = PixelBody::from_indexed_rgba(3, 2, &cells, 6).unwrap();
        assert_eq!(
            source.cells_indexed().collect::<Vec<_>>(),
            imported.cells_indexed().collect::<Vec<_>>()
        );
        assert!(imported.is_occupied(2, 0));
        assert!(!imported.is_occupied(1, 0));
        assert!(
            imported
                .hit_segment(Vec2::new(0.0, -1.0), Vec2::new(0.0, -0.5))
                .is_none()
        );
        assert!(
            imported
                .hit_segment(Vec2::new(1.0, -1.0), Vec2::new(1.0, 0.0))
                .is_some()
        );
    }

    #[test]
    fn packed_import_rejects_unbounded_dimensions_and_aliased_cells() {
        assert!(PixelBody::from_indexed_rgba(usize::MAX, 2, &[], usize::MAX).is_err());
        assert!(PixelBody::from_indexed_rgba(512, 512, &[], 1024).is_err());
        assert!(PixelBody::from_indexed_rgba(2, 2, &[[4, 1]], 4).is_err());
        assert!(PixelBody::from_indexed_rgba(2, 2, &[[1, 1], [1, 2]], 4).is_err());
        assert!(PixelBody::from_indexed_rgba(2, 2, &[[2, 1], [0, 2]], 4).is_err());
        assert!(
            PixelBody::from_indexed_rgba(0, 0, &[], 0)
                .unwrap()
                .cells()
                .next()
                .is_none()
        );
    }
    use crate::Transform;
    const WHITE: u32 = 0xff_ffffff;
    fn body(rows: &[&str]) -> PixelBody {
        PixelBody::from_ascii(rows, &[('#', WHITE)])
    }
    fn near(a: Vec2, b: Vec2) {
        assert!((a - b).length() < 0.0001, "{a:?} != {b:?}");
    }

    #[test]
    fn ascii_centering_ragged_rows_colors_and_occupancy_are_independent() {
        let b = PixelBody::from_ascii(&["#x", "#"], &[('#', 0), ('x', WHITE)]);
        assert_eq!((b.width, b.height, b.occupied_count()), (2, 2, 3));
        assert_eq!(b.color_at(0, 0), Some(0));
        assert_eq!(b.color_at(1, 1), None);
        assert_eq!(b.cells().next(), Some((Vec2::new(-0.5, -0.5), 0)));
    }

    #[test]
    fn swept_projectiles_hit_first_cell_and_pass_through_holes() {
        let b = body(&["#.#", "...", "#.#"]);
        assert_eq!(
            b.hit_segment(Vec2::new(-100.0, 0.0), Vec2::new(100.0, 0.0)),
            None
        );
        near(
            b.hit_segment(Vec2::new(-100.0, -1.0), Vec2::new(100.0, -1.0))
                .unwrap(),
            Vec2::new(-1.5, -1.0),
        );
        near(
            b.hit_segment(Vec2::new(100.0, -1.0), Vec2::new(-100.0, -1.0))
                .unwrap(),
            Vec2::new(1.5, -1.0),
        );
    }

    #[test]
    fn exact_edges_corners_and_zero_length_hits_are_consistent() {
        let b = body(&["#.", ".."]).clone();
        near(
            b.hit_segment(Vec2::new(-2.0, 0.0), Vec2::new(2.0, 0.0))
                .unwrap(),
            Vec2::new(-1.0, 0.0),
        );
        near(
            b.hit_segment(Vec2::new(1.0, -1.0), Vec2::new(-1.0, 1.0))
                .unwrap(),
            Vec2::ZERO,
        );
        assert_eq!(
            b.hit_segment(Vec2::new(0.5, 0.5), Vec2::new(0.5, 0.5)),
            None
        );
        assert!(
            b.hit_segment(Vec2::new(-0.5, -0.5), Vec2::new(-0.5, -0.5))
                .is_some()
        );
    }

    #[test]
    fn transformed_hit_matches_world_contact_with_shear() {
        let b = body(&["#.#"]);
        let parent = Transform {
            position: Vec2::new(40.0, -8.0),
            rotation: 0.6,
            scale: Vec2::new(3.0, 0.4),
        };
        let child = Transform {
            position: Vec2::new(4.0, 2.0),
            rotation: -0.9,
            scale: Vec2::new(2.0, 1.2),
        };
        let t = parent.compose_affine(child);
        let from = t.point(Vec2::new(-8.0, 0.0));
        let to = t.point(Vec2::new(8.0, 0.0));
        near(
            b.hit_segment_world(from, to, t).unwrap(),
            t.point(Vec2::new(-1.5, 0.0)),
        );
    }

    #[test]
    fn localized_damage_preserves_holes_and_returns_exact_debris() {
        let mut b = body(&["#####", "##.##", "#####"]);
        assert!(b.damage(Vec2::ZERO, 0.0).is_empty());
        let removed = b.damage(Vec2::new(-2.0, 0.0), 0.1);
        assert_eq!(removed, vec![(Vec2::new(-2.0, 0.0), WHITE)]);
        assert_eq!(b.occupied_count(), 13);
        assert!(b.damage(Vec2::new(-2.0, 0.0), 0.1).is_empty());
        assert_eq!(b.damage(Vec2::ZERO, 100.0).len(), 13);
        assert_eq!(b.occupied_count(), 0);
    }

    #[test]
    fn damage_fragments_conserve_surviving_cells_and_separate_diagonal_cells() {
        let mut b = body(&["#####"]);
        assert_eq!(b.damage(Vec2::ZERO, 0.1).len(), 1);
        let components = b.connected_components();
        assert_eq!(components.len(), 2);
        assert_eq!(
            components.iter().map(Vec::len).sum::<usize>(),
            b.occupied_count()
        );
        assert_eq!(body(&["#.", ".#"]).connected_components().len(), 2);
    }

    #[test]
    fn empty_nonfinite_and_singular_inputs_do_not_hit_or_damage() {
        let mut empty = body(&[]);
        assert!(empty.damage(Vec2::ZERO, 1.0).is_empty());
        assert_eq!(empty.hit_segment(Vec2::ZERO, Vec2::ONE), None);
        let mut b = body(&["#"]);
        assert_eq!(b.hit_segment(Vec2::splat(f32::NAN), Vec2::ONE), None);
        assert!(b.damage(Vec2::ZERO, f32::INFINITY).is_empty());
        let t = Transform {
            scale: Vec2::ZERO,
            ..Transform::default()
        }
        .affine();
        assert_eq!(b.hit_segment_world(Vec2::ZERO, Vec2::ONE, t), None);
    }

    #[test]
    fn grid_traversal_matches_independent_per_cell_slab_reference() {
        // Many directions and holes: catches reverse traversal and corner-step errors.
        let mut random = 17_u32;
        let mut next = || {
            random = random.wrapping_mul(1664525).wrapping_add(1013904223);
            (random >> 8) as f32 / 16777216.0
        };
        for _ in 0..30 {
            let rows: Vec<String> = (0..5)
                .map(|_| {
                    (0..7)
                        .map(|_| if next() > 0.6 { '#' } else { '.' })
                        .collect()
                })
                .collect();
            let refs: Vec<&str> = rows.iter().map(String::as_str).collect();
            let b = body(&refs);
            for _ in 0..50 {
                let a = Vec2::new(next() * 16.0 - 8.0, next() * 12.0 - 6.0);
                let end = Vec2::new(next() * 16.0 - 8.0, next() * 12.0 - 6.0);
                let delta = end - a;
                let reference = b
                    .cells()
                    .filter_map(|(center, _)| {
                        clip_segment(
                            a,
                            delta,
                            center - Vec2::splat(0.5),
                            center + Vec2::splat(0.5),
                        )
                        .map(|(t, _)| t)
                    })
                    .min_by(f32::total_cmp)
                    .map(|t| a + delta * t);
                match (b.hit_segment(a, end), reference) {
                    (Some(actual), Some(expected)) => near(actual, expected),
                    (None, None) => {}
                    pair => panic!("traversal mismatch: {pair:?}, from {a:?}, to {end:?}"),
                }
            }
        }
    }
}
