//! Standalone presentation prototype. No renderer, gameplay, or persistence.
//! Coordinates and returned rectangles are full logical UI pixels, not CSS
//! backing pixels. Panel geometry never scales with world zoom.

pub const PANEL_WIDTH: f64 = 220.0;
pub const PANEL_HEIGHT: f64 = 40.0;
pub const HOLD_MS: u64 = 350;
pub const IMPROVEMENT_PIXELS: f64 = 24.0;
pub const WIDGET_CLEARANCE: f64 = 4.0;
pub const MAX_RESERVED_RECTS: usize = 64;
pub const MAX_SCREEN_WIDTH: f64 = 4096.0;
pub const MAX_SCREEN_HEIGHT: f64 = 2160.0;
pub const COMPACT_WIDTHS: [f64; 3] = [168.0, 140.0, 100.0];
pub const SCREEN_MARGIN: f64 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

fn anchor_priority(anchor: Anchor) -> u8 {
    match anchor {
        Anchor::UpperLeft | Anchor::UpperRight => 0,
        Anchor::LowerCenter => 1,
        Anchor::MiddleLeft | Anchor::MiddleRight => 2,
        Anchor::LowerLeft | Anchor::LowerRight => 3,
        Anchor::UpperCenter => 4,
        Anchor::Search => 5,
    }
}

fn anchored(safe: Rect, width: f64, anchor: Anchor) -> Option<Placement> {
    let center_x = safe.x + ((safe.width - width) * 0.5).floor();
    let (x, y) = match anchor {
        Anchor::UpperLeft => (safe.x + 12.0, safe.y + 68.0),
        Anchor::UpperRight => (safe.right() - 12.0 - width, safe.y + 68.0),
        Anchor::LowerCenter => (center_x, safe.bottom() - 98.0),
        Anchor::MiddleLeft => (safe.x + 12.0, safe.y + (safe.height * 0.5 - 20.0).floor()),
        Anchor::MiddleRight => (
            safe.right() - 12.0 - width,
            safe.y + (safe.height * 0.5 - 20.0).floor(),
        ),
        Anchor::LowerLeft => (safe.x + 12.0, safe.bottom() - 98.0),
        Anchor::LowerRight => (safe.right() - 12.0 - width, safe.bottom() - 98.0),
        Anchor::UpperCenter => (center_x, safe.y + 12.0),
        Anchor::Search => return None,
    };
    Some(Placement {
        anchor,
        screen_rect: Rect::new(x, y, width, PANEL_HEIGHT),
    })
}

fn fixed_preference(
    safe: Rect,
    width: f64,
    head: Point,
    clear: &impl Fn(Placement) -> bool,
    compact: bool,
) -> Option<Placement> {
    let candidate = |anchor| anchored(safe, width, anchor).filter(|c| clear(*c));
    let pair = |left, right| match (candidate(left), candidate(right)) {
        (Some(left), Some(right)) => Some(
            if right.screen_rect.distance_to(head) > left.screen_rect.distance_to(head) {
                right
            } else {
                left
            },
        ),
        (Some(c), None) | (None, Some(c)) => Some(c),
        (None, None) => None,
    };
    pair(Anchor::UpperLeft, Anchor::UpperRight)
        .or_else(|| candidate(Anchor::LowerCenter))
        .or_else(|| {
            compact
                .then(|| pair(Anchor::MiddleLeft, Anchor::MiddleRight))
                .flatten()
        })
        .or_else(|| {
            compact
                .then(|| pair(Anchor::LowerLeft, Anchor::LowerRight))
                .flatten()
        })
        .or_else(|| compact.then(|| candidate(Anchor::UpperCenter)).flatten())
}

#[derive(Clone, Copy, Default)]
struct Interval {
    left: f64,
    right: f64,
}

/// Exhaustive integer-origin compact search. For each permitted y row, sorted
/// forbidden x intervals skip obstructed spans without scanning screen pixels.
/// At most2160rows,66intervals/row,3widths; arrays/sort use no heap allocation.
fn search_free_rectangle(
    width: f64,
    usable: Rect,
    reserved: &[Rect],
    discs: [Option<ScreenDisc>; 2],
    clear: &impl Fn(Placement) -> bool,
) -> Option<Placement> {
    if usable.width < width || usable.height < PANEL_HEIGHT || !usable.valid() {
        return None;
    }
    if reserved
        .iter()
        .any(|r| r.expanded(WIDGET_CLEARANCE).contains(usable))
    {
        return None;
    }
    let corners = [
        Point::new(usable.x, usable.y),
        Point::new(usable.right(), usable.y),
        Point::new(usable.x, usable.bottom()),
        Point::new(usable.right(), usable.bottom()),
    ];
    if discs.into_iter().flatten().any(|d| {
        corners
            .iter()
            .all(|c| (c.x - d.center.x).hypot(c.y - d.center.y) <= d.radius)
    }) {
        return None;
    }
    let max_x = usable.right() - width;
    for y in usable.y as i32..=(usable.bottom() - PANEL_HEIGHT) as i32 {
        let y = f64::from(y);
        let mut intervals = [Interval::default(); MAX_RESERVED_RECTS + 2];
        let mut count = 0;
        for reserved in reserved {
            let blocked = reserved.expanded(WIDGET_CLEARANCE);
            if y <= blocked.bottom() && y + PANEL_HEIGHT >= blocked.y {
                intervals[count] = Interval {
                    left: blocked.x - width,
                    right: blocked.right(),
                };
                count += 1;
            }
        }
        for disc in discs.into_iter().flatten() {
            let dy = (disc.center.y - disc.center.y.clamp(y, y + PANEL_HEIGHT)).abs();
            if dy <= disc.radius {
                let reach = if disc.radius == 0.0 {
                    0.0
                } else {
                    disc.radius * (1.0 - (dy / disc.radius).powi(2)).max(0.0).sqrt()
                };
                intervals[count] = Interval {
                    left: disc.center.x - reach - width,
                    right: disc.center.x + reach,
                };
                count += 1;
            }
        }
        intervals[..count].sort_unstable_by(|a, b| a.left.total_cmp(&b.left));
        let mut x = usable.x;
        while x <= max_x {
            let probe = |x| {
                let candidate = Placement {
                    anchor: Anchor::Search,
                    screen_rect: Rect::new(x, y, width, PANEL_HEIGHT),
                };
                clear(candidate).then_some(candidate)
            };
            // Validate interval boundaries with the collision predicate itself:
            // sqrt rounding can either underestimate or overestimate a tangent.
            if let Some(candidate) = probe(x) {
                return Some(candidate);
            }
            for interval in &intervals[..count] {
                if x < interval.left {
                    break;
                }
                if x <= interval.right {
                    let edge = interval.right.floor().min(max_x);
                    if edge >= x
                        && let Some(candidate) = probe(edge)
                    {
                        return Some(candidate);
                    }
                    if interval.right >= max_x {
                        x = max_x + 1.0;
                        break;
                    }
                    x = interval.right.floor() + 1.0;
                }
            }
            if x > max_x {
                break;
            }
            let candidate = Placement {
                anchor: Anchor::Search,
                screen_rect: Rect::new(x, y, width, PANEL_HEIGHT),
            };
            if clear(candidate) {
                return Some(candidate);
            }
            // sqrt can round a tangent interval below an integer while hypot
            // correctly rejects it. Continue the row instead of the width.
            x += 1.0;
        }
    }
    None
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn valid(self) -> bool {
        [
            self.x,
            self.y,
            self.width,
            self.height,
            self.right(),
            self.bottom(),
        ]
        .into_iter()
        .all(f64::is_finite)
            && self.width > 0.0
            && self.height > 0.0
    }

    pub fn right(self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(self) -> f64 {
        self.y + self.height
    }

    pub fn contains(self, other: Self) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    /// Touching edges count as occupied to preserve explicit clearance.
    pub fn intersects(self, other: Self) -> bool {
        self.x <= other.right()
            && self.right() >= other.x
            && self.y <= other.bottom()
            && self.bottom() >= other.y
    }

    pub fn expanded(self, padding: f64) -> Self {
        Self::new(
            self.x - padding,
            self.y - padding,
            self.width + padding * 2.0,
            self.height + padding * 2.0,
        )
    }

    pub fn distance_to(self, point: Point) -> f64 {
        let dx = point.x - point.x.clamp(self.x, self.right());
        let dy = point.y - point.y.clamp(self.y, self.bottom());
        dx.hypot(dy)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Screen {
    pub width: f64,
    pub height: f64,
    /// Top, right, bottom, left; all measured in logical UI pixels.
    pub insets: [f64; 4],
}

impl Screen {
    pub const fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            insets: [0.0; 4],
        }
    }

    fn safe_rect(self) -> Option<Rect> {
        if !self.width.is_finite()
            || !self.height.is_finite()
            || self.width <= 0.0
            || self.height <= 0.0
            || self.width > MAX_SCREEN_WIDTH
            || self.height > MAX_SCREEN_HEIGHT
            || self.insets.iter().any(|v| !v.is_finite() || *v < 0.0)
        {
            return None;
        }
        let [top, right, bottom, left] = self.insets;
        // Snap inward: a fractional DOM safe inset must not produce soft rails
        // or move occupied panel cells under the excluded edge.
        let safe = Rect::new(
            left.ceil(),
            top.ceil(),
            (self.width - right).floor() - left.ceil(),
            (self.height - bottom).floor() - top.ceil(),
        );
        safe.valid().then_some(safe)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projection {
    /// Use the exact presentation camera, including enabled shake.
    pub camera: Point,
    /// Actual eased camera zoom, not its requested destination.
    pub zoom: f64,
}

impl Projection {
    pub fn project(self, screen: Screen, world: Point) -> Option<Point> {
        if screen.safe_rect().is_none()
            || !self.camera.finite()
            || !self.zoom.is_finite()
            // Production CameraZoom is f32: its legitimate 1.6 endpoint
            // promotes to 1.600000023841858, not exactly f64 1.6.
            || !(0.6..=f64::from(1.6_f32)).contains(&self.zoom)
            || !world.finite()
        {
            return None;
        }
        let result = Point::new(
            (world.x - self.camera.x) * self.zoom + screen.width * 0.5,
            (world.y - self.camera.y) * self.zoom + screen.height * 0.5,
        );
        result.finite().then_some(result)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldDisc {
    pub center: Point,
    /// Already includes actor scale; caller can use actual body/core radius.
    pub world_radius: f64,
    pub padding_pixels: f64,
    pub minimum_pixels: f64,
}

impl WorldDisc {
    pub fn core(center: Point, world_radius: f64) -> Self {
        Self {
            center,
            world_radius,
            padding_pixels: 8.0,
            minimum_pixels: 32.0,
        }
    }

    pub fn player(center: Point, world_radius: f64) -> Self {
        Self {
            center,
            world_radius,
            padding_pixels: 4.0,
            minimum_pixels: 24.0,
        }
    }

    pub fn project(self, screen: Screen, projection: Projection) -> Option<ScreenDisc> {
        if [self.world_radius, self.padding_pixels, self.minimum_pixels]
            .into_iter()
            .any(|v| !v.is_finite() || v < 0.0)
        {
            return None;
        }
        let radius =
            (self.world_radius * projection.zoom + self.padding_pixels).max(self.minimum_pixels);
        let center = projection.project(screen, self.center)?;
        radius.is_finite().then_some(ScreenDisc { center, radius })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenDisc {
    pub center: Point,
    pub radius: f64,
}

impl ScreenDisc {
    pub fn intersects(self, rect: Rect) -> bool {
        rect.distance_to(self.center) <= self.radius
    }

    pub fn visible(self, screen: Screen) -> bool {
        self.intersects(Rect::new(0.0, 0.0, screen.width, screen.height))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    UpperLeft,
    UpperRight,
    LowerCenter,
    MiddleLeft,
    MiddleRight,
    LowerLeft,
    LowerRight,
    UpperCenter,
    Search,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub anchor: Anchor,
    pub screen_rect: Rect,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    Placed(Placement),
    /// No clear compact rectangle within twelve-pixel safe-screen margins.
    /// The prototype never covers a protected core or control to force a fit.
    NoClearAnchor,
    /// Invalid input, including extents beyond4096x2160, preserves state.
    InvalidInput,
}

pub struct FrameInput<'a> {
    pub screen: Screen,
    pub projection: Projection,
    pub head: Point,
    pub core: Option<WorldDisc>,
    pub player: Option<WorldDisc>,
    /// Actual full logical-screen bounds of visible widgets only.
    pub reserved: &'a [Rect],
    /// Presentation clock; rewind resets pending safe-anchor promotion.
    pub now_ms: u64,
    /// A caller-assigned epoch changed only at real encounter/phase changes.
    pub phase_epoch: u64,
}

/// Only no-fit results require exact input geometry. Successful compact
/// preferences may be reused briefly after both rectangles pass fresh safety
/// checks. Fixed-size storage avoids an allocation on the presentation path.
#[derive(Clone, Debug, PartialEq)]
struct SearchGeometry {
    screen: Screen,
    projection: Projection,
    core: Option<WorldDisc>,
    player: Option<WorldDisc>,
    reserved: [Rect; MAX_RESERVED_RECTS],
    count: usize,
}

impl SearchGeometry {
    fn capture(input: &FrameInput<'_>) -> Self {
        let mut reserved = [Rect::new(0.0, 0.0, 0.0, 0.0); MAX_RESERVED_RECTS];
        reserved[..input.reserved.len()].copy_from_slice(input.reserved);
        Self {
            screen: input.screen,
            projection: input.projection,
            core: input.core,
            player: input.player,
            reserved,
            count: input.reserved.len(),
        }
    }

    fn matches(&self, input: &FrameInput<'_>) -> bool {
        self.screen == input.screen
            && self.projection == input.projection
            && self.core == input.core
            && self.player == input.player
            && self.reserved[..self.count] == *input.reserved
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SearchMemo {
    geometry: SearchGeometry,
    preferred: Option<Placement>,
    at_ms: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutController {
    retained: Option<Placement>,
    challenger: Option<(Placement, u64)>,
    last_now: Option<u64>,
    last_phase: Option<u64>,
    search_memo: Option<SearchMemo>,
}

impl LayoutController {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn retained_anchor(&self) -> Option<Anchor> {
        self.retained.map(|p| p.anchor)
    }

    pub fn update(&mut self, input: &FrameInput<'_>) -> Outcome {
        let Some(safe) = input.screen.safe_rect() else {
            return Outcome::InvalidInput;
        };
        let Some(head) = input.projection.project(input.screen, input.head) else {
            return Outcome::InvalidInput;
        };
        if input.reserved.len() > MAX_RESERVED_RECTS
            || input
                .reserved
                .iter()
                .any(|r| !r.valid() || !r.expanded(WIDGET_CLEARANCE).valid())
        {
            return Outcome::InvalidInput;
        }
        let project = |disc: Option<WorldDisc>| -> Result<Option<ScreenDisc>, ()> {
            match disc {
                Some(disc) => disc
                    .project(input.screen, input.projection)
                    .map(Some)
                    .ok_or(()),
                None => Ok(None),
            }
        };
        let (Ok(core), Ok(player)) = (project(input.core), project(input.player)) else {
            return Outcome::InvalidInput;
        };
        let discs = [core, player];
        let usable = Rect::new(
            safe.x + SCREEN_MARGIN,
            safe.y + SCREEN_MARGIN,
            safe.width - SCREEN_MARGIN * 2.0,
            safe.height - SCREEN_MARGIN * 2.0,
        );
        let clear = |candidate: Placement| -> bool {
            usable.valid()
                && usable.contains(candidate.screen_rect)
                && !discs
                    .into_iter()
                    .flatten()
                    .any(|d| d.visible(input.screen) && d.intersects(candidate.screen_rect))
                && !input.reserved.iter().any(|r| {
                    candidate
                        .screen_rect
                        .intersects(r.expanded(WIDGET_CLEARANCE))
                })
        };
        let rewound = self.last_now.is_some_and(|last| input.now_ms < last);
        let phase_changed = self
            .last_phase
            .is_some_and(|last| input.phase_epoch != last);
        self.last_now = Some(input.now_ms);
        self.last_phase = Some(input.phase_epoch);
        if rewound {
            self.challenger = None;
        }
        // Check the retained rectangle before the expensive rescue search.
        // Any unsafe retained placement forces an immediate fresh preference.
        let current = self.retained.and_then(|retained| {
            let reflowed =
                anchored(safe, retained.screen_rect.width, retained.anchor).unwrap_or(retained);
            clear(reflowed).then_some(reflowed)
        });
        // Primary fixed anchors are always evaluated with this frame's head,
        // target discs and actual reserved widgets, even on a cache hit.
        let mut preferred = fixed_preference(safe, PANEL_WIDTH, head, &clear, false);
        if preferred.is_none() {
            let memo = self
                .search_memo
                .as_ref()
                .filter(|memo| match memo.preferred {
                    Some(candidate) => {
                        current.is_some()
                            && !rewound
                            && !phase_changed
                            && memo.geometry.screen == input.screen
                            && input.now_ms.saturating_sub(memo.at_ms) <= HOLD_MS
                            && clear(candidate)
                    }
                    None => memo.geometry.matches(input),
                });
            if let Some(memo) = memo {
                preferred = memo.preferred;
            } else {
                for width in COMPACT_WIDTHS {
                    preferred = fixed_preference(safe, width, head, &clear, true).or_else(|| {
                        search_free_rectangle(width, usable, input.reserved, discs, &clear)
                    });
                    if preferred.is_some() {
                        break;
                    }
                }
                self.search_memo = Some(SearchMemo {
                    geometry: SearchGeometry::capture(input),
                    preferred,
                    at_ms: input.now_ms,
                });
            }
        }
        let Some(preferred) = preferred else {
            self.challenger = None;
            return Outcome::NoClearAnchor;
        };
        let Some(current) = current else {
            self.retained = Some(preferred);
            self.challenger = None;
            return Outcome::Placed(preferred);
        };
        self.retained = Some(current);
        if current == preferred {
            self.challenger = None;
            return Outcome::Placed(current);
        }
        let improvement =
            preferred.screen_rect.distance_to(head) - current.screen_rect.distance_to(head);
        let promotion = preferred.screen_rect.width > current.screen_rect.width
            || anchor_priority(preferred.anchor) < anchor_priority(current.anchor);
        if !promotion && improvement < IMPROVEMENT_PIXELS {
            self.challenger = None;
            return Outcome::Placed(current);
        }
        let since = match self.challenger {
            Some((candidate, since))
                if candidate == preferred
                    || (candidate.anchor != Anchor::Search
                        && candidate.anchor == preferred.anchor
                        && candidate.screen_rect.width == preferred.screen_rect.width) =>
            {
                since
            }
            _ => input.now_ms,
        };
        if phase_changed || input.now_ms.saturating_sub(since) >= HOLD_MS {
            self.retained = Some(preferred);
            self.challenger = None;
            Outcome::Placed(preferred)
        } else {
            self.challenger = Some((preferred, since));
            Outcome::Placed(current)
        }
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;

    const COMPACT_WIDGETS: [Rect; 7] = [
        Rect::new(12.0, 12.0, 160.0, 28.0),
        Rect::new(106.0, 12.0, 142.0, 24.0),
        Rect::new(278.0, 24.0, 28.0, 36.0),
        Rect::new(72.0, 132.0, 176.0, 36.0),
        Rect::new(260.0, 36.0, 48.0, 48.0),
        Rect::new(28.0, 102.0, 56.0, 56.0),
        Rect::new(244.0, 102.0, 56.0, 56.0),
    ];

    fn compact_frame() -> FrameInput<'static> {
        FrameInput {
            screen: Screen::new(320.0, 180.0),
            projection: Projection {
                camera: Point::new(0.0, 0.0),
                zoom: f64::from(1.6_f32),
            },
            head: Point::new(0.0, -210.0),
            core: None,
            player: Some(WorldDisc::player(Point::new(0.0, 0.0), 20.0)),
            reserved: &COMPACT_WIDGETS,
            now_ms: 0,
            phase_epoch: 0,
        }
    }

    #[test]
    fn retained_rescue_reuses_only_a_freshly_clear_preference_for_350_ms() {
        let mut frame = compact_frame();
        let mut controller = LayoutController::default();
        let initial = controller.update(&frame);
        assert!(matches!(initial, Outcome::Placed(p) if p.anchor == Anchor::Search));
        for time in [1, 100, 349, 350] {
            frame.now_ms = time;
            frame.head.x += 0.1;
            assert_eq!(controller.update(&frame), initial);
            assert_eq!(controller.search_memo.as_ref().unwrap().at_ms, 0);
        }
        frame.now_ms = 351;
        assert_eq!(controller.update(&frame), initial);
        assert_eq!(controller.search_memo.as_ref().unwrap().at_ms, 351);
    }

    #[test]
    fn moved_core_or_new_widget_that_blocks_retained_rescue_invalidates_immediately() {
        for block_with_core in [true, false] {
            let mut frame = compact_frame();
            let mut controller = LayoutController::default();
            let Outcome::Placed(initial) = controller.update(&frame) else {
                panic!("missing compact rescue")
            };
            let extra = [initial.screen_rect];
            if block_with_core {
                let rect = initial.screen_rect;
                frame.core = Some(WorldDisc::core(
                    Point::new(
                        (rect.x + rect.width * 0.5 - frame.screen.width * 0.5)
                            / frame.projection.zoom,
                        (rect.y + rect.height * 0.5 - frame.screen.height * 0.5)
                            / frame.projection.zoom,
                    ),
                    0.0,
                ));
            } else {
                frame.reserved = &extra;
            }
            frame.now_ms = 1;
            let outcome = controller.update(&frame);
            assert_ne!(outcome, Outcome::Placed(initial));
            assert_eq!(controller.search_memo.as_ref().unwrap().at_ms, 1);
            if let Outcome::Placed(p) = outcome {
                assert!(reference_clear(
                    p.screen_rect,
                    frame.reserved,
                    [
                        frame
                            .core
                            .map(|d| d.project(frame.screen, frame.projection).unwrap()),
                        frame
                            .player
                            .map(|d| d.project(frame.screen, frame.projection).unwrap())
                    ]
                ));
            }
        }
    }

    #[test]
    fn exact_no_fit_cache_invalidates_on_every_geometry_input_change() {
        let mut frame = compact_frame();
        let full = [Rect::new(0.0, 0.0, 400.0, 240.0)];
        frame.reserved = &full;
        let mut controller = LayoutController::default();
        assert_eq!(controller.update(&frame), Outcome::NoClearAnchor);
        frame.now_ms = 100_000;
        frame.head.x = 99.0; // Preference changes cannot create empty space.
        assert_eq!(controller.update(&frame), Outcome::NoClearAnchor);
        assert_eq!(controller.search_memo.as_ref().unwrap().at_ms, 0);
        frame.core = Some(WorldDisc::core(Point::new(0.0, 0.0), 0.0));
        frame.now_ms += 1;
        assert_eq!(controller.update(&frame), Outcome::NoClearAnchor);
        assert_eq!(controller.search_memo.as_ref().unwrap().at_ms, frame.now_ms);
        frame.player.as_mut().unwrap().world_radius += 1.0;
        frame.now_ms += 1;
        assert_eq!(controller.update(&frame), Outcome::NoClearAnchor);
        assert_eq!(controller.search_memo.as_ref().unwrap().at_ms, frame.now_ms);
        frame.screen.insets[0] = 1.0;
        frame.now_ms += 1;
        assert_eq!(controller.update(&frame), Outcome::NoClearAnchor);
        assert_eq!(controller.search_memo.as_ref().unwrap().at_ms, frame.now_ms);
        frame.projection.camera.x += 1.0;
        frame.now_ms += 1;
        assert_eq!(controller.update(&frame), Outcome::NoClearAnchor);
        assert_eq!(controller.search_memo.as_ref().unwrap().at_ms, frame.now_ms);
        let moved = [Rect::new(0.0, 0.0, 401.0, 240.0)];
        frame.reserved = &moved;
        frame.now_ms += 1;
        assert_eq!(controller.update(&frame), Outcome::NoClearAnchor);
        assert_eq!(controller.search_memo.as_ref().unwrap().at_ms, frame.now_ms);
        frame.reserved = &[];
        frame.core = None;
        frame.now_ms += 1;
        assert!(matches!(controller.update(&frame), Outcome::Placed(_)));
    }

    #[test]
    fn phase_change_and_clock_rewind_refresh_compact_preference() {
        let mut frame = compact_frame();
        frame.now_ms = 100;
        let mut controller = LayoutController::default();
        let initial = controller.update(&frame);
        frame.now_ms = 101;
        frame.phase_epoch = 1;
        assert_eq!(controller.update(&frame), initial);
        assert_eq!(controller.search_memo.as_ref().unwrap().at_ms, 101);
        frame.now_ms = 50;
        assert_eq!(controller.update(&frame), initial);
        assert_eq!(controller.search_memo.as_ref().unwrap().at_ms, 50);
        controller.reset();
        assert!(controller.search_memo.is_none());
    }

    #[test]
    fn cached_rescue_never_suppresses_newly_clear_primary_promotion() {
        let mut frame = FrameInput {
            screen: Screen::new(400.0, 240.0),
            projection: Projection {
                camera: Point::new(0.0, 0.0),
                zoom: 1.0,
            },
            head: Point::new(0.0, -210.0),
            core: None,
            player: None,
            reserved: &[],
            now_ms: 0,
            phase_epoch: 0,
        };
        let blocked = [
            Rect::new(0.0, 60.0, 400.0, 180.0),
            Rect::new(120.0, 0.0, 10.0, 24.0),
        ];
        frame.reserved = &blocked;
        let mut controller = LayoutController::default();
        let Outcome::Placed(compact) = controller.update(&frame) else {
            panic!("no rescue")
        };
        assert_eq!(compact.anchor, Anchor::Search);
        frame.reserved = &[];
        frame.now_ms = 10;
        assert_eq!(controller.update(&frame), Outcome::Placed(compact));
        frame.now_ms = 359;
        assert_eq!(controller.update(&frame), Outcome::Placed(compact));
        frame.now_ms = 360;
        assert!(
            matches!(controller.update(&frame), Outcome::Placed(p) if p.screen_rect.width == 220.0)
        );
    }

    #[test]
    fn opening_each_geometry_class_escapes_cached_no_fit_immediately() {
        for change in 0..5 {
            let mut frame = compact_frame();
            frame.reserved = &[];
            let full = [Rect::new(0.0, 0.0, 400.0, 240.0)];
            match change {
                0 => frame.screen = Screen::new(96.0, 60.0),
                1 => frame.core = Some(WorldDisc::core(Point::new(0.0, 0.0), 1000.0)),
                2 => frame.player.as_mut().unwrap().world_radius = 1000.0,
                3 => frame.core = Some(WorldDisc::core(Point::new(0.0, 0.0), 80.0)),
                4 => frame.reserved = &full,
                _ => unreachable!(),
            }
            let mut controller = LayoutController::default();
            assert_eq!(
                controller.update(&frame),
                Outcome::NoClearAnchor,
                "class{change}"
            );
            frame.now_ms = 1;
            match change {
                0 => frame.screen = Screen::new(320.0, 180.0),
                1 => frame.core.as_mut().unwrap().world_radius = 0.0,
                2 => frame.player.as_mut().unwrap().world_radius = 20.0,
                3 => frame.projection.zoom = 0.6,
                4 => frame.reserved = &[],
                _ => unreachable!(),
            }
            assert!(
                matches!(controller.update(&frame), Outcome::Placed(_)),
                "class{change}"
            );
        }
    }

    #[test]
    fn unsafe_cached_preference_refreshes_while_current_remains_safe() {
        let current = Placement {
            anchor: Anchor::Search,
            screen_rect: Rect::new(12.0, 12.0, 100.0, 40.0),
        };
        let mut frame = FrameInput {
            screen: Screen::new(400.0, 240.0),
            projection: Projection {
                camera: Point::new(0.0, 0.0),
                zoom: 1.0,
            },
            head: Point::new(0.0, -210.0),
            core: None,
            player: None,
            reserved: &[],
            now_ms: 0,
            phase_epoch: 0,
        };
        let fixed = [
            Rect::new(0.0, 60.0, 400.0, 180.0),
            Rect::new(120.0, 0.0, 10.0, 24.0),
        ];
        frame.reserved = &fixed;
        let mut controller = LayoutController {
            retained: Some(current),
            ..Default::default()
        };
        assert_eq!(controller.update(&frame), Outcome::Placed(current));
        let old_preference = controller.search_memo.as_ref().unwrap().preferred.unwrap();
        assert_eq!(old_preference.screen_rect.width, 168.0);
        let changed = [fixed[0], fixed[1], Rect::new(300.0, 12.0, 1.0, 40.0)];
        frame.reserved = &changed;
        frame.now_ms = 1;
        assert_eq!(controller.update(&frame), Outcome::Placed(current));
        let memo = controller.search_memo.as_ref().unwrap();
        assert_eq!(memo.at_ms, 1);
        assert_ne!(memo.preferred.unwrap(), old_preference);
        assert_eq!(memo.preferred.unwrap().screen_rect.width, 140.0);
        assert!(reference_clear(
            current.screen_rect,
            frame.reserved,
            [None, None]
        ));
    }

    fn reference_clear(rect: Rect, reserved: &[Rect], discs: [Option<ScreenDisc>; 2]) -> bool {
        for d in discs.into_iter().flatten() {
            let dx = (d.center.x - rect.x).min(0.0).abs() + (d.center.x - rect.right()).max(0.0);
            let dy = (d.center.y - rect.y).min(0.0).abs() + (d.center.y - rect.bottom()).max(0.0);
            if dx.hypot(dy) <= d.radius {
                return false;
            }
        }
        !reserved
            .iter()
            .any(|r| rect.intersects(r.expanded(WIDGET_CLEARANCE)))
    }

    fn brute(
        width: f64,
        usable: Rect,
        reserved: &[Rect],
        discs: [Option<ScreenDisc>; 2],
    ) -> Option<Rect> {
        if usable.width < width || usable.height < PANEL_HEIGHT {
            return None;
        }
        for y in usable.y as i32..=(usable.bottom() - PANEL_HEIGHT) as i32 {
            for x in usable.x as i32..=(usable.right() - width) as i32 {
                let rect = Rect::new(f64::from(x), f64::from(y), width, PANEL_HEIGHT);
                if reference_clear(rect, reserved, discs) {
                    return Some(rect);
                }
            }
        }
        None
    }

    #[test]
    fn rounded_disc_tangency_continues_search_to_next_integer() {
        let usable = Rect::new(64.0, 0.0, 101.0, 40.0);
        let disc = ScreenDisc {
            center: Point::new(44.8, -25.6),
            radius: 32.0,
        };
        assert_eq!((64.0_f64 - 44.8).hypot(25.6), 32.0);
        let discs = [Some(disc), None];
        let clear = |p: Placement| reference_clear(p.screen_rect, &[], discs);
        let found = search_free_rectangle(100.0, usable, &[], discs, &clear).unwrap();
        assert_eq!(found.screen_rect.x, 65.0);
        assert_eq!(Some(found.screen_rect), brute(100.0, usable, &[], discs));
    }

    #[test]
    fn overestimated_interval_keeps_integer_boundary_accepted_by_hypot() {
        let usable = Rect::new(64.0, 0.0, 100.0, 40.0);
        let discs = [
            Some(ScreenDisc {
                center: Point::new(40.000012000003, -0.024),
                radius: 24.0,
            }),
            None,
        ];
        assert!((64.0_f64 - 40.000012000003).hypot(0.024) > 24.0);
        let clear = |p: Placement| reference_clear(p.screen_rect, &[], discs);
        let found = search_free_rectangle(100.0, usable, &[], discs, &clear).unwrap();
        assert_eq!(found.screen_rect.x, 64.0);
        assert_eq!(Some(found.screen_rect), brute(100.0, usable, &[], discs));
    }

    #[test]
    fn forbidden_interval_search_matches_direct_integer_origin_oracle() {
        let mut state = 0x4752_4f59_u32;
        let mut next = || {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            state
        };
        let mut checked = 0;
        for case in 0..400 {
            let usable = Rect::new(
                (case % 7) as f64,
                (case % 3) as f64,
                104.0 + (next() % 80) as f64,
                40.0 + (next() % 42) as f64,
            );
            let reserved: [Rect; 3] = std::array::from_fn(|_| {
                Rect::new(
                    usable.x + (next() % 200) as f64 - 15.0,
                    usable.y + (next() % 100) as f64 - 10.0,
                    1.0 + (next() % 42) as f64,
                    1.0 + (next() % 50) as f64,
                )
            });
            let discs = std::array::from_fn(|_| {
                Some(ScreenDisc {
                    center: Point::new(
                        usable.x + (next() % 200) as f64 - 20.0 + (next() % 5) as f64 * 0.2,
                        usable.y + (next() % 100) as f64 - 20.0 + (next() % 5) as f64 * 0.2,
                    ),
                    radius: (next() % 42) as f64 + (next() % 5) as f64 * 0.2,
                })
            });
            for width in COMPACT_WIDTHS {
                let clear = |p: Placement| reference_clear(p.screen_rect, &reserved, discs);
                let found = search_free_rectangle(width, usable, &reserved, discs, &clear)
                    .map(|p| p.screen_rect);
                assert_eq!(
                    found,
                    brute(width, usable, &reserved, discs),
                    "case{case},width{width}"
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 1200);
    }

    #[test]
    fn moving_search_challenger_does_not_inherit_previous_rectangle_hold() {
        // Isolate the private retained/challenger representation so every
        // frame has a stable safe compact current but a moving wider rescue.
        let current = Placement {
            anchor: Anchor::Search,
            screen_rect: Rect::new(12.0, 12.0, 100.0, 40.0),
        };
        let mut controller = LayoutController {
            retained: Some(current),
            ..LayoutController::default()
        };
        let mut frame = FrameInput {
            screen: Screen::new(400.0, 240.0),
            projection: Projection {
                camera: Point::new(0.0, 0.0),
                zoom: 1.0,
            },
            head: Point::new(0.0, -210.0),
            core: None,
            player: None,
            reserved: &[],
            now_ms: 0,
            phase_epoch: 0,
        };
        // Reserve all fixed anchor regions, leaving a narrow horizontal strip
        // with a left edge that can move without touching the retained panel.
        let fixed = [
            Rect::new(0.0, 60.0, 400.0, 180.0),
            Rect::new(120.0, 0.0, 10.0, 24.0),
        ];
        frame.reserved = &fixed;
        let first = controller.update(&frame);
        assert_eq!(first, Outcome::Placed(current));
        let first_pending = controller.challenger.expect("wider search challenger");
        let moved = [fixed[0], Rect::new(120.0, 0.0, 11.0, 24.0)];
        frame.reserved = &moved;
        frame.now_ms = 349;
        assert_eq!(controller.update(&frame), Outcome::Placed(current));
        let pending = controller.challenger.unwrap();
        assert_ne!(pending.0.screen_rect, first_pending.0.screen_rect);
        assert_eq!(pending.1, 349);
        frame.now_ms = 350;
        assert_eq!(controller.update(&frame), Outcome::Placed(current));
        frame.now_ms = 699;
        let Outcome::Placed(promoted) = controller.update(&frame) else {
            panic!("missing promotion")
        };
        assert_eq!(promoted.screen_rect, pending.0.screen_rect);
    }
}
