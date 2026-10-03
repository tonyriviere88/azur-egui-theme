//! The chrome glyphs Azur's components draw, as painters.
//!
//! Azur ships its icons as inline SVG, so the faithful port is to paint them
//! rather than to depend on an icon font: nothing to install, nothing to embed,
//! and a checkmark that scales with the box it sits in.
//!
//! Every icon here has the same shape — it fills the rect it is handed, in the
//! color it is handed — so they are interchangeable, and so an application can
//! pass its own:
//!
//! ```
//! use azur_egui_theme::icons::Icon;
//! fn my_glyph(p: &egui::Painter, r: egui::Rect, c: egui::Color32) {
//!     p.circle_filled(r.center(), r.width() * 0.4, c);
//! }
//! let icon: Icon = &my_glyph;
//! ```
//!
//! This module is only about the glyphs the *system* needs — chevrons, checks,
//! window buttons, status marks. Domain icons stay in the application.
//!
//! # Drawing your own
//!
//! An application's icons sit right next to these — a `Print` icon in a
//! `CommandBar` between two built-in ones — so they have to be built the same
//! way: same grid, same weight, same terminals, one color from the caller.
//! [`Pen`] is that geometry, so an icon can be transcribed from an SVG rather
//! than eyeballed, and [`canvas`] / [`stroke_width`] are the two rules on their
//! own for anything hand-rolled.
//!
//! `docs/icons.md` in the repository is the full brief: what the contract is,
//! how big to draw, which color to use where, and a checklist to run an icon set
//! against.

use crate::tokens::focus_ring;
use egui::{pos2, Color32, CornerRadius, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};

/// Anything that can paint an icon into a rect: every function in this module,
/// and any closure with the same shape.
pub type Icon<'a> = &'a dyn Fn(&Painter, Rect, Color32);

/// **The brief for an application drawing its own icons**, in two parts: the
/// glyphs inside the window — the contract, the grid, the weight, which color
/// belongs where, how to transcribe an SVG — and the icon on the executable
/// itself, which is a raster mark under entirely different rules. A checklist
/// each.
///
/// This page is `docs/icons.md` in the repository, included here so its examples
/// are compiled by `cargo test` and cannot drift from the API they describe.
#[doc = include_str!("../docs/icons.md")]
pub mod guidelines {}

/// The stroke weight an icon drawn into `rect` uses: an eighth of the box, never
/// thinner than 1px, never heavier than 2px.
///
/// Azur's SVGs stroke at 1.4–1.6 in a 16-unit viewBox, and a hairline at 12px
/// still has to read; this is that, expressed as a function of the box so one
/// glyph body works at every size it is asked for.
///
/// An application-supplied icon should use this rather than a weight of its own,
/// so it does not read bolder or lighter than the chevron beside it.
pub fn stroke_width(rect: Rect) -> f32 {
    (rect.width() / 8.0).clamp(1.0, 2.0)
}

/// The square an icon is laid out in: the largest centred square inside `rect`.
///
/// Every glyph here starts with this, so an icon handed a non-square box stays
/// round instead of stretching.
pub fn canvas(rect: Rect) -> Rect {
    let side = rect.width().min(rect.height());
    Rect::from_center_size(rect.center(), Vec2::splat(side))
}

fn stroke_path(p: &Painter, points: Vec<Pos2>, color: Color32, width: f32) {
    p.add(Shape::line(points, Stroke::new(width, color)));
}

// ---------------------------------------------------------------------------
// Drawing an application's own icons
// ---------------------------------------------------------------------------

/// Azur's icon grid, for an application drawing its own glyphs.
///
/// Every coordinate is in **grid units** — the same numbers as the `viewBox` of
/// the SVGs Azur ships, so an icon is transcribed rather than re-invented. The
/// grid is 16 units by default; `<svg viewBox="0 0 20 20">` art needs
/// [`units(20.0)`](Self::units).
///
/// The pen fills the largest centred square in the rect it is handed, strokes at
/// [`stroke_width`], and paints everything in the one color the caller passed —
/// which is what makes an application icon follow variant, hover, disabled and
/// the light/dark switch without knowing they exist.
///
/// ```
/// use azur_egui_theme::icons::{Icon, Pen};
/// use egui::{Color32, Painter, Rect};
///
/// // <svg viewBox="0 0 20 20">
/// //   <path d="M3 9.5L10 4l7 5.5V16a1 1 0 01-1 1h-3v-5H7v5H4a1 1 0 01-1-1V9.5z"/>
/// // </svg>
/// fn home(painter: &Painter, rect: Rect, color: Color32) {
///     let pen = Pen::new(painter, rect, color).units(20.0);
///     // Roof, walls and door in one outline, the way the `d` attribute has it.
///     // Its `a1 1 0 01` corners are 1px at a 20px box, so they are dropped.
///     pen.closed_path(&[
///         (3.0, 9.5),
///         (10.0, 4.0),
///         (17.0, 9.5),
///         (17.0, 17.0),
///         (12.0, 17.0),
///         (12.0, 12.0),
///         (8.0, 12.0),
///         (8.0, 17.0),
///         (3.0, 17.0),
///     ]);
/// }
///
/// let icon: Icon = &home;
/// ```
pub struct Pen<'a> {
    painter: &'a Painter,
    canvas: Rect,
    color: Color32,
    units: f32,
    width: f32,
}

impl<'a> Pen<'a> {
    /// A pen on a 16-unit grid, at the weight an icon this size is drawn with.
    pub fn new(painter: &'a Painter, rect: Rect, color: Color32) -> Self {
        let canvas = canvas(rect);
        Self {
            painter,
            canvas,
            color,
            units: 16.0,
            width: stroke_width(canvas),
        }
    }

    /// Change the grid — `20.0` for `viewBox="0 0 20 20"` art, `12.0` for 12.
    ///
    /// This rescales the coordinates, not the icon: the glyph still fills the same
    /// box, so a 20-unit icon sits beside a 16-unit one at the same visual size.
    pub fn units(mut self, units: f32) -> Self {
        self.units = units.max(1.0);
        self
    }

    /// Take the weight from the artwork's own `stroke-width`, in grid units.
    ///
    /// Faithful to the SVG, and half a pixel lighter than the icons in this module
    /// at 16px and up — use it for a whole set or not at all, never for one glyph
    /// in a row of others. Never thinner than 1px, or it disappears.
    ///
    /// Call it after [`units`](Self::units): the width is converted with the grid
    /// in force at the time.
    pub fn stroke_units(mut self, width: f32) -> Self {
        self.width = (width * self.scale()).max(1.0);
        self
    }

    /// Set the weight in pixels, bypassing both rules above.
    pub fn stroke_px(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// The square being drawn into.
    pub fn canvas(&self) -> Rect {
        self.canvas
    }

    /// Pixels per grid unit.
    pub fn scale(&self) -> f32 {
        self.canvas.width() / self.units
    }

    /// The stroke every method here paints with, for anything they do not cover.
    pub fn stroke(&self) -> Stroke {
        Stroke::new(self.width, self.color)
    }

    /// A grid coordinate as a screen position.
    pub fn at(&self, x: f32, y: f32) -> Pos2 {
        let s = self.scale();
        pos2(self.canvas.left() + x * s, self.canvas.top() + y * s)
    }

    fn points(&self, pts: &[(f32, f32)]) -> Vec<Pos2> {
        pts.iter().map(|&(x, y)| self.at(x, y)).collect()
    }

    /// `<line>`.
    pub fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.painter
            .line_segment([self.at(x1, y1), self.at(x2, y2)], self.stroke());
    }

    /// An open stroked polyline — the `M … L …` of a `<path>`, with round joins.
    pub fn path(&self, pts: &[(f32, f32)]) {
        self.painter
            .add(Shape::line(self.points(pts), self.stroke()));
    }

    /// A stroked polyline that closes back on itself — a `<path>` ending in `Z`,
    /// or a `<polygon>`.
    pub fn closed_path(&self, pts: &[(f32, f32)]) {
        self.painter
            .add(Shape::closed_line(self.points(pts), self.stroke()));
    }

    /// A filled polygon, for the solid marks — a caret, a sort triangle. Must be
    /// convex.
    pub fn polygon(&self, pts: &[(f32, f32)]) {
        self.painter.add(Shape::convex_polygon(
            self.points(pts),
            self.color,
            Stroke::NONE,
        ));
    }

    /// `<circle>`, stroked.
    pub fn circle(&self, cx: f32, cy: f32, r: f32) {
        self.painter
            .circle_stroke(self.at(cx, cy), r * self.scale(), self.stroke());
    }

    /// `<circle>`, filled — a dot, a badge, the tittle of an `i`.
    pub fn circle_filled(&self, cx: f32, cy: f32, r: f32) {
        self.painter
            .circle_filled(self.at(cx, cy), r * self.scale(), self.color);
    }

    /// `<rect>`, stroked. `rx` is the corner radius in grid units.
    pub fn rect(&self, x: f32, y: f32, width: f32, height: f32, rx: f32) {
        self.painter.rect_stroke(
            self.box_of(x, y, width, height),
            self.corner(rx),
            self.stroke(),
            StrokeKind::Middle,
        );
    }

    /// `<rect>`, filled.
    pub fn rect_filled(&self, x: f32, y: f32, width: f32, height: f32, rx: f32) {
        self.painter.rect_filled(
            self.box_of(x, y, width, height),
            self.corner(rx),
            self.color,
        );
    }

    fn box_of(&self, x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect::from_min_max(self.at(x, y), self.at(x + width, y + height))
    }

    fn corner(&self, rx: f32) -> CornerRadius {
        CornerRadius::same((rx * self.scale()).round().clamp(0.0, 255.0) as u8)
    }
}

// ---------------------------------------------------------------------------
// Selection marks
// ---------------------------------------------------------------------------

/// A checkmark. Checkbox, and the tick beside a chosen `Select` option.
pub fn check(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect);
    let w = stroke_width(r).max(1.4);
    stroke_path(
        p,
        vec![
            pos2(r.left() + r.width() * 0.18, r.top() + r.height() * 0.52),
            pos2(r.left() + r.width() * 0.42, r.top() + r.height() * 0.76),
            pos2(r.left() + r.width() * 0.84, r.top() + r.height() * 0.26),
        ],
        color,
        w,
    );
}

/// A dash — a checkbox in its indeterminate state.
pub fn dash(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect);
    let w = stroke_width(r).max(1.4);
    p.line_segment(
        [
            pos2(r.left() + r.width() * 0.22, r.center().y),
            pos2(r.right() - r.width() * 0.22, r.center().y),
        ],
        Stroke::new(w, color),
    );
}

/// A filled dot — the selected radio button, and a status dot.
pub fn dot(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect);
    p.circle_filled(r.center(), r.width() * 0.5, color);
}

// ---------------------------------------------------------------------------
// Chevrons and carets
// ---------------------------------------------------------------------------

fn chevron(p: &Painter, rect: Rect, color: Color32, turns: f32) {
    let r = canvas(rect);
    let w = stroke_width(r).max(1.3);
    // Drawn pointing down, then rotated: `turns` is quarter turns clockwise.
    let (sin, cos) = (turns * std::f32::consts::FRAC_PI_2).sin_cos();
    let rotate = |dx: f32, dy: f32| {
        Pos2::new(
            r.center().x + dx * cos - dy * sin,
            r.center().y + dx * sin + dy * cos,
        )
    };
    let h = r.width() * 0.28;
    stroke_path(
        p,
        vec![
            rotate(-h, -h * 0.55),
            rotate(0.0, h * 0.55),
            rotate(h, -h * 0.55),
        ],
        color,
        w,
    );
}

/// A chevron pointing down — `Select` and `ComboBox` triggers, an expanded tree
/// node.
pub fn chevron_down(p: &Painter, rect: Rect, color: Color32) {
    chevron(p, rect, color, 0.0);
}

/// A chevron pointing up — an open `Select`.
pub fn chevron_up(p: &Painter, rect: Rect, color: Color32) {
    chevron(p, rect, color, 2.0);
}

/// A chevron pointing right — a collapsed tree node, a breadcrumb separator, a
/// submenu.
pub fn chevron_right(p: &Painter, rect: Rect, color: Color32) {
    chevron(p, rect, color, 3.0);
}

/// A chevron pointing left.
pub fn chevron_left(p: &Painter, rect: Rect, color: Color32) {
    chevron(p, rect, color, 1.0);
}

/// A small solid triangle pointing up — a `Table` sorted ascending.
pub fn sort_asc(p: &Painter, rect: Rect, color: Color32) {
    triangle(p, rect, color, true);
}

/// A small solid triangle pointing down — a `Table` sorted descending.
pub fn sort_desc(p: &Painter, rect: Rect, color: Color32) {
    triangle(p, rect, color, false);
}

/// A small solid triangle pointing right — the submenu marker.
///
/// Azur's own affordance is [`chevron_right`], which is what a web menu uses. A
/// desktop menu before Windows 11 used a filled caret, and at the 12px this is drawn
/// at, a solid shape holds together where a two-stroke chevron goes fuzzy.
pub fn caret_right(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect);
    let (hw, hh) = (r.width() * 0.19, r.height() * 0.3);
    let c = r.center();
    p.add(Shape::convex_polygon(
        vec![
            pos2(c.x - hw, c.y - hh),
            pos2(c.x - hw, c.y + hh),
            pos2(c.x + hw, c.y),
        ],
        color,
        Stroke::NONE,
    ));
}

fn triangle(p: &Painter, rect: Rect, color: Color32, up: bool) {
    // Azur builds these out of CSS borders: 8px wide, 5px tall.
    let r = canvas(rect);
    let (hw, hh) = (r.width() * 0.3, r.height() * 0.19);
    // Two points below the middle of the rect it was handed, which is not the same as two
    // points off-centre. A header cell centres its label and its sort indicator on the same
    // line, and a line of text does not fill its own box: the ascender and descender room that
    // `Modified` and `Type` never reach is counted into the height being centred, so the ink
    // lands lower than a shape that fills its box does. The label is the thing there is a
    // column of, so the shape is what moves.
    //
    // Here rather than in `Table`'s header because it is the same correction wherever a sort
    // indicator is centred beside a label, and an application drawing its own table header —
    // which is the case this was found in — should not have to know about it.
    const DROP: f32 = 2.0;
    let c = r.center() + egui::vec2(0.0, DROP);
    let pts = if up {
        vec![
            pos2(c.x - hw, c.y + hh),
            pos2(c.x + hw, c.y + hh),
            pos2(c.x, c.y - hh),
        ]
    } else {
        vec![
            pos2(c.x - hw, c.y - hh),
            pos2(c.x + hw, c.y - hh),
            pos2(c.x, c.y + hh),
        ]
    };
    p.add(Shape::convex_polygon(pts, color, Stroke::NONE));
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

/// An ✕ — the close button on a dialog, drawer, toast, message bar or field.
pub fn close(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect).shrink(canvas(rect).width() * 0.28);
    let w = stroke_width(canvas(rect)).max(1.2);
    p.line_segment([r.left_top(), r.right_bottom()], Stroke::new(w, color));
    p.line_segment([r.right_top(), r.left_bottom()], Stroke::new(w, color));
}

/// A magnifier — `SearchBox`.
pub fn search(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect);
    let w = stroke_width(r).max(1.2);
    let radius = r.width() * 0.26;
    let center = pos2(r.left() + r.width() * 0.42, r.top() + r.height() * 0.42);
    p.circle_stroke(center, radius, Stroke::new(w, color));
    let start = center + Vec2::splat(radius * 0.72);
    p.line_segment(
        [
            start,
            pos2(r.right() - r.width() * 0.14, r.bottom() - r.height() * 0.14),
        ],
        Stroke::new(w, color),
    );
}

/// A plus — the up half of a `NumberField` stepper.
pub fn plus(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect);
    let w = stroke_width(r).max(1.2);
    let arm = r.width() * 0.28;
    let c = r.center();
    p.line_segment(
        [pos2(c.x - arm, c.y), pos2(c.x + arm, c.y)],
        Stroke::new(w, color),
    );
    p.line_segment(
        [pos2(c.x, c.y - arm), pos2(c.x, c.y + arm)],
        Stroke::new(w, color),
    );
}

/// A minus — the down half of a `NumberField` stepper.
pub fn minus(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect);
    let w = stroke_width(r).max(1.2);
    let arm = r.width() * 0.28;
    let c = r.center();
    p.line_segment(
        [pos2(c.x - arm, c.y), pos2(c.x + arm, c.y)],
        Stroke::new(w, color),
    );
}

/// Three dots in a row — an overflow affordance, and a truncated breadcrumb.
pub fn ellipsis(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect);
    let radius = (r.width() * 0.075).max(1.0);
    for i in -1..=1 {
        p.circle_filled(
            pos2(r.center().x + i as f32 * r.width() * 0.28, r.center().y),
            radius,
            color,
        );
    }
}

/// Three stacked lines — a menu affordance.
pub fn hamburger(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect);
    let w = stroke_width(r).max(1.2);
    for i in -1..=1 {
        let y = r.center().y + i as f32 * r.height() * 0.26;
        p.line_segment(
            [
                pos2(r.left() + r.width() * 0.2, y),
                pos2(r.right() - r.width() * 0.2, y),
            ],
            Stroke::new(w, color),
        );
    }
}

/// Three dots in a column — a row's overflow menu.
pub fn more_vertical(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect);
    let radius = (r.width() * 0.075).max(1.0);
    for i in -1..=1 {
        p.circle_filled(
            pos2(r.center().x, r.center().y + i as f32 * r.height() * 0.28),
            radius,
            color,
        );
    }
}

// ---------------------------------------------------------------------------
// Status marks — `MessageBar` and `Toast` intents
// ---------------------------------------------------------------------------

/// An ⓘ — the `info` intent.
pub fn info(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect);
    let w = stroke_width(r).max(1.2);
    p.circle_stroke(r.center(), r.width() * 0.42, Stroke::new(w, color));
    p.circle_filled(
        pos2(r.center().x, r.top() + r.height() * 0.28),
        w * 0.6,
        color,
    );
    p.line_segment(
        [
            pos2(r.center().x, r.center().y - r.height() * 0.04),
            pos2(r.center().x, r.bottom() - r.height() * 0.24),
        ],
        Stroke::new(w, color),
    );
}

/// A ✓ in a circle — the `success` intent.
pub fn success(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect);
    p.circle_stroke(
        r.center(),
        r.width() * 0.42,
        Stroke::new(stroke_width(r).max(1.2), color),
    );
    check(p, r.shrink(r.width() * 0.26), color);
}

/// A ! in a triangle — the `warning` intent.
pub fn warning(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect);
    let w = stroke_width(r).max(1.2);
    let (top, left, right) = (
        pos2(r.center().x, r.top() + r.height() * 0.12),
        pos2(r.left() + r.width() * 0.06, r.bottom() - r.height() * 0.16),
        pos2(r.right() - r.width() * 0.06, r.bottom() - r.height() * 0.16),
    );
    p.add(Shape::closed_line(
        vec![top, right, left],
        Stroke::new(w, color),
    ));
    p.line_segment(
        [
            pos2(r.center().x, r.top() + r.height() * 0.38),
            pos2(r.center().x, r.top() + r.height() * 0.66),
        ],
        Stroke::new(w, color),
    );
    p.circle_filled(
        pos2(r.center().x, r.bottom() - r.height() * 0.28),
        w * 0.6,
        color,
    );
}

/// An ✕ in a circle — the `danger` intent.
pub fn error(p: &Painter, rect: Rect, color: Color32) {
    let r = canvas(rect);
    p.circle_stroke(
        r.center(),
        r.width() * 0.42,
        Stroke::new(stroke_width(r).max(1.2), color),
    );
    close(p, r.shrink(r.width() * 0.2), color);
}

// ---------------------------------------------------------------------------
// Window buttons — `TitleBar`
// ---------------------------------------------------------------------------

/// The four window buttons are the one group here drawn at a *fixed* weight: the
/// platform draws its caption glyphs as 1px hairlines whether the title bar is
/// 32px or 48px tall, so scaling the stroke with the box — which is what
/// [`line`] does for everything else — would make them read as bold.
///
/// They also fill the box they are handed rather than insetting inside it. Windows
/// sizes these glyphs at about 10px in a 46×32 button, so
/// [`TitleBar`](crate::components::TitleBar) hands them a 10px box and they use
/// all of it; anything less and they come out visibly smaller than the buttons on
/// every other window on the desktop.
fn hairline(rect: Rect) -> Rect {
    // Snapped to whole pixels and offset by half, so a 1px stroke lands on one row
    // of pixels rather than straddling two and going gray.
    let r = canvas(rect);
    Rect::from_min_max(
        pos2(r.left().round() + 0.5, r.top().round() + 0.5),
        pos2(r.right().round() - 0.5, r.bottom().round() - 0.5),
    )
}

/// A single line — minimize.
pub fn window_minimize(p: &Painter, rect: Rect, color: Color32) {
    let r = hairline(rect);
    let y = r.center().y.round() + 0.5;
    p.line_segment(
        [pos2(r.left(), y), pos2(r.right(), y)],
        Stroke::new(1.0, color),
    );
}

/// An empty square — maximize.
pub fn window_maximize(p: &Painter, rect: Rect, color: Color32) {
    let r = hairline(rect);
    p.rect_stroke(
        r,
        CornerRadius::ZERO,
        Stroke::new(1.0, color),
        StrokeKind::Middle,
    );
}

/// Two offset squares — restore down.
///
/// The back square shows only its top and right edges, which is how the platform
/// glyph reads at this size: two full outlines overlapping in 10px is mush.
pub fn window_restore(p: &Painter, rect: Rect, color: Color32) {
    let r = hairline(rect);
    let off = (r.width() * 0.25).round().max(2.0);
    let stroke = Stroke::new(1.0, color);

    let front = Rect::from_min_max(
        pos2(r.left(), r.top() + off),
        pos2(r.right() - off, r.bottom()),
    );
    p.line_segment(
        [pos2(front.left() + off, r.top()), pos2(r.right(), r.top())],
        stroke,
    );
    p.line_segment(
        [
            pos2(r.right(), r.top()),
            pos2(r.right(), front.bottom() - off),
        ],
        stroke,
    );
    p.rect_stroke(front, CornerRadius::ZERO, stroke, StrokeKind::Middle);
}

/// An ✕ at window-button weight — close.
pub fn window_close(p: &Painter, rect: Rect, color: Color32) {
    let r = hairline(rect);
    p.line_segment([r.left_top(), r.right_bottom()], Stroke::new(1.0, color));
    p.line_segment([r.right_top(), r.left_bottom()], Stroke::new(1.0, color));
}

// ---------------------------------------------------------------------------
// The focus ring
// ---------------------------------------------------------------------------

/// Paint Azur's focus ring around `rect`.
///
/// CSS gets this from `outline` + `outline-offset`, which paint outside the
/// border box without taking part in layout. egui has no equivalent, so this
/// strokes a rect grown by `--azur-focus-ring-offset` — which means the ring can
/// spill into a neighbour if the layout is tight. Where Azur itself is tight it
/// uses a negative offset instead; that is [`focus_ring_inset`].
pub fn focus_ring(p: &Painter, rect: Rect, corner_radius: CornerRadius, color: Color32) {
    let grow = focus_ring::OFFSET + focus_ring::WIDTH * 0.5;
    let radius = corner_radius.nw.saturating_add(grow as u8);
    p.rect_stroke(
        rect.expand(grow),
        CornerRadius::same(radius),
        Stroke::new(focus_ring::WIDTH, color),
        StrokeKind::Middle,
    );
}

/// The focus ring drawn *inside* `rect`, for the rows, tabs and table headers
/// where Azur uses `outline-offset: -2px` so the ring cannot overlap a
/// neighbour.
pub fn focus_ring_inset(p: &Painter, rect: Rect, corner_radius: CornerRadius, color: Color32) {
    p.rect_stroke(
        rect.shrink(focus_ring::WIDTH * 0.5),
        corner_radius,
        Stroke::new(focus_ring::WIDTH, color),
        StrokeKind::Middle,
    );
}
