//! The semantic layer: what a surface, a line or a word *is for*, rather than
//! which gray it happens to be.
//!
//! Every field here is one `--azur-color-*` custom property, grouped by the same
//! namespaces the token file uses, so `--azur-color-background-card-hover` is
//! `t.bg.card_hover` and `--azur-color-stroke-subtle` is `t.stroke.subtle`. That
//! one-to-one shape is the point: it is what makes [`Theme::dark`] and
//! [`Theme::light`] checkable against `tokens.css` by reading them side by side.

use crate::tokens::{palette, paper, typography};
use egui::{Color32, FontFamily, FontId};

/// Named font roles: Azur's type ramp, one [`FontId`] per `Text` variant.
///
/// `FontId` is cheap to clone, but building one per cell per frame still means
/// re-hashing it, so they are kept here rather than constructed at the call site.
///
/// Azur asks for four weights and egui selects a weight by choosing a font
/// family, so the ramp resolves onto the two families [`crate::fonts`] registers:
/// regular, and the semibold face behind [`crate::fonts::SEMIBOLD`].
///
/// `font-weight-medium` (500) has no face of its own — Segoe UI goes 400, then 600 —
/// and rounds **down** to regular rather than up to semibold. That covers a button
/// label, a tab, a field label and a badge: the roles Azur wants slightly firm, not
/// bold. See [`crate::style::medium`](mod@crate::style) for why up was wrong.
///
/// If you install fonts yourself, register that family name too (egui panics on
/// an unknown [`FontFamily::Name`]).
#[derive(Clone, Debug)]
pub struct Fonts {
    /// 12/16 regular — captions, secondary row text, table units.
    pub caption: FontId,
    /// 14/20 regular — body copy, table cells, control labels. The default.
    pub body: FontId,
    /// 14/20 semibold — `body-strong`: toast titles, table headers, tab labels.
    pub body_strong: FontId,
    /// 16/22 regular — `subtitle`.
    pub subtitle: FontId,
    /// 18/24 semibold — `title-small`: dialog and drawer titles.
    pub title_small: FontId,
    /// 20/26 semibold — `title`: section headings.
    pub title: FontId,
    /// 24/32 semibold — `title-large`.
    pub title_large: FontId,
    /// 32/40 semibold — `display`.
    pub display: FontId,
    /// Paths, ids, anything where alignment carries meaning. Azur names a
    /// monospace family but no size for it; this follows body.
    pub mono: FontId,
}

impl Default for Fonts {
    fn default() -> Self {
        let strong = || FontFamily::Name(crate::fonts::SEMIBOLD.into());
        Self {
            caption: FontId::new(typography::SIZE_CAPTION, FontFamily::Proportional),
            body: FontId::new(typography::SIZE_BODY, FontFamily::Proportional),
            body_strong: FontId::new(typography::SIZE_BODY, strong()),
            subtitle: FontId::new(typography::SIZE_SUBTITLE, FontFamily::Proportional),
            title_small: FontId::new(typography::SIZE_TITLE_SMALL, strong()),
            title: FontId::new(typography::SIZE_TITLE, strong()),
            title_large: FontId::new(typography::SIZE_TITLE_LARGE, strong()),
            display: FontId::new(typography::SIZE_DISPLAY, strong()),
            mono: FontId::new(typography::SIZE_BODY, FontFamily::Monospace),
        }
    }
}

/// `--azur-color-background-*`: the surface ladder.
#[derive(Clone, Copy, Debug)]
pub struct Backgrounds {
    /// The window itself. The furthest back anything goes.
    pub canvas: Color32,
    /// Chrome and structure: title bar, navigation rail, dialog and drawer
    /// panels, the body of a list or table.
    pub layer: Color32,
    /// One step in from `layer`: popovers, menus, tooltips, a table's header
    /// strip, a neutral badge.
    pub layer_alt: Color32,
    pub card: Color32,
    /// A card or a list row under the pointer.
    pub card_hover: Color32,
    /// The scrim behind a modal. Translucent — it is painted *over* the page.
    pub overlay: Color32,
    /// The fill of a control at rest: button, text field, checkbox, switch track.
    pub control: Color32,
    pub control_hover: Color32,
    /// A control being pressed.
    pub control_active: Color32,
    pub control_disabled: Color32,
}

/// `--azur-color-accent-*`.
///
/// `default` / `hover` / `active` are **surfaces** — a primary button, a checked
/// checkbox, a slider's trailing fill — and carry `text.on_accent` on top.
/// [`mark`](Self::mark) is the same accent used as **ink**. The two cannot be one
/// value in a dark theme: a fill dark enough for white text to read on it is too
/// dark to be read *as* text itself. See [`Theme::dark`].
#[derive(Clone, Copy, Debug)]
pub struct Accents {
    pub default: Color32,
    pub hover: Color32,
    pub active: Color32,
    /// A disabled accent surface — a checked-but-disabled checkbox or switch.
    /// Neutral, not a faded blue: the point is that it stops reading as live.
    pub disabled: Color32,
    /// The accent as ink rather than as a surface: a pressed button's label, a
    /// selected navigation entry's icon, a `Select` tick, a 2px selection bar, a
    /// spinner, accent-colored text.
    ///
    /// Azur's CSS spells all of these `accent-default`, because on the web the
    /// same blue happens to work both ways. It does not here — see
    /// [`Theme::dark`] — so this is the rung that stays legible *on* a surface,
    /// and it equals `default` in the light theme, where one value does serve
    /// both.
    pub mark: Color32,
    /// An accent-tinted fill quiet enough to put text on: a selected row, an
    /// active navigation entry, an accent badge.
    pub subtle: Color32,
    pub subtle_hover: Color32,
    pub subtle_active: Color32,
}

/// `--azur-color-text-*`.
#[derive(Clone, Copy, Debug)]
pub struct TextColors {
    /// Body copy and anything the eye should land on first.
    pub primary: Color32,
    /// Labels, captions, column headers, inactive tabs.
    pub secondary: Color32,
    /// Metadata and placeholder text.
    pub tertiary: Color32,
    pub disabled: Color32,
    /// Text and icons drawn on top of `accent.default`.
    pub on_accent: Color32,
    /// A hyperlink. Deliberately not `accent.default`: a link has to clear body
    /// text on the canvas, which the accent proper does not in either theme.
    pub link: Color32,
}

/// `--azur-color-stroke-*`.
#[derive(Clone, Copy, Debug)]
pub struct Strokes {
    /// Panel edges, table rules, the outline of a secondary button.
    pub default: Color32,
    /// Separators inside a surface, and the border of a popover — where a full
    /// stroke would read as a second frame. Also Azur's slider rail and, by
    /// extension, the gridlines of a plot.
    pub subtle: Color32,
    /// The outline of a control under the pointer.
    pub strong: Color32,
    /// The outline of a control at rest: text field, checkbox, switch.
    pub control: Color32,
    /// The focus ring, and the border of a focused field.
    pub focus: Color32,
}

/// `--azur-color-success|warning|danger|info*`.
///
/// The `*_subtle` fills are the translucent versions Azur puts behind a message
/// bar, so they compose over whatever surface they land on.
#[derive(Clone, Copy, Debug)]
pub struct Status {
    pub success: Color32,
    pub success_subtle: Color32,
    pub warning: Color32,
    pub warning_subtle: Color32,
    /// Danger as ink: an error message under a field, a `danger` menu entry, the
    /// ✕ on a danger message bar, a busy status dot.
    pub danger: Color32,
    /// Danger as a surface, with `text.on_accent` on top: a destructive button,
    /// and the close button's hover in the title bar.
    ///
    /// Split from `danger` for the reason [`Accents::mark`] is split from
    /// `Accents::default`, and equal to it in the light theme.
    pub danger_solid: Color32,
    /// The hover state of `danger_solid`.
    pub danger_hover: Color32,
    pub danger_subtle: Color32,
    pub info: Color32,
    pub info_subtle: Color32,
}

/// `--azur-color-scrollbar-*`.
///
/// Azur's track is transparent in both themes; only the thumb has a color.
#[derive(Clone, Copy, Debug)]
pub struct Scrollbar {
    pub thumb: Color32,
    pub thumb_hover: Color32,
}

/// Every color the theme decides, by role.
///
/// Fields are public and the struct is `Clone`, so an application can take
/// [`Theme::dark`] and move a single role without forking the file:
///
/// ```
/// use azur_egui_theme::{tokens::palette, Theme};
/// let mut theme = Theme::dark();
/// theme.bg.canvas = palette::GRAY_2; // a less black window
/// theme.bg.layer = palette::GRAY_3;
/// ```
///
/// The whole of [`mod@crate::style`] and [`crate::components`] is derived from
/// these, so an override here reaches egui's own widgets, Azur's components and
/// whatever you paint by hand, together.
#[derive(Clone, Debug)]
pub struct Theme {
    /// Whether this is the dark side of the token set. Drives
    /// [`egui::Visuals::dark_mode`].
    pub dark: bool,

    pub bg: Backgrounds,
    pub accent: Accents,
    pub text: TextColors,
    pub stroke: Strokes,
    pub status: Status,
    pub scrollbar: Scrollbar,

    /// The alternating row of a zebra-striped list.
    ///
    /// The one role with no token behind it: Azur's `Table` separates rows with
    /// a rule rather than a stripe, and has no component that stripes. Desktop
    /// tables wide enough to need help tracking across do, so this is here —
    /// one step off `bg.layer`, because stripes that announce themselves are
    /// harder to read across, not easier.
    pub row_alt: Color32,

    pub fonts: Fonts,
}

impl Theme {
    /// The dark theme — Azur's primary. Transcribed from the `:root` /
    /// `[data-theme='dark']` block of `tokens.css`.
    ///
    /// `bg.canvas` is `GRAY_0`, which is very nearly black. That is the token,
    /// and this is what compliance means; an app that would rather not leave a
    /// near-black window open all day should lift `canvas` and `layer` a step or
    /// two, which no other role depends on.
    ///
    /// # The one deviation: filled accent and danger
    ///
    /// The CSS fills a primary button with `azure-accent` (`#3AA0FF`) and a
    /// destructive one with `red-70` (`#EA4D54`), both carrying white text. On a
    /// near-black canvas those are light sources — and white on them measures
    /// **2.7:1** and **3.7:1**, under the 4.5:1 a label needs. A web page is
    /// mostly light surfaces, so the same swatches behave differently there.
    ///
    /// So the *fills* move two rungs down their ramps, and the same hues stay
    /// bright where they are **ink** rather than a surface: [`Accents::mark`] and
    /// [`Status::danger`]. `stroke.focus`, `status.info` and `text.link` stay
    /// bright for that reason too — a ring, a glyph and a link are all read *on* a
    /// dark surface, never through one.
    ///
    /// | | primary | danger | white on them |
    /// | --- | --- | --- | --- |
    /// | CSS | `#3AA0FF` | `#EA4D54` | 2.7:1 / 3.7:1 |
    /// | here, at rest | `#1D609D` | `#C22932` | 6.5:1 / 5.7:1 |
    /// | hovered | `#2678C2` | `#D63A41` | 4.6:1 / 4.6:1 |
    /// | pressed | `#184E80` | `#B2252E` | 8.6:1 / 6.6:1 |
    ///
    /// Every state clears AA for its label, where the CSS cleared none of them.
    ///
    /// What it costs: a resting fill this dark is 3.1:1 against the canvas but only
    /// **2.6:1** against a card, under the 3:1 WCAG asks of a control's boundary.
    /// That figure is about controls identified *by* their boundary; these are
    /// identified by a label at 6.5:1, and a saturated blue against a neutral gray
    /// separates by hue in a way a luminance ratio does not measure. If it ever
    /// does read as flat, the fix is a 1px `accent.mark` border on the button
    /// rather than a lighter fill.
    ///
    /// To put the CSS values back, four lines after construction:
    ///
    /// ```
    /// use azur_egui_theme::{tokens::palette, Theme};
    /// let mut t = Theme::dark();
    /// t.accent.default = palette::AZURE_ACCENT;
    /// t.accent.hover = palette::AZURE_ACCENT_HOVER;
    /// t.accent.active = palette::AZURE_ACCENT_ACTIVE;
    /// t.status.danger_solid = palette::RED_70;
    /// ```
    ///
    /// Or one rung back up — `AZURE_60` and `RED_60`, the fills the light theme
    /// uses — for 4.6:1 labels and 3.7:1 against every surface.
    pub fn dark() -> Self {
        use palette::*;
        Self {
            dark: true,

            bg: Backgrounds {
                canvas: GRAY_0,
                layer: GRAY_2,
                layer_alt: GRAY_3,
                card: GRAY_3,
                card_hover: GRAY_4,
                overlay: Color32::from_rgba_unmultiplied(5, 6, 7, 184), // rgba(5,6,7,.72)
                control: GRAY_4,
                control_hover: GRAY_5,
                control_active: GRAY_6,
                control_disabled: GRAY_2,
            },

            accent: Accents {
                // The fill ramp, two rungs down from the CSS: `AZURE_60` is where
                // white text starts to pass, and this is one further, for a window
                // that is open all day. Direction is unchanged — hover lifts off
                // the surface, pressing sinks into it — and every state now clears
                // AA for its label rather than only the resting one.
                default: AZURE_50,
                hover: AZURE_60,
                active: AZURE_40,
                disabled: GRAY_6,
                // Ink. This is the CSS's `accent-default`, kept where it works.
                mark: AZURE_ACCENT,
                subtle: AZURE_10,
                subtle_hover: AZURE_20,
                subtle_active: AZURE_30,
            },

            text: TextColors {
                primary: GRAY_16,
                secondary: GRAY_12,
                tertiary: GRAY_10,
                disabled: GRAY_8,
                on_accent: WHITE,
                link: AZURE_90,
            },

            stroke: Strokes {
                default: GRAY_6,
                subtle: GRAY_4,
                strong: GRAY_9,
                control: GRAY_7,
                focus: AZURE_ACCENT,
            },

            status: Status {
                success: GREEN_70,
                success_subtle: Color32::from_rgba_unmultiplied(63, 200, 95, 36), // .14
                warning: AMBER_70,
                warning_subtle: Color32::from_rgba_unmultiplied(224, 165, 48, 36),
                danger: RED_70,
                // The red ramp stops at `RED_60`, so the rung below it is the one
                // `tokens.css` writes out longhand for the light theme's
                // `danger-hover` — a design-system value, just not one the palette
                // block names. Kept in step with the accent, which is two rungs
                // down from its own CSS fill.
                danger_solid: Color32::from_rgb(0xc2, 0x29, 0x32),
                // `RED_60` is the one-rung lift from there, and the CSS's own dark
                // fill was `RED_70` above that — which is now out of reach of the
                // button entirely, and reads much better as ink.
                danger_hover: RED_60,
                danger_subtle: Color32::from_rgba_unmultiplied(234, 77, 84, 36),
                info: AZURE_ACCENT,
                info_subtle: AZURE_10,
            },

            scrollbar: Scrollbar {
                thumb: GRAY_7,
                thumb_hover: GRAY_9,
            },

            row_alt: GRAY_3,
            fonts: Fonts::default(),
        }
    }

    /// The light theme. Transcribed from the `[data-theme='light']` block.
    ///
    /// Note which way the accent moves: `hover` and `active` step *down* the
    /// azure ramp here (60 → 50 → 40) where dark steps up, because on a light
    /// page a control that brightens when you press it reads as releasing.
    ///
    /// # The one deviation: no surface is white
    ///
    /// The surface ladder is [`tokens::paper`](mod@crate::tokens::paper) rather
    /// than `WHITE` / `GRAY_16` / `GRAY_15` — the same three rungs, the same
    /// spacing, all of it about a third of a rung lower, so a window that is open
    /// all day is not a white one. That module says why; the short version is that
    /// `WHITE` is a document's brightest surface, and an application window is
    /// mostly chrome.
    ///
    /// What it costs is measured, not assumed: against the softer canvas a primary
    /// button's edge is 3.97:1 where it was 4.27:1, still over the 3:1 WCAG asks of
    /// a boundary; secondary text is 5.9:1 at worst against the 3:1 it needs, and
    /// body text 14.5:1 against 4.5:1. `theme.rs`'s tests hold each of those floors
    /// on every surface.
    ///
    /// To put the CSS's white back:
    ///
    /// ```
    /// use azur_egui_theme::{tokens::palette, Theme};
    /// let mut t = Theme::light();
    /// t.bg.canvas = palette::GRAY_16;
    /// t.bg.layer = palette::WHITE;
    /// t.bg.card = palette::WHITE;
    /// t.bg.control = palette::WHITE;
    /// t.bg.layer_alt = palette::GRAY_15;
    /// t.bg.card_hover = palette::GRAY_15;
    /// t.bg.control_hover = palette::GRAY_15;
    /// t.bg.control_disabled = palette::GRAY_15;
    /// t.row_alt = palette::GRAY_15;
    /// ```
    pub fn light() -> Self {
        use palette::*;
        Self {
            dark: false,

            bg: Backgrounds {
                canvas: paper::CANVAS,
                layer: paper::SHEET,
                layer_alt: paper::SHADE,
                card: paper::SHEET,
                card_hover: paper::SHADE,
                overlay: Color32::from_rgba_unmultiplied(18, 19, 22, 102), // rgba(18,19,22,.4)
                control: paper::SHEET,
                control_hover: paper::SHADE,
                control_active: GRAY_14,
                // The CSS gives a disabled control the same gray as a hovered one
                // — it is `text.disabled` and the missing pointer that say it is
                // dead, not the fill — so it moves with the hover rather than
                // staying behind on `GRAY_15`.
                control_disabled: paper::SHADE,
            },

            accent: Accents {
                default: AZURE_60,
                hover: AZURE_50,
                active: AZURE_40,
                disabled: GRAY_13,
                // One value, both jobs, exactly as the CSS has it: on a white page
                // `AZURE_60` is dark enough to hold white text (4.6:1) and dark
                // enough to be text. It is not a *strong* ink on `AZURE_100`
                // (3.4:1, the CSS's own pairing) — but that is the light theme's
                // to answer, and no darker rung would fix it without taking the
                // fill with it.
                mark: AZURE_60,
                subtle: AZURE_100,
                subtle_hover: AZURE_90,
                subtle_active: AZURE_80,
            },

            text: TextColors {
                primary: GRAY_1,
                secondary: GRAY_8,
                tertiary: GRAY_9,
                disabled: GRAY_12,
                on_accent: WHITE,
                link: AZURE_60,
            },

            stroke: Strokes {
                default: GRAY_13,
                subtle: GRAY_14,
                strong: GRAY_10,
                control: GRAY_12,
                focus: AZURE_60,
            },

            status: Status {
                success: GREEN_60,
                success_subtle: Color32::from_rgba_unmultiplied(41, 167, 68, 31), // .12
                warning: AMBER_60,
                warning_subtle: Color32::from_rgba_unmultiplied(201, 138, 18, 31),
                danger: RED_60,
                danger_solid: RED_60,
                danger_hover: Color32::from_rgb(0xc2, 0x29, 0x32),
                danger_subtle: Color32::from_rgba_unmultiplied(214, 58, 65, 26), // .10
                info: AZURE_60,
                info_subtle: AZURE_100,
            },

            scrollbar: Scrollbar {
                thumb: GRAY_13,
                thumb_hover: GRAY_11,
            },

            row_alt: paper::SHADE,
            fonts: Fonts::default(),
        }
    }

    /// Pick a side by [`egui::Theme`], for an app that follows the system.
    pub fn of(theme: egui::Theme) -> Self {
        match theme {
            egui::Theme::Dark => Self::dark(),
            egui::Theme::Light => Self::light(),
        }
    }

    /// Which side of the token set this is, as egui names it.
    pub fn kind(&self) -> egui::Theme {
        if self.dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        }
    }

    /// The text color for a piece of copy, given whether its widget is enabled.
    ///
    /// Azur disables by swapping in `text-disabled`, not by fading — a faded
    /// near-white on a near-black canvas turns gray anyway, and loses the
    /// contrast floor while doing it.
    pub fn text_for(&self, enabled: bool) -> Color32 {
        if enabled {
            self.text.primary
        } else {
            self.text.disabled
        }
    }

    /// Below this fraction of full scale, a cell is left alone. See [`Self::heat`].
    pub const HEAT_FLOOR: f32 = 0.12;

    /// A magnitude tint for the cell behind a number: nothing at all until the value
    /// is worth noticing, then `hue`, deepening as the value climbs.
    ///
    /// `t` is the reading as a fraction of as much as that column ever shows. `hue`
    /// is the colour that column's own graph is drawn in — so a heavy memory cell is
    /// the violet of the memory trace and a heavy disk cell is the green of the disk
    /// trace, and the table and the graphs agree about what each resource looks like.
    /// The older ramp went amber-into-red for every column alike, which is Task
    /// Manager's own scheme and says *alarm* about a process doing nothing wrong.
    /// Severity is the number's job, not the cell's: see [`Self::threshold_text`].
    ///
    /// Nothing below [`Self::HEAT_FLOOR`]. A tint that starts at the first non-zero
    /// reading paints most of the list, and a list where everything is coloured tells
    /// a reader nothing — the whole point is that the eye lands on the few rows that
    /// are actually busy.
    pub fn heat(&self, t: f32, hue: Color32) -> Color32 {
        let t = t.clamp(0.0, 1.0);
        if t < Self::HEAT_FLOOR {
            return Color32::TRANSPARENT;
        }
        // Ramped from the floor rather than from zero, so the first cell to be
        // tinted is faintly tinted rather than arriving already a fifth of the way
        // up the scale.
        let over = (t - Self::HEAT_FLOOR) / (1.0 - Self::HEAT_FLOOR);
        let alpha = (14.0 + 64.0 * over) as u8;
        Color32::from_rgba_unmultiplied(hue.r(), hue.g(), hue.b(), alpha)
    }

    /// Text color for a percentage: quiet until it matters, then `warning`, then
    /// `danger`. `warn_at` and `crit_at` are in the same units as `pct`.
    pub fn threshold_text(&self, pct: f32, warn_at: f32, crit_at: f32) -> Color32 {
        if pct >= crit_at {
            self.status.danger
        } else if pct >= warn_at {
            self.status.warning
        } else {
            self.text.primary
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // The same darkening the danger button's pressed state paints with, so the
    // state measured here is the state drawn.
    use crate::components::darken;

    // The ruler, from [`crate::contrast`] rather than a copy per file: `ratio` for ink
    // on a fill, `over` first for anything translucent, and the two WCAG floors.
    // Both arguments to `ratio` must be opaque, which is what `over` is for — the
    // `*_subtle` fills are translucent and mean nothing measured on their own.
    use crate::contrast::{over, ratio as contrast, SHAPE, TEXT};

    fn assert_at_least(min: f32, got: f32, what: &str) {
        assert!(
            got >= min,
            "{what}: {got:.2}:1, wanted at least {min}:1 — see `Theme::dark`"
        );
    }

    /// The reason `accent.mark` and `status.danger_solid` exist: a fill that
    /// carries white text has to be dark enough for the white, and the same hue
    /// used as ink has to be light enough to read on a dark surface. One value
    /// cannot do both on a near-black canvas.
    /// Every state, not just the resting one: the fills are far enough down their
    /// ramps that hover and pressed clear AA too, which the CSS's values did not.
    #[test]
    fn filled_controls_hold_their_label() {
        for t in [Theme::dark(), Theme::light()] {
            let side = if t.dark { "dark" } else { "light" };
            for (fill, what) in [
                (t.accent.default, "primary button"),
                (t.accent.hover, "primary button, hovered"),
                (t.accent.active, "primary button, pressed"),
                (t.status.danger_solid, "danger button"),
                (t.status.danger_hover, "danger button, hovered"),
                (
                    darken(t.status.danger_solid, 0.92),
                    "danger button, pressed",
                ),
            ] {
                assert_at_least(
                    TEXT,
                    contrast(t.text.on_accent, fill),
                    &format!("{side}: on_accent on {what}"),
                );
            }
        }
    }

    /// The other half of the split: what the fills gave up, the ink keeps.
    ///
    /// Dark only, because dark is what the split was made for and light was never
    /// broken in the same way. Light's own numbers are the CSS's, unchanged by this
    /// port and weaker: `AZURE_60` ink on an `AZURE_100` tint is 3.4:1, and 2.7:1
    /// on the hovered tint. Fixing those means moving the light theme's subtle
    /// fills, which is a separate change with a much wider blast radius — every
    /// selected row, active nav entry and accent badge on that side.
    #[test]
    fn accent_ink_reads_on_the_surfaces_it_lands_on() {
        for t in [Theme::dark()] {
            let side = if t.dark { "dark" } else { "light" };
            // A pressed button's label and a selected row's bar sit on the subtle
            // accent tint; accent text and a spinner sit on the plain surfaces.
            for (bg, what) in [
                (t.accent.subtle, "accent-subtle"),
                (t.accent.subtle_hover, "accent-subtle, hovered"),
                (t.bg.canvas, "the canvas"),
                (t.bg.card, "a card"),
                (t.bg.layer_alt, "a popover"),
            ] {
                // 6.4:1 on `accent-subtle`, 5.3:1 hovered, 7.3:1 on the canvas —
                // where the darkened fill would have been 3.8:1, 3.2:1 and 4.3:1.
                assert_at_least(
                    TEXT,
                    contrast(t.accent.mark, bg),
                    &format!("{side}: accent.mark on {what}"),
                );
            }
        }
    }

    /// The status hues as ink, each on the translucent fill of its own intent —
    /// a `MessageBar`, a `Toast`, a `Badge` — composited over the surfaces those
    /// actually appear on.
    ///
    /// Dark only, and at 4:1 rather than 4.5:1, because these are the CSS's values
    /// untouched and that is where they land: `danger` measures 4.0–4.8:1 over the
    /// four surfaces, the other three 6.1:1 and up. The light theme is materially
    /// worse — `success` and `warning` ink on their own 12%-alpha tints measure as
    /// low as **2.2:1** over `layer_alt` — which is a `tokens.css` problem in its
    /// own right, not one this port introduced, and not one the accent and danger
    /// fills touch.
    #[test]
    fn status_ink_reads_on_its_own_subtle_fill() {
        for t in [Theme::dark()] {
            let side = if t.dark { "dark" } else { "light" };
            for (bg, what) in [
                (t.bg.canvas, "the canvas"),
                (t.bg.layer, "a layer"),
                (t.bg.card, "a card"),
                (t.bg.layer_alt, "a popover"),
            ] {
                for (ink, tint, which) in [
                    (t.status.danger, t.status.danger_subtle, "danger"),
                    (t.status.success, t.status.success_subtle, "success"),
                    (t.status.warning, t.status.warning_subtle, "warning"),
                    (t.status.info, t.status.info_subtle, "info"),
                ] {
                    assert_at_least(
                        4.0,
                        contrast(ink, over(tint, bg)),
                        &format!("{side}: {which} on {which}-subtle over {what}"),
                    );
                }
            }
        }
    }

    /// A filled control still has to be findable as a shape, which is the other
    /// half of the trade — a fill dark enough for white text but too dark to see
    /// against the window would be no improvement.
    ///
    /// Against the canvas, where a dialog's primary action and a toolbar's buttons
    /// sit, the resting fills clear the 3:1 WCAG asks of a boundary: 3.1:1 blue and
    /// 3.5:1 red in dark.
    ///
    /// Against a *raised* surface — a card, a popover — dark measures 2.6:1 blue
    /// and 3.0:1 red, so the floor here is lower, deliberately. WCAG's figure is
    /// for controls identified by their boundary; these carry a white label at
    /// 5.7–6.5:1, and a saturated hue against a neutral gray separates in a way a
    /// luminance ratio does not measure. `Theme::dark` says what to do if that ever
    /// stops being true.
    #[test]
    fn filled_controls_stay_visible_against_their_surface() {
        /// The floor for a labeled fill on a raised surface. Not a standard —
        /// a measured position, low enough to permit the chosen fills and high
        /// enough that a fill sliding toward its background still fails.
        const LABELED_FILL: f32 = 2.5;

        for t in [Theme::dark(), Theme::light()] {
            let side = if t.dark { "dark" } else { "light" };
            for (bg, what, floor) in [
                (t.bg.canvas, "the canvas", SHAPE),
                (t.bg.card, "a card", LABELED_FILL),
                (t.bg.layer_alt, "a popover", LABELED_FILL),
            ] {
                for (fill, which) in [
                    (t.accent.default, "primary button"),
                    (t.status.danger_solid, "danger button"),
                ] {
                    assert_at_least(
                        floor,
                        contrast(fill, bg),
                        &format!("{side}: {which} against {what}"),
                    );
                }
            }
        }
    }

    /// The focus ring is the one accent surface that must stay bright: it is a 2px
    /// outline read against whatever it encircles, never a fill with text on it.
    #[test]
    fn focus_ring_and_links_stay_bright() {
        for t in [Theme::dark(), Theme::light()] {
            let side = if t.dark { "dark" } else { "light" };
            assert_at_least(
                SHAPE,
                contrast(t.stroke.focus, t.bg.canvas),
                &format!("{side}: focus ring on the canvas"),
            );
            assert_at_least(
                SHAPE,
                contrast(t.stroke.focus, t.bg.control),
                &format!("{side}: focus ring on a control"),
            );
            // `text.link` is `AZURE_90` in dark and `AZURE_60` in light, both
            // straight from the CSS. Dark clears AA comfortably; light measures
            // 3.97:1 against its canvas — 4.27:1 against the CSS's own white-ish
            // one — a shortfall this port inherits rather than invents, and one
            // that is the *link's* to fix, not the surface's: moving it would
            // change every hyperlink in both themes.
            assert_at_least(
                if t.dark { TEXT } else { SHAPE },
                contrast(t.text.link, t.bg.canvas),
                &format!("{side}: a link on the canvas"),
            );
        }
    }

    /// Body text, for completeness: if this ever fails the palette is broken in a
    /// way no button tuning would explain.
    #[test]
    fn body_text_passes_on_every_surface() {
        for t in [Theme::dark(), Theme::light()] {
            let side = if t.dark { "dark" } else { "light" };
            for (bg, what) in [
                (t.bg.canvas, "the canvas"),
                (t.bg.layer, "a layer"),
                (t.bg.card, "a card"),
                (t.bg.control, "a control"),
            ] {
                assert_at_least(
                    TEXT,
                    contrast(t.text.primary, bg),
                    &format!("{side}: primary text on {what}"),
                );
                assert_at_least(
                    SHAPE,
                    contrast(t.text.secondary, bg),
                    &format!("{side}: secondary text on {what}"),
                );
            }
        }
    }
}
