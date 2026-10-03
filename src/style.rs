//! Turning a [`Theme`] into an [`egui::Style`].
//!
//! Everything here is derived from the theme's roles, so there is exactly one
//! place a color is decided. Nothing in this module picks a color of its own; it
//! reaches for the token scales only for shape, type and motion.
//!
//! # Where egui's vocabulary is smaller than Azur's
//!
//! [`egui::Visuals`] has one field where Azur has two or three, and a handful of
//! its fields are read by widgets that Azur styles differently. Each of those
//! collisions is resolved the same way — the field goes to whichever consumer is
//! most common in egui, and [`crate::components`] paints the exact Azur spec for
//! the rest — and each is called out at the line where it happens. The three
//! worth knowing about up front:
//!
//! - `widgets.*.bg_fill` is a checkbox's box, a slider's rail *and* a scrollbar's
//!   handle — three different tokens through one field.
//! - `selection.bg_fill` is a text-selection highlight, a slider's trailing
//!   fill, a progress bar's fill *and* a selected [`egui::Button::selectable`].
//! - `window_fill` is both a popover and a dialog panel.

use crate::theme::Theme;
use crate::tokens::{control, motion, radius, shadow, space, typography};
use egui::{CornerRadius, FontId, Margin, Stroke, Style, TextStyle};
use std::collections::BTreeMap;

/// The [`TextStyle::Name`] keys this theme registers, for the ramp steps egui
/// has no built-in name for.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::text_style;
/// ui.label(egui::RichText::new("Section").text_style(text_style::title()));
/// # }
/// ```
pub mod text_style {
    use egui::TextStyle;

    pub const BODY_STRONG: &str = "azur-body-strong";
    pub const SUBTITLE: &str = "azur-subtitle";
    pub const TITLE: &str = "azur-title";
    pub const TITLE_LARGE: &str = "azur-title-large";
    pub const DISPLAY: &str = "azur-display";

    /// 14px semibold.
    pub fn body_strong() -> TextStyle {
        TextStyle::Name(BODY_STRONG.into())
    }
    /// 16px regular.
    pub fn subtitle() -> TextStyle {
        TextStyle::Name(SUBTITLE.into())
    }
    /// 20px semibold.
    pub fn title() -> TextStyle {
        TextStyle::Name(TITLE.into())
    }
    /// 24px semibold.
    pub fn title_large() -> TextStyle {
        TextStyle::Name(TITLE_LARGE.into())
    }
    /// 32px semibold.
    pub fn display() -> TextStyle {
        TextStyle::Name(DISPLAY.into())
    }
}

/// The handful of choices that are density and taste rather than palette.
///
/// [`Default`] is Azur as documented; a dense, table-heavy window will want
/// `control_height: control::SMALL` and `selectable_labels: false`.
#[derive(Clone, Copy, Debug)]
pub struct StyleOptions {
    /// Minimum height of a button, combo box or menu entry. One of
    /// [`control::SMALL`], [`control::MEDIUM`] (Azur's default) or
    /// [`control::LARGE`].
    pub control_height: f32,
    /// `--azur-*::-webkit-scrollbar { width: 10px }`.
    pub scroll_bar_width: f32,
    /// Overlay the scrollbar on the content instead of reserving a gutter for
    /// it. Off by default, which is what a desktop scrollbar does — and a
    /// floating bar sits on top of the last column of a table.
    pub floating_scroll_bar: bool,
    /// Let egui treat its own label text as selectable. Turn it off in an app
    /// that paints its rows by hand, where the only visible effect is a stray
    /// highlight.
    pub selectable_labels: bool,
    /// Seconds before a hover tooltip appears. Azur's `Tooltip` defaults to
    /// 300ms.
    pub tooltip_delay: f32,
    /// Width a tooltip wraps at. Azur specifies `max-width: 280px`, which suits
    /// the label-length hints the token was written for; raise it in an app whose
    /// tooltips carry a sentence, where 280px turns one line into six.
    pub tooltip_width: f32,
    /// How long egui takes to cross-fade a state change.
    /// [`motion::FAST`] is the duration Azur's components transition on.
    pub animation_time: f32,
}

impl Default for StyleOptions {
    fn default() -> Self {
        Self {
            control_height: control::MEDIUM,
            scroll_bar_width: 10.0,
            floating_scroll_bar: false,
            selectable_labels: true,
            tooltip_delay: 0.3,
            tooltip_width: 280.0,
            animation_time: motion::FAST,
        }
    }
}

/// Azur's type ramp as egui's text styles.
///
/// egui names five styles and Azur names eight, so the five are pointed at their
/// nearest Azur step and the rest are registered under the keys in
/// [`text_style`].
pub fn text_styles(t: &Theme) -> BTreeMap<TextStyle, FontId> {
    let f = &t.fonts;
    [
        (TextStyle::Small, f.caption.clone()),
        (TextStyle::Body, f.body.clone()),
        // Azur's button label is `font-weight-medium`, which rounds down to regular
        // here — see `medium`. egui reads this style for every button, checkbox
        // label, radio label and combo box, so it is most of what an app looks like.
        (TextStyle::Button, f.body.clone()),
        // egui's only heading. Azur's `title-small` is what a dialog, a drawer
        // and a card header use, so it is the one that earns the name.
        (TextStyle::Heading, f.title_small.clone()),
        (TextStyle::Monospace, f.mono.clone()),
        (text_style::body_strong(), f.body_strong.clone()),
        (text_style::subtitle(), f.subtitle.clone()),
        (text_style::title(), f.title.clone()),
        (text_style::title_large(), f.title_large.clone()),
        (text_style::display(), f.display.clone()),
    ]
    .into()
}

/// Build [`egui::Visuals`] from a theme.
pub fn visuals(t: &Theme) -> egui::Visuals {
    let mut v = if t.dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    v.dark_mode = t.dark;

    // ---- Surfaces -------------------------------------------------------
    v.panel_fill = t.bg.canvas;
    // `window_fill` is the fill of `Frame::popup` and `Frame::menu` as well as
    // `egui::Window`, and popovers outnumber floating windows in an egui app by
    // a wide margin -- so this is `background-layer-alt`, the popover surface.
    // `components::Dialog` and `components::Drawer` set `background-layer`
    // themselves, which is what Azur's panels use.
    v.window_fill = t.bg.layer_alt;
    // Every Azur overlay -- popover, tooltip, dialog, drawer, toast -- is
    // bordered with `stroke-subtle`, not `stroke-default`: at that elevation the
    // shadow is doing the separating and a strong line reads as a second frame.
    v.window_stroke = Stroke::new(1.0, t.stroke.subtle);
    v.window_corner_radius = CornerRadius::same(radius::LARGE);
    // Azur makes a menu `radius-large` like a popover, which is the Windows 11
    // shape. A menu here is square instead -- see
    // [`components::menu_frame`](crate::components::menu_frame) for why, and note
    // that this is what a nested submenu is painted with, so the two have to agree.
    v.menu_corner_radius = CornerRadius::ZERO;
    v.window_shadow = shadow::S16;
    v.popup_shadow = shadow::S16;

    // What egui fills a text edit with, and the fallback for a scroll groove.
    // Azur's fields are `background-control`, the same as a button at rest.
    v.extreme_bg_color = t.bg.control;
    v.text_edit_bg_color = Some(t.bg.control);
    v.code_bg_color = t.bg.layer_alt;
    v.faint_bg_color = t.row_alt;

    // ---- Text -----------------------------------------------------------
    // egui otherwise derives "weak" text by fading body text, which on a
    // near-black canvas lands somewhere between two of Azur's steps.
    v.weak_text_color = Some(t.text.secondary);
    v.hyperlink_color = t.text.link;
    v.warn_fg_color = t.status.warning;
    v.error_fg_color = t.status.danger;
    v.text_cursor.stroke = Stroke::new(1.0, t.text.primary);
    v.text_cursor.blink = true;

    // Glyph coverage goes into the atlas as it comes out of the rasteriser, in both
    // palettes -- so Azur's regular weight is drawn at its regular weight.
    //
    // egui's dark default is `TwoCoverageMinusCoverageSq`, `alpha = 2c - c²`, which lifts
    // every partially covered pixel: at c = 0.25 it writes 0.44. It is compensating for a
    // real effect -- light-on-dark text loses apparent weight when glyphs are composited
    // with a plain alpha multiply rather than in linear light -- but the correction is
    // tuned for white on black, and Azur's dark canvas is `#14171A` with text at
    // `#F5F6F8`. Over that much smaller range it over-corrects, and the result is a window
    // whose body text reads as semibold: measured over this design system's own sidebar,
    // 10% more lit pixels than the glyphs actually cover.
    //
    // The same function on both sides also means **the weight does not change when the
    // palette does**. egui's own defaults differ per side (`Off` for light,
    // `2c - c²` for dark), so toggling the theme visibly reweighted the text.
    //
    // The cost is stated rather than hidden: at `SIZE_CAPTION` in `text-tertiary` this is
    // the faintest text the system produces, and it is now drawn at its true coverage.
    // `FontColorTransferFunction::Gamma(0.85)` is the halfway house if that reads too
    // light on a particular display.
    v.text_options.color_transfer_function = egui::epaint::FontColorTransferFunction::Off;

    // One rasterisation per glyph, at a whole pixel.
    //
    // egui otherwise renders each glyph at up to four fractional horizontal offsets and
    // picks the one matching where the glyph lands, which evens out kerning and — its own
    // documentation says so — "lead to text looking more blurry". At the sizes a desktop
    // application actually uses, that trade is the wrong way round: a stem rendered at a
    // half-pixel offset is two grey columns instead of one dark one, and the whole window
    // reads soft next to a native one. Windows itself snaps glyph positions at these sizes
    // rather than sampling between them.
    //
    // Kerning is a fraction of a pixel worse over a long line. A file name is not a long
    // line, and nothing in this design system sets body text as paragraphs.
    v.text_options.subpixel_binning = false;

    // Azur disables by swapping in `text-disabled` rather than by fading, and
    // egui can only fade -- so this is the fade that lands nearest that token
    // over each theme's canvas. `components` paint the exact color instead.
    v.disabled_alpha = if t.dark { 0.32 } else { 0.45 };

    // ---- Selection -------------------------------------------------------
    // Read by four different widgets. `accent-default` is right for three of
    // them -- a slider's trailing fill, a progress bar, and a text-selection
    // highlight that has to stay legible -- and wrong only for a selected
    // `SelectableLabel`, where Azur wants `accent-subtle`. `widgets::list_items`
    // fixes that for the popups where it matters.
    v.selection.bg_fill = t.accent.default;
    // `selection.stroke` is two unrelated things in one field, and only one of them is a
    // stroke:
    //
    // - `text_selection::visuals` reads **`stroke.color` as the colour selected text is drawn
    //   in** — the ink that lands on `selection.bg_fill` above.
    // - `TextEdit` reads the whole stroke as the border of a *framed* field, which is Azur's
    //   `.fieldWrapper:focus-within { border-color: stroke-focus }`.
    //
    // This was `stroke.focus`, for the second of those, and that made the first illegible:
    // `stroke-focus` is an accent and `selection.bg_fill` is `accent-default`, so selected text
    // was accent on accent at **2.3:1** — measured off a screenshot, where a selected filename
    // read as a flat blue block. `text.on_accent` is the token the design system defines for ink
    // on `accent-default`, and `filled_controls_hold_their_label` already holds it to 4.5:1
    // there.
    //
    // The cost is the border: a *bare* framed `egui::TextEdit` now takes its focus ring from
    // `text-on_accent` rather than `stroke-focus`. Azur's own [`components::TextField`] is
    // unaffected — it wraps a frameless `TextEdit` and paints `stroke-focus` itself — so this
    // only shows up in an application reaching past the components for a plain framed field,
    // where one line puts it back. Illegible text is a defect and a focus ring is a preference,
    // so the field goes to the text.
    v.selection.stroke = Stroke::new(1.0, t.text.on_accent);

    // ---- Widget states ---------------------------------------------------
    //
    // egui splits a widget's background in two. `weak_bg_fill` is for the
    // optional kind -- a button, which reads as a raised surface. `bg_fill` is
    // for widgets that must have a fill: a slider rail, a checkbox box, a
    // scrollbar handle.
    //
    // Those do not want the same Azur token. `bg_fill`'s consumers are a slider
    // rail (`stroke-subtle`), a checkbox box (`background-control`) and a
    // scrollbar handle (`scrollbar-thumb`) -- three different tokens through one
    // field. It goes to the scrollbar, because a scrollbar is the only one of the
    // three egui paints that `components` cannot replace: `Slider`, `Checkbox`
    // and `Radio` here paint their own, to Azur's spec.
    // Every widget state below is a control — egui's buttons, text edits, combo
    // boxes — so they take the control radius rather than `radius-medium`, and match
    // the ones `components` paints. See
    // [`components::control_radius`](crate::components::control_radius).
    let round = crate::components::control_radius();

    // `noninteractive` is what a plain label and a `Frame::group` are drawn
    // with, so its foreground is body text -- `text-secondary` here would dim all
    // copy in the app.
    v.widgets.noninteractive.bg_fill = t.scrollbar.thumb;
    v.widgets.noninteractive.weak_bg_fill = t.bg.card;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.stroke.subtle);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.text.primary);
    v.widgets.noninteractive.corner_radius = round;

    v.widgets.inactive.bg_fill = t.scrollbar.thumb;
    v.widgets.inactive.weak_bg_fill = t.bg.control;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, t.stroke.control);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text.primary);
    v.widgets.inactive.corner_radius = round;

    v.widgets.hovered.bg_fill = t.scrollbar.thumb_hover;
    v.widgets.hovered.weak_bg_fill = t.bg.control_hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, t.stroke.strong);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, t.text.primary);
    v.widgets.hovered.corner_radius = round;
    // egui grows a hovered widget by default. Azur's controls hold their size
    // and change color, which also keeps a hovered entry from re-laying out the
    // popup it sits in.
    v.widgets.hovered.expansion = 0.0;

    // Pressed. `weak_bg_fill` is the button, so it takes `control-active`;
    // `bg_fill` is the furniture being dragged -- a slider handle -- so it takes
    // the accent, as Azur's does.
    v.widgets.active.bg_fill = t.accent.default;
    v.widgets.active.weak_bg_fill = t.bg.control_active;
    v.widgets.active.bg_stroke = Stroke::new(1.0, t.stroke.strong);
    v.widgets.active.fg_stroke = Stroke::new(1.0, t.text.primary);
    v.widgets.active.corner_radius = round;
    v.widgets.active.expansion = 0.0;

    // Open: a combo box or menu button whose popup is showing. Azur outlines the
    // trigger in `stroke-focus` for exactly this state.
    v.widgets.open.bg_fill = t.scrollbar.thumb_hover;
    v.widgets.open.weak_bg_fill = t.bg.control_hover;
    v.widgets.open.bg_stroke = Stroke::new(1.0, t.stroke.focus);
    v.widgets.open.fg_stroke = Stroke::new(1.0, t.text.primary);
    v.widgets.open.corner_radius = round;

    // ---- Controls --------------------------------------------------------
    // Azur's slider fills the track up to the value and rides a round handle.
    // egui does neither by default.
    v.slider_trailing_fill = true;
    v.handle_shape = egui::style::HandleShape::Circle;
    // Azur's buttons and menu entries are `cursor: pointer`.
    v.interact_cursor = Some(egui::CursorIcon::PointingHand);
    v.striped = false; // Azur's tables rule their rows rather than stripe them.

    v
}

/// Build [`egui::style::Spacing`] on Azur's 4px rhythm.
pub fn spacing(opts: &StyleOptions) -> egui::style::Spacing {
    let mut s = egui::style::Spacing {
        item_spacing: egui::vec2(space::S3, space::S3),
        button_padding: egui::vec2(space::S4, space::S2),
        // Azur's `Menu.list` is `padding: space-2; gap: space-1`. A desktop menu has
        // neither: entries meet, and they reach the border so a highlight spans the
        // row. The 2px is vertical only. Matches
        // [`components::menu_frame`](crate::components::menu_frame).
        menu_margin: Margin::symmetric(0, space::S1 as i8),
        menu_spacing: 0.0,
        window_margin: Margin::same(space::S4 as i8),
        // `TreeView` indents by `space-6` per level.
        indent: space::S6,
        // A 16px checkbox box, and the `space-3` gap Azur puts after it.
        icon_width: control::BOX_MEDIUM,
        icon_width_inner: 10.0,
        icon_spacing: space::S3,
        // `Slider.medium` rail is 6px.
        slider_rail_height: 6.0,
        // `Menu { max-height: 320px }`. The tooltip's own `max-width: 280px` is
        // the default of `StyleOptions::tooltip_width`, applied below.
        combo_height: 320.0,
        ..Default::default()
    };
    // Height only: egui also uses `interact_size.x` as a minimum width for combo
    // boxes and drag values, and zeroing it lets those collapse onto their text.
    s.interact_size.y = opts.control_height;
    s.tooltip_width = opts.tooltip_width;

    s.scroll.bar_width = opts.scroll_bar_width;
    s.scroll.floating = opts.floating_scroll_bar;
    // egui otherwise paints the scrollbar handle with `fg_stroke.color`, which in
    // this theme is the body text color: a near-white bar down the side of every
    // scroll area. Off means the handle takes `bg_fill` instead, which is where
    // `widgets::scrollbar` puts the scrollbar roles.
    s.scroll.foreground_color = false;
    // Azur's track is `background: transparent` in both themes; only the thumb
    // is painted.
    s.scroll.dormant_background_opacity = 0.0;
    s.scroll.active_background_opacity = 0.0;
    s.scroll.interact_background_opacity = 0.0;
    s.scroll.dormant_handle_opacity = 1.0;
    s.scroll.active_handle_opacity = 1.0;
    s.scroll.interact_handle_opacity = 1.0;
    s
}

/// Build a whole [`egui::Style`].
///
/// `Style` owns its own `visuals`, `spacing` and `text_styles`, so this exists to
/// make all of them land in one value: installing them separately means whichever
/// call comes second discards the first.
pub fn style(t: &Theme, opts: &StyleOptions) -> Style {
    let mut style = Style {
        visuals: visuals(t),
        spacing: spacing(opts),
        text_styles: text_styles(t),
        animation_time: opts.animation_time,
        ..Default::default()
    };
    style.interaction.tooltip_delay = opts.tooltip_delay;
    style.interaction.selectable_labels = opts.selectable_labels;

    // egui's rect/id heuristic is off, because in an application with more than one
    // view it is mostly wrong, and when it is wrong it paints over the product.
    //
    // `warn_if_rect_changes_id` draws a two-pixel `Color32::RED` border around any
    // widget that occupies a rectangle a *differently identified* widget held on the
    // previous pass, and it is on by default in every debug build. The reasoning is
    // sound — a widget whose id moves loses its state — but the test is positional,
    // so it also fires every time one view is replaced by another at the same
    // geometry: switching tabs puts a second table's scrollbar exactly where the
    // first one's was, and dropping a column as a window narrows leaves a different
    // column's resize handle on the vanished one's pixels. Both are correct code with
    // stable ids, and both flash red across the middle of the window.
    //
    // It is a real diagnostic, so it is worth switching back on deliberately when a
    // control is losing state — `ctx.style_mut(|s| s.debug.warn_if_rect_changes_id =
    // true)` after `install`, and read the `log::warn!` it emits alongside the
    // border, which names the rect and both ids.
    //
    // `show_unaligned` is deliberately left alone: an unaligned rect is a fact about
    // the geometry, it fires where the geometry is actually wrong, and this design
    // system cares about pixel alignment.
    //
    // Gated, because `Style::debug` is itself `#[cfg(debug_assertions)]` in egui — the
    // field does not exist in a release build, so an unguarded assignment compiles under
    // `cargo test` and fails under `cargo build --release`.
    #[cfg(debug_assertions)]
    {
        style.debug.warn_if_rect_changes_id = false;
    }

    style
}

/// Install a theme on a context, and pin egui to that side of the palette.
///
/// Replaces the `Style` for the theme's own side only, so a context can hold both
/// and follow the system. Call after any [`crate::fonts`] setup — that registers
/// the family [`crate::Fonts`] names for Azur's heavier weights.
pub fn install(ctx: &egui::Context, t: &Theme, opts: &StyleOptions) {
    ctx.set_style_of(t.kind(), style(t, opts));
    // So `components` can find the roles without every call site carrying them.
    crate::components::set_theme(ctx, t);
    // Azur is the product's palette, not the operating system's, so the
    // preference is pinned rather than left to follow.
    ctx.set_theme(match t.kind() {
        egui::Theme::Dark => egui::ThemePreference::Dark,
        egui::Theme::Light => egui::ThemePreference::Light,
    });
    pin_pixel_grid(ctx);
}

/// Hold text, lines and rectangles on the physical pixel grid.
///
/// Every one of these is already egui's default, and this sets them anyway. A design
/// system specified in one-pixel strokes and 14px body text has its whole appearance
/// resting on them: turn `round_text_to_pixels` off and every glyph in every Azur
/// window softens, and there would be nothing in this crate to say it had happened.
/// They are cheap to state and they are checked by a test, so an upstream default
/// that moves is a failing build rather than a window that quietly goes blurry.
///
/// Feathering stays *on*, which looks like the odd one out and is not: it is what
/// antialiases a rounded corner or a circle, it is fixed at one physical pixel, and
/// egui's own documentation notes it does not touch text at all. Widening it is what
/// blurs an edge, so the width is pinned rather than the feature disabled.
fn pin_pixel_grid(ctx: &egui::Context) {
    ctx.tessellation_options_mut(|o| {
        o.round_text_to_pixels = true;
        o.round_line_segments_to_pixels = true;
        o.round_rects_to_pixels = true;
        o.feathering = true;
        o.feathering_size_in_pixels = 1.0;
    });
}

/// Fonts, dark palette and default density in one call, for an app that wants
/// the theme as it ships. Returns it, since you need the same roles to paint
/// with.
pub fn install_dark(ctx: &egui::Context) -> Theme {
    crate::fonts::install(ctx);
    let t = Theme::dark();
    install(ctx, &t, &StyleOptions::default());
    t
}

/// The same, on the light side.
pub fn install_light(ctx: &egui::Context) -> Theme {
    crate::fonts::install(ctx);
    let t = Theme::light();
    install(ctx, &t, &StyleOptions::default());
    t
}

/// A [`FontId`] at one of Azur's sizes, in the theme's heavier family.
///
/// Small helper for the components, which need "this size but semibold" in a few
/// places the ramp does not name. For `font-weight-medium` use [`medium`] instead —
/// the two are not the same weight, however alike the CSS makes them look.
pub(crate) fn strong(size: f32) -> FontId {
    FontId::new(size, egui::FontFamily::Name(crate::fonts::SEMIBOLD.into()))
}

/// A [`FontId`] at one of Azur's sizes for a `font-weight-medium` role: a button
/// label, a tab, a field label, a badge.
///
/// Medium rounds **down** to regular. No UI family this port loads has a 500 face —
/// Segoe UI goes 400, then 600 — so 500 has to resolve to one of its neighbours,
/// and 600 at 14px on a dark surface reads as bold, which is more emphasis than
/// these roles are asking for. Rounding up made every button label in the app
/// heavier than its own body text.
pub(crate) fn medium(size: f32) -> FontId {
    FontId::proportional(size)
}

/// The line height Azur pairs with a font size, for anything allocating its own
/// row. Falls back to `1.4 × size` for a size that is not on the ramp.
pub fn line_height(size: f32) -> f32 {
    use typography::*;
    for (s, l) in [
        (SIZE_CAPTION, LINE_CAPTION),
        (SIZE_BODY, LINE_BODY),
        (SIZE_SUBTITLE, LINE_SUBTITLE),
        (SIZE_TITLE_SMALL, LINE_TITLE_SMALL),
        (SIZE_TITLE, LINE_TITLE),
        (SIZE_TITLE_LARGE, LINE_TITLE_LARGE),
        (SIZE_DISPLAY, LINE_DISPLAY),
    ] {
        if (size - s).abs() < 0.01 {
            return l;
        }
    }
    (size * 1.4).round()
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::epaint::FontColorTransferFunction;

    #[test]
    fn installing_holds_text_and_lines_on_the_pixel_grid() {
        // These are egui's defaults today. The test is here so that the day one of them
        // stops being the default, this fails instead of every Azur window going soft.
        let ctx = egui::Context::default();
        install(&ctx, &Theme::dark(), &StyleOptions::default());
        ctx.tessellation_options(|o| {
            assert!(o.round_text_to_pixels, "glyphs must land on whole pixels");
            assert!(
                o.round_line_segments_to_pixels,
                "a 1px rule must land on one row, not across two"
            );
            assert!(
                o.round_rects_to_pixels,
                "panel fills must tile without a seam of their own"
            );
            // Feathering is what antialiases a rounded corner, and per epaint's own
            // documentation it does not affect text. One physical pixel is the width
            // that antialiases without blurring; anything wider is a soft edge.
            assert!(o.feathering, "corners and circles still need antialiasing");
            assert_eq!(o.feathering_size_in_pixels, 1.0);
        });
    }

    #[test]
    fn text_weighs_the_same_in_both_palettes() {
        // egui's own defaults differ per side, so this is not something to leave alone:
        // inherit them and the text visibly reweights when the theme is toggled, which
        // reads as the font having changed.
        let dark = visuals(&Theme::dark()).text_options.color_transfer_function;
        let light = visuals(&Theme::light())
            .text_options
            .color_transfer_function;
        assert_eq!(dark, light, "text is rendered at two different weights");
        assert_eq!(
            dark,
            FontColorTransferFunction::Off,
            "glyph coverage should reach the atlas unmodified -- a boost draws Azur's \
             regular weight heavier than it is"
        );
    }

    /// Selected text has to be readable on the selection.
    ///
    /// `selection.stroke.color` is not only a stroke: `text_selection::visuals` uses it as the
    /// colour of selected text, on `selection.bg_fill`. Pointed at `stroke-focus` — for the other
    /// thing egui reads that field for, a framed `TextEdit`'s focus border — it was an accent on
    /// an accent, and a selected filename in a rename field came out as a flat blue block with
    /// the name invisible inside it. Nothing in either crate noticed, because nothing was looking
    /// at the two halves of the pair together. This does.
    #[test]
    fn selected_text_reads_on_the_selection_behind_it() {
        use crate::contrast::{ratio as contrast, TEXT};

        for t in [Theme::dark(), Theme::light()] {
            let side = if t.dark { "dark" } else { "light" };
            let v = visuals(&t);
            let got = contrast(v.selection.stroke.color, v.selection.bg_fill);
            assert!(
                got >= TEXT,
                "{side}: selected text is {got:.2}:1 against the selection behind it \
                 ({:?} on {:?}) -- a selected filename would read as a coloured block",
                v.selection.stroke.color,
                v.selection.bg_fill
            );
            // And it must not be transparent, which is the other way to lose the text and the
            // one an application reaches for when it wants the focus border gone.
            assert!(
                v.selection.stroke.color.a() == 255,
                "{side}: selected text is drawn in something translucent"
            );
        }
    }

    #[test]
    fn glyphs_are_rasterised_once_at_a_whole_pixel() {
        // The other half of "why does this look soft next to a native window". egui's
        // default renders each glyph at four fractional offsets, which its own docs say
        // looks blurrier; both palettes opt out.
        for t in [Theme::dark(), Theme::light()] {
            assert!(
                !visuals(&t).text_options.subpixel_binning,
                "subpixel binning trades sharpness for kerning, which is the wrong way \
                 round at UI sizes"
            );
            // And hinting stays on, which is the other half of the same bargain.
            assert!(visuals(&t).text_options.font_hinting);
        }
    }
}
