//! Original sports-league co-op symbols. No session, invitation or identity data.
//! Cached source cells fit continuously; admitted roster count is caller-owned.
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Transform, Vec2};
use std::sync::LazyLock;

const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    u32::from_le_bytes([r, g, b, 255])
}
const RECESS: u32 = rgb(18, 18, 18);
const GRAPHITE: u32 = rgb(36, 38, 41);
const ALLOY: u32 = rgb(88, 99, 110);
const PLATINUM: u32 = rgb(184, 201, 213);
const SPECULAR: u32 = rgb(235, 242, 244);
const CYAN_DARK: u32 = rgb(32, 101, 115);
const CYAN: u32 = rgb(82, 213, 234);
const GOLD: u32 = rgb(214, 175, 102);

/// Roster count is the actual admitted population, including the local pilot.
/// It is not a transport-connected count, capacity promise or ownership flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Invite,
    Join,
    Roster(u8),
}

pub struct Art {
    pub body: PixelBody,
    signals: Box<[bool]>,
}

// Authoring only: allocated once by LazyLock, never cloned by append.
struct Canvas {
    width: usize,
    height: usize,
    cells: Vec<u32>,
    signals: Vec<bool>,
}
impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![0; width * height],
            signals: vec![false; width * height],
        }
    }
    fn cell(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            let i = y as usize * self.width + x as usize;
            self.cells[i] = color;
            self.signals[i] = color == CYAN;
        }
    }
    fn rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: u32) {
        for yy in y..y + height {
            for xx in x..x + width {
                self.cell(xx, yy, color);
            }
        }
    }
    fn polygon(&mut self, vertices: &[(i32, i32)], color: u32) {
        // Convex authored facets, evaluated at source-cell centers.
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let mut sign = 0;
                let mut inside = true;
                for i in 0..vertices.len() {
                    let (ax, ay) = vertices[i];
                    let (bx, by) = vertices[(i + 1) % vertices.len()];
                    let cross = (bx - ax) * (2 * y + 1 - 2 * ay) - (by - ay) * (2 * x + 1 - 2 * ax);
                    if cross != 0 {
                        let next = cross.signum();
                        if sign != 0 && sign != next {
                            inside = false;
                            break;
                        }
                        sign = next;
                    }
                }
                if inside {
                    self.cell(x, y, color);
                }
            }
        }
    }
    fn line(&mut self, a: (i32, i32), b: (i32, i32), radius: i32, color: u32) {
        let steps = (b.0 - a.0).abs().max((b.1 - a.1).abs()).max(1);
        for t in 0..=steps {
            let x = a.0 + (b.0 - a.0) * t / steps;
            let y = a.1 + (b.1 - a.1) * t / steps;
            for yy in -radius..=radius {
                for xx in -radius..=radius {
                    if xx.abs() + yy.abs() <= radius {
                        self.cell(x + xx, y + yy, color);
                    }
                }
            }
        }
    }
    fn fastener(&mut self, x: i32, y: i32) {
        self.rect(x - 1, y - 1, 3, 3, RECESS);
        self.cell(x, y, GOLD);
        self.cell(x - 1, y - 1, ALLOY);
    }
    fn finish(self) -> Art {
        let packed: Vec<_> = self
            .cells
            .into_iter()
            .enumerate()
            .filter(|(_, color)| *color != 0)
            .map(|(index, color)| [index as u32, color])
            .collect();
        Art {
            body: PixelBody::from_indexed_rgba(self.width, self.height, &packed, 4096)
                .expect("bounded original co-op art"),
            signals: self.signals.into_boxed_slice(),
        }
    }
}

fn invite() -> Art {
    let mut c = Canvas::new(64, 64);
    // Split orbital wings leave a real dark gap around the departing beacon.
    for mirror in [false, true] {
        let point = |x, y| (if mirror { 63 - x } else { x }, y);
        for (a, b) in [((18, 24), (7, 32)), ((7, 32), (8, 45)), ((8, 45), (20, 52))] {
            c.line(point(a.0, a.1), point(b.0, b.1), 3, ALLOY);
            c.line(point(a.0, a.1 - 1), point(b.0, b.1 - 1), 1, PLATINUM);
            c.line(point(a.0, a.1), point(b.0, b.1), 0, CYAN_DARK);
        }
        c.fastener(point(9, 39).0, 39);
    }
    // Athletic outward arrow, with a broad pale facet and recessed cyan stem.
    // Separate convex head and stem retain the full arrow silhouette.
    c.polygon(&[(32, 5), (47, 22), (17, 22)], ALLOY);
    c.rect(26, 20, 12, 20, ALLOY);
    c.polygon(&[(32, 7), (44, 21), (20, 21)], PLATINUM);
    c.polygon(&[(34, 13), (43, 21), (34, 21)], ALLOY);
    c.line((32, 7), (21, 20), 0, SPECULAR);
    c.rect(28, 21, 8, 17, PLATINUM);
    c.rect(30, 22, 4, 15, CYAN);
    c.rect(28, 34, 2, 4, GRAPHITE);
    // Beveled launch plinth, not a card surrounding the symbol.
    c.polygon(
        &[(18, 43), (46, 43), (51, 48), (45, 56), (19, 56), (13, 48)],
        ALLOY,
    );
    c.polygon(
        &[(20, 45), (44, 45), (48, 48), (43, 53), (21, 53), (16, 48)],
        GRAPHITE,
    );
    c.line((19, 44), (44, 44), 0, SPECULAR);
    c.rect(25, 46, 14, 2, CYAN);
    for x in [22, 30, 38] {
        c.rect(x, 50, 4, 1, RECESS);
    }
    c.fastener(19, 49);
    c.fastener(44, 49);
    c.finish()
}

fn join() -> Art {
    let mut c = Canvas::new(64, 64);
    // Two inward competition arms frame a recessed docking receiver.
    for mirror in [false, true] {
        let points = |v: &[(i32, i32)]| -> Vec<_> {
            v.iter()
                .map(|&(x, y)| (if mirror { 63 - x } else { x }, y))
                .collect()
        };
        c.rect(if mirror { 49 } else { 5 }, 13, 10, 26, ALLOY);
        c.rect(if mirror { 49 } else { 7 }, 15, 7, 11, PLATINUM);
        c.rect(if mirror { 54 } else { 7 }, 15, 2, 10, SPECULAR);
        c.rect(if mirror { 52 } else { 9 }, 16, 2, 8, CYAN);
        c.rect(if mirror { 52 } else { 9 }, 19, 2, 1, CYAN_DARK);
        c.rect(if mirror { 52 } else { 9 }, 23, 2, 1, GRAPHITE);
        c.polygon(
            &points(&[(14, 25), (23, 25), (29, 31), (23, 37), (14, 37)]),
            PLATINUM,
        );
        c.polygon(&points(&[(20, 27), (24, 31), (20, 35)]), CYAN);
        c.fastener(if mirror { 55 } else { 8 }, 35);
    }
    c.polygon(
        &[
            (25, 18),
            (39, 18),
            (44, 23),
            (44, 43),
            (38, 50),
            (26, 50),
            (20, 43),
            (20, 23),
        ],
        ALLOY,
    );
    c.polygon(
        &[
            (27, 20),
            (37, 20),
            (41, 24),
            (41, 41),
            (37, 46),
            (27, 46),
            (23, 41),
            (23, 24),
        ],
        RECESS,
    );
    c.line((25, 19), (38, 19), 0, SPECULAR);
    c.line((22, 24), (22, 42), 0, PLATINUM);
    c.line((41, 24), (41, 42), 0, GRAPHITE);
    // The inward arrows cross the side of the receiver, not its aperture.
    for mirror in [false, true] {
        let x = |n| if mirror { 63 - n } else { n };
        c.polygon(
            &[
                (x(15), 27),
                (x(23), 27),
                (x(28), 31),
                (x(23), 35),
                (x(15), 35),
            ],
            PLATINUM,
        );
        c.line((x(17), 31), (x(24), 31), 0, CYAN);
        c.line((x(16), 27), (x(22), 27), 0, SPECULAR);
        c.line((x(16), 34), (x(22), 34), 0, ALLOY);
    }
    c.rect(30, 26, 4, 13, CYAN_DARK);
    c.rect(30, 28, 4, 7, CYAN);
    c.rect(29, 40, 6, 2, GOLD);
    c.polygon(
        &[(18, 49), (46, 49), (50, 53), (45, 57), (19, 57), (14, 53)],
        ALLOY,
    );
    c.line((20, 50), (43, 50), 0, PLATINUM);
    c.rect(25, 52, 14, 2, GRAPHITE);
    c.rect(30, 53, 4, 2, GOLD);
    c.finish()
}

fn roster(members: u8) -> Art {
    let mut c = Canvas::new(128, 24);
    for slot in 0..8 {
        let x = 3 + slot * 15;
        c.polygon(
            &[
                (x + 2, 2),
                (x + 10, 2),
                (x + 13, 5),
                (x + 13, 16),
                (x + 10, 19),
                (x + 2, 19),
                (x, 16),
                (x, 5),
            ],
            ALLOY,
        );
        c.polygon(
            &[
                (x + 3, 4),
                (x + 9, 4),
                (x + 11, 6),
                (x + 11, 15),
                (x + 9, 17),
                (x + 3, 17),
                (x + 2, 15),
                (x + 2, 6),
            ],
            RECESS,
        );
        c.line((x + 3, 3), (x + 9, 3), 0, PLATINUM);
        // Same occupied pilot glyph in every state. Only a known admitted seat
        // illuminates; empty seats cannot reveal an identity or pretend readiness.
        let admitted = slot < members as i32;
        let signal = if admitted { CYAN } else { GRAPHITE };
        c.polygon(
            &[
                (x + 6, 6),
                (x + 8, 8),
                (x + 8, 10),
                (x + 6, 12),
                (x + 4, 10),
                (x + 4, 8),
            ],
            signal,
        );
        c.polygon(
            &[(x + 4, 12), (x + 8, 12), (x + 10, 16), (x + 2, 16)],
            signal,
        );
        c.rect(x + 4, 18, 5, 1, if admitted { CYAN_DARK } else { GRAPHITE });
        c.cell(x + 11, 5, GOLD);
    }
    c.finish()
}

static INVITE: LazyLock<Art> = LazyLock::new(invite);
static JOIN: LazyLock<Art> = LazyLock::new(join);
static ROSTER: LazyLock<[Art; 9]> = LazyLock::new(|| std::array::from_fn(|i| roster(i as u8)));

pub fn art(kind: Kind) -> Option<&'static Art> {
    match kind {
        Kind::Invite => Some(&INVITE),
        Kind::Join => Some(&JOIN),
        Kind::Roster(count) => ROSTER.get(count as usize),
    }
}

#[derive(Clone, Copy, Default)]
pub struct Style {
    pub hovered: bool,
    pub busy: bool,
    pub disabled: bool,
    pub time: f32,
    pub reduced_motion: bool,
}
impl Art {
    pub fn color(&self, cell: Vec2, color: u32, style: Style) -> u32 {
        let x = (cell.x + self.body.width as f32 * 0.5).floor();
        let y = (cell.y + self.body.height as f32 * 0.5).floor();
        if !cell.is_finite()
            || x < 0.0
            || y < 0.0
            || x >= self.body.width as f32
            || y >= self.body.height as f32
        {
            return color;
        }
        let index = y as usize * self.body.width + x as usize;
        let [r, g, b, _] = color.to_le_bytes();
        if style.disabled {
            let gray = ((r as u16 + g as u16 + b as u16) / 5) as u8;
            return rgb(gray, gray, gray);
        }
        if !self.signals.get(index).copied().unwrap_or(false) {
            return color;
        }
        let time = if style.time.is_finite() {
            style.time.rem_euclid(100.0)
        } else {
            0.0
        };
        let mut strength = if style.hovered { 1.05 } else { 1.0 };
        if (style.hovered || style.busy) && !style.reduced_motion {
            strength *= 0.96 + 0.06 * (time * 2.5 + (index % self.body.width) as f32 * 0.10).sin();
        }
        let channel = |v: u8| (v as f32 * strength).clamp(0.0, 255.0) as u8;
        rgb(channel(r), channel(g), channel(b))
    }
}

pub struct Input {
    pub kind: Kind,
    /// Measured semantic rectangle; never a new interactive hitbox.
    pub rect: [f32; 4],
    /// Clip in the same local UI coordinate space as rect, before parent.
    pub clip: Option<[f32; 4]>,
    pub parent: AffineTransform,
    pub style: Style,
}
fn valid_rect(r: [f32; 4]) -> bool {
    r.iter().all(|v| v.is_finite() && v.abs() <= 8192.0) && r[2] > 0.0 && r[3] > 0.0
}

/// Admits one complete cached object and charges all submitted occupied cells,
/// even when local whole-cell clipping masks their alpha. One transform, unlit.
/// No per-frame body/role allocation; FrameBuilder may grow its own buffers.
pub fn append(
    frame: &mut FrameBuilder,
    input: &Input,
    pixels: &mut usize,
    work: &mut usize,
) -> usize {
    if !valid_rect(input.rect) || input.clip.is_some_and(|c| !valid_rect(c)) {
        return 0;
    }
    let Some(art) = art(input.kind) else {
        return 0;
    };
    let count = art.body.occupied_count();
    let scale = (input.rect[2] / art.body.width as f32).min(input.rect[3] / art.body.height as f32);
    let local = Transform {
        position: Vec2::new(
            input.rect[0] + input.rect[2] * 0.5,
            input.rect[1] + input.rect[3] * 0.5,
        ),
        scale: Vec2::splat(scale),
        ..Default::default()
    }
    .affine();
    let composed = input.parent.compose(local);
    let determinant = composed.basis_x.perp_dot(composed.basis_y);
    if count > *pixels
        || count > *work
        || !determinant.is_finite()
        || determinant == 0.0
        || !composed.position.is_finite()
        || !composed.basis_x.is_finite()
        || !composed.basis_y.is_finite()
        || art.body.cells().any(|(cell, _)| {
            [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::splat(0.5),
            ]
            .iter()
            .any(|&corner| !composed.point(cell + corner).is_finite())
        })
    {
        return 0;
    }
    frame.push_body_affine_colored(&art.body, composed, |cell, color| {
        let p = local.point(cell);
        let half = scale * 0.5;
        if input.clip.is_some_and(|c| {
            p.x - half < c[0]
                || p.y - half < c[1]
                || p.x + half > c[0] + c[2]
                || p.y + half > c[1] + c[3]
        }) {
            0
        } else {
            art.color(cell, color, input.style)
        }
    });
    *pixels -= count;
    *work -= count;
    count
}

#[cfg(test)]
#[path = "co_op_art_tests.rs"]
mod tests;
