//! Original packed-cell collectible fittings. DOM owns layout and hit targets.
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Transform, Vec2};
use std::sync::LazyLock;

const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    u32::from_le_bytes([r, g, b, 255])
}
const BLACK: u32 = rgb(4, 8, 11);
const GRAPHITE: u32 = rgb(23, 32, 38);
const STEEL: u32 = rgb(76, 90, 96);
const PLATINUM: u32 = rgb(162, 175, 176);
const LIGHT: u32 = rgb(213, 222, 217);
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

fn plate(size: usize, alternate: bool) -> PixelBody {
    let mut canvas = Canvas::new(size, size);
    // Cut outer corner, layered platinum L, recessed graphite, enamel inlay.
    for y in 0..size {
        for x in 0..size {
            if x + y < 6 || (x >= 6 && y >= 6 && x + y > 17) {
                continue;
            }
            let color = match x.min(y) {
                0 => 'w',
                1 => 'p',
                2 => 's',
                3 => 'g',
                4 => 'p',
                _ => 's',
            };
            canvas.put(x, y, color);
        }
    }
    for n in 8..size - 2 {
        canvas.put(3, n, if n % 5 == 0 { 'f' } else { 'e' });
        canvas.put(n, 3, if alternate && n % 4 == 0 { 'f' } else { 'd' });
    }
    for (x, y) in [(7, 7), (8, 7), (7, 8), (11, 4), (4, 11)] {
        canvas.put(x, y, 'b');
    }
    canvas.put(8, 8, 'w');
    if alternate {
        for n in 9..14 {
            canvas.put(n, 13, if n % 2 == 0 { 'p' } else { 'g' });
        }
    }
    canvas.body()
}

fn socket(slot: usize) -> PixelBody {
    let mut canvas = Canvas::new(40, 40);
    for y in 0..40 {
        for x in 0..40 {
            let cut = x + y >= 6 && x + 39 - y >= 6 && 39 - x + y >= 6 && 78 - x - y >= 6;
            if !cut {
                continue;
            }
            let edge = x.min(y).min(39 - x).min(39 - y);
            if edge < 2 {
                canvas.put(x, y, if edge == 0 { 'p' } else { 's' });
            } else if edge == 3 && (!(8..=31).contains(&x) || !(8..=31).contains(&y)) {
                canvas.put(x, y, if x + y < 40 { 'w' } else { 'g' });
            }
        }
    }
    // Enamel catches in the recessed side walls; no champion-derived imagery.
    for y in [12, 13, 25, 26] {
        for x in [2, 37] {
            canvas.put(x, y, 'e');
        }
    }
    for bit in 0..3 {
        canvas.put(
            17 + bit * 2,
            37,
            if slot & (1 << bit) != 0 { 'e' } else { 'g' },
        );
    }
    for x in 16..24 {
        canvas.put(x, 2, if x == 19 || x == 20 { 'f' } else { 'd' });
    }
    canvas.body()
}

fn seal(corporation: usize) -> PixelBody {
    let mut canvas = Canvas::new(32, 32);
    for y in 0..32 {
        for x in 0..32 {
            let dx = x as f32 - 15.5;
            let dy = y as f32 - 15.5;
            let ring = (dx * dx + dy * dy).sqrt();
            let mark = match corporation {
                // PAN: orbital flow and a four-point seed.
                0 => {
                    ((11.0..12.5).contains(&ring) && dy.abs() < 8.0)
                        || dx.abs() + dy.abs() * 2.0 < 8.0
                        || dy.abs() + dx.abs() * 3.0 < 11.0
                }
                // RENKO: open reticle and a diagonal optical vane.
                1 => {
                    ((9.0..11.0).contains(&ring) && dx.abs() > 3.0 && dy.abs() > 3.0)
                        || ((dx + dy).abs() < 2.0 && ring < 12.0)
                }
                // HUGO: shield shoulders converge on an inset core.
                _ => {
                    ((-10.0..8.0).contains(&dy)
                        && (dx.abs() - (10.0 - (dy + 3.0).max(0.0) * 0.65)).abs() < 1.5)
                        || (dy.abs() < 2.0 && dx.abs() < 6.0)
                        || (dy > -10.0 && dy < -8.0 && dx.abs() < 10.0)
                }
            };
            if mark {
                canvas.put(x, y, if x + y < 30 { 'l' } else { 'e' });
            }
        }
    }
    canvas.put(15, 15, 'f');
    canvas.put(16, 15, 'f');
    canvas.body()
}

struct Assets {
    unit: PixelBody,
    plates: [PixelBody; 2],
    portrait_corner: PixelBody,
    sockets: [PixelBody; 8],
    seals: [PixelBody; 3],
}
static ART: LazyLock<Assets> = LazyLock::new(|| Assets {
    unit: PixelBody::from_ascii(&["b"], PALETTE),
    plates: [plate(24, false), plate(24, true)],
    portrait_corner: plate(16, false),
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
            let corner = if panel { 12.0_f32 } else { 6.0_f32 }.min(r[2].min(r[3]) * 0.25);
            let thickness = corner / 12.0;
            if panel {
                // Quiet opaque deck, clipped external corners, never a dense texture grid.
                add(
                    &ART.unit,
                    [r[0] + corner * 0.25, r[1], r[2] - corner * 0.5, r[3]],
                    [1.0; 2],
                    Some(BLACK),
                );
                add(
                    &ART.unit,
                    [r[0], r[1] + corner * 0.25, r[2], r[3] - corner * 0.5],
                    [1.0; 2],
                    Some(BLACK),
                );
            }
            for rect in [
                [
                    r[0] + corner,
                    r[1] + thickness,
                    r[2] - corner * 2.0,
                    thickness,
                ],
                [
                    r[0] + corner,
                    r[1] + r[3] - thickness * 2.0,
                    r[2] - corner * 2.0,
                    thickness,
                ],
                [
                    r[0] + thickness,
                    r[1] + corner,
                    thickness,
                    r[3] - corner * 2.0,
                ],
                [
                    r[0] + r[2] - thickness * 2.0,
                    r[1] + corner,
                    thickness,
                    r[3] - corner * 2.0,
                ],
            ] {
                add(&ART.unit, rect, [1.0; 2], Some(PLATINUM));
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
            let width = (r[2] - corner * 2.0).min(if panel { 28.0 } else { 10.0 });
            for (offset, color) in [(2.0, ENAMEL_DARK), (3.0, ENAMEL), (4.0, FOIL)] {
                add(
                    &ART.unit,
                    [r[0] + corner, r[1] + thickness * offset, width, thickness],
                    [1.0; 2],
                    Some(color),
                );
            }
        }
        "meishi-socket" => {
            let size = r[2].min(r[3]);
            let x = r[0] + (r[2] - size) * 0.5;
            let y = r[1] + (r[3] - size) * 0.5;
            let fit = size / 40.0;
            add(
                &ART.unit,
                [x + fit * 6.0, y, size - fit * 12.0, size],
                [1.0; 2],
                Some(BLACK),
            );
            add(
                &ART.unit,
                [x, y + fit * 6.0, size, size - fit * 12.0],
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
    for part in parts[..used].iter().flatten() {
        frame.push_body_affine_colored(
            part.body,
            input.parent.compose(part.local),
            |cell, original| {
                let outside = [
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
        assert!(ART.plates.iter().all(|b| b.occupied_count() < 300));
        assert!(ART.portrait_corner.occupied_count() < 200);
        assert!(ART.seals.iter().all(|b| b.occupied_count() < 350));
        for body in &ART.sockets {
            assert_eq!([body.width, body.height], [40, 40]);
            assert!(body.occupied_count() <= 400);
            for y in 4..36 {
                for x in 4..36 {
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
                385
            );
            let center = Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5);
            let covering: Vec<_> = frame
                .frame()
                .batches
                .iter()
                .flat_map(|batch| {
                    batch.pixels.iter().filter_map(|p| {
                        let t = batch.transforms[p.transform as usize];
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
}
