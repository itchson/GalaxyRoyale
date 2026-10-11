//! Screen-space HUD built from the same affine pixel quads as the world.
//! Coordinates are logical pixels; DOM controls retain interaction/accessibility.
use engine_core::{AffineTransform, CameraView, FrameBuilder, PixelBody, Transform, Vec2};
use game_core::Hud;
use std::sync::LazyLock;

const INK: u32 = 0xef_24130b;
const EDGE: u32 = 0xff_856548;
const DIM: u32 = 0xff_a48563;
const CYAN: u32 = 0xff_f4e887;
const WHITE: u32 = 0xff_f6ead9;
const GOLD: u32 = 0xff_85d4ff;
const RED: u32 = 0xff_8890ff;
const MINT: u32 = 0xff_b5f0a1;
const MAGENTA: u32 = 0xff_cf69f4;
static CELL: LazyLock<PixelBody> =
    LazyLock::new(|| PixelBody::from_ascii(&["#"], &[('#', 0xff_ffffff)]));

/// RoyaleMono: our authored 5×7 fixed-advance pixel font. Every surviving
/// letter is a reusable object-local cell body, rather than a cloud of dots.
pub(crate) struct RoyaleMono;
impl RoyaleMono {
    pub const WIDTH: f32 = 5.0;
    pub const HEIGHT: f32 = 7.0;
    pub const ADVANCE: f32 = 6.0;

    pub fn body(character: char) -> &'static PixelBody {
        static BODIES: LazyLock<[PixelBody; 128]> = LazyLock::new(|| {
            std::array::from_fn(|index| {
                let masks = glyph(index as u8 as char);
                let rows: [String; 7] = std::array::from_fn(|y| {
                    (0..5)
                        .map(|x| {
                            if masks[y] & (1 << (4 - x)) != 0 {
                                '#'
                            } else {
                                ' '
                            }
                        })
                        .collect()
                });
                let rows: Vec<_> = rows.iter().map(String::as_str).collect();
                PixelBody::from_ascii(&rows, &[('#', WHITE)])
            })
        });
        let character = character.to_ascii_uppercase();
        &BODIES[if character.is_ascii() {
            character as usize
        } else {
            0
        }]
    }

    /// Child translation is measured in unscaled font cells. A text object's
    /// transform owns its size and the UI group owns its screen placement.
    pub fn letter_transform(parent: AffineTransform, column: usize) -> AffineTransform {
        parent.compose(
            Transform {
                position: Vec2::new(
                    column as f32 * Self::ADVANCE + Self::WIDTH * 0.5,
                    Self::HEIGHT * 0.5,
                ),
                ..Transform::default()
            }
            .affine(),
        )
    }
}

struct Ui<'a> {
    frame: &'a mut FrameBuilder,
    origin: Vec2,
}
impl Ui<'_> {
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: u32) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        self.frame.push_body_colored(
            &CELL,
            Transform {
                position: self.origin + Vec2::new(x + w * 0.5, y + h * 0.5),
                scale: Vec2::new(w, h),
                ..Transform::default()
            },
            |_, _| color,
        );
    }

    fn dot(&mut self, x: f32, y: f32, size: f32, color: u32) {
        self.frame.push_pixel(
            self.origin + Vec2::new(x + size * 0.5, y + size * 0.5),
            size,
            color,
        );
    }

    fn panel(&mut self, x: f32, y: f32, w: f32, h: f32) {
        // A clipped two-step silhouette, with one-pixel inner/outer edges.
        self.rect(x + 3.0, y, w - 6.0, h, INK);
        self.rect(x, y + 3.0, w, h - 6.0, INK);
        for (dy, color) in [(0.0, EDGE), (h - 1.0, DIM)] {
            self.rect(x + 3.0, y + dy, w - 6.0, 1.0, color);
        }
        self.rect(x, y + 3.0, 1.0, h - 6.0, EDGE);
        self.rect(x + w - 1.0, y + 3.0, 1.0, h - 6.0, DIM);
        for (dx, dy) in [
            (1.0, 1.0),
            (w - 3.0, 1.0),
            (1.0, h - 3.0),
            (w - 3.0, h - 3.0),
        ] {
            self.dot(x + dx, y + dy, 2.0, EDGE);
        }
        self.rect(x + 5.0, y + 3.0, 12.0, 1.0, CYAN);
    }

    fn text(&mut self, x: f32, y: f32, text: &str, size: f32, color: u32) {
        self.text_bounded(x, y, text, size, color, f32::INFINITY);
    }

    fn text_bounded(&mut self, x: f32, y: f32, text: &str, size: f32, color: u32, width: f32) {
        let columns = ((width + size) / (RoyaleMono::ADVANCE * size)).floor() as usize;
        let group = Transform {
            position: self.origin,
            ..Transform::default()
        }
        .affine();
        let text_transform = group.compose(
            Transform {
                position: Vec2::new(x, y),
                scale: Vec2::splat(size),
                ..Transform::default()
            }
            .affine(),
        );
        for (index, character) in text.chars().take(columns.min(256)).enumerate() {
            let body = RoyaleMono::body(character);
            if body.occupied_count() != 0 {
                self.frame.push_body_affine_colored(
                    body,
                    RoyaleMono::letter_transform(text_transform, index),
                    |_, _| color,
                );
            }
        }
    }

    fn icon(&mut self, x: f32, y: f32, category: usize) {
        self.icon_scaled(x, y, category, 2.0);
    }

    fn icon_scaled(&mut self, x: f32, y: f32, category: usize, size: f32) {
        let rows = match category {
            0 => [4, 14, 31, 4, 4],   // weapon/star
            1 => [31, 17, 17, 10, 4], // shield
            _ => [4, 14, 31, 14, 4],  // passive crystal
        };
        for (row, bits) in rows.iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    self.dot(
                        x + column as f32 * size,
                        y + row as f32 * size,
                        size,
                        [RED, CYAN, MINT][category],
                    );
                }
            }
        }
    }

    fn bar(&mut self, x: f32, y: f32, width: f32, value: f32, maximum: f32, color: u32) {
        self.rect(x, y, width, 4.0, 0xff_4d3827);
        let fill = (width * ratio(value, maximum)).floor();
        self.rect(x, y, fill, 4.0, color);
        // Discrete cells keep a readable ten-segment meter at small resolutions.
        for i in 1..10 {
            self.rect(x + (width * i as f32 / 10.0).floor(), y, 1.0, 4.0, INK);
        }
    }

    fn pad(&mut self, center: Vec2, radius: i32, edge: u32, fill: u32) {
        // Rasterized octagon: horizontal two-pixel bands and stepped 45° corners.
        for y in (-radius..radius).step_by(2) {
            let half = radius.min(radius + radius / 2 - (y + 1).abs()) as f32;
            self.rect(center.x - half, center.y + y as f32, half * 2.0, 2.0, fill);
            self.rect(center.x - half, center.y + y as f32, 2.0, 2.0, edge);
            self.rect(center.x + half - 2.0, center.y + y as f32, 2.0, 2.0, edge);
        }
        let cap = (radius / 2 + 1) as f32;
        self.rect(
            center.x - cap,
            center.y - radius as f32,
            cap * 2.0,
            2.0,
            edge,
        );
        self.rect(
            center.x - cap,
            center.y + radius as f32 - 2.0,
            cap * 2.0,
            2.0,
            edge,
        );
    }
}

/// One stack-only source of truth for painted telemetry and boss exclusions.
/// All rectangles are local to the already-inset logical UI viewport.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FlightLayout {
    pub biome: Option<[f32; 4]>,
    pub currency: Option<[f32; 4]>,
    pub radar: Option<[f32; 4]>,
    pub meter: Option<[f32; 4]>,
    pub compact_header: bool,
    pub meter_style: MeterStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MeterStyle {
    Full,
    Compact,
    Micro,
    Slim,
}

/// Actual semantic header art, converted once to the inset HUD coordinates.
/// No assumed CSS hit size or toolbar width is duplicated in the painter.
pub(crate) fn flight_header(
    scene: &crate::ui_scene::UiScene,
    insets: [f32; 4],
) -> Option<[f32; 4]> {
    let [top, _, _, left] = insets;
    if !top.is_finite() || !left.is_finite() {
        return None;
    }
    let mut union: Option<[f32; 4]> = None;
    let mut visited = 0;
    for group in scene.groups.iter().take(128) {
        if group
            .position
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 8192.0)
        {
            continue;
        }
        for node in &group.nodes {
            if visited == 512 {
                return union;
            }
            visited += 1;
            if !matches!(
                node.kind.as_str(),
                "save-icon" | "status-icon" | "map-icon" | "settings-icon"
            ) {
                continue;
            }
            let mut r = node.rect;
            if r.iter().any(|v| !v.is_finite() || v.abs() > 8192.0) || r[2] <= 0.0 || r[3] <= 0.0 {
                continue;
            }
            if let Some(c) = node.clip {
                if c.iter().any(|v| !v.is_finite() || v.abs() > 8192.0)
                    || c[2] <= 0.0
                    || c[3] <= 0.0
                {
                    continue;
                }
                let x = r[0].max(c[0]);
                let y = r[1].max(c[1]);
                r = [
                    x,
                    y,
                    (r[0] + r[2]).min(c[0] + c[2]) - x,
                    (r[1] + r[3]).min(c[1] + c[3]) - y,
                ];
            }
            if r[2] <= 0.0 || r[3] <= 0.0 {
                continue;
            }
            r[0] += group.position[0] - left;
            r[1] += group.position[1] - top;
            union = Some(union.map_or(r, |u| {
                let x = u[0].min(r[0]);
                let y = u[1].min(r[1]);
                [
                    x,
                    y,
                    (u[0] + u[2]).max(r[0] + r[2]) - x,
                    (u[1] + u[3]).max(r[1] + r[3]) - y,
                ]
            }));
        }
    }
    union
}

fn hud_overlap(a: [f32; 4], b: [f32; 4], gap: f32) -> bool {
    a[0] < b[0] + b[2] + gap
        && a[0] + a[2] > b[0] - gap
        && a[1] < b[1] + b[3] + gap
        && a[1] + a[3] > b[1] - gap
}

/// Presentation geometry shared by the native painter, DOM bounds and boss
/// allocator. Rectangles are safe-area local; world projection happens before
/// subtracting the safe origin. No movement, health or camera state is changed.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FlightStatusLayout {
    pub strip: Option<[f32; 4]>,
    pub stats: Option<[f32; 4]>,
    pub context: Option<[f32; 4]>,
    pub vitals: Option<[f32; 4]>,
    pub viewport: [f32; 2],
}

pub struct FlightStatusInput {
    pub viewport: [f32; 2],
    pub insets: [f32; 4],
    pub header: Option<[f32; 4]>,
    pub status_hits: [Option<[f32; 4]>; 2],
    pub touch: bool,
    /// Compact boss allocation may use the footer's H/S readout instead.
    pub attached_vitals: bool,
    pub player: Vec2,
    pub player_radius: f32,
    pub protected_core: Option<(Vec2, f32)>,
    /// Interpolated unshaken world camera, including the current eased zoom.
    pub camera: [f32; 2],
    pub zoom: f32,
}

impl FlightStatusLayout {
    pub fn new(input: &FlightStatusInput) -> Option<Self> {
        if input.insets.iter().any(|v| !v.is_finite() || *v < 0.0)
            || input.viewport.iter().any(|v| !v.is_finite() || *v > 8192.0)
            || !input.player.is_finite()
            || !input.player_radius.is_finite()
            || input.player_radius < 0.0
            || input
                .protected_core
                .is_some_and(|(p, r)| !p.is_finite() || !r.is_finite() || r < 0.0)
            || !(engine_core::MIN_CAMERA_ZOOM..=engine_core::MAX_CAMERA_ZOOM).contains(&input.zoom)
        {
            return None;
        }
        let [top, right, bottom, left] = input.insets;
        let viewport = [
            input.viewport[0] - left - right,
            input.viewport[1] - top - bottom,
        ];
        if viewport[0] < 180.0 || viewport[1] < 140.0 {
            return None;
        }
        let header = input
            .header
            .filter(|r| r.iter().all(|v| v.is_finite()) && r[2] > 0.0 && r[3] > 0.0);
        let hits = touch_hit_rects(viewport);
        let clear = |r: [f32; 4]| {
            r[0] >= 0.0
                && r[1] >= 0.0
                && r[0] + r[2] <= viewport[0]
                && r[1] + r[3] <= viewport[1]
                && header.is_none_or(|h| !hud_overlap(r, h, 2.0))
                && (!input.touch || hits.iter().all(|h| !hud_overlap(r, *h, 2.0)))
        };
        let [width, height] = viewport;
        let central_left = hits[0][0] + hits[0][2] + 4.0;
        let central_width = hits[1][0] - 4.0 - central_left;
        let central = [central_left, height - 28.0, central_width, 22.0];
        let bottom_strip = [6.0, height - 28.0, width - 12.0, 22.0];
        let above_pads = [6.0, hits[0][1].min(hits[1][1]) - 26.0, width - 12.0, 22.0];
        let slim = [6.0, 0.0, width - 12.0, 10.0];
        let strip = if input.touch {
            [
                (central_width >= 120.0).then_some(central),
                Some(above_pads),
                Some(slim),
            ]
        } else {
            [Some(bottom_strip), Some(slim), None]
        }
        .into_iter()
        .flatten()
        .find(|r| clear(*r));
        let stats = strip.map(|r| [r[0] + 4.0, r[1] + 2.0, r[2] - 8.0, 7.0]);
        let context = strip
            .filter(|r| r[3] >= 22.0)
            .map(|r| [r[0] + 4.0, r[1] + 12.0, r[2] - 8.0, 7.0]);
        let view = CameraView::new(
            Vec2::from_array(input.camera),
            Vec2::from_array(input.viewport),
            input.zoom,
        )?;
        let center = view.world_to_screen(input.player) - Vec2::new(left, top);
        let radius = input.player_radius * input.zoom;
        let core = input.protected_core.map(|(p, r)| {
            (
                view.world_to_screen(p) - Vec2::new(left, top),
                r * input.zoom + 4.0,
            )
        });
        // Never clamp an offscreen ship to a false marker at the viewport edge.
        let vitals = (input.attached_vitals
            && center.x >= 0.0
            && center.y >= 0.0
            && center.x <= width
            && center.y <= height)
            .then(|| {
                [
                    [center.x - 22.0, center.y - radius - 14.0, 44.0, 9.0],
                    [center.x - 22.0, center.y + radius + 5.0, 44.0, 9.0],
                ]
                .into_iter()
                .find(|r| {
                    clear(*r)
                        && strip.is_none_or(|s| !hud_overlap(*r, s, 2.0))
                        && input
                            .status_hits
                            .iter()
                            .flatten()
                            .all(|hit| !hud_overlap(*r, *hit, 2.0))
                        && core.is_none_or(|(p, radius)| {
                            let nearest =
                                p.clamp(Vec2::new(r[0], r[1]), Vec2::new(r[0] + r[2], r[1] + r[3]));
                            p.distance_squared(nearest) > radius * radius
                        })
                })
            })
            .flatten();
        Some(Self {
            strip,
            stats,
            context,
            vitals,
            viewport,
        })
    }

    pub fn full_bounds(rect: Option<[f32; 4]>, insets: [f32; 4]) -> Option<[f32; 4]> {
        rect.map(|[x, y, w, h]| [x + insets[3], y + insets[0], w, h])
    }
}

/// Real semantic contact hits are taller than their single-line font slot.
/// Preserve their measured, clipped bounds when avoiding attached ship bars.
pub(crate) fn flight_status_hits(
    scene: &crate::ui_scene::UiScene,
    insets: [f32; 4],
) -> [Option<[f32; 4]>; 2] {
    let mut hits = [None; 2];
    let mut count = 0;
    let mut visited = 0;
    for group in scene.groups.iter().take(128) {
        if group
            .position
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 8192.0)
        {
            continue;
        }
        for node in &group.nodes {
            if visited == 512 || count == 2 {
                return hits;
            }
            visited += 1;
            if node.kind != "status-hit" {
                continue;
            }
            let mut r = node.rect;
            if r.iter().any(|v| !v.is_finite() || v.abs() > 8192.0) || r[2] <= 0.0 || r[3] <= 0.0 {
                continue;
            }
            if let Some(c) = node.clip {
                if c.iter().any(|v| !v.is_finite() || v.abs() > 8192.0)
                    || c[2] <= 0.0
                    || c[3] <= 0.0
                {
                    continue;
                }
                let x = r[0].max(c[0]);
                let y = r[1].max(c[1]);
                r = [
                    x,
                    y,
                    (r[0] + r[2]).min(c[0] + c[2]) - x,
                    (r[1] + r[3]).min(c[1] + c[3]) - y,
                ];
            }
            if r[2] <= 0.0 || r[3] <= 0.0 {
                continue;
            }
            r[0] += group.position[0] - insets[3];
            r[1] += group.position[1] - insets[0];
            hits[count] = Some(r);
            count += 1;
        }
    }
    hits
}

/// Color-only warning cue. Reduced Motion keeps the full readable accent;
/// normal presentation varies by 18%, once a second, without flashing off.
pub(crate) fn warning_color(kind: u8, time: f32, reduced_motion: bool) -> u32 {
    let base = match kind {
        1 => RED,
        2 => GOLD,
        _ => CYAN,
    };
    if reduced_motion || !time.is_finite() || kind == 0 {
        return base;
    }
    let strength = 0.91 + 0.09 * (time * std::f32::consts::TAU).sin();
    let [r, g, b, a] = base.to_le_bytes();
    u32::from_le_bytes([
        (r as f32 * strength) as u8,
        (g as f32 * strength) as u8,
        (b as f32 * strength) as u8,
        a,
    ])
}

/// Small stack formatter for the single-line native footer. UI strings never
/// exceed the font's 256-character bound and extraction allocates no strings.
struct StatusText {
    bytes: [u8; 160],
    len: usize,
}
impl StatusText {
    fn new() -> Self {
        Self {
            bytes: [0; 160],
            len: 0,
        }
    }
    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}
impl std::fmt::Write for StatusText {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        if self.len + text.len() > self.bytes.len() {
            return Err(std::fmt::Error);
        }
        self.bytes[self.len..self.len + text.len()].copy_from_slice(text.as_bytes());
        self.len += text.len();
        Ok(())
    }
}

/// Current flight painter: one docked strip and two compact ship-relative
/// tracks. Detailed information belongs to Ship Status rather than flight.
#[allow(clippy::too_many_arguments)]
pub fn draw_flight_status(
    frame: &mut FrameBuilder,
    hud: &Hud,
    camera: [f32; 2],
    layout: &FlightStatusLayout,
    boss_panel: Option<[f32; 4]>,
    time: f32,
    reduced_motion: bool,
    semantic_context: bool,
) {
    use std::fmt::Write;
    let camera = Vec2::from_array(camera);
    if !camera.is_finite() {
        return;
    }
    let mut ui = Ui {
        frame,
        origin: camera - Vec2::from_array(layout.viewport) * 0.5,
    };
    let low_hull = hud.hull.is_finite()
        && hud.hull > 0.0
        && hud.max_hull.is_finite()
        && hud.max_hull > 0.0
        && ratio(hud.hull, hud.max_hull) <= 0.25;
    let shield_down = hud.hull.is_finite()
        && hud.hull > 0.0
        && hud.max_shield.is_finite()
        && hud.max_shield > 0.0
        && hud.shield.is_finite()
        && hud.shield <= 0.0;
    let warning = if low_hull {
        1
    } else if shield_down {
        2
    } else {
        0
    };
    if let Some([x, y, w, h]) = layout.strip {
        ui.rect(x, y, w, h, 0xee_0e0907);
        ui.rect(x, y, w, 1.0, EDGE);
        ui.rect(x, y, 3.0, 1.0, CYAN);
    }
    if let Some([x, y, w, _]) = layout.stats {
        let mut text = StatusText::new();
        if layout.vitals.is_none() {
            let _ = write!(
                text,
                "H{} S{} ",
                (ratio(hud.hull, hud.max_hull) * 100.0).ceil() as u32,
                (ratio(hud.shield, hud.max_shield) * 100.0).ceil() as u32
            );
        } else if w >= 260.0 {
            let _ = write!(text, "{}  ", hud.biome.name);
        }
        let _ = write!(
            text,
            "L{} C{}/{}",
            hud.ship_level, hud.cargo_materials, hud.cargo_capacity
        );
        let mut points = StatusText::new();
        let _ = points.write_str(" ");
        for (index, value) in hud.currency.into_iter().enumerate() {
            if index > 0 {
                let _ = points.write_str("/");
            }
            let _ = if value >= 1_000_000 {
                write!(points, "{}M", value / 1_000_000)
            } else if value >= 1000 {
                write!(points, "{}K", value / 1000)
            } else {
                write!(points, "{value}")
            };
        }
        // Very narrow rows prioritize essential health, level and carried loot.
        if (text.len + points.len) as f32 * RoyaleMono::ADVANCE - 1.0 <= w {
            let _ = text.write_str(points.as_str());
        }
        ui.text_bounded(
            x,
            y,
            text.as_str(),
            1.0,
            if layout.vitals.is_none() && warning != 0 {
                warning_color(warning, time, reduced_motion)
            } else {
                DIM
            },
            w,
        );
    }
    if !semantic_context && let Some([x, y, w, _]) = layout.context {
        let mut text = StatusText::new();
        let color = if warning != 0 {
            let _ = text.write_str(if warning == 1 {
                "LOW HULL"
            } else {
                "SHIELD DOWN"
            });
            warning_color(warning, time, reduced_motion)
        } else if hud.safe {
            let _ = write!(text, "{} / SAFE", hud.biome.name);
            MINT
        } else {
            let _ = write!(text, "{} / THREAT {}", hud.biome.name, hud.danger);
            GOLD
        };
        ui.text_bounded(x, y, text.as_str(), 1.0, color, w);
    }
    if let Some([x, y, w, _]) = layout.vitals {
        // A three-cell reticle and a shield rune, each cached as a local body.
        static ICONS: LazyLock<[PixelBody; 2]> = LazyLock::new(|| {
            [
                PixelBody::from_ascii(&[".#.", "###", ".#."], &[('#', WHITE)]),
                PixelBody::from_ascii(&["###", "#.#", ".#."], &[('#', WHITE)]),
            ]
        });
        for (row, (value, maximum, color)) in [
            (
                hud.hull,
                hud.max_hull,
                if low_hull {
                    warning_color(1, time, reduced_motion)
                } else {
                    MINT
                },
            ),
            (
                hud.shield,
                hud.max_shield,
                if shield_down {
                    warning_color(2, time, reduced_motion)
                } else {
                    CYAN
                },
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let row_y = y + row as f32 * 4.0;
            ui.frame.push_body_colored(
                &ICONS[row],
                Transform {
                    position: ui.origin + Vec2::new(x + 1.5, row_y + 2.5),
                    ..Transform::default()
                },
                |_, _| color,
            );
            ui.rect(x + 6.0, row_y + 1.0, w - 6.0, 2.0, 0xee_24130b);
            ui.rect(
                x + 6.0,
                row_y + 1.0,
                (w - 6.0) * ratio(value, maximum),
                2.0,
                color,
            );
        }
    }
    draw_boss(&mut ui, hud, Vec2::from_array(layout.viewport), boss_panel);
}

/// Uses the same measured toolbar rectangle for extraction and exclusion.
/// Legacy callers without a scene keep the original layout exactly.
pub(crate) fn flight_layout_with_header(
    hud: &Hud,
    viewport: [f32; 2],
    minimal: bool,
    touch: bool,
    header: Option<[f32; 4]>,
) -> Option<FlightLayout> {
    let mut layout = flight_layout(hud, viewport, minimal, touch)?;
    let Some(header) =
        header.filter(|r| r.iter().all(|v| v.is_finite()) && r[2] > 0.0 && r[3] > 0.0)
    else {
        return Some(layout);
    };
    let [width, height] = viewport;
    let hits = touch_hit_rects(viewport);
    let clear = |r: [f32; 4]| {
        r[0] >= 0.0
            && r[1] >= 0.0
            && r[0] + r[2] <= width
            && r[1] + r[3] <= height
            && !hud_overlap(r, header, 2.0)
            && (!touch || hits.iter().all(|h| !hud_overlap(r, *h, 2.0)))
    };
    if let Some(meter) = layout.meter
        && !clear(meter)
    {
        // Tiny touch layouts use the space above fire and beside the joystick.
        let x = hits[0][0] + hits[0][2] + 2.0;
        let micro = [x, hits[1][1] - 18.0, (width - 12.0 - x).min(176.0), 16.0];
        let slim = [12.0, 0.0, (width - 24.0).min(176.0), 10.0];
        if touch && micro[2] >= 64.0 && clear(micro) {
            layout.meter = Some(micro);
            layout.meter_style = MeterStyle::Micro;
        } else if clear(slim) {
            layout.meter = Some(slim);
            layout.meter_style = MeterStyle::Slim;
        } else {
            // There is no supported essential-meter slot under this supplied
            // geometry. Never paint over real controls to manufacture a fit.
            return None;
        }
    }
    let meter = layout.meter;
    let radar = layout.radar;
    let clear_telemetry = |r: [f32; 4], other: Option<[f32; 4]>| {
        clear(r)
            && meter.is_none_or(|m| !hud_overlap(r, m, 2.0))
            && radar.is_none_or(|m| !hud_overlap(r, m, 2.0))
            && other.is_none_or(|m| !hud_overlap(r, m, 4.0))
    };
    for biome in [true, false] {
        let original = if biome { layout.biome } else { layout.currency };
        let Some(r) = original else {
            continue;
        };
        let other = if biome {
            layout.currency.filter(|c| clear(*c))
        } else {
            layout.biome
        };
        let mut beside = r;
        beside[0] = if biome { 12.0 } else { header[0] - 4.0 - r[2] };
        if biome {
            beside[2] = beside[2].min(header[0] - 16.0);
        }
        let below = [12.0, header[1] + header[3] + 4.0, r[2], r[3]];
        let below_second = [12.0, below[1] + 32.0, r[2], r[3]];
        let chosen = [r, beside, below, below_second]
            .into_iter()
            .find(|c| c[2] >= if biome { 80.0 } else { r[2] } && clear_telemetry(*c, other));
        if biome {
            layout.biome = chosen;
        } else {
            layout.currency = chosen;
        }
    }
    Some(layout)
}

#[cfg(test)]
#[path = "ui_header_tests.rs"]
pub(crate) mod header_tests;

pub(crate) fn touch_hit_rects(viewport: [f32; 2]) -> [[f32; 4]; 2] {
    [
        [
            14.0,
            viewport[1] - 92.0,
            (viewport[0] - 98.0).clamp(0.0, 84.0),
            84.0,
        ],
        [viewport[0] - 84.0, viewport[1] - 86.0, 72.0, 72.0],
    ]
}

pub(crate) fn flight_layout(
    hud: &Hud,
    viewport: [f32; 2],
    minimal: bool,
    touch: bool,
) -> Option<FlightLayout> {
    let [width, height] = viewport;
    if !width.is_finite() || !height.is_finite() || width < 180.0 || height < 140.0 {
        return None;
    }
    let minimal = minimal || width < 220.0;
    let compact_header = width < 392.0;
    let mut layout = FlightLayout {
        biome: None,
        currency: None,
        radar: None,
        meter: None,
        compact_header,
        meter_style: MeterStyle::Full,
    };
    if !minimal {
        if !compact_header {
            layout.biome = Some([12.0, 12.0, 160.0, 28.0]);
            layout.currency = Some([width - 214.0, 12.0, 142.0, 24.0]);
        } else if width >= 300.0 {
            layout.biome = Some([12.0, 12.0, 100.0, 20.0]);
            layout.currency = Some([118.0, 12.0, 110.0, 20.0]);
        } else if !touch || height >= 160.0 {
            layout.biome = Some([12.0, 12.0, (width - 96.0).clamp(80.0, 160.0), 20.0]);
        }
    }
    if !hud.dock || ratio(hud.hull, hud.max_hull) < 1.0 {
        if touch && width < 290.0 {
            // Above the actual hitpads, rather than shrinking below readable
            // bars or painting beneath input regions. The status line is optional.
            layout.meter = Some([12.0, height - 112.0, (width - 24.0).min(176.0), 16.0]);
            layout.meter_style = MeterStyle::Micro;
        } else {
            let meter_width = if touch {
                (width - 190.0).min(176.0)
            } else {
                176.0
            };
            let centered = (width * 0.5 - meter_width * 0.5).floor();
            let x = if touch {
                centered.clamp(102.0, width - 88.0 - meter_width)
            } else {
                centered
            };
            layout.meter = Some([x, height - 48.0, meter_width, 36.0]);
            layout.meter_style = if meter_width < 176.0 {
                MeterStyle::Compact
            } else {
                MeterStyle::Full
            };
        }
    }
    let position = Vec2::from_array(hud.position);
    if !minimal
        && !compact_header
        && height >= 240.0
        && position.is_finite()
        && position.length_squared() > 240.0 * 240.0
    {
        layout.radar = Some([width - 60.0, height - 144.0, 48.0, 48.0]);
    }
    Some(layout)
}

/// Visual touch controls. The DOM handles pointer capture and hit testing.
/// Centers follow the shared hit rectangles; normally [56,h-50] / [w-48,h-50].
/// Joystick width contracts to82..83 at safe widths180..181; radius remains28.
pub fn draw_controls(
    frame: &mut FrameBuilder,
    camera: [f32; 2],
    viewport: [f32; 2],
    joystick: Option<[f32; 2]>,
    fire: bool,
) {
    let camera = Vec2::from_array(camera);
    let viewport = Vec2::from_array(viewport);
    if !camera.is_finite() || !viewport.is_finite() || viewport.x < 180.0 || viewport.y < 140.0 {
        return;
    }
    let mut ui = Ui {
        frame,
        origin: camera - viewport * 0.5,
    };
    let stick = joystick
        .map(Vec2::from_array)
        .filter(|v| v.is_finite() && v.abs().max_element() <= 4.0)
        .unwrap_or(Vec2::ZERO)
        .clamp_length_max(1.0);
    let [joystick_hit, fire_hit] = touch_hit_rects(viewport.to_array());
    let center = Vec2::new(
        joystick_hit[0] + joystick_hit[2] * 0.5,
        joystick_hit[1] + joystick_hit[3] * 0.5,
    );
    let active = stick.length_squared() > 0.0064;
    ui.pad(center, 28, if active { CYAN } else { EDGE }, INK);
    for offset in [-18.0, 18.0] {
        ui.rect(center.x + offset - 1.0, center.y - 3.0, 2.0, 6.0, DIM);
        ui.rect(center.x - 3.0, center.y + offset - 1.0, 6.0, 2.0, DIM);
    }
    let knob = center + (stick * 18.0).round();
    ui.pad(knob, 8, if active { CYAN } else { DIM }, 0xff_493322);
    ui.rect(
        knob.x - 2.0,
        knob.y - 2.0,
        4.0,
        4.0,
        if active { WHITE } else { CYAN },
    );
    let fire_center = Vec2::new(
        fire_hit[0] + fire_hit[2] * 0.5,
        fire_hit[1] + fire_hit[3] * 0.5,
    );
    ui.pad(
        fire_center,
        28,
        if fire { MAGENTA } else { EDGE },
        if fire { 0xf0_452342 } else { INK },
    );
    ui.text(
        fire_center.x - 11.0,
        fire_center.y - 3.0,
        "FIRE",
        1.0,
        if fire { WHITE } else { MAGENTA },
    );
    for offset in [-17.0, 17.0] {
        ui.rect(
            fire_center.x - 3.0,
            fire_center.y + offset,
            6.0,
            2.0,
            MAGENTA,
        );
    }
}

/// 0 hidden, 1 flight HUD, 2 launch backplate, 3 menu backplate.
pub fn draw_ui(
    frame: &mut FrameBuilder,
    hud: &Hud,
    camera: [f32; 2],
    viewport: [f32; 2],
    mode: u8,
) {
    draw_ui_with_boss_panel(
        frame,
        hud,
        camera,
        viewport,
        mode,
        Some([(viewport[0] * 0.5 - 110.0).floor(), 44.0, 220.0, 40.0]),
        false,
    );
}

/// The browser supplies a presentation-only, screen-space boss placement.
/// A compact panel retains its health meter and phase using the same font cells.
pub fn draw_ui_with_boss_panel(
    frame: &mut FrameBuilder,
    hud: &Hud,
    camera: [f32; 2],
    viewport: [f32; 2],
    mode: u8,
    boss_panel: Option<[f32; 4]>,
    minimal: bool,
) {
    draw_ui_with_boss_panel_and_touch(
        frame, hud, camera, viewport, mode, boss_panel, minimal, false,
    );
}

/// Touch-aware variant; the original public wrapper keeps desktop behavior.
#[allow(clippy::too_many_arguments)]
pub fn draw_ui_with_boss_panel_and_touch(
    frame: &mut FrameBuilder,
    hud: &Hud,
    camera: [f32; 2],
    viewport: [f32; 2],
    mode: u8,
    boss_panel: Option<[f32; 4]>,
    minimal: bool,
    touch: bool,
) {
    draw_ui_with_boss_panel_and_touch_and_header(
        frame, hud, camera, viewport, mode, boss_panel, minimal, touch, None,
    );
}

/// Actual semantic header-aware painter. The previous APIs remain wrappers.
#[allow(clippy::too_many_arguments)]
pub fn draw_ui_with_boss_panel_and_touch_and_header(
    frame: &mut FrameBuilder,
    hud: &Hud,
    camera: [f32; 2],
    viewport: [f32; 2],
    mode: u8,
    boss_panel: Option<[f32; 4]>,
    minimal: bool,
    touch: bool,
    header: Option<[f32; 4]>,
) {
    let camera = Vec2::from_array(camera);
    let viewport = Vec2::from_array(viewport);
    if mode == 0
        || !camera.is_finite()
        || !viewport.is_finite()
        || viewport.x < if mode == 1 { 180.0 } else { 220.0 }
        || viewport.y < 140.0
    {
        return;
    }
    let mut ui = Ui {
        frame,
        origin: camera - viewport * 0.5,
    };
    if mode == 2 || mode == 3 {
        let size = if mode == 2 {
            Vec2::new(440.0, 250.0)
        } else {
            Vec2::new(540.0, 300.0)
        }
        .min(viewport - Vec2::splat(if mode == 2 { 32.0 } else { 24.0 }));
        let top_left = ((viewport - size) * 0.5).floor();
        ui.panel(top_left.x, top_left.y, size.x.floor(), size.y.floor());
        // Accent brackets leave headings, choices and buttons to semantic DOM.
        ui.rect(top_left.x + 9.0, top_left.y + 10.0, 2.0, 18.0, CYAN);
        ui.rect(
            top_left.x + size.x - 11.0,
            top_left.y + size.y - 28.0,
            2.0,
            18.0,
            GOLD,
        );
        return;
    }
    if mode != 1 {
        return;
    }
    let Some(layout) = flight_layout_with_header(hud, viewport.to_array(), minimal, touch, header)
    else {
        return;
    };
    if let Some([x, y, w, h]) = layout.biome {
        ui.panel(x, y, w, h);
        let padding = if layout.compact_header { 5.0 } else { 8.0 };
        ui.text_bounded(
            x + padding,
            y + if layout.compact_header { 3.0 } else { 6.0 },
            hud.biome.name,
            1.0,
            CYAN,
            w - padding * 2.0,
        );
        let status = if hud.safe {
            "SAFE".into()
        } else {
            format!("THREAT {:02}", hud.danger)
        };
        ui.text_bounded(
            x + padding,
            y + if layout.compact_header { 11.0 } else { 17.0 },
            &status,
            1.0,
            if hud.safe { MINT } else { GOLD },
            w - padding * 2.0,
        );
    }
    draw_boss(&mut ui, hud, viewport, boss_panel);

    if let Some([currency_x, y, width, height]) = layout.currency {
        ui.panel(currency_x, y, width, height);
        for category in 0..3 {
            let step = if layout.compact_header { 32.0 } else { 43.0 };
            let x = currency_x + 8.0 + category as f32 * step;
            if layout.compact_header {
                ui.icon_scaled(x, y + 7.0, category, 1.0);
            } else {
                ui.icon(x, y + 7.0, category);
            }
            ui.text_bounded(
                x + if layout.compact_header { 8.0 } else { 14.0 },
                y + if layout.compact_header { 6.0 } else { 8.0 },
                &compact(hud.currency[category]),
                1.0,
                [RED, CYAN, MINT][category],
                if layout.compact_header { 24.0 } else { 26.0 },
            );
        }
    }

    if let Some([meter_x, y, width, height]) = layout.meter {
        ui.panel(meter_x, y, width, height);
        if layout.meter_style == MeterStyle::Slim {
            for (column, label, value, maximum, color) in [
                (0.0, "H", hud.hull, hud.max_hull, MINT),
                (1.0, "S", hud.shield, hud.max_shield, CYAN),
            ] {
                let x = meter_x + column * width * 0.5;
                ui.text(x + 3.0, y + 1.0, label, 1.0, DIM);
                ui.bar(x + 12.0, y + 3.0, width * 0.5 - 16.0, value, maximum, color);
            }
        } else if layout.meter_style == MeterStyle::Micro {
            for (row, label, value, maximum, color) in [
                (
                    0.0,
                    "H",
                    hud.hull,
                    hud.max_hull,
                    if ratio(hud.hull, hud.max_hull) < 0.3 {
                        RED
                    } else {
                        MINT
                    },
                ),
                (7.0, "S", hud.shield, hud.max_shield, CYAN),
            ] {
                ui.text(meter_x + 5.0, y + 1.0 + row, label, 1.0, DIM);
                ui.bar(
                    meter_x + 16.0,
                    y + 3.0 + row,
                    width - 48.0,
                    value,
                    maximum,
                    color,
                );
                ui.text_bounded(
                    meter_x + width - 29.0,
                    y + 1.0 + row,
                    &format!("{}%", (ratio(value, maximum) * 100.0).ceil() as u32),
                    1.0,
                    WHITE,
                    24.0,
                );
            }
        } else {
            let full = layout.meter_style == MeterStyle::Full;
            let status = if full {
                format!(
                    "LV {} / CARGO {}/{}",
                    compact(hud.ship_level),
                    compact(hud.cargo_materials),
                    compact(hud.cargo_capacity)
                )
            } else {
                format!(
                    "L{} C{}/{}",
                    compact(hud.ship_level),
                    compact(hud.cargo_materials),
                    compact(hud.cargo_capacity)
                )
            };
            let status_width =
                (status.chars().count() as f32 * RoyaleMono::ADVANCE - 1.0).min(width - 14.0);
            ui.text_bounded(
                if full {
                    (viewport.x - status_width) * 0.5
                } else {
                    meter_x + (width - status_width) * 0.5
                },
                y + 5.0,
                &status,
                1.0,
                DIM,
                width - 14.0,
            );
            ui.text(
                meter_x + 7.0,
                y + 17.0,
                if full { "HULL" } else { "H" },
                1.0,
                DIM,
            );
            ui.bar(
                meter_x + if full { 37.0 } else { 18.0 },
                y + 18.0,
                if full { 99.0 } else { width - 54.0 },
                hud.hull,
                hud.max_hull,
                if ratio(hud.hull, hud.max_hull) < 0.3 {
                    RED
                } else {
                    MINT
                },
            );
            ui.text_bounded(
                meter_x + width - 33.0,
                y + 17.0,
                &format!("{}%", (ratio(hud.hull, hud.max_hull) * 100.0).ceil() as u32),
                1.0,
                WHITE,
                24.0,
            );
            ui.text(
                meter_x + 7.0,
                y + 27.0,
                if full { "SHLD" } else { "S" },
                1.0,
                DIM,
            );
            ui.bar(
                meter_x + if full { 37.0 } else { 18.0 },
                y + 28.0,
                if full { 99.0 } else { width - 54.0 },
                hud.shield,
                hud.max_shield,
                CYAN,
            );
        }
    }

    let position = Vec2::from_array(hud.position);
    let Some([x, y, _, _]) = layout.radar else {
        return;
    };
    let radar = Vec2::new(x, y);
    ui.panel(radar.x, radar.y, 48.0, 48.0);
    ui.text(radar.x + 8.0, radar.y + 6.0, "HOME", 1.0, DIM);
    let center = radar + Vec2::new(24.0, 29.0);
    for i in -12_i32..=12 {
        if i % 4 == 0 {
            ui.dot(center.x + i as f32, center.y, 1.0, EDGE);
            ui.dot(center.x, center.y + i as f32, 1.0, EDGE);
        }
    }
    let dock = (-position / 70.0).clamp_length_max(12.0).round();
    ui.dot(center.x + dock.x - 1.0, center.y + dock.y - 1.0, 3.0, GOLD);
    ui.dot(center.x - 1.0, center.y - 1.0, 3.0, CYAN);
    if hud.heading.is_finite() {
        let direction = Vec2::new(hud.heading.cos(), hud.heading.sin());
        for i in 3..9 {
            let p = (direction * i as f32).round();
            ui.dot(center.x + p.x, center.y + p.y, 1.0, WHITE);
        }
    }
}

fn draw_boss(ui: &mut Ui<'_>, hud: &Hud, viewport: Vec2, boss_panel: Option<[f32; 4]>) {
    if let (Some(boss), Some([x, y, width, height])) = (&hud.boss, boss_panel)
        && [x, y, width, height].iter().all(|v| v.is_finite())
        && width >= 100.0
        && height == 40.0
        && x >= 0.0
        && y >= 0.0
        && x + width <= viewport.x
        && y + height <= viewport.y
    {
        let limit = ((width - 14.0) / RoyaleMono::ADVANCE).floor() as usize;
        let name = if width < 140.0 && boss.generation > 0 {
            format!(
                "G{} / {}",
                boss.generation,
                ["DART", "BASTION", "COIL"][boss.family.min(2)]
            )
        } else if boss.generation > 0 {
            format!("G{} / {}", boss.generation, boss.name)
        } else {
            boss.name.clone()
        };
        let label = boss_phase_label(&boss.phase, width);
        let phase = if boss.telegraph {
            format!(
                "{} / {}",
                if width < 168.0 { "WARN" } else { "WARNING" },
                label
            )
        } else {
            label.into()
        };
        ui.panel(x, y, width, height);
        ui.text(
            x + 7.0,
            y + 5.0,
            &name.chars().take(limit).collect::<String>(),
            1.0,
            MAGENTA,
        );
        ui.text(
            x + 7.0,
            y + 17.0,
            &phase.chars().take(limit).collect::<String>(),
            1.0,
            if boss.telegraph { GOLD } else { DIM },
        );
        ui.bar(
            x + 7.0,
            y + 31.0,
            width - 14.0,
            boss.health,
            boss.max_health,
            RED,
        );
    }
}

fn boss_phase_label(phase: &str, width: f32) -> &str {
    if width >= 168.0 {
        return phase;
    }
    match phase {
        "Attack charging" => "CHARGE",
        "Siege broadside" => "BROADSIDE",
        "Serpent pursuit" => "PURSUIT",
        "Sidewinder rush" => "COIL RUSH",
        _ => phase,
    }
}

fn ratio(value: f32, maximum: f32) -> f32 {
    if !value.is_finite() || !maximum.is_finite() || maximum <= 0.0 {
        0.0
    } else {
        (value / maximum).clamp(0.0, 1.0)
    }
}
fn compact(value: u32) -> String {
    if value >= 1_000_000_000 {
        format!("{}B", value / 1_000_000_000)
    } else if value >= 1_000_000 {
        format!("{}M", value / 1_000_000)
    } else if value >= 1_000 {
        format!("{}K", value / 1_000)
    } else {
        value.to_string()
    }
}

/// RoyaleMono source masks. Unknown characters occupy a blank advance.
pub(crate) fn glyph(character: char) -> [u8; 7] {
    match character {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '/' => [1, 1, 2, 4, 8, 16, 16],
        '%' => [25, 25, 2, 4, 8, 19, 19],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '.' => [0, 0, 0, 0, 0, 12, 12],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        '>' => [16, 8, 4, 2, 4, 8, 16],
        '<' => [1, 2, 4, 8, 4, 2, 1],
        '(' => [2, 4, 8, 8, 8, 4, 2],
        ')' => [8, 4, 2, 2, 2, 4, 8],
        '[' => [14, 8, 8, 8, 8, 8, 14],
        ']' => [14, 2, 2, 2, 2, 2, 14],
        '|' => [4, 4, 4, 4, 4, 4, 4],
        '_' => [0, 0, 0, 0, 0, 0, 31],
        '?' => [14, 17, 1, 2, 4, 0, 4],
        '=' => [0, 0, 31, 0, 31, 0, 0],
        '#' => [10, 31, 10, 10, 31, 10, 0],
        ',' => [0, 0, 0, 0, 0, 4, 8],
        '\'' => [4, 4, 8, 0, 0, 0, 0],
        _ => [0; 7],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn intersects(a: [f32; 4], b: [f32; 4]) -> bool {
        a[0] < b[0] + b[2] && a[0] + a[2] > b[0] && a[1] < b[1] + b[3] && a[1] + a[3] > b[1]
    }

    #[test]
    fn minimum_touch_hitboxes_are_disjoint_and_share_the_painted_centers() {
        for (width, joystick_width, center_x) in [
            (180.0, 82.0, 55.0),
            (181.0, 83.0, 55.5),
            (182.0, 84.0, 56.0),
        ] {
            for height in [140.0, 180.0] {
                let viewport = [width, height];
                let [joystick, fire] = touch_hit_rects(viewport);
                assert_eq!(joystick, [14.0, height - 92.0, joystick_width, 84.0]);
                assert_eq!(fire, [width - 84.0, height - 86.0, 72.0, 72.0]);
                assert!(
                    !intersects(joystick, fire),
                    "input ownership must not overlap"
                );
                assert_eq!(joystick[0] + joystick[2], fire[0]);
                for stick in [
                    None,
                    Some([0.5, 0.0]),
                    Some([1.0, -1.0]),
                    Some([f32::NAN, 0.0]),
                ] {
                    let camera = Vec2::new(17.25, -33.5);
                    let origin = camera - Vec2::from_array(viewport) * 0.5;
                    let mut frame = FrameBuilder::new();
                    draw_controls(&mut frame, camera.to_array(), viewport, stick, true);
                    let normalized = stick
                        .map(Vec2::from_array)
                        .filter(|value| value.is_finite())
                        .unwrap_or(Vec2::ZERO)
                        .clamp_length_max(1.0);
                    let expected_knob =
                        Vec2::new(center_x, height - 50.0) + (normalized * 18.0).round();
                    let mut knob_seen = false;
                    let mut fire_glyph_cells = 0usize;
                    for batch in &frame.frame().batches {
                        for pixel in batch.pixels.iter().filter(|pixel| pixel.color >> 24 != 0) {
                            let transform = batch.transforms[pixel.transform as usize];
                            let center = transform.point(Vec2::from_array(pixel.local)) - origin;
                            if transform.a[2] == 4.0 && transform.b[1] == 4.0 {
                                assert_eq!(center, expected_knob);
                                knob_seen = true;
                            }
                            if pixel.color == WHITE
                                && transform.a[2] == 1.0
                                && transform.b[1] == 1.0
                            {
                                assert!(center.x >= fire[0] && center.x <= fire[0] + fire[2]);
                                assert!(center.y >= fire[1] && center.y <= fire[1] + fire[3]);
                                fire_glyph_cells += 1;
                            }
                            for corner in [
                                Vec2::new(-0.5, -0.5),
                                Vec2::new(0.5, -0.5),
                                Vec2::new(-0.5, 0.5),
                                Vec2::new(0.5, 0.5),
                            ] {
                                let point = transform.point(Vec2::from_array(pixel.local) + corner)
                                    - origin;
                                assert!(
                                    point.x >= 0.0
                                        && point.x <= width
                                        && point.y >= 0.0
                                        && point.y <= height
                                );
                                assert!([joystick, fire].iter().any(|rect| point.x >= rect[0]
                                    && point.x <= rect[0] + rect[2]
                                    && point.y >= rect[1]
                                    && point.y <= rect[1] + rect[3]));
                            }
                        }
                    }
                    assert!(knob_seen);
                    assert_eq!(
                        fire_glyph_cells,
                        "FIRE"
                            .chars()
                            .map(|c| RoyaleMono::body(c).occupied_count())
                            .sum::<usize>()
                    );
                }
            }
        }
    }

    #[test]
    fn normal_touch_hit_rect_centers_retain_the_original_control_coordinates() {
        for width in [182.0, 200.0, 220.0, 320.0, 533.0, 720.0] {
            for height in [140.0, 180.0, 300.0] {
                let [joystick, fire] = touch_hit_rects([width, height]);
                assert_eq!(joystick[2], 84.0);
                assert_eq!(
                    Vec2::new(
                        joystick[0] + joystick[2] * 0.5,
                        joystick[1] + joystick[3] * 0.5
                    ),
                    Vec2::new(56.0, height - 50.0),
                );
                assert_eq!(
                    Vec2::new(fire[0] + fire[2] * 0.5, fire[1] + fire[3] * 0.5),
                    Vec2::new(width - 48.0, height - 50.0),
                );
                assert!(!intersects(joystick, fire));
            }
        }
    }

    #[test]
    fn shared_flight_layout_preserves_normal_rects_and_keeps_touch_telemetry_clear() {
        let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
        hud.dock = false;
        hud.position = [1600.0, 0.0];
        let normal = flight_layout(&hud, [533.0, 300.0], false, true).unwrap();
        assert_eq!(normal.biome, Some([12.0, 12.0, 160.0, 28.0]));
        assert_eq!(normal.currency, Some([319.0, 12.0, 142.0, 24.0]));
        assert_eq!(normal.radar, Some([473.0, 156.0, 48.0, 48.0]));
        assert_eq!(normal.meter, Some([178.0, 252.0, 176.0, 36.0]));
        assert_eq!(normal.meter_style, MeterStyle::Full);
        let narrow = flight_layout(&hud, [320.0, 180.0], false, true).unwrap();
        assert_eq!(narrow.meter, Some([102.0, 132.0, 130.0, 36.0]));
        assert_eq!(narrow.meter_style, MeterStyle::Compact);
        assert!(narrow.radar.is_none());
        let tiny = flight_layout(&hud, [180.0, 140.0], false, true).unwrap();
        assert_eq!(tiny.meter, Some([12.0, 28.0, 156.0, 16.0]));
        assert_eq!(tiny.meter_style, MeterStyle::Micro);
        assert!(tiny.biome.is_none() && tiny.currency.is_none() && tiny.radar.is_none());
    }

    #[test]
    fn actual_hud_and_control_corners_fit_narrow_fractional_viewports_and_shared_rects() {
        let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
        hud.dock = false;
        hud.position = [1600.0, 0.0];
        hud.ship_level = u32::MAX;
        hud.currency = [999_999, u32::MAX, 1_000];
        hud.cargo_materials = u32::MAX;
        hud.cargo_capacity = u32::MAX;
        hud.danger = u32::MAX;
        for width in [
            180.0, 200.0, 220.0, 239.75, 280.0, 289.75, 290.0, 300.0, 320.0, 365.75, 366.0, 391.75,
            392.0, 400.0, 533.0, 720.0,
        ] {
            for height in [140.0, 159.0, 160.0, 180.0, 240.0, 300.0] {
                for touch in [false, true] {
                    for minimal in [false, true] {
                        let viewport = [width, height];
                        let layout = flight_layout(&hud, viewport, minimal, touch).unwrap();
                        let rects: Vec<_> =
                            [layout.biome, layout.currency, layout.radar, layout.meter]
                                .into_iter()
                                .flatten()
                                .collect();
                        for (index, a) in rects.iter().enumerate() {
                            assert!(
                                a[0] >= 0.0
                                    && a[1] >= 0.0
                                    && a[0] + a[2] <= width
                                    && a[1] + a[3] <= height
                            );
                            assert!(
                                rects[index + 1..].iter().all(|b| !intersects(*a, *b)),
                                "native panels overlap at{viewport:?}"
                            );
                            if touch {
                                assert!(
                                    touch_hit_rects(viewport)
                                        .iter()
                                        .all(|b| !intersects(*a, *b)),
                                    "telemetry beneath hitpad at{viewport:?}: {a:?}"
                                );
                            }
                        }
                        let camera = Vec2::new(999_000.25, -998_000.5);
                        let origin = camera - Vec2::from_array(viewport) * 0.5;
                        let mut frame = FrameBuilder::new();
                        draw_ui_with_boss_panel_and_touch(
                            &mut frame,
                            &hud,
                            camera.to_array(),
                            viewport,
                            1,
                            None,
                            minimal,
                            touch,
                        );
                        assert!(frame.frame().pixel_count > 0);
                        for b in &frame.frame().batches {
                            for p in b.pixels.iter().filter(|p| p.color >> 24 != 0) {
                                let t = b.transforms[p.transform as usize];
                                for offset in [
                                    Vec2::new(-0.5, -0.5),
                                    Vec2::new(0.5, -0.5),
                                    Vec2::new(-0.5, 0.5),
                                    Vec2::new(0.5, 0.5),
                                ] {
                                    let point =
                                        t.point(Vec2::from_array(p.local) + offset) - origin;
                                    assert!(
                                        point.x >= -0.125
                                            && point.y >= -0.125
                                            && point.x <= width + 0.125
                                            && point.y <= height + 0.125,
                                        "actual HUD corner outside{viewport:?}: {point:?}"
                                    );
                                    assert!(
                                        rects.iter().any(|r| point.x >= r[0] - 0.125
                                            && point.x <= r[0] + r[2] + 0.125
                                            && point.y >= r[1] - 0.125
                                            && point.y <= r[1] + r[3] + 0.125),
                                        "paint/reservation divergence{viewport:?}: {point:?}"
                                    );
                                }
                            }
                        }
                        if touch {
                            frame.clear();
                            draw_controls(
                                &mut frame,
                                camera.to_array(),
                                viewport,
                                Some([1.0, -1.0]),
                                true,
                            );
                            for b in &frame.frame().batches {
                                for p in &b.pixels {
                                    for offset in [Vec2::new(-0.5, -0.5), Vec2::new(0.5, 0.5)] {
                                        let point = b.transforms[p.transform as usize]
                                            .point(Vec2::from_array(p.local) + offset)
                                            - origin;
                                        assert!(
                                            point.x >= -0.125
                                                && point.y >= -0.125
                                                && point.x <= width + 0.125
                                                && point.y <= height + 0.125
                                        );
                                        assert!(touch_hit_rects(viewport).iter().any(|r| point.x
                                            >= r[0] - 0.125
                                            && point.x <= r[0] + r[2] + 0.125
                                            && point.y >= r[1] - 0.125
                                            && point.y <= r[1] + r[3] + 0.125));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn extreme_biome_currency_and_cargo_glyphs_stay_in_their_small_panels() {
        let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
        hud.dock = false;
        hud.ship_level = u32::MAX;
        hud.cargo_materials = u32::MAX;
        hud.cargo_capacity = u32::MAX;
        for viewport in [[300.0, 180.0], [320.0, 180.0], [220.0, 140.0]] {
            let mut frame = FrameBuilder::new();
            draw_ui_with_boss_panel_and_touch(
                &mut frame, &hud, [0.0; 2], viewport, 1, None, false, true,
            );
            let layout = flight_layout(&hud, viewport, false, true).unwrap();
            let count = frame.frame().pixel_count;
            assert!(count > 100, "essential bars retain occupied glyph objects");
            assert!(layout.meter.is_some());
        }
        assert!(flight_layout(&hud, [179.0, 140.0], false, true).is_none());
        assert!(flight_layout(&hud, [180.0, 139.0], false, true).is_none());
        assert!(flight_layout(&hud, [f32::NAN, 140.0], false, true).is_none());
    }

    #[test]
    fn royale_mono_reuses_body_and_composes_letter_spacing() {
        assert!(std::ptr::eq(RoyaleMono::body('A'), RoyaleMono::body('a')));
        assert!(std::ptr::eq(RoyaleMono::body('A'), RoyaleMono::body('A')));
        assert_eq!(RoyaleMono::body(' ').occupied_count(), 0);
        assert_eq!(RoyaleMono::body('é').occupied_count(), 0);
        let parent = Transform {
            position: Vec2::new(9.0, -5.0),
            rotation: 0.4,
            scale: Vec2::new(2.0, 0.5),
        }
        .affine();
        let letter = RoyaleMono::letter_transform(parent, 2);
        let expected = parent.point(Vec2::new(14.5, 3.5));
        assert!(letter.position.distance(expected) < 0.0001);
        let mut frame = FrameBuilder::new();
        let mut ui = Ui {
            frame: &mut frame,
            origin: Vec2::new(10.0, 20.0),
        };
        ui.text(5.0, 7.0, "A A", 2.0, WHITE);
        let batch = &frame.frame().batches[0];
        assert_eq!(
            batch.transforms.len(),
            2,
            "blank advance must not create an object"
        );
        assert_eq!(
            frame.frame().pixel_count,
            RoyaleMono::body('A').occupied_count() * 2
        );
        assert_eq!(batch.transforms[1].a[0] - batch.transforms[0].a[0], 24.0);
    }

    #[test]
    fn overlay_stays_fixed_when_world_camera_moves() {
        let hud = game_core::Game::new(game_core::Profile::default()).hud();
        let positions = |camera: Vec2| {
            let mut frame = FrameBuilder::new();
            draw_ui(&mut frame, &hud, camera.to_array(), [640.0, 360.0], 1);
            let mut result = Vec::new();
            for batch in &frame.frame().batches {
                for pixel in &batch.pixels {
                    result.push(
                        batch.transforms[pixel.transform as usize]
                            .point(Vec2::from_array(pixel.local))
                            - camera,
                    );
                }
            }
            result
        };
        let stationary = positions(Vec2::ZERO);
        let moving = positions(Vec2::new(1700.25, -900.5));
        assert!(stationary.len() > 100);
        assert_eq!(stationary.len(), moving.len());
        assert!(
            stationary
                .iter()
                .zip(moving)
                .all(|(a, b)| a.distance(b) < 0.001)
        );
    }

    #[test]
    fn hidden_mode_and_invalid_viewport_emit_no_overlay() {
        let hud = game_core::Game::new(game_core::Profile::default()).hud();
        let mut frame = FrameBuilder::new();
        draw_ui(&mut frame, &hud, [0.0; 2], [640.0, 360.0], 0);
        draw_ui(&mut frame, &hud, [f32::NAN, 0.0], [640.0, 360.0], 1);
        draw_ui(&mut frame, &hud, [0.0; 2], [f32::INFINITY, 360.0], 1);
        assert_eq!(frame.frame().pixel_count, 0);
        assert_eq!(ratio(100.0, 0.0), 0.0);
        assert_eq!(ratio(f32::NAN, 100.0), 0.0);
        assert_eq!(ratio(130.0, 100.0), 1.0);
    }

    #[test]
    fn full_dock_hides_meters_and_home_compass_until_needed() {
        let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
        let positions = |hud: &Hud| {
            let mut frame = FrameBuilder::new();
            draw_ui(&mut frame, hud, [0.0; 2], [640.0, 360.0], 1);
            frame
                .frame()
                .batches
                .iter()
                .flat_map(|batch| {
                    batch.pixels.iter().map(|pixel| {
                        batch.transforms[pixel.transform as usize]
                            .point(Vec2::from_array(pixel.local))
                            + Vec2::new(320.0, 180.0)
                    })
                })
                .collect::<Vec<_>>()
        };
        assert!(positions(&hud).iter().all(|p| p.y < 100.0));
        hud.hull *= 0.5;
        assert!(
            positions(&hud)
                .iter()
                .any(|p| p.y > 320.0 && p.x > 232.0 && p.x < 408.0)
        );
        hud.dock = false;
        hud.position = [1200.0, 0.0];
        assert!(
            positions(&hud)
                .iter()
                .any(|p| p.x > 580.0 && p.y > 216.0 && p.y < 264.0)
        );
    }

    #[test]
    fn flight_level_and_cargo_line_stays_between_touch_pads() {
        let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
        hud.dock = false;
        hud.ship_level = u32::MAX;
        hud.cargo_materials = u32::MAX;
        hud.cargo_capacity = u32::MAX;
        let mut frame = FrameBuilder::new();
        draw_ui(&mut frame, &hud, [0.0; 2], [640.0, 360.0], 1);
        let mut status_cells = 0;
        let mut status_backing = false;
        for batch in &frame.frame().batches {
            for pixel in &batch.pixels {
                let transform = batch.transforms[pixel.transform as usize];
                let p = transform.point(Vec2::from_array(pixel.local)) + Vec2::new(320.0, 180.0);
                if pixel.color == INK && transform.a[2] >= 170.0 && transform.b[1] >= 36.0 {
                    status_backing = p.y - transform.b[1] * 0.5 <= 312.0;
                }
                if pixel.color == DIM && p.y > 310.0 && p.y < 324.0 {
                    assert!(p.x > 232.0 && p.x < 408.0);
                    status_cells += 1;
                }
            }
        }
        assert!(status_cells > 100);
        assert!(
            status_backing,
            "busy actors must remain behind the status caption"
        );
    }

    #[test]
    fn generation_and_warning_fit_the_boss_panel_at_landscape_extents() {
        let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
        hud.boss = Some(game_core::BossHud {
            name: "A DELIBERATELY LONG BOSS TITLE TO TEST CLIPPING".into(),
            health: 90.0,
            max_health: 100.0,
            family: 2,
            generation: 5,
            phase: "TELEGRAPH THAT MUST FIT ITS PANEL".into(),
            telegraph: true,
            position: [0.0; 2],
            core_position: [0.0; 2],
        });
        for viewport in [[533.0, 300.0], [720.0, 360.0], [844.0, 390.0]] {
            let camera = Vec2::new(1250.0, -345.0);
            let origin = camera - Vec2::from_array(viewport) * 0.5;
            let panel_x = (viewport[0] * 0.5 - 110.0).floor();
            let mut frame = FrameBuilder::new();
            draw_ui(&mut frame, &hud, camera.to_array(), viewport, 1);
            let mut warning_cells = 0;
            for batch in &frame.frame().batches {
                for pixel in &batch.pixels {
                    let position = batch.transforms[pixel.transform as usize]
                        .point(Vec2::from_array(pixel.local))
                        - origin;
                    if position.y >= 44.0 && position.y < 84.0 {
                        assert!(position.x >= panel_x && position.x < panel_x + 220.0);
                        if position.y >= 61.0 && pixel.color == GOLD {
                            warning_cells += 1;
                        }
                    }
                }
            }
            assert!(
                warning_cells > 30,
                "actual warning glyph cells must be visible"
            );
        }
    }

    #[test]
    fn compact_boss_panels_keep_real_phase_glyphs_health_and_inset_padding() {
        let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
        hud.boss = Some(game_core::BossHud {
            name: "Bastion Oblivion".into(),
            health: 50.0,
            max_health: 100.0,
            family: 1,
            generation: 5,
            phase: "Core exposed".into(),
            telegraph: false,
            position: [0.0; 2],
            core_position: [0.0; 2],
        });
        for width in [100.0, 140.0, 168.0, 220.0] {
            let mut frame = FrameBuilder::new();
            draw_ui_with_boss_panel(
                &mut frame,
                &hud,
                [0.0; 2],
                [400.0, 240.0],
                1,
                Some([12.0, 68.0, width, 40.0]),
                false,
            );
            let mut phase_cells = 0;
            let mut health_cells = 0;
            for batch in &frame.frame().batches {
                for pixel in &batch.pixels {
                    let t = batch.transforms[pixel.transform as usize];
                    let p = t.point(Vec2::from_array(pixel.local)) + Vec2::new(200.0, 120.0);
                    if (68.0..108.0).contains(&p.y) {
                        assert!((12.0..12.0 + width).contains(&p.x));
                        let glyph = t.a[2] == 1.0 && t.b[1] == 1.0;
                        if pixel.color == MAGENTA || (pixel.color == DIM && glyph) {
                            assert!(p.x >= 19.0 && p.x < 12.0 + width - 7.0);
                        }
                        if pixel.color == DIM && glyph && (85.0..92.0).contains(&p.y) {
                            phase_cells += 1;
                        }
                        if pixel.color == RED && (99.0..103.0).contains(&p.y) {
                            health_cells += 1;
                        }
                    }
                }
            }
            let expected_phase_cells: usize = "CORE EXPOSED"
                .chars()
                .map(|c| RoyaleMono::body(c).occupied_count())
                .sum();
            assert_eq!(
                phase_cells, expected_phase_cells,
                "phase must be complete, including at 100px"
            );
            assert!(
                health_cells > 0,
                "compact panel retains the actual health fill"
            );
        }
    }

    #[test]
    fn every_authored_attack_has_a_complete_compact_label() {
        for phase in [
            "Radial weave",
            "Locked dash",
            "Scissor pass",
            "Gap spiral",
            "Royal cross",
            "Aimed fan",
            "Siege broadside",
            "Pincer gates",
            "Quadrant wall",
            "Rotating rails",
            "Serpent pursuit",
            "Sidewinder rush",
            "Ouroboros loop",
            "Tail barrage",
            "Vortex lunge",
            "Core exposed",
        ] {
            let compact = boss_phase_label(phase, 100.0);
            assert!(
                compact.chars().count() <= 14,
                "incomplete compact cue: {phase}"
            );
            assert_eq!(boss_phase_label(phase, 220.0), phase);
        }
        assert_eq!(
            format!("WARN / {}", boss_phase_label("Attack charging", 100.0)),
            "WARN / CHARGE"
        );
    }

    #[test]
    fn invalid_boss_placement_cannot_emit_stray_boss_pixels() {
        let mut game = game_core::Game::new(game_core::Profile::default());
        game.diagnostic("boss-coil-5");
        let hud = game.hud();
        let mut baseline = FrameBuilder::new();
        draw_ui_with_boss_panel(
            &mut baseline,
            &hud,
            [0.0; 2],
            [400.0, 240.0],
            1,
            None,
            false,
        );
        for rect in [
            [-1.0, 60.0, 220.0, 40.0],
            [300.0, 60.0, 220.0, 40.0],
            [12.0, 60.0, 99.0, 40.0],
            [12.0, 60.0, 220.0, 41.0],
            [f32::NAN, 60.0, 220.0, 40.0],
        ] {
            let mut frame = FrameBuilder::new();
            draw_ui_with_boss_panel(
                &mut frame,
                &hud,
                [0.0; 2],
                [400.0, 240.0],
                1,
                Some(rect),
                false,
            );
            assert_eq!(frame.frame().pixel_count, baseline.frame().pixel_count);
        }
    }

    #[test]
    fn minimal_flight_hides_only_optional_telemetry_and_keeps_hull() {
        let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
        hud.dock = false;
        hud.position = [1800.0, 0.0];
        let mut frame = FrameBuilder::new();
        draw_ui_with_boss_panel(&mut frame, &hud, [0.0; 2], [400.0, 240.0], 1, None, true);
        let mut hull = 0;
        for batch in &frame.frame().batches {
            for pixel in &batch.pixels {
                let p = batch.transforms[pixel.transform as usize]
                    .point(Vec2::from_array(pixel.local))
                    + Vec2::new(200.0, 120.0);
                assert!(
                    p.y >= 192.0,
                    "biome/currency/compass quads must be absent in minimal mode"
                );
                if pixel.color == MINT {
                    hull += 1;
                }
            }
        }
        assert!(hull > 0, "minimal mode retains real hull fill");
    }

    #[test]
    fn touch_knob_moves_proportionally_and_invalid_input_returns_to_center() {
        let knob = |stick: [f32; 2]| {
            let mut frame = FrameBuilder::new();
            draw_controls(&mut frame, [0.0; 2], [640.0, 360.0], Some(stick), true);
            for batch in &frame.frame().batches {
                for pixel in &batch.pixels {
                    let t = batch.transforms[pixel.transform as usize];
                    if t.a[2] == 4.0 && t.b[1] == 4.0 {
                        return t.point(Vec2::from_array(pixel.local)) + Vec2::new(320.0, 180.0);
                    }
                }
            }
            panic!("knob marker missing");
        };
        assert_eq!(knob([1.0, 0.0]), Vec2::new(74.0, 310.0));
        assert_eq!(knob([0.5, 0.0]), Vec2::new(65.0, 310.0));
        assert_eq!(knob([f32::NAN, 0.0]), Vec2::new(56.0, 310.0));
    }
}

#[cfg(test)]
#[path = "ui_flight_status_tests.rs"]
mod flight_status_tests;
