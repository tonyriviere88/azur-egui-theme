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

/// The light theme's own neutral ramp: the accent's hue, at the lightnesses paper
/// needs.
///
/// Eleven rungs, [`SLATE_0`](paper::SLATE_0) the ink and [`SLATE_10`](paper::SLATE_10)
/// the brightest sheet, and the light theme takes **every** neutral out of here —
/// surfaces, text, strokes, scrollbars, hovers. That is the deviation: [`palette`] is
/// `tokens.css` transcribed and this is not in it.
///
/// # Why the light theme cannot use the gray ramp
///
/// Two separate problems, and the second is the one that made this module a ramp
/// rather than three surfaces.
///
/// **No surface is white.** Azur's light surfaces are `WHITE`,
/// [`GRAY_16`](palette::GRAY_16) and [`GRAY_15`](palette::GRAY_15), which is a
/// *document's* ladder: a browser window is one page, mostly text, looked at for as
/// long as it takes to read. An application window is a screen of chrome that stays
/// open all day, and pure white across that much area is what people mean when they
/// call a light theme aggressive.
///
/// **And the gray ramp has no colour left at the end a light theme lives at.** This is
/// the measurement the rework turned on. Chroma in `C*` — CIELAB, the same reason
/// [`crate::contrast`] measures surfaces in `L*` rather than in bytes:
///
/// | | dark theme | light theme, before | here |
/// | --- | --- | --- | --- |
/// | the panel content sits on | `GRAY_2` C\* **2.65** | `#F7F8FA` C\* **1.07** | [`SHEET`](paper::SHEET) C\* **4.89** |
/// | the window behind it | `GRAY_0` C\* 0.49 | `#EBEEF3` C\* **2.76** | [`CANVAS`](paper::CANVAS) C\* **8.17** |
/// | a step off the sheet | `GRAY_4` C\* 4.46 | `#DCE0E8` C\* **4.37** | [`SHADE`](paper::SHADE) C\* **10.44** |
/// | body text | `GRAY_16` C\* 1.08 | `GRAY_1` C\* **0.64** | [`SLATE_0`](paper::SLATE_0) C\* **13.24** |
/// | a hovered row | `GRAY_8` C\* 6.51 | `GRAY_13` C\* **6.27** | [`SLATE_6`](paper::SLATE_6) C\* **13.61** |
///
/// The gray ramp is a *cool* ramp where it is dark — the dark theme is a blue theme,
/// and reads as one — and it gives that up as it climbs, because the top of a neutral
/// scale is where chroma goes to die: `GRAY_16` is C\* 1.08, which is grey. So the
/// light theme was the same design system with the colour drained out of it, and the
/// ink on nearly every pixel of text in the window was a *neutral* near-black.
///
/// # What this ramp is
///
/// **The dark theme's hue, held constant.** Every rung is Lab hue **272**, which is
/// where the dark ramp's own middle sits (`GRAY_3` 273.0, `GRAY_4` 273.5, `GRAY_8`
/// 273.2) and, within a degree, [`AZURE_60`](palette::AZURE_60)'s (272.7). The light
/// theme is now the same blue the dark one already was, and "inspired by the dark
/// theme" is a number rather than a taste.
///
/// **At the old ramp's lightnesses**, deliberately. Contrast is a function of `L*`
/// alone, so keeping the `L*` skeleton is what let the chroma roughly double without a
/// single measured floor moving: no rung is more than 3.5 `L*` from the gray it
/// replaces. The exceptions are stated rather than hidden:
///
/// - **The ink moves, and it moved twice.** The rework took it to `L*` 16.3 (13.3:1) on
///   the reasoning that `GRAY_1`'s `L*` 4.3 — 18.1:1 on the sheet — is the harshest
///   thing a light theme can do. Beside Explorer's own file listing, which sets pure
///   black on pure white at 19.0:1, `L*` 16.3 is not restrained but *washed out*: the
///   ink is the one rung that is read rather than looked at, and it was the one rung
///   given away. [`SLATE_0`] is now `L*` 9.0 at **15.9:1** — a near-black that still
///   carries the ramp's hue, where below about `L*` 6 the chroma stops being
///   perceptible at text sizes and is paid for with nothing.
///
///   It is not taken all the way to black, and that is the ladder's doing rather than
///   taste: `#000000` is `C*` 0, so the ink would be the one rung off the hue every
///   other rung is defined by. What that costs is 3.1:1 of contrast nobody is short of.
/// - **The ladder opens up a little**: sheet to canvas 1.12:1 against the CSS's 1.08:1,
///   sheet to shade 1.27:1 against 1.24:1. Panels read as panels at a glance, which is
///   most of what "modern" means about a window divided by seams.
///
/// [`SLATE_0`]: paper::SLATE_0
pub mod paper {
    use egui::Color32;

    /// Body text — `text.primary`. A near-black that keeps the ramp's hue; see the
    /// [module documentation](self) for why the ink is the one rung that really moved,
    /// and moved twice.
    pub const SLATE_0: Color32 = Color32::from_rgb(0x0d, 0x1a, 0x2b);
    /// `text.secondary` — the metadata columns, and the weak text egui derives.
    pub const SLATE_1: Color32 = Color32::from_rgb(0x4a, 0x58, 0x6e);
    /// `text.tertiary` — a mark, a unit, a figure nobody is reading closely.
    pub const SLATE_2: Color32 = Color32::from_rgb(0x5d, 0x6b, 0x83);
    /// `stroke.strong` — the one line meant to be seen as a line.
    pub const SLATE_3: Color32 = Color32::from_rgb(0x67, 0x75, 0x8e);
    /// A scrollbar thumb under the pointer.
    pub const SLATE_4: Color32 = Color32::from_rgb(0x7c, 0x8b, 0xa5);
    /// `text.disabled` and `stroke.control`, and what a control takes while it is
    /// pressed — [`crate::desktop::press_fill`].
    pub const SLATE_5: Color32 = Color32::from_rgb(0x93, 0xa1, 0xbb);
    /// `stroke.default`, a scrollbar at rest, a disabled accent — and the fill under
    /// the pointer in an application window, [`crate::desktop::hover_fill`].
    pub const SLATE_6: Color32 = Color32::from_rgb(0xab, 0xb8, 0xd0);
    /// `stroke.subtle` — so, the seam between two panes — and `bg.control_active`.
    pub const SLATE_7: Color32 = Color32::from_rgb(0xc1, 0xcd, 0xe2);
    /// One step off a sheet: `bg.layer_alt`, `row_alt`, a disabled control. Named
    /// [`SHADE`].
    pub const SLATE_8: Color32 = Color32::from_rgb(0xd0, 0xda, 0xed);
    /// The window itself. Named [`CANVAS`].
    pub const SLATE_9: Color32 = Color32::from_rgb(0xe0, 0xe8, 0xf7);
    /// The surfaces content sits on. Named [`SHEET`].
    pub const SLATE_10: Color32 = Color32::from_rgb(0xef, 0xf4, 0xfd);

    /// The surfaces content sits *on*: `bg.layer`, `bg.card`, `bg.control`.
    pub const SHEET: Color32 = SLATE_10;
    /// The window itself — `bg.canvas`, behind every sheet.
    pub const CANVAS: Color32 = SLATE_9;
    /// One step off a sheet: `bg.layer_alt`, the hover fills, `row_alt`.
    pub const SHADE: Color32 = SLATE_8;
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
