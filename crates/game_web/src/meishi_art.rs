//! Original packed-cell collectible fittings. DOM owns layout and hit targets.
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Transform, Vec2};
use std::sync::LazyLock;

const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    u32::from_le_bytes([r, g, b, 255])
}
const BLACK: u32 = rgb(4, 8, 11);
const DECK: u32 = rgb(6, 11, 10);
const GRAPHITE: u32 = rgb(23, 32, 38);
const STEEL: u32 = rgb(76, 90, 96);
const PLATINUM: u32 = rgb(162, 175, 176);
const LIGHT: u32 = rgb(213, 222, 217);
const WARM_ALLOY: u32 = rgb(163, 151, 126);
const SHADOW_METAL: u32 = rgb(43, 51, 54);
const STAR_DIM: u32 = rgb(28, 42, 43);
const STAR_LIGHT: u32 = rgb(74, 104, 92);
// Explicit enamel/foil roles; metal and the anonymous socket floor stay neutral.
const ENAMEL_DARK: u32 = rgb(21, 71, 35);
const ENAMEL: u32 = rgb(55, 151, 85);
const ENAMEL_LIGHT: u32 = rgb(135, 230, 168);
const FOIL: u32 = rgb(149, 203, 195);
const PALETTE: &[(char, u32)] = &[
    ('b', BLACK),
    ('g', GRAPHITE),
    ('s', STEEL),
    ('p', PLATINUM),
    ('w', LIGHT),
    ('d', ENAMEL_DARK),
    ('e', ENAMEL),
    ('l', ENAMEL_LIGHT),
    ('f', FOIL),
    ('a', WARM_ALLOY),
    ('h', SHADOW_METAL),
    ('t', STAR_DIM),
    ('v', STAR_LIGHT),
];

struct Canvas {
    width: usize,
    cells: Vec<char>,
}
impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            cells: vec![' '; width * height],
        }
    }
    fn put(&mut self, x: usize, y: usize, color: char) {
        self.cells[y * self.width + x] = color;
    }
    fn rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: char) {
        for yy in y..y + height {
            for xx in x..x + width {
                self.put(xx, yy, color);
            }
        }
    }
    fn rivet(&mut self, x: usize, y: usize) {
        self.rect(x - 1, y - 1, 3, 3, 'h');
        self.put(x - 1, y - 1, 'p');
        self.put(x, y, 'w');
        self.put(x + 1, y, 'b');
    }
    fn body(self) -> PixelBody {
        let rows: Vec<String> = self
            .cells
            .chunks_exact(self.width)
            .map(|row| row.iter().collect())
            .collect();
        PixelBody::from_ascii(
            &rows.iter().map(String::as_str).collect::<Vec<_>>(),
            PALETTE,
        )
    }
}

fn chassis_corner(alternate: bool) -> PixelBody {
    let mut canvas = Canvas::new(64, 64);
    // Wide machined L with a real diagonal shoulder, not a rectangular outline.
    for y in 0..64 {
        for x in 0..64 {
            if x + y < 18 || (x >= 7 && y >= 7 && x + y > 32) {
                continue;
            }
            let edge = x.min(y).min((x + y - 18) / 2);
            let color = match edge {
                0 => 'w',
                1 => 'p',
                2 => 'a',
                3 => 's',
                4 => 'h',
                5 => 'p',
                6 => 's',
                _ => 'b',
            };
            canvas.put(x, y, color);
        }
    }
    // Three asymmetric planes form an inset corporate crystal inside the shoulder.
    for y in 12..=25 {
        for x in 12..=25 {
            if x + y > 37 {
                continue;
            }
            let color = if x == 12 || y == 12 || x + y == 37 {
                'b'
            } else if x < y {
                'd'
            } else if x + y < 30 {
                'l'
            } else {
                'e'
            };
            canvas.put(x, y, color);
        }
    }
    // Separate recessed rail segments, latches and catches retain neutral alloy.
    for start in [34, 46, 56] {
        canvas.rect(start, 3, 6, 3, 'b');
        canvas.rect(start, 4, 5, 1, 'e');
        canvas.put(start, 4, 'l');
        canvas.rect(3, start, 3, 6, 'b');
        canvas.rect(4, start, 1, 5, if alternate { 'e' } else { 'd' });
        canvas.put(4, start, 'f');
    }
    for (x, y) in [(9, 25), (25, 9), (5, 29), (29, 5)] {
        canvas.rivet(x, y);
    }
    canvas.rect(17, 9, 5, 1, 'a');
    canvas.rect(9, 17, 1, 5, 'a');
    canvas.body()
}

fn rail(portrait: bool, vertical: bool) -> PixelBody {
    let (width, height) = if portrait { (32, 4) } else { (64, 8) };
    let mut canvas = Canvas::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let color = if portrait {
                ['w', 'p', 'h', 'b'][y]
            } else {
                ['w', 'p', 'a', 's', 'h', 'd', 'e', 'b'][y]
            };
            canvas.put(x, y, color);
        }
    }
    if !portrait {
        for start in [4, 20, 36, 52] {
            canvas.rect(start, 4, 8, 2, 'b');
            canvas.rect(start + 1, 4, 6, 1, 's');
            canvas.rect(start + 2, 6, 5, 1, 'd');
            canvas.put(start + 2, 6, 'f');
        }
        // A deliberate machining seam and gold keeper break the straight rail.
        canvas.rect(30, 1, 2, 3, 'h');
        canvas.put(31, 1, 'a');
    }
    if vertical {
        let mut rotated = Canvas::new(height, width);
        for y in 0..height {
            for x in 0..width {
                rotated.put(y, x, canvas.cells[y * width + x]);
            }
        }
        rotated.body()
    } else {
        canvas.body()
    }
}

fn portrait_corner() -> PixelBody {
    let mut canvas = Canvas::new(32, 32);
    for y in 0..32 {
        for x in 0..32 {
            if x + y < 8 || (x >= 4 && y >= 4 && x + y > 18) {
                continue;
            }
            canvas.put(x, y, ['w', 'p', 'h', 'b'][x.min(y).min(3)]);
        }
    }
    canvas.rivet(5, 9);
    canvas.rivet(9, 5);
    canvas.rect(2, 15, 1, 9, 'd');
    canvas.put(2, 16, 'f');
    canvas.rect(15, 2, 9, 1, 'e');
    canvas.put(16, 2, 'l');
    canvas.body()
}

fn socket(slot: usize) -> PixelBody {
    let mut canvas = Canvas::new(48, 48);
    for y in 0..48 {
        for x in 0..48 {
            let cut = x + y >= 7 && x + 47 - y >= 7 && 47 - x + y >= 7 && 94 - x - y >= 7;
            if !cut {
                continue;
            }
            let edge = x.min(y).min(47 - x).min(47 - y);
            let diagonal = (x + y).min(x + 47 - y).min(47 - x + y).min(94 - x - y);
            if edge < 4 || diagonal < 11 {
                let depth = edge.min((diagonal - 7) / 2).min(3);
                let top_light = x + y < 48;
                let color = if top_light {
                    ['w', 'p', 'a', 'h'][depth]
                } else {
                    ['p', 's', 'h', 'b'][depth]
                };
                canvas.put(x, y, color);
            }
        }
    }
    // Four warm keeper assemblies over the recessed wall, no champion-derived art.
    for y in [15, 30] {
        for x in [3, 43] {
            canvas.rect(x, y, 2, 4, 'b');
            canvas.rect(x, y, 1, 3, 'a');
            canvas.put(x, y, 'w');
            canvas.put(x + 1, y + 2, 'e');
        }
    }
    canvas.rect(19, 2, 10, 2, 'b');
    canvas.rect(20, 2, 8, 1, 'd');
    canvas.put(22, 2, 'f');
    canvas.rect(19, 44, 10, 2, 'b');
    for bit in 0..3 {
        canvas.put(
            21 + bit * 2,
            44,
            if slot & (1 << bit) != 0 { 'e' } else { 'h' },
        );
    }
    canvas.body()
}

fn seal(corporation: usize) -> PixelBody {
    let mut canvas = Canvas::new(48, 48);
    for y in 0..48 {
        for x in 0..48 {
            let dx = x as f32 - 23.5;
            let dy = y as f32 - 23.5;
            let ring = (dx * dx + dy * dy).sqrt();
            let mark = match corporation {
                // PAN: orbit with a long asymmetric orbital vane and seed star.
                0 => {
                    ((16.0..18.0).contains(&ring) && dy.abs() < 11.0)
                        || dx.abs() + dy.abs() * 2.0 < 12.0
                        || dy.abs() + dx.abs() * 3.0 < 16.0
                        || ((dy + dx * 0.23).abs() < 1.4 && dx.abs() < 20.0)
                }
                // RENKO: open optical reticle and a bevelled diagonal vane.
                1 => {
                    ((14.0..17.0).contains(&ring) && dx.abs() > 4.0 && dy.abs() > 4.0)
                        || ((dx + dy).abs() < 2.5 && ring < 18.0)
                }
                // HUGO: wide mechanical shield shoulders around an inset core.
                _ => {
                    ((-15.0..13.0).contains(&dy)
                        && (dx.abs() - (15.0 - (dy + 4.0).max(0.0) * 0.65)).abs() < 2.0)
                        || (dy.abs() < 2.5 && dx.abs() < 9.0)
                        || (dy > -15.0 && dy < -12.0 && dx.abs() < 15.0)
                }
            };
            if mark {
                canvas.put(x, y, if x + y < 44 { 'l' } else { 'e' });
            }
        }
    }
    // Segmented mechanical locator pins keep the seal distinct from a flat logo.
    for (x, y) in [(23, 3), (3, 23), (43, 23), (23, 43)] {
        canvas.rect(x, y, 2, 2, 'a');
        canvas.put(x, y, 'w');
    }
    canvas.put(23, 23, 'f');
    canvas.put(24, 23, 'f');
    canvas.body()
}

fn portrait_backing() -> PixelBody {
    let mut canvas = Canvas::new(72, 96);
    for (x, y) in [
        (7, 9),
        (19, 17),
        (58, 8),
        (65, 27),
        (51, 34),
        (8, 40),
        (63, 56),
        (16, 67),
        (55, 77),
        (8, 85),
        (64, 89),
        (34, 6),
        (42, 91),
    ] {
        canvas.put(x, y, 't');
        if x % 2 == 0 {
            for (xx, yy) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                canvas.put(xx, yy, 't');
            }
            canvas.put(x, y, 'v');
        }
    }
    // A broken orbit gives the backing structure without filling it with noise.
    for y in 8..48 {
        for x in 42..68 {
            let dx = x as f32 - 49.0;
            let dy = y as f32 - 31.0;
            let distance = (dx * dx + dy * dy).sqrt();
            if (17.0..17.7).contains(&distance) && !(23..32).contains(&y) {
                canvas.put(x, y, 't');
            }
        }
    }
    canvas.body()
}

struct Assets {
    unit: PixelBody,
    plates: [PixelBody; 2],
    portrait_corner: PixelBody,
    rails: [PixelBody; 2],
    portrait_rails: [PixelBody; 2],
    portrait_backing: PixelBody,
    sockets: [PixelBody; 8],
    seals: [PixelBody; 3],
}
static ART: LazyLock<Assets> = LazyLock::new(|| Assets {
    unit: PixelBody::from_ascii(&["b"], PALETTE),
    plates: [chassis_corner(false), chassis_corner(true)],
    portrait_corner: portrait_corner(),
    rails: [rail(false, false), rail(false, true)],
    portrait_rails: [rail(true, false), rail(true, true)],
    portrait_backing: portrait_backing(),
    sockets: std::array::from_fn(socket),
    seals: std::array::from_fn(seal),
});

pub(super) struct Input<'a> {
    pub kind: &'a str,
    pub rect: [f32; 4],
    pub parent: AffineTransform,
    pub clip: Option<[f32; 4]>,
    pub value: f32,
    pub earned: bool,
    pub time: f32,
    pub reduced: bool,
}
#[derive(Clone, Copy)]
struct Part {
    body: &'static PixelBody,
    local: AffineTransform,
    shade: Option<u32>,
}

fn shade(original: u32, corporation: usize, earned: bool, phase: f32) -> u32 {
    if original == DECK {
        return [rgb(6, 11, 10), rgb(12, 8, 10), rgb(7, 10, 14)][corporation];
    }
    let accent = [
        [111.0, 201.0, 137.0],
        [227.0, 123.0, 117.0],
        [106.0, 176.0, 231.0],
    ][corporation];
    let strength = match original {
        ENAMEL_DARK => 0.35,
        ENAMEL => {
            if earned {
                0.85
            } else {
                0.55
            }
        }
        ENAMEL_LIGHT => 1.0,
        FOIL => 0.80 + phase * 0.05,
        _ => return original,
    };
    u32::from_le_bytes([
        (accent[0] * strength) as u8,
        (accent[1] * strength) as u8,
        (accent[2] * strength) as u8,
        255,
    ])
}

/// Returns submitted cells, including cells hidden by clipping. No frame-time heap work.
pub(super) fn append(
    frame: &mut FrameBuilder,
    input: Input<'_>,
    remaining: &mut usize,
    work: &mut usize,
) -> usize {
    let r = input.rect;
    if r.iter().any(|v| !v.is_finite() || v.abs() > 8192.0)
        || r[2] <= 0.0
        || r[3] <= 0.0
        || !input.parent.position.is_finite()
        || input.parent.position.abs().max_element() > 1_100_000.0
        || !input.parent.basis_x.is_finite()
        || !input.parent.basis_y.is_finite()
        || input.parent.basis_x.abs().max_element() > 64.0
        || input.parent.basis_y.abs().max_element() > 64.0
    {
        return 0;
    }
    if input
        .clip
        .is_some_and(|c| c.iter().any(|v| !v.is_finite()) || c[2] <= 0.0 || c[3] <= 0.0)
    {
        return 0;
    }
    let valid = |value: f32, count: usize| {
        (value.is_finite() && value.fract() == 0.0 && value >= 0.0 && value < count as f32)
            .then_some(value as usize)
    };
    let (corporation, slot) = if input.kind == "meishi-socket" {
        let Some(value) = valid(input.value, 24) else {
            return 0;
        };
        (value / 8, value % 8)
    } else {
        (valid(input.value, 3).unwrap_or(0), 0)
    };
    let mut parts = [None; 24];
    let mut used = 0;
    let mut add = |body: &'static PixelBody, rect: [f32; 4], flip: [f32; 2], color| {
        let local = Transform {
            position: Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5),
            scale: Vec2::new(
                rect[2] / body.width as f32 * flip[0],
                rect[3] / body.height as f32 * flip[1],
            ),
            ..Transform::default()
        }
        .affine();
        parts[used] = Some(Part {
            body,
            local,
            shade: color,
        });
        used += 1;
    };
    match input.kind {
        "meishi-panel" | "meishi-portrait-frame" => {
            let panel = input.kind == "meishi-panel";
            let corner = if panel { 36.0_f32 } else { 10.0_f32 }.min(r[2].min(r[3]) * 0.25);
            let thickness = corner * if panel { 7.0 / 36.0 } else { 0.125 };
            // Wider rails overlap the long corner arms. The corner's authored
            // bevel, crystals and catches render over the joint; no gap or
            // added decorative submission is needed.
            let rail_start = if panel { corner * 0.5 } else { corner };
            let cut = corner * if panel { 18.0 / 64.0 } else { 8.0 / 32.0 };
            // Near-black portrait aperture remains neutral; corporate enamel
            // belongs to the chassis. Two fill quads preserve the cut corners.
            for rect in [
                [r[0] + cut, r[1], r[2] - cut * 2.0, r[3]],
                [r[0], r[1] + cut, r[2], r[3] - cut * 2.0],
            ] {
                add(
                    &ART.unit,
                    rect,
                    [1.0; 2],
                    Some(if panel { DECK } else { BLACK }),
                );
            }
            if !panel {
                add(&ART.portrait_backing, r, [1.0; 2], None);
            }
            let rails = if panel {
                &ART.rails
            } else {
                &ART.portrait_rails
            };
            for (axis, rect, flip) in [
                (
                    0,
                    [r[0] + rail_start, r[1], r[2] - rail_start * 2.0, thickness],
                    [1.0, 1.0],
                ),
                (
                    0,
                    [
                        r[0] + rail_start,
                        r[1] + r[3] - thickness,
                        r[2] - rail_start * 2.0,
                        thickness,
                    ],
                    [1.0, -1.0],
                ),
                (
                    1,
                    [r[0], r[1] + rail_start, thickness, r[3] - rail_start * 2.0],
                    [1.0, 1.0],
                ),
                (
                    1,
                    [
                        r[0] + r[2] - thickness,
                        r[1] + rail_start,
                        thickness,
                        r[3] - rail_start * 2.0,
                    ],
                    [-1.0, 1.0],
                ),
            ] {
                add(&rails[axis], rect, flip, None);
            }
            for (i, (x, y, flip)) in [
                (r[0], r[1], [1.0, 1.0]),
                (r[0] + r[2] - corner, r[1], [-1.0, 1.0]),
                (r[0], r[1] + r[3] - corner, [1.0, -1.0]),
                (r[0] + r[2] - corner, r[1] + r[3] - corner, [-1.0, -1.0]),
            ]
            .into_iter()
            .enumerate()
            {
                add(
                    if panel {
                        &ART.plates[i % 2]
                    } else {
                        &ART.portrait_corner
                    },
                    [x, y, corner, corner],
                    flip,
                    None,
                );
            }
        }
        "meishi-socket" => {
            let size = r[2].min(r[3]);
            let x = r[0] + (r[2] - size) * 0.5;
            let y = r[1] + (r[3] - size) * 0.5;
            let fit = size / 48.0;
            add(
                &ART.unit,
                [x + fit * 7.0, y, size - fit * 14.0, size],
                [1.0; 2],
                Some(BLACK),
            );
            add(
                &ART.unit,
                [x, y + fit * 7.0, size, size - fit * 14.0],
                [1.0; 2],
                Some(BLACK),
            );
            add(&ART.sockets[slot], [x, y, size, size], [1.0; 2], None);
        }
        "meishi-seal" => {
            let size = r[2].min(r[3]);
            add(
                &ART.seals[corporation],
                [
                    r[0] + (r[2] - size) * 0.5,
                    r[1] + (r[3] - size) * 0.5,
                    size,
                    size,
                ],
                [1.0; 2],
                None,
            );
        }
        _ => return 0,
    }
    let count: usize = parts[..used]
        .iter()
        .flatten()
        .map(|p| p.body.occupied_count())
        .sum();
    if count > *remaining || count > *work {
        return 0;
    }
    // Validate composed rectangular source corners before appending any part.
    for part in parts[..used].iter().flatten() {
        let transform = input.parent.compose(part.local);
        for x in [-0.5, 0.5] {
            for y in [-0.5, 0.5] {
                let corner = transform.point(Vec2::new(
                    x * part.body.width as f32,
                    y * part.body.height as f32,
                ));
                if !corner.is_finite() || corner.abs().max_element() > 1_100_000.0 {
                    return 0;
                }
            }
        }
    }
    let phase = if input.reduced || !input.time.is_finite() {
        0.0
    } else {
        (input.time * 1.4).sin()
    };
    let clip = input.clip.unwrap_or(r);
    // Every cached part is fitted inside r by construction. When that entire
    // rectangle is admitted, avoid classifying a boundary source cell using a
    // second, rounded f32 reconstruction. A genuine partial clip still checks
    // all four local corners before applying the parent affine.
    let fully_contained = clip[0] <= r[0]
        && clip[1] <= r[1]
        && clip[0] + clip[2] >= r[0] + r[2]
        && clip[1] + clip[3] >= r[1] + r[3];
    for part in parts[..used].iter().flatten() {
        frame.push_body_affine_colored(
            part.body,
            input.parent.compose(part.local),
            |cell, original| {
                let outside = !fully_contained
                    && [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::splat(0.5),
                    ]
                    .into_iter()
                    .map(|corner| part.local.point(cell + corner))
                    .any(|p| {
                        p.x < clip[0]
                            || p.y < clip[1]
                            || p.x > clip[0] + clip[2]
                            || p.y > clip[1] + clip[3]
                    });
                if outside {
                    0
                } else {
                    shade(
                        part.shade.unwrap_or(original),
                        corporation,
                        input.earned,
                        phase,
                    )
                }
            },
        );
    }
    *remaining -= count;
    *work -= count;
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_meishi_source_counts_and_anonymous_socket_floor_are_bounded() {
        println!(
            "MEISHI plates {:?}; portrait_corner {}; seals {:?}; sockets {:?}",
            ART.plates.each_ref().map(PixelBody::occupied_count),
            ART.portrait_corner.occupied_count(),
            ART.seals.each_ref().map(PixelBody::occupied_count),
            ART.sockets.each_ref().map(PixelBody::occupied_count)
        );
        assert!(ART.plates.iter().all(|b| b.occupied_count() <= 950));
        assert_eq!(ART.portrait_corner.occupied_count(), 270);
        assert_eq!(
            ART.rails.each_ref().map(PixelBody::occupied_count),
            [512; 2]
        );
        assert_eq!(
            ART.portrait_rails.each_ref().map(PixelBody::occupied_count),
            [128; 2]
        );
        assert!(ART.portrait_backing.occupied_count() < 100);
        assert!(ART.seals.iter().all(|b| b.occupied_count() < 400));
        for body in &ART.sockets {
            assert_eq!([body.width, body.height], [48, 48]);
            assert_eq!(body.occupied_count(), 632);
            // The centered75% aperture is clear; a66.667% emblem leaves
            // additional negative space around the full bevel and catches.
            for y in 6..42 {
                for x in 6..42 {
                    assert!(!body.is_occupied(x, y));
                }
            }
        }
    }

    #[test]
    fn meishi_socket_has_opaque_anonymous_floor_and_rejects_unknown_tokens() {
        for value in [0.0, 8.0, 16.0] {
            let mut frame = FrameBuilder::new();
            let mut pixels = 60000;
            let mut work = 120000;
            let rect = [7.125, 9.375, 22.5, 22.5];
            assert_eq!(
                append(
                    &mut frame,
                    Input {
                        kind: "meishi-socket",
                        rect,
                        parent: AffineTransform::default(),
                        clip: None,
                        value,
                        earned: false,
                        time: 0.0,
                        reduced: true
                    },
                    &mut pixels,
                    &mut work
                ),
                634
            );
            let center = Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5);
            let covering: Vec<_> = frame
                .frame()
                .batches
                .iter()
                .flat_map(|batch| {
                    batch.pixels.iter().filter_map(|p| {
                        let t = batch.transforms[p.transform_slot()];
                        let min = t.point(Vec2::from_array(p.local) - Vec2::splat(0.5));
                        let max = t.point(Vec2::from_array(p.local) + Vec2::splat(0.5));
                        (center.x >= min.x
                            && center.x <= max.x
                            && center.y >= min.y
                            && center.y <= max.y)
                            .then_some(p.color)
                    })
                })
                .collect();
            assert!(!covering.is_empty());
            assert!(covering.iter().all(|&color| color == BLACK));
        }
        for value in [-1.0, 24.0, 0.5, f32::NAN, f32::INFINITY] {
            let mut frame = FrameBuilder::new();
            let mut pixels = 60000;
            let mut work = 120000;
            assert_eq!(
                append(
                    &mut frame,
                    Input {
                        kind: "meishi-socket",
                        rect: [0.0, 0.0, 32.0, 32.0],
                        parent: AffineTransform::default(),
                        clip: None,
                        value,
                        earned: true,
                        time: 0.0,
                        reduced: false
                    },
                    &mut pixels,
                    &mut work
                ),
                0
            );
            assert_eq!(frame.frame().pixel_count, 0);
            assert_eq!((pixels, work), (60000, 120000));
        }
    }

    fn extract(
        kind: &str,
        value: f32,
        rect: [f32; 4],
        time: f32,
        reduced: bool,
    ) -> engine_core::RenderFrame {
        let mut frame = FrameBuilder::new();
        let (mut pixels, mut work) = (60_000, 120_000);
        let count = append(
            &mut frame,
            Input {
                kind,
                rect,
                parent: AffineTransform::default(),
                clip: None,
                value,
                earned: true,
                time,
                reduced,
            },
            &mut pixels,
            &mut work,
        );
        assert!(count > 0);
        assert_eq!(frame.frame().pixel_count, count);
        assert_eq!((pixels, work), (60_000 - count, 120_000 - count));
        frame.finish()
    }

    type PacketCell = ([f32; 2], u32, [f32; 4], [f32; 4]);
    fn packet(frame: &engine_core::RenderFrame) -> Vec<PacketCell> {
        frame
            .batches
            .iter()
            .flat_map(|batch| {
                batch.pixels.iter().map(|p| {
                    let t = batch.transforms[p.transform_slot()];
                    (p.local, p.color, t.a, t.b)
                })
            })
            .collect()
    }

    #[test]
    fn authored_crystal_bevel_keeper_and_negative_space_landmarks_are_real_cells() {
        let plate = &ART.plates[0];
        assert_eq!([plate.width, plate.height], [64, 64]);
        assert_eq!(plate.color_at(18, 0), Some(LIGHT));
        assert_eq!(plate.color_at(20, 1), Some(PLATINUM));
        assert_eq!(plate.color_at(14, 13), Some(ENAMEL_LIGHT));
        assert_eq!(plate.color_at(13, 15), Some(ENAMEL_DARK));
        assert_eq!(plate.color_at(19, 14), Some(ENAMEL));
        assert_eq!(plate.color_at(9, 25), Some(LIGHT));
        assert_eq!(plate.color_at(17, 9), Some(WARM_ALLOY));
        assert!(!plate.is_occupied(0, 0));
        assert!(!plate.is_occupied(48, 48));
        // All four corners fit the desktop10.8logical content gutter. The
        // long arms extend36logical, but their perpendicular depth is smaller.
        let maximum_diagonal_intrusion = plate
            .cells_indexed()
            .map(|(i, _, _)| ((i % 64).min(i / 64) + 1) as f32 * 36.0 / 64.0)
            .fold(0.0_f32, f32::max);
        assert!(maximum_diagonal_intrusion <= 10.6875);
        let socket = &ART.sockets[0];
        assert_eq!(socket.color_at(12, 0), Some(LIGHT));
        assert_eq!(socket.color_at(12, 1), Some(PLATINUM));
        assert_eq!(socket.color_at(12, 2), Some(WARM_ALLOY));
        assert_eq!(socket.color_at(3, 15), Some(LIGHT));
        assert_eq!(socket.color_at(3, 16), Some(WARM_ALLOY));
        assert_eq!(socket.color_at(22, 2), Some(FOIL));
        assert!(!socket.is_occupied(24, 24));
        assert!(!socket.is_occupied(0, 0));
        assert!(
            ART.portrait_backing
                .cells()
                .all(|(_, c)| c == STAR_DIM || c == STAR_LIGHT)
        );
        assert!(ART.portrait_backing.color_at(24, 48).is_none());
        for color in [
            BLACK,
            GRAPHITE,
            STEEL,
            PLATINUM,
            LIGHT,
            WARM_ALLOY,
            SHADOW_METAL,
            STAR_DIM,
            STAR_LIGHT,
        ] {
            for corporation in 0..3 {
                for phase in [-1.0, 0.0, 1.0] {
                    assert_eq!(shade(color, corporation, true, phase), color);
                }
            }
        }
    }

    #[test]
    fn all_fittings_preflight_both_quotas_and_invalid_affines_atomically() {
        for kind in [
            "meishi-panel",
            "meishi-portrait-frame",
            "meishi-socket",
            "meishi-seal",
        ] {
            let rect = [7.125, 9.375, 173.25, 219.75];
            let count = extract(kind, 0.0, rect, 0.0, true).pixel_count;
            for (mut pixels, mut work) in [(count - 1, 120_000), (60_000, count - 1)] {
                let before = (pixels, work);
                let mut frame = FrameBuilder::new();
                assert_eq!(
                    append(
                        &mut frame,
                        Input {
                            kind,
                            rect,
                            parent: AffineTransform::default(),
                            clip: None,
                            value: 0.0,
                            earned: true,
                            time: 0.0,
                            reduced: true,
                        },
                        &mut pixels,
                        &mut work
                    ),
                    0
                );
                assert_eq!((pixels, work), before);
                assert!(frame.frame().batches.is_empty());
            }
            let mut frame = FrameBuilder::new();
            let (mut pixels, mut work) = (60_000, 120_000);
            assert_eq!(
                append(
                    &mut frame,
                    Input {
                        kind,
                        rect,
                        parent: AffineTransform {
                            position: Vec2::splat(1_099_999.0),
                            basis_x: Vec2::new(64.0, 64.0),
                            basis_y: Vec2::new(64.0, -64.0)
                        },
                        clip: None,
                        value: 0.0,
                        earned: true,
                        time: 0.0,
                        reduced: true,
                    },
                    &mut pixels,
                    &mut work
                ),
                0
            );
            assert_eq!((pixels, work), (60_000, 120_000));
            assert!(frame.frame().batches.is_empty());
        }
    }

    #[test]
    fn fractional_fit_retains_source_cells_and_real_clip_uses_all_four_corners() {
        for size in [4.8, 22.5, 32.0, 73.5] {
            let rect = [7.125, 9.375, size, size];
            let frame = extract("meishi-socket", 0.0, rect, 0.0, true);
            assert_eq!(frame.pixel_count, 634);
            assert!(
                frame
                    .batches
                    .iter()
                    .flat_map(|b| &b.pixels)
                    .all(|p| p.color >> 24 == 255)
            );
        }
        let rect = [7.125, 9.375, 48.0, 48.0];
        let clip = [7.125, 9.375, 24.0, 25.0];
        let parent = AffineTransform {
            position: Vec2::new(120.0, -50.0),
            basis_x: Vec2::new(-1.25, 0.25),
            basis_y: Vec2::new(0.5, 0.75),
        };
        let baseline = extract("meishi-socket", 0.0, rect, 0.0, true);
        let mut clipped = FrameBuilder::new();
        let (mut pixels, mut work) = (60_000, 120_000);
        assert_eq!(
            append(
                &mut clipped,
                Input {
                    kind: "meishi-socket",
                    rect,
                    parent,
                    clip: Some(clip),
                    value: 0.0,
                    earned: true,
                    time: 0.0,
                    reduced: true
                },
                &mut pixels,
                &mut work
            ),
            634
        );
        let raw: Vec<_> = baseline
            .batches
            .iter()
            .flat_map(|b| {
                b.pixels
                    .iter()
                    .map(|p| (*p, b.transforms[p.transform_slot()]))
            })
            .collect();
        let actual: Vec<_> = clipped
            .frame()
            .batches
            .iter()
            .flat_map(|b| {
                b.pixels
                    .iter()
                    .map(|p| (*p, b.transforms[p.transform_slot()]))
            })
            .collect();
        let mut hidden = 0;
        for ((original, local), (pixel, transformed)) in raw.into_iter().zip(actual) {
            assert_eq!(pixel.local, original.local);
            assert_eq!(&transformed.b[2..], &[0.0, 0.0]);
            let corners = [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::splat(0.5),
            ];
            let outside = corners.iter().any(|corner| {
                let point = local.point(Vec2::from_array(pixel.local) + *corner);
                point.x < clip[0]
                    || point.y < clip[1]
                    || point.x > clip[0] + clip[2]
                    || point.y > clip[1] + clip[3]
            });
            assert_eq!(pixel.color, if outside { 0 } else { original.color });
            hidden += usize::from(outside);
            for corner in corners {
                let point = Vec2::from_array(pixel.local) + corner;
                assert!(
                    transformed
                        .point(point)
                        .distance(parent.point(local.point(point)))
                        < 0.0001
                );
            }
        }
        assert!(hidden > 0 && hidden < 634);
    }

    #[test]
    fn reduced_motion_freezes_foil_and_animation_never_changes_geometry_or_metal() {
        for kind in [
            "meishi-panel",
            "meishi-portrait-frame",
            "meishi-socket",
            "meishi-seal",
        ] {
            let rect = [0.0, 0.0, 192.0, 256.0];
            let value = if kind == "meishi-socket" { 16.0 } else { 2.0 };
            let a = extract(kind, value, rect, 0.0, true);
            let b = extract(kind, value, rect, 100.0, true);
            assert_eq!(packet(&a), packet(&b));
            let live = extract(kind, value, rect, 1.0, false);
            for (static_cell, animated) in packet(&a).into_iter().zip(packet(&live)) {
                assert_eq!(static_cell.0, animated.0);
                assert_eq!(static_cell.2, animated.2);
                assert_eq!(static_cell.3, animated.3);
                if static_cell.1 != animated.1 {
                    let source_foil = shade(FOIL, 2, true, 0.0);
                    assert_eq!(static_cell.1, source_foil);
                }
            }
        }
    }

    #[test]
    fn redesigned_close_only_card_retains_every_component_for_nine_profiles() {
        use crate::ui_scene::{UiGroup, UiNode, UiScene, draw};
        let node = |kind: &str, value, rect| UiNode {
            kind: kind.into(),
            value,
            rect,
            ..Default::default()
        };
        let render = |scene: &UiScene| {
            let mut frame = FrameBuilder::new();
            draw(&mut frame, scene, [0.0; 2], [640.0, 360.0], 0.0, true);
            frame.finish()
        };
        for kind in 0..3 {
            for corp in 0..3 {
                let mut nodes = vec![
                    node("meishi-panel", corp as f32, [5.0, 5.0, 240.0, 324.0]),
                    node(
                        "meishi-portrait-frame",
                        corp as f32,
                        [15.0, 30.0, 220.0, 154.0],
                    ),
                    UiNode {
                        portrait_kind: Some(kind),
                        portrait_main: Some(corp),
                        portrait_sub: Some((corp + 1) % 3),
                        ..node("portrait", 9.0, [20.0, 34.0, 210.0, 146.0])
                    },
                    node("meishi-seal", corp as f32, [205.0, 36.0, 22.0, 22.0]),
                    node("licensed-ship", corp as f32, [18.0, 188.0, 24.0, 24.0]),
                    node("close-icon", 0.0, [215.0, 11.0, 22.0, 22.0]),
                ];
                for (index, text) in [
                    "MEISHI CARD",
                    "APR#0123456789ABCDEF",
                    "FACTORY RAICHO LV 1",
                    "RENKO 13 / HUGO 29 / PAN 47",
                ]
                .into_iter()
                .enumerate()
                {
                    nodes.push(UiNode {
                        text: text.into(),
                        font: 0.75,
                        ..node(
                            "text",
                            0.0,
                            [18.0, [15.0, 175.0, 188.0, 210.0][index], 204.0, 9.0],
                        )
                    });
                }
                for slot in 0..8 {
                    let x = 18.0 + (slot % 4) as f32 * 54.0;
                    let y = 232.0 + (slot / 4) as f32 * 44.0;
                    nodes.push(UiNode {
                        selected: true,
                        ..node(
                            "meishi-socket",
                            (corp as usize * 8 + slot) as f32,
                            [x, y, 36.0, 36.0],
                        )
                    });
                    nodes.push(UiNode {
                        active: true,
                        ..node(
                            "campaign-patch",
                            slot as f32,
                            [x + 6.0, y + 6.0, 24.0, 24.0],
                        )
                    });
                }
                let mut expected = Vec::new();
                let mut decoration = 0;
                let mut joined = Vec::new();
                for n in nodes {
                    let decorative = n.kind.starts_with("meishi-");
                    let mut isolated = UiScene {
                        groups: vec![UiGroup {
                            position: [0.0; 2],
                            nodes: vec![n],
                        }],
                    };
                    let part = render(&isolated);
                    assert!(part.pixel_count > 0);
                    if decorative {
                        decoration += part.pixel_count;
                    }
                    expected.extend(packet(&part));
                    joined.push(isolated.groups[0].nodes.remove(0));
                }
                let frame = render(&UiScene {
                    groups: vec![UiGroup {
                        position: [0.0; 2],
                        nodes: joined,
                    }],
                });
                assert!(
                    decoration <= 13_000,
                    "decoration exceeded agreed local target: {decoration}"
                );
                assert!(frame.pixel_count < 60_000);
                assert_eq!(
                    packet(&frame),
                    expected,
                    "whole component lost or changed under shared admission"
                );
                println!(
                    "REFINED MEISHI kind{kind}/corp{corp}: decoration {decoration}, complete {}, transforms {} /60000",
                    frame.pixel_count,
                    frame
                        .batches
                        .iter()
                        .map(|b| b.transforms.len())
                        .sum::<usize>()
                );
                if let Some(directory) = std::env::var_os("GALAXY_MEISHI_REVIEW_OUTPUT") {
                    let output = std::path::PathBuf::from(directory);
                    std::fs::create_dir_all(&output).unwrap();
                    let batches: Vec<_> = frame.batches.iter().map(|b| serde_json::json!({
                        "transforms": b.transforms.iter().map(|t| serde_json::json!({"a":t.a,"b":t.b})).collect::<Vec<_>>(),
                        "pixels": b.pixels.iter().map(|p| serde_json::json!({"local":p.local,"color":p.color,"transform":p.transform})).collect::<Vec<_>>()
                    })).collect();
                    let value = serde_json::json!({"scope":"actual native UiScene extraction; no GPU observation", "kind":kind,"corporation":corp,
                        "pixel_count":frame.pixel_count,"decoration":decoration,"batches":batches});
                    std::fs::write(
                        output.join(format!("kind{kind}-corp{corp}.json")),
                        serde_json::to_vec(&value).unwrap(),
                    )
                    .unwrap();
                }
            }
        }
    }
}
