//! The raw Azur token scales, transcribed from
//! `azur-design-system/src/styles/tokens.css`.
//!
//! These are values, not decisions: a widget should never reach for `GRAY_5`
//! directly, it should ask the [`Theme`](crate::Theme) for the *role* it means
//! (`bg.card`, `stroke.subtle`, `text.secondary`). Roles are what the other
//! theme remaps; a gray step is not.
//!
//! Every `--azur-*` custom property in the source file appears here, under the
//! same name, in the same order — including steps no role currently uses. That
//! is deliberate: this module is the diff surface against `tokens.css`, and a
//! scale with holes in it cannot be checked by reading.
//!
//! # What does not survive the port
//!
//! Three groups of tokens have no egui equivalent and are transcribed for
//! reference rather than use:
//!
//! - **Line heights.** [`egui::FontId`] carries a size and a family, not a
//!   leading. The values in [`typography`] are what a hand-painted row should
//!   use for its height ([`crate::components`] does).
//! - **Easing curves.** egui animates on a fixed ease-out; [`motion`] keeps the
//!   control points and a [`motion::cubic_bezier`] evaluator for anything an app
//!   animates itself.
//! - **Z-index.** egui orders by [`egui::Order`], not by number. [`layer`] maps
//!   each `--azur-z-*` stop onto the order that holds the same rank.

/// Neutrals, accent and status hues.
///
/// Theme-independent: both themes pick roles out of this one scale.
#[allow(dead_code)]
pub mod palette {
    use egui::Color32;

    // ---- Neutrals ----
    pub const GRAY_0: Color32 = Color32::from_rgb(0x07, 0x08, 0x09);
    pub const GRAY_1: Color32 = Color32::from_rgb(0x0e, 0x0f, 0x10);
    pub const GRAY_2: Color32 = Color32::from_rgb(0x14, 0x17, 0x1a);
    pub const GRAY_3: Color32 = Color32::from_rgb(0x19, 0x1b, 0x1f);
    pub const GRAY_4: Color32 = Color32::from_rgb(0x20, 0x23, 0x29);
    pub const GRAY_5: Color32 = Color32::from_rgb(0x28, 0x2b, 0x32);
    pub const GRAY_6: Color32 = Color32::from_rgb(0x33, 0x37, 0x3f);
    pub const GRAY_7: Color32 = Color32::from_rgb(0x3f, 0x43, 0x4c);
    pub const GRAY_8: Color32 = Color32::from_rgb(0x4d, 0x52, 0x5c);
    pub const GRAY_9: Color32 = Color32::from_rgb(0x5c, 0x62, 0x6d);
    pub const GRAY_10: Color32 = Color32::from_rgb(0x6d, 0x73, 0x80);
    pub const GRAY_11: Color32 = Color32::from_rgb(0x83, 0x8a, 0x97);
    pub const GRAY_12: Color32 = Color32::from_rgb(0x9a, 0xa1, 0xad);
    pub const GRAY_13: Color32 = Color32::from_rgb(0xb3, 0xb9, 0xc4);
    pub const GRAY_14: Color32 = Color32::from_rgb(0xcc, 0xd1, 0xd9);
    pub const GRAY_15: Color32 = Color32::from_rgb(0xe4, 0xe7, 0xec);
    pub const GRAY_16: Color32 = Color32::from_rgb(0xf5, 0xf6, 0xf8);
    pub const WHITE: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
    pub const BLACK: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);

    // ---- Azure accent ----
    pub const AZURE_10: Color32 = Color32::from_rgb(0x07, 0x1a, 0x2c);
    pub const AZURE_20: Color32 = Color32::from_rgb(0x0d, 0x2a, 0x47);
    pub const AZURE_30: Color32 = Color32::from_rgb(0x13, 0x3c, 0x63);
    pub const AZURE_40: Color32 = Color32::from_rgb(0x18, 0x4e, 0x80);
    pub const AZURE_50: Color32 = Color32::from_rgb(0x1d, 0x60, 0x9d);
    pub const AZURE_60: Color32 = Color32::from_rgb(0x26, 0x78, 0xc2);
    pub const AZURE_70: Color32 = Color32::from_rgb(0x3a, 0xa0, 0xff);
    pub const AZURE_80: Color32 = Color32::from_rgb(0x5c, 0xb0, 0xff);
    pub const AZURE_90: Color32 = Color32::from_rgb(0x8e, 0xca, 0xff);
    pub const AZURE_100: Color32 = Color32::from_rgb(0xc2, 0xe2, 0xff);
    pub const AZURE_ACCENT: Color32 = Color32::from_rgb(0x3a, 0xa0, 0xff);
    pub const AZURE_ACCENT_HOVER: Color32 = Color32::from_rgb(0x5c, 0xb0, 0xff);
    pub const AZURE_ACCENT_ACTIVE: Color32 = Color32::from_rgb(0x24, 0x88, 0xe6);

    // ---- Status ----
    pub const GREEN_60: Color32 = Color32::from_rgb(0x29, 0xa7, 0x44);
    pub const GREEN_70: Color32 = Color32::from_rgb(0x3f, 0xc8, 0x5f);
    pub const AMBER_60: Color32 = Color32::from_rgb(0xc9, 0x8a, 0x12);
    pub const AMBER_70: Color32 = Color32::from_rgb(0xe0, 0xa5, 0x30);
    pub const RED_60: Color32 = Color32::from_rgb(0xd6, 0x3a, 0x41);
    pub const RED_70: Color32 = Color32::from_rgb(0xea, 0x4d, 0x54);
}

/// The light theme's surface ladder, a third of a rung off the token file.
///
/// Azur's light surfaces are `WHITE`, [`GRAY_16`](palette::GRAY_16) and
/// [`GRAY_15`](palette::GRAY_15), which is a *document's* ladder: a browser window
/// is one page, mostly text, looked at for as long as it takes to read. An
/// application window is a screen of chrome that stays open all day, and pure
/// white across that much area is what people mean when they call a light theme
/// aggressive.
///
/// So the whole ladder moves down together, and it moves *as* a ladder: what makes
/// a card read as raised and a menu as set into its panel is the distance between
/// the rungs, not where the top one is. Those distances are the CSS's, within a
/// hundredth — sheet to canvas 1.09:1 against 1.08:1, sheet to shade 1.25:1
/// against 1.24:1 — so only the whiteness is gone.
///
/// | | here | `tokens.css` |
/// | --- | --- | --- |
/// | [`SHEET`](paper::SHEET) — panels, cards, controls | `#F7F8FA` | `WHITE` |
/// | [`CANVAS`](paper::CANVAS) — the window behind them | `#EBEEF3` | `GRAY_16` `#F5F6F8` |
/// | [`SHADE`](paper::SHADE) — a step off a sheet: menus, hovers, stripes | `#DCE0E8` | `GRAY_15` `#E4E7EC` |
///
/// The gray scale has no steps this fine at the top — `GRAY_15` → `GRAY_16` →
/// `WHITE` is 15, then 9 — so these are interpolated on it, keeping its cool drift
/// (blue > green > red).
///
/// Nothing else moves: text, strokes, accent and status are the CSS's values, and
/// [`Theme::light`](crate::Theme::light) says how to put the white back.
pub mod paper {
    use egui::Color32;

    /// The surfaces content sits *on*: `bg.layer`, `bg.card`, `bg.control`.
    pub const SHEET: Color32 = Color32::from_rgb(0xf7, 0xf8, 0xfa);
    /// The window itself — `bg.canvas`, behind every sheet.
    pub const CANVAS: Color32 = Color32::from_rgb(0xeb, 0xee, 0xf3);
    /// One step off a sheet: `bg.layer_alt`, the hover fills, `row_alt`.
    pub const SHADE: Color32 = Color32::from_rgb(0xdc, 0xe0, 0xe8);
}

/// The type ramp (`--azur-font-size-*` / `--azur-line-height-*`).
///
/// Sizes go into [`egui::FontId`]; the line heights are for anything that
/// allocates its own row, since egui has nowhere to put a leading.
#[allow(dead_code)]
pub mod typography {
    pub const SIZE_CAPTION: f32 = 12.0;
    pub const LINE_CAPTION: f32 = 16.0;
    pub const SIZE_BODY: f32 = 14.0;
    pub const LINE_BODY: f32 = 20.0;
    pub const SIZE_SUBTITLE: f32 = 16.0;
    pub const LINE_SUBTITLE: f32 = 22.0;
    pub const SIZE_TITLE_SMALL: f32 = 18.0;
    pub const LINE_TITLE_SMALL: f32 = 24.0;
    pub const SIZE_TITLE: f32 = 20.0;
    pub const LINE_TITLE: f32 = 26.0;
    pub const SIZE_TITLE_LARGE: f32 = 24.0;
    pub const LINE_TITLE_LARGE: f32 = 32.0;
    pub const SIZE_DISPLAY: f32 = 32.0;
    pub const LINE_DISPLAY: f32 = 40.0;

    /// `--azur-font-weight-*`, for the record. egui selects a weight by picking a
    /// font family, not by asking for a number, so [`crate::fonts`] registers the
    /// two faces a desktop UI family actually has: regular (400) and semibold
    /// (600). Medium (500) rounds down to regular — a button label is meant to read
    /// as firm, not bold — and bold (700) is not used by any role here.
    pub const WEIGHT_REGULAR: u16 = 400;
    pub const WEIGHT_MEDIUM: u16 = 500;
    pub const WEIGHT_SEMIBOLD: u16 = 600;
    pub const WEIGHT_BOLD: u16 = 700;
}

/// The 4px spacing scale (`--azur-space-*`).
///
/// Named `S1`..`S10` after the token index rather than the pixel count, so a
/// call site can be checked against the CSS by eye.
#[allow(dead_code)]
pub mod space {
    pub const S1: f32 = 2.0;
    pub const S2: f32 = 4.0;
    pub const S3: f32 = 8.0;
    pub const S4: f32 = 12.0;
    pub const S5: f32 = 16.0;
    pub const S6: f32 = 20.0;
    pub const S7: f32 = 24.0;
    pub const S8: f32 = 32.0;
    pub const S9: f32 = 40.0;
    pub const S10: f32 = 48.0;
}

/// Corner radii, in px (`--azur-radius-*`).
///
/// `u8` because that is what [`egui::CornerRadius`] holds. `CIRCULAR` stands in
/// for the CSS `999px`: egui clamps a corner radius to half the shorter side, so
/// the largest value it can hold is already a pill.
#[allow(dead_code)]
pub mod radius {
    pub const SMALL: u8 = 4;
    pub const MEDIUM: u8 = 6;
    pub const LARGE: u8 = 8;
    pub const XLARGE: u8 = 12;
    pub const CIRCULAR: u8 = u8::MAX;
}

/// Elevation (`--azur-shadow-*`), as [`egui::Shadow`].
///
/// A CSS `box-shadow: 0 Ypx Bpx rgba(0,0,0,A)` becomes `offset: [0, Y]`,
/// `blur: B`, `color: black * A`. Azur's shadows are theme-independent — the
/// same six values in light and dark — so these are not remapped per theme.
#[allow(dead_code)]
pub mod shadow {
    use egui::{Color32, Shadow};

    const fn drop(y: i8, blur: u8, alpha: u8) -> Shadow {
        Shadow {
            offset: [0, y],
            blur,
            spread: 0,
            color: Color32::from_black_alpha(alpha),
        }
    }

    /// `0 1px 2px rgba(0,0,0,.36)` — a resting edge.
    pub const S2: Shadow = drop(1, 2, 92);
    /// `0 2px 4px rgba(0,0,0,.40)` — a slider handle.
    pub const S4: Shadow = drop(2, 4, 102);
    /// `0 4px 8px rgba(0,0,0,.44)` — an elevated card.
    pub const S8: Shadow = drop(4, 8, 112);
    /// `0 8px 16px rgba(0,0,0,.48)` — popovers, menus, tooltips, toasts.
    pub const S16: Shadow = drop(8, 16, 122);
    /// `0 14px 28px rgba(0,0,0,.52)` — a drawer.
    pub const S28: Shadow = drop(14, 28, 133);
    /// `0 32px 64px rgba(0,0,0,.56)` — a modal dialog.
    pub const S64: Shadow = drop(32, 64, 143);
}

/// Motion (`--azur-duration-*`, `--azur-easing-*`).
///
/// Durations are seconds, which is what [`egui::Style::animation_time`] and
/// [`egui::Context::animate_bool_with_time`] take. The easing curves are the CSS
/// control points; [`motion::cubic_bezier`] evaluates them for an app animating by
/// hand.
#[allow(dead_code)]
pub mod motion {
    pub const ULTRA_FAST: f32 = 0.050;
    pub const FASTER: f32 = 0.100;
    /// The duration nearly every Azur component uses for a state change.
    pub const FAST: f32 = 0.150;
    pub const NORMAL: f32 = 0.200;
    pub const SLOW: f32 = 0.300;
    pub const SLOWER: f32 = 0.400;

    /// `cubic-bezier` control points, as `[x1, y1, x2, y2]`.
    pub type Easing = [f32; 4];

    pub const ACCELERATE: Easing = [0.9, 0.1, 1.0, 0.2];
    pub const DECELERATE: Easing = [0.1, 0.9, 0.2, 1.0];
    pub const STANDARD: Easing = [0.8, 0.0, 0.2, 1.0];
    pub const MAX: Easing = [0.7, 0.0, 1.0, 0.5];

    /// Evaluate a CSS-style easing at `t` in `0..=1`.
    ///
    /// The curve is `y(x)` with implicit endpoints at `(0,0)` and `(1,1)`, so `t`
    /// has to be solved for before `y` can be read. Newton's method converges in
    /// a handful of steps on curves this shape; the bisection fallback is there
    /// for `ACCELERATE`, whose near-vertical start can leave the derivative at
    /// zero.
    pub fn cubic_bezier(easing: Easing, t: f32) -> f32 {
        let [x1, y1, x2, y2] = easing;
        let t = t.clamp(0.0, 1.0);

        // B(u) for one axis, with p0 = 0 and p3 = 1.
        let axis = |p1: f32, p2: f32, u: f32| {
            let v = 1.0 - u;
            3.0 * v * v * u * p1 + 3.0 * v * u * u * p2 + u * u * u
        };
        let dx = |u: f32| {
            let v = 1.0 - u;
            3.0 * v * v * x1 + 6.0 * v * u * (x2 - x1) + 3.0 * u * u * (1.0 - x2)
        };

        let mut u = t;
        for _ in 0..8 {
            let err = axis(x1, x2, u) - t;
            if err.abs() < 1e-5 {
                return axis(y1, y2, u);
            }
            let slope = dx(u);
            if slope.abs() < 1e-6 {
                break;
            }
            u -= err / slope;
        }

        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        for _ in 0..24 {
            u = 0.5 * (lo + hi);
            if axis(x1, x2, u) < t {
                lo = u;
            } else {
                hi = u;
            }
        }
        axis(y1, y2, u)
    }
}

/// Stacking order (`--azur-z-*`), as [`egui::Order`].
///
/// egui has six orders where Azur has seven stops, and they are ranks rather
/// than numbers: anything in a higher order paints over everything below it, so
/// two stops that share an order are ordered between themselves by which was
/// added last. Dropdowns, popovers and menus all land on `Foreground`, which is
/// the order egui's own popups use.
#[allow(dead_code)]
pub mod layer {
    use egui::Order;

    pub const DROPDOWN: Order = Order::Foreground;
    pub const STICKY: Order = Order::Middle;
    /// A modal scrim. egui paints it in the same layer as the panel it dims,
    /// just below it, so it shares the modal's order.
    pub const OVERLAY: Order = Order::Foreground;
    pub const MODAL: Order = Order::Foreground;
    pub const POPOVER: Order = Order::Foreground;
    pub const TOOLTIP: Order = Order::Tooltip;
    pub const TOAST: Order = Order::Tooltip;
}

/// Control heights, in px, and the icon size that goes with each.
///
/// Azur sizes a control by fixing its height (`Button`, `TextField`, `Select`,
/// `NumberField` and `SearchBox` all use the same three), so this is one scale
/// rather than one per component.
#[allow(dead_code)]
pub mod control {
    /// 24px — `Button.small`, dense toolbars.
    pub const SMALL: f32 = 24.0;
    /// 32px — Azur's default.
    pub const MEDIUM: f32 = 32.0;
    /// 40px — a primary call to action.
    pub const LARGE: f32 = 40.0;

    /// Icon box for each height, from the component CSS.
    pub const ICON_SMALL: f32 = 14.0;
    pub const ICON_MEDIUM: f32 = 16.0;
    pub const ICON_LARGE: f32 = 20.0;

    /// Checkbox / radio box, per size.
    pub const BOX_SMALL: f32 = 14.0;
    pub const BOX_MEDIUM: f32 = 16.0;
    pub const BOX_LARGE: f32 = 20.0;

    /// Switch track, per size: `(width, height)`.
    pub const SWITCH_SMALL: (f32, f32) = (32.0, 18.0);
    pub const SWITCH_MEDIUM: (f32, f32) = (40.0, 22.0);
    pub const SWITCH_LARGE: (f32, f32) = (48.0, 26.0);
}

/// Row heights for the collection components, in px.
#[allow(dead_code)]
pub mod row {
    /// `ListView` / `TreeView` at `density="compact"`.
    pub const COMPACT: f32 = 28.0;
    /// `ListView` at `density="comfortable"`.
    pub const COMFORTABLE: f32 = 40.0;
    /// A `Table` cell: `space-3` above and below a body line.
    pub const TABLE: f32 = 36.0;
    /// A `NavigationRail` entry.
    pub const NAV: f32 = 36.0;
    /// The `TitleBar`, and its window buttons.
    pub const TITLE_BAR: f32 = 32.0;

    /// A row in a listing scanned by the hundred: 24px, `space-1` either side of a
    /// body line.
    ///
    /// Below the CSS's smallest, and the two below have no token behind them at all —
    /// Azur's densest row is [`COMPACT`] and its table row is [`TABLE`], which are
    /// sized for a form. A file listing, a process table or a log is read by scanning,
    /// and every point of row height is a line that did not fit on the screen. See
    /// [`crate::desktop`], which is where the rest of that argument lives.
    pub const DENSE: f32 = 24.0;
    /// One body line and a point of air: 22px, for a sidebar entry or a status line —
    /// a row that is *listed* rather than read across.
    pub const TIGHT: f32 = 22.0;
}

/// The focus ring (`--azur-focus-ring-*`).
///
/// CSS gets this from `outline`, which paints outside the border box without
/// affecting layout. egui has no such thing, so [`crate::components`] paints the
/// ring itself, expanding the widget's rect by `OFFSET` and stroking it.
pub mod focus_ring {
    pub const WIDTH: f32 = 2.0;
    pub const OFFSET: f32 = 2.0;
}

/// A categorical series palette for charts, graphs and sparklines.
///
/// Azur's token file has no chart scale. These are the accent and the two status
/// hues — so that a red line and a red number mean the same thing — plus a
/// violet and a magenta picked to hold their own beside the azure at roughly the
/// same lightness. They are ordered for use as a cycle: adjacent entries are the
/// furthest apart in hue.
#[allow(dead_code)]
pub mod chart {
    use super::palette;
    use egui::Color32;

    pub const AZURE: Color32 = palette::AZURE_ACCENT;
    pub const VIOLET: Color32 = Color32::from_rgb(0xa2, 0x7b, 0xf0);
    pub const GREEN: Color32 = palette::GREEN_70;
    pub const AMBER: Color32 = palette::AMBER_70;
    pub const MAGENTA: Color32 = Color32::from_rgb(0xf0, 0x6c, 0xb0);

    /// The five above, in cycle order.
    pub const SERIES: [Color32; 5] = [AZURE, VIOLET, GREEN, AMBER, MAGENTA];
}
