//! The decisions an *application window* makes that a web page does not.
//!
//! Azur is specified for the browser, and this crate already leaves the CSS in a
//! handful of places because a desktop window is not a page — square menus, controls
//! at `radius-small`, no white surface, filled accents two rungs down their ramps.
//! Those are unconditional; every Azur window gets them.
//!
//! This module is the next layer of the same argument, and it is **opt-in**, because
//! it only holds for one shape of window: dense, navigational, open all day — a file
//! manager, a process table, a launcher, a mixer. Panes divided by seams rather than
//! floating cards. Hundreds of rows the pointer sweeps across. Nothing here is a new
//! colour; it is Azur's own rungs, chosen for that window instead of for a page.
//!
//! ```no_run
//! # fn demo(ctx: &egui::Context) {
//! // Fonts, the dark palette adjusted for an application window, and the density
//! // that goes with it.
//! let theme = azur_egui_theme::desktop::install(ctx, true);
//! # }
//! ```
//!
//! Spelled out, for a window that also has roles of its own to move:
//!
//! ```no_run
//! use azur_egui_theme::{desktop, fonts, install, Theme};
//! # fn demo(ctx: &egui::Context) {
//! fonts::install(ctx);
//! let mut theme = Theme::dark();
//! desktop::apply(&mut theme);
//! install(ctx, &theme, &desktop::density());
//! # }
//! ```
//!
//! # 1. One hover, and it is a place rather than a control
//!
//! [`apply`] moves `background-control-hover` and `background-card-hover` three rungs
//! up the neutral ramp — [`hover_fill`] — and that single move is the whole of the
//! colour change.
//!
//! Azur's hover ladder is written for a control that wants to look liftable off
//! `background-layer`. Most of what an application window hovers is not a control: it
//! is a row, a path segment, a folder in a tree, a tab — a *place*, on a listing that
//! is nearly black, where one rung is not enough to see. Measured in
//! [`contrast::apart`](crate::contrast::apart), the token's own rung is ΔL\* 11.3 off
//! the panel it sits on and this is **28.5**.
//!
//! **Set the token, never the call sites.** That is not a style preference, it is the
//! bug this module was extracted after: the window it came from moved its rows, its
//! tree and its path bar by hand, and Back, Forward, Up and Refresh went on wearing
//! the old grey for a week — four buttons in the middle of the window, a rung and a
//! half off everything around them, and nothing in the code looking wrong at either
//! end. They reached their fill through the *token*, which nobody had moved. One
//! token is what egui's own widget styles, [`MenuItem`](crate::components::MenuItem),
//! [`ListRow`](crate::components::ListRow), the column headers, the caption buttons
//! and a hand-painted row all read.
//!
//! A press is one rung further out again — [`press_fill`] — in whichever direction
//! "out" is on that side of the palette. Naming both rungs outright is what lets
//! [`control_fills`] ignore the surface it is painted on: Azur's `control-active`
//! *is* [`seam`] in the light theme, so a pressed segment on a path bar used to paint
//! the surface's own colour over the surface and the fill vanished at the moment of
//! the press.
//!
//! # 2. A selected row is the accent as a surface — in the dark theme
//!
//! [`row_fill`]. Azur's list vocabulary is `accent-subtle`, which is right on a light
//! page and, in the dark theme, a navy barely off the panel. So the dark side takes
//! the accent's *surface* ramp instead, at `active` with `default` under the pointer.
//!
//! The light side keeps `accent-subtle`, and that is not an omission. The ink on a
//! row does not change when the row is selected — a listing paints `text-primary` and
//! `text-secondary` either way — so on a light window the selected fill has to stay
//! light or the metadata goes with it. Sampled: `accent-default` in the light theme
//! carries near-black primary text at 4.3:1 and secondary at **1.7:1**, which is the
//! same failure the dark theme's first attempt had.
//!
//! Every figure here was sampled off the framebuffer rather than reasoned about, and
//! the dark ramp was walked from the wrong end: `AZURE_ACCENT` (`#3AA0FF`) put a
//! selected row's name at **2.5:1** and its metadata columns at **1.06:1**;
//! `accent-default` was halfway back at 6.05 and 2.51; `accent-active` is **7.96 and
//! 3.31**. Every step down bought legibility instead of costing it, which is the tell
//! that the bright first guess was three rungs past where it should have stopped.
//! The metadata columns remain the weakest thing on a selected row.
//!
//! # 3. Nothing structural is rounded
//!
//! [`panel`] is square and unringed: `radius-medium` and `stroke-subtle` are right for
//! a card floating on canvas and wrong for a panel that *is* the window's structure.
//! What separates two panels is [`seam`] — one point of `stroke-subtle` — not a gap
//! with a border on each side of it, which is three lines wide and reads as a row of
//! loose cards.
//!
//! [`squared`] is the same rule for the fields that stand in for something square: a
//! rename box over a row, a path field over a breadcrumb, a filter box in a bar of
//! flush panels. [`control_radius`](crate::components::control_radius) is right for a
//! control you go to; a 4px radius on a 20px box that has *replaced* a square thing
//! reads as a bubble.
//!
//! # 4. Rows are 24 points, and they touch
//!
//! [`row::DENSE`](crate::tokens::row::DENSE) rather than
//! [`row::TABLE`](crate::tokens::row::TABLE)'s 36: Azur's table row is sized for a
//! form, and a listing is read by scanning hundreds of lines, where every point of
//! row height is a line that did not fit.
//!
//! And no `item_spacing` between them. The 4px scale is right for controls and wrong
//! for a list of names — the hover fill *is* the row, so 8 points between rows turns a
//! sidebar into a column of buttons and costs a third of the panel. [`density`] cannot
//! say that for you, because it is a property of how a list is painted rather than a
//! field of [`egui::style::Spacing`]; the components here already do it, and a
//! hand-painted list should compute `row.top() + index * height` and nothing else.
//!
//! # 5. Everything in a row is centred on its ink
//!
//! The rule that has to be said the most often, so it is code:
//! [`ink_baseline`](crate::components::ink_baseline) for text that shares a row with a
//! glyph, [`row_baseline`](crate::components::row_baseline) for a row of text alone,
//! and one baseline per row in both cases — not per column, not per font.
//!
//! A line box reserves room under the baseline for descenders and above the capitals
//! for accents. A file name uses neither, so text centred by its *box* hangs 1.5
//! points below an icon centred on its *ink*, at every size, in every row, and it
//! reads as the window being slightly out of true without ever looking like a bug.
//! The glyph has the matching obligation, in
//! [`icons::guidelines`](crate::icons::guidelines): art is centred in its box
//! optically, because the box is what a caller centres.

use crate::style::StyleOptions;
use crate::theme::Theme;
use crate::tokens::{control, palette};
use egui::{Color32, CornerRadius, Painter, Rect, Stroke, StrokeKind, Ui};

/// The line between two panels, in points. One physical pixel of [`seam`].
pub const SEAM: f32 = 1.0;

/// Azur, adjusted for an application window: the theme, ready to install.
pub fn theme(dark: bool) -> Theme {
    let mut t = if dark { Theme::dark() } else { Theme::light() };
    apply(&mut t);
    t
}

/// Fonts, the adjusted palette and the dense density, in one call. Returns the theme,
/// since you need the same roles to paint with.
pub fn install(ctx: &egui::Context, dark: bool) -> Theme {
    crate::fonts::install(ctx);
    let t = theme(dark);
    crate::style::install(ctx, &t, &density());
    t
}

/// Move the hover rungs onto [`hover_fill`], which is the one role change this whole
/// module makes to the palette.
///
/// Both hover tokens, so that a hovered button, a hovered row, a hovered menu entry
/// and a hovered card are one colour: `background-control-hover` is what egui's widget
/// styles and most of [`crate::components`] read, and `background-card-hover` is what
/// a [`ListRow`](crate::components::ListRow) and a hoverable
/// [`Card`](crate::components::Card) read. Leaving the second one behind is how a
/// window ends up with two hover greys and no line of code that looks wrong.
///
/// Nothing else moves. In particular `background-control-active` is left where it is,
/// for the places that use it as an *inert* fill rather than as a press — a row that
/// has gone quiet while it is dragged — and [`press_fill`] is what a press takes.
pub fn apply(t: &mut Theme) {
    let hover = place_hover(t);
    t.bg.control_hover = hover;
    t.bg.card_hover = hover;
}

/// The rung [`apply`] moves the two hover tokens to — three up Azur's neutral ramp
/// from the `GRAY_5` its `control-hover` names, and three *down* the light one,
/// because "away from the surface" points in opposite directions on the two sides.
///
/// | | hover | ΔL\* off `background-layer` | primary text on it | secondary text |
/// | --- | --- | --- | --- | --- |
/// | dark | `GRAY_8` `#4D525C` | 28.5 (the token's own rung: 11.3) | 7.5:1 | 2.9:1 |
/// | light | `GRAY_13` `#B3B9C4` | 22.4 (the token's own rung: 8.9) | 9.8:1 | 3.9:1 |
///
/// The light rung is not the dark one mirrored by eye but by measurement: the dark
/// move is 2.5× the distance the token asks for, and this is 2.5× of the light one.
/// Going as dark as `GRAY_9` on that side — which is what the first attempt did, on
/// the strength of a hex value picked while looking at the dark theme — puts
/// near-black body text on the hovered row at **3.26:1**, under AA. A hover band is
/// not an inverted row: the ink does not change when the pointer arrives, so the fill
/// has to stay on the ink's side of the palette.
///
/// Private on purpose. [`hover_fill`] is what a call site asks, and it reads the token
/// back, so an application that moves `control-hover` again moves every hover with it.
fn place_hover(t: &Theme) -> Color32 {
    if t.dark {
        palette::GRAY_8
    } else {
        palette::GRAY_13
    }
}

/// The density an application window installs with: 24px controls, and egui's own
/// label selection off because the rows are painted by hand.
pub fn density() -> StyleOptions {
    StyleOptions {
        control_height: control::SMALL,
        selectable_labels: false,
        ..Default::default()
    }
}

/// The fill under the pointer on anything you hover in order to *go* somewhere: a row
/// in a listing, an entry in a tree, a segment on a path bar, a toolbar button.
///
/// `background-control-hover`, read back from the theme rather than named here, so
/// that there is one value and not two — which is the whole of rule 1 in the
/// [module documentation](self). [`apply`] is what puts the rung there, and the table
/// beside `place_hover` in this file is where the rung came from.
pub fn hover_fill(t: &Theme) -> Color32 {
    t.bg.control_hover
}

/// A control being pressed: one rung past the hover it came from, in the same
/// direction, so a press reads as *more* of what the hover was rather than a reversal.
pub fn press_fill(t: &Theme) -> Color32 {
    if t.dark {
        palette::GRAY_9
    } else {
        palette::GRAY_12
    }
}

/// The hover and pressed fills for a control, whatever surface it is painted on.
///
/// `(hover, pressed)`. Deliberately not a function of the surface — see rule 1 in the
/// [module documentation](self) for the fill that vanished when it was.
pub fn control_fills(t: &Theme) -> (Color32, Color32) {
    (hover_fill(t), press_fill(t))
}

/// The fill for a row in a given state, or `None` to leave the surface showing.
///
/// | state | dark | light |
/// | --- | --- | --- |
/// | hovered | [`hover_fill`] | [`hover_fill`] |
/// | selected | `accent-active` | `accent-subtle` |
/// | selected + hovered | `accent-default` | `accent-subtle-hover` |
///
/// Rule 2 in the [module documentation](self) is why the two sides differ, and why
/// this exists beside [`ListRow`](crate::components::ListRow)'s own rules rather than
/// replacing them: a window that has not adopted this module should keep Azur's.
pub fn row_fill(t: &Theme, selected: bool, hovered: bool) -> Option<Color32> {
    let (rest, lifted) = chosen(t);
    match (selected, hovered) {
        (true, true) => Some(lifted),
        (true, false) => Some(rest),
        (false, true) => Some(hover_fill(t)),
        (false, false) => None,
    }
}

/// The fill and the ink for a control that is **on**: a latched toggle button.
///
/// `(fill, ink)`, and the fill is the one a *selected row* gets. That is the whole rule, and it
/// is the same argument as rule 1 in the [module documentation](self) pointed at the accent
/// instead of the neutrals: **one surface in this window means "on", whatever is wearing it.** A
/// selected row and a latched button are the same statement, and a window that says it two ways
/// is a window where you have to learn which is which.
///
/// Azur paints a latched [`IconButton`](crate::components::IconButton) in the `accent-subtle`
/// ramp with `accent-mark` ink, which is right on a light page and, in the dark theme, is the
/// navy that [`row_fill`] already declined for exactly this reason: on a near-black panel it is
/// barely off the surface, so a toggle wearing it is one you have to look twice at to read.
///
/// The ink is `text-primary` rather than `accent-mark` — the same ink a selected row's name
/// carries, measured at **7.96:1** on the dark fill and 14.5:1 on the light one, where
/// `accent-mark` on the dark fill is 3.13:1. That clears what WCAG asks of a *shape*, and a
/// 14px glyph is a shape, but a toggle's whole job is to be read at a glance.
///
/// Two states rather than three: there is no separate pressed rung, because pressing a latch is
/// answered by the latch changing. `hovered` lifts it the same way a hovered selected row lifts.
pub fn latched(t: &Theme, hovered: bool) -> (Color32, Color32) {
    let (rest, lifted) = chosen(t);
    (if hovered { lifted } else { rest }, t.text.primary)
}

/// The accent surface that means *chosen* — selected, or on — at rest and hovered.
///
/// One place, so a latched control and a selected row cannot drift apart. See [`row_fill`] for
/// why the two sides of the palette take different rungs.
fn chosen(t: &Theme) -> (Color32, Color32) {
    if t.dark {
        (t.accent.active, t.accent.default)
    } else {
        (t.accent.subtle, t.accent.subtle_hover)
    }
}

/// The 2px bar down the left edge of a selected row.
///
/// `accent-mark` — the accent as ink, which is the role's own job description. Not
/// `accent-default`: with [`row_fill`] painting the *fill* from the accent's surface
/// ramp, a bar from the same ramp is a bar the colour of what it is drawn on.
pub fn selection_bar(painter: &Painter, row: Rect, t: &Theme) {
    painter.rect_filled(
        Rect::from_min_size(row.min, egui::vec2(2.0, row.height())),
        CornerRadius::ZERO,
        t.accent.mark,
    );
}

/// The colour a panel boundary shows, and the fill of the surfaces welded to it: a
/// selected tab, the bar directly under it.
///
/// `stroke-subtle`, which is what Azur calls a separator *inside* a surface — which is
/// what a seam is. It is squeezed from both sides, by the panel it divides
/// (`background-layer`) and by anything set into it (`background-layer-alt`), and
/// [`crate::contrast::SAME`] is the floor it has to clear against both;
/// `the_seam_is_told_apart_from_what_it_sits_between` measures it.
pub fn seam(t: &Theme) -> Color32 {
    t.stroke.subtle
}

/// A structural panel: `background-layer`, square, unringed, no margin of its own.
///
/// Rule 3 in the [module documentation](self). Pass it to `SidePanel::frame` and
/// friends, or use it for a pane you lay out yourself.
pub fn panel(t: &Theme) -> egui::Frame {
    egui::Frame::new()
        .fill(t.bg.layer)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::ZERO)
        .inner_margin(egui::Margin::ZERO)
}

/// Draw the widgets `add` puts up with square corners.
///
/// For a text field that has *replaced* something square and takes its shape while it
/// is up. Scoped to this `Ui`, and going through the style rather than through each
/// widget's builder, because that is the one place both kinds of field read it from:
/// [`egui::TextEdit`] takes its frame from `Style::interact(..).corner_radius` and
/// [`TextField`](crate::components::TextField) takes it from `widgets.inactive`. Every
/// state, because `Style::interact` picks by response and a field's corners must not
/// change under the pointer.
pub fn squared<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    let widgets = ui.visuals().widgets.clone();
    {
        let w = &mut ui.visuals_mut().widgets;
        for state in [
            &mut w.noninteractive,
            &mut w.inactive,
            &mut w.hovered,
            &mut w.active,
            &mut w.open,
        ] {
            state.corner_radius = CornerRadius::ZERO;
        }
    }
    let out = add(ui);
    ui.visuals_mut().widgets = widgets;
    out
}

/// Put every egui popup away when the window loses focus, and say whether it did.
///
/// A menu belongs to a moment. Click into another window with one open and it is still
/// there when you come back — pinned over content it no longer describes, and in the
/// case of a context menu, over a *selection* the click may have changed. Every
/// platform menu closes on deactivation; egui's do not, because egui has no idea the
/// window went away.
///
/// The return value is for the caller's own transient state — a latched toolbar
/// button, a dropdown it tracks itself — so both go away in the same breath:
///
/// ```no_run
/// # struct App { menu_open: bool }
/// # impl App {
/// # fn frame(&mut self, ctx: &egui::Context) {
/// if azur_egui_theme::desktop::close_popups_on_blur(ctx) {
///     self.menu_open = false;
/// }
/// # }
/// # }
/// ```
///
/// Reads [`egui::InputState::focused`], which is *window* focus and which an
/// integration has to report — `eframe` does. Note that
/// [`egui::RawInput::default`] has it `true`, so a bare test harness is focused until
/// it says otherwise.
pub fn close_popups_on_blur(ctx: &egui::Context) -> bool {
    if ctx.input(|i| i.focused) {
        return false;
    }
    egui::Popup::close_all(ctx);
    true
}

/// The corner radius of anything in this window that is *not* a control: none.
///
/// Here so the rule can be read beside
/// [`control_radius`](crate::components::control_radius), which is the only radius a
/// dense application window has. A row, a panel, a seam, a header, a drop target and a
/// docking preview are all square: a rounded corner on a full-width band reads as a
/// floating pill, and rounding a rectangle that marks *where something will land* only
/// blurs where the edge is.
pub fn structural_radius() -> CornerRadius {
    CornerRadius::ZERO
}

/// The rectangle a drag would land in: an accent wash and a 2px outline, square.
///
/// For a drop target, a docking preview, or the pane a gesture is currently over.
/// Two things about it are corrections rather than taste, both learned the hard way:
///
/// - **Painted over the content, not under it.** A selected row's fill is an opaque
///   surface drawn after the row's own background, so a wash underneath it vanished
///   completely — dropping onto a folder that happened to be selected showed nothing
///   at all.
/// - **Square**, per [`structural_radius`]. This carried `radius-medium` for as long
///   as the panes did, and it is showing where a rectangle will go.
pub fn drop_target(painter: &Painter, rect: Rect, t: &Theme) {
    let accent = t.accent.default;
    painter.rect_filled(
        rect,
        structural_radius(),
        Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 40),
    );
    painter.rect_stroke(
        rect,
        structural_radius(),
        Stroke::new(2.0, accent),
        StrokeKind::Inside,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contrast::{apart, over, ratio, SAME, SHAPE, TEXT};
    use crate::tokens::row;

    fn both() -> [Theme; 2] {
        [theme(true), theme(false)]
    }

    fn side(t: &Theme) -> &'static str {
        if t.dark {
            "dark"
        } else {
            "light"
        }
    }

    /// The one role move, and the reason it is a role move: every hover in the window
    /// reads one value, including the ones nobody remembered to look at.
    #[test]
    fn applying_moves_both_hover_rungs_and_nothing_else() {
        for dark in [true, false] {
            let plain = if dark { Theme::dark() } else { Theme::light() };
            let ours = theme(dark);
            assert_eq!(ours.bg.control_hover, hover_fill(&ours));
            assert_eq!(ours.bg.card_hover, hover_fill(&ours));
            assert_ne!(
                plain.bg.control_hover, ours.bg.control_hover,
                "{}: the preset is not moving the hover at all",
                side(&ours)
            );
            // Everything a press, a drag or a surface reads is left alone.
            assert_eq!(plain.bg.control_active, ours.bg.control_active);
            assert_eq!(plain.bg.control, ours.bg.control);
            assert_eq!(plain.bg.layer, ours.bg.layer);
            assert_eq!(plain.bg.layer_alt, ours.bg.layer_alt);
            assert_eq!(plain.accent.subtle, ours.accent.subtle);
        }
    }

    /// The rungs, pinned. The consistency assertion above would pass just as happily
    /// if [`apply`] were deleted and everything agreed on Azur's own `GRAY_5`.
    #[test]
    fn the_hover_and_press_rungs_are_the_measured_ones() {
        let (dark, light) = (theme(true), theme(false));
        assert_eq!(hover_fill(&dark), palette::GRAY_8);
        assert_eq!(press_fill(&dark), palette::GRAY_9);
        assert_eq!(hover_fill(&light), palette::GRAY_13);
        assert_eq!(press_fill(&light), palette::GRAY_12);
    }

    /// A hover has to be visible against every surface it can land on, a press has to
    /// be visible against the hover, and the press has to be *further out* rather than
    /// back the way it came.
    ///
    /// The bug: a path bar painted [`seam`] — which in the light theme *is*
    /// `background-control-active` — put a pressed control's standard fill on the
    /// surface's own colour, and the fill vanished at the moment of the press. Nothing
    /// was wrong with either token; they simply met.
    #[test]
    fn the_fills_step_away_from_every_surface() {
        for t in both() {
            let (hover, press) = control_fills(&t);
            for (surface, what) in [
                (t.bg.layer, "background-layer"),
                (t.bg.layer_alt, "background-layer-alt"),
                (t.bg.card, "a card"),
                (seam(&t), "a seam"),
            ] {
                for (fill, state) in [(hover, "a hover"), (press, "a press")] {
                    assert!(
                        apart(fill, surface) >= SAME,
                        "{}: {state} on {what} is only ΔL* {:.1} off it",
                        side(&t),
                        apart(fill, surface)
                    );
                }
                // And in one direction, whichever direction the palette makes it.
                let out = (crate::contrast::lstar(hover) - crate::contrast::lstar(surface)).signum();
                let on = (crate::contrast::lstar(press) - crate::contrast::lstar(hover)).signum();
                assert_eq!(
                    out,
                    on,
                    "{}: on {what} a press turns back on the hover it came from",
                    side(&t)
                );
            }
        }
    }

    /// The failure that made the light rung a measurement rather than a mirror: the ink
    /// on a row does not change when the pointer arrives, so the fill cannot cross to
    /// the other side of the ink.
    #[test]
    fn a_row_can_still_be_read_when_it_is_hovered_or_pressed() {
        for t in both() {
            // A press is a state you cannot read in anyway — the pointer is down and
            // the fill is one rung past the hover — so it is held to the name alone.
            // Everything a row is read *in* holds both columns.
            for (fill, what, metadata) in [
                (hover_fill(&t), "a hover", true),
                (press_fill(&t), "a press", false),
                (row_fill(&t, true, false).unwrap(), "a selected row", true),
                (
                    row_fill(&t, true, true).unwrap(),
                    "a selected row, hovered",
                    true,
                ),
            ] {
                assert!(
                    ratio(t.text.primary, fill) >= TEXT,
                    "{}: a name on {what} is {:.2}:1",
                    side(&t),
                    ratio(t.text.primary, fill)
                );
                // Metadata is the weakest thing on a filled row and always has been.
                // Measured, dark: 2.88:1 hovered, 3.15:1 selected, 2.51:1 selected and
                // hovered at once — where the same column on the bright first attempt
                // was 1.06:1, which is to say gone. Light clears 4.5:1 everywhere,
                // because its selected fills are tints rather than surfaces.
                //
                // 2.5 is a floor at where they actually sit, not a standard: WCAG has
                // nothing to say about a 12px column on a filled row, and the honest
                // fix if this ever has to pass is to lift the *ink* on a selected row
                // — `text-primary` for all four columns measures about 7:1 — which is
                // a decision about the window's look, not about the palette.
                if metadata {
                    assert!(
                        ratio(t.text.secondary, fill) >= 2.5,
                        "{}: a metadata column on {what} is {:.2}:1",
                        side(&t),
                        ratio(t.text.secondary, fill)
                    );
                }
            }
        }
    }

    /// A latched control wears the same surface a selected row does, and its glyph reads on it.
    ///
    /// The pair being equal is the point: two accent surfaces meaning "on" is one too many.
    #[test]
    fn a_latched_control_is_painted_like_a_selected_row() {
        for t in both() {
            for hovered in [false, true] {
                let (fill, ink) = latched(&t, hovered);
                assert_eq!(
                    Some(fill),
                    row_fill(&t, true, hovered),
                    "{}: a latched button and a selected row are different blues",
                    side(&t)
                );
                assert!(
                    ratio(ink, fill) >= TEXT,
                    "{}: a latched glyph is {:.2}:1 on its own fill",
                    side(&t),
                    ratio(ink, fill)
                );
                // And it is not the tint Azur latches with, which is what this deviates from.
                assert_ne!(fill, t.accent.subtle_active);
                if t.dark {
                    assert_ne!(
                        fill, t.accent.subtle,
                        "dark took the navy this exists to avoid"
                    );
                }
            }
        }
    }

    /// A selected row has to read as selected — against the surface, against a merely
    /// hovered row, and its bar against its own fill.
    #[test]
    fn a_selected_row_is_told_apart_from_a_hovered_one() {
        for t in both() {
            let selected = row_fill(&t, true, false).unwrap();
            let hovered = row_fill(&t, false, true).unwrap();
            assert!(
                apart(selected, t.bg.layer) >= SAME,
                "{}: a selected row is ΔL* {:.1} off the panel",
                side(&t),
                apart(selected, t.bg.layer)
            );
            assert_ne!(selected, hovered);
            assert!(
                ratio(t.accent.mark, selected) >= SHAPE,
                "{}: the selection bar is {:.2}:1 on the row it marks",
                side(&t),
                ratio(t.accent.mark, selected)
            );
        }
    }

    /// The seam is the whole of what divides one pane from the next, so it has to read
    /// as a surface of its own against both of the surfaces it sits between.
    #[test]
    fn the_seam_is_told_apart_from_what_it_sits_between() {
        for t in both() {
            for (other, what) in [
                (t.bg.layer, "the panel it divides"),
                (t.bg.layer_alt, "a header set into it"),
            ] {
                assert!(
                    apart(seam(&t), other) >= SAME,
                    "{}: the seam is only ΔL* {:.1} from {what}",
                    side(&t),
                    apart(seam(&t), other)
                );
            }
        }
    }

    /// Nothing structural is rounded, and a dense window has exactly two radii.
    #[test]
    fn only_controls_are_rounded() {
        let control = crate::components::control_radius();
        assert_eq!(structural_radius(), CornerRadius::ZERO);
        assert_eq!(control, CornerRadius::same(crate::tokens::radius::SMALL));
        assert_ne!(control, structural_radius(), "then there is one radius");
        // And a panel takes the structural one, with nothing ringing it.
        for t in both() {
            let panel = panel(&t);
            assert_eq!(panel.corner_radius, structural_radius());
            assert_eq!(panel.stroke, Stroke::NONE);
            assert_eq!(panel.fill, t.bg.layer);
        }
    }

    /// A drop target's wash goes *over* the row it marks, so it is measured composited
    /// rather than on its own — and it has to survive landing on a selected row, which
    /// is an opaque accent surface of its own.
    #[test]
    fn a_drop_target_still_reads_on_a_selected_row() {
        for t in both() {
            // The wash `drop_target` paints, which is the accent at alpha 40.
            let accent = t.accent.default;
            let wash = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 40);
            for (under, what) in [
                (row_fill(&t, true, false).unwrap(), "a selected row"),
                (t.bg.layer, "the panel"),
            ] {
                let got = ratio(t.text.primary, over(wash, under));
                assert!(
                    got >= TEXT,
                    "{}: a name under a drop wash on {what} is {got:.2}:1",
                    side(&t)
                );
            }
        }
    }

    #[test]
    fn the_density_is_the_dense_one() {
        let d = density();
        assert_eq!(d.control_height, control::SMALL);
        assert!(!d.selectable_labels);
        // At compile time, since all three are constants: a listing row is shorter
        // than the form-sized one Azur names, and a sidebar row is shorter again.
        const {
            assert!(row::DENSE < row::TABLE);
            assert!(row::TIGHT < row::DENSE);
        }
    }

    #[test]
    fn a_blurred_window_puts_its_popups_away() {
        let ctx = egui::Context::default();
        install(&ctx, true);

        // Focused: nothing happens, and the caller is told nothing happened.
        let focused = egui::RawInput {
            focused: true,
            ..Default::default()
        };
        let mut asked = false;
        let _ = ctx.run_ui(focused, |ui| {
            asked = close_popups_on_blur(ui.ctx());
        });
        assert!(!asked, "a focused window was told to put its menus away");

        let blurred = egui::RawInput {
            focused: false,
            ..Default::default()
        };
        let mut asked = false;
        let _ = ctx.run_ui(blurred, |ui| {
            asked = close_popups_on_blur(ui.ctx());
        });
        assert!(asked, "a window that lost focus kept its menus");
    }
}
