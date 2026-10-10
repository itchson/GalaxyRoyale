//! Screen-space HUD built from the same affine pixel quads as the world.
//! Coordinates are logical pixels; DOM controls retain interaction/accessibility.
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Transform, Vec2};
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
        for (index, character) in text.chars().enumerate() {
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
        let rows = match category {
            0 => [4, 14, 31, 4, 4],   // weapon/star
            1 => [31, 17, 17, 10, 4], // shield
            _ => [4, 14, 31, 14, 4],  // passive crystal
        };
        for (row, bits) in rows.iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    self.dot(
                        x + column as f32 * 2.0,
                        y + row as f32 * 2.0,
                        2.0,
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

/// Visual touch controls. The DOM handles pointer capture and hit testing.
/// Centers: joystick [56,h-50], fire [w-48,h-50], both radius 28.
pub fn draw_controls(
    frame: &mut FrameBuilder,
    camera: [f32; 2],
    viewport: [f32; 2],
    joystick: Option<[f32; 2]>,
    fire: bool,
) {
    let camera = Vec2::from_array(camera);
    let viewport = Vec2::from_array(viewport);
    if !camera.is_finite() || !viewport.is_finite() || viewport.x < 220.0 || viewport.y < 140.0 {
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
    let center = Vec2::new(56.0, viewport.y - 50.0);
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
    let fire_center = Vec2::new(viewport.x - 48.0, viewport.y - 50.0);
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
    let camera = Vec2::from_array(camera);
    let viewport = Vec2::from_array(viewport);
    if mode == 0
        || !camera.is_finite()
        || !viewport.is_finite()
        || viewport.x < 220.0
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
    ui.panel(12.0, 12.0, 160.0, 28.0);
    ui.text(20.0, 18.0, hud.biome.name, 1.0, CYAN);
    let status = if hud.safe {
        "SAFE".into()
    } else {
        format!("THREAT {:02}", hud.danger)
    };
    ui.text(20.0, 29.0, &status, 1.0, if hud.safe { MINT } else { GOLD });
    if let Some(boss) = &hud.boss {
        let x = (viewport.x * 0.5 - 110.0).floor();
        ui.panel(x, 44.0, 220.0, 24.0);
        ui.text(x + 7.0, 49.0, &boss.name, 1.0, MAGENTA);
        ui.bar(x + 7.0, 59.0, 206.0, boss.health, boss.max_health, RED);
    }

    let currency_x = viewport.x - 214.0;
    ui.panel(currency_x, 12.0, 142.0, 24.0);
    for category in 0..3 {
        let x = currency_x + 8.0 + category as f32 * 43.0;
        ui.icon(x, 19.0, category);
        ui.text(
            x + 14.0,
            20.0,
            &compact(hud.currency[category]),
            1.0,
            [RED, CYAN, MINT][category],
        );
    }

    if !hud.dock || ratio(hud.hull, hud.max_hull) < 1.0 {
        let meter_x = (viewport.x * 0.5 - 88.0).floor();
        let meter_y = viewport.y - 36.0;
        let status = format!(
            "LV {} / CARGO {}/{}",
            compact(hud.ship_level),
            compact(hud.cargo_materials),
            compact(hud.cargo_capacity)
        );
        let status_width = status.chars().count() as f32 * RoyaleMono::ADVANCE - 1.0;
        ui.text(
            (viewport.x - status_width) * 0.5,
            meter_y - 10.0,
            &status,
            1.0,
            DIM,
        );
        ui.panel(meter_x, meter_y, 176.0, 24.0);
        ui.text(meter_x + 7.0, meter_y + 5.0, "HULL", 1.0, DIM);
        ui.bar(
            meter_x + 37.0,
            meter_y + 6.0,
            99.0,
            hud.hull,
            hud.max_hull,
            if ratio(hud.hull, hud.max_hull) < 0.3 {
                RED
            } else {
                MINT
            },
        );
        ui.text(
            meter_x + 143.0,
            meter_y + 5.0,
            &format!("{}%", (ratio(hud.hull, hud.max_hull) * 100.0).ceil() as u32),
            1.0,
            WHITE,
        );
        ui.text(meter_x + 7.0, meter_y + 15.0, "SHLD", 1.0, DIM);
        ui.bar(
            meter_x + 37.0,
            meter_y + 16.0,
            99.0,
            hud.shield,
            hud.max_shield,
            CYAN,
        );
    }

    let position = Vec2::from_array(hud.position);
    if !position.is_finite() || position.length_squared() <= 240.0 * 240.0 {
        return;
    }
    let radar = Vec2::new(viewport.x - 60.0, viewport.y - 144.0);
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
        assert!(stationary
            .iter()
            .zip(moving)
            .all(|(a, b)| a.distance(b) < 0.001));
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
        assert!(positions(&hud)
            .iter()
            .any(|p| p.y > 320.0 && p.x > 232.0 && p.x < 408.0));
        hud.dock = false;
        hud.position = [1200.0, 0.0];
        assert!(positions(&hud)
            .iter()
            .any(|p| p.x > 580.0 && p.y > 216.0 && p.y < 264.0));
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
        for batch in &frame.frame().batches {
            for pixel in &batch.pixels {
                let p = batch.transforms[pixel.transform as usize]
                    .point(Vec2::from_array(pixel.local))
                    + Vec2::new(320.0, 180.0);
                if p.y > 310.0 && p.y < 324.0 {
                    assert!(p.x > 232.0 && p.x < 408.0);
                    status_cells += 1;
                }
            }
        }
        assert!(status_cells > 100);
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
