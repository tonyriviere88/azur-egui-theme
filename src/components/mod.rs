//! Azur's components, as egui widgets.
//!
//! [`mod@crate::style`] makes egui's own widgets wear the token set, which is enough
//! for a button or a text field. It is not enough for the rest: a checked Azur
//! checkbox is an accent-filled box, egui's is a checkmark on the control fill;
//! Azur's selected list row is `accent-subtle` with a 2px accent bar, egui has no
//! list rows at all; `MessageBar`, `Badge`, `Toast`, `Avatar`, `Breadcrumb`,
//! `NavigationRail` and `TitleBar` have no egui counterpart to restyle.
//!
//! So these paint themselves, from the roles in [`crate::Theme`] and the scales
//! in [`crate::tokens`] — every fill, line, radius, height and font is the token
//! the matching `*.module.css` names, and where a component deviates the comment
//! says which line of CSS it is deviating from and why.
//!
//! # Getting the theme
//!
//! Components read the theme out of the [`egui::Context`], which
//! [`crate::install`] puts there — so a call site does not have to carry it:
//!
//! ```no_run
//! # fn demo(ctx: &egui::Context, ui: &mut egui::Ui) {
//! use azur_egui_theme::components::{Button, Variant};
//! azur_egui_theme::install_dark(ctx);
//! if ui.add(Button::new("Save").variant(Variant::Primary)).clicked() {
//!     // ...
//! }
//! # }
//! ```
//!
//! If nothing was installed, [`theme`] falls back to the side egui is currently
//! on, so a component never draws with the wrong palette — only, possibly, with
//! an unstyled egui around it.
//!
//! # What is not ported
//!
//! Azur's transitions and keyframe animations. egui cross-fades widget state on
//! [`egui::Style::animation_time`], which the installed style sets to
//! [`crate::tokens::motion::FAST`], and the components that genuinely animate —
//! `Spinner`, an indeterminate `ProgressBar`, `Toast` — drive themselves off
//! [`egui::InputState::time`]. Nothing here interpolates a color per property the
//! way CSS does.

use crate::theme::{Fonts, Theme};
use crate::tokens::{control, space, typography};
use egui::{Color32, FontId, Id, Rect, Ui};
use std::sync::Arc;

mod actions;
mod chrome;
mod collections;
mod feedback;
mod inputs;
mod menus;
mod overlays;
mod selection;

pub use actions::{Button, Card, CardVariant, Divider, IconButton, Padding};
pub use chrome::{
    chrome_edge, resize_handles, Avatar, AvatarShape, AvatarStatus, Breadcrumb, CommandBar,
    CommandItem, NavItem, NavigationRail, Text, TextColor, TextVariant, TitleBar, TitleBarAction,
};
pub use collections::{
    collection_label, collection_outline, tabs_height, tabs_label_band, tabs_width, ListRow,
    ListView, Sort, Table, TableColumn, Tabs, TreeNode, TreeNodeState, TreeView,
};
pub use feedback::{
    Badge, BadgeVariant, MessageBar, ProgressBar, ProgressColor, Spinner, Toast, Toaster,
    ToasterPosition,
};
pub use inputs::{
    field_frame, field_hovered, FieldLook, NumberField, SearchBox, TextArea, TextField,
};
pub use menus::{
    menu_divider, menu_divider_height, menu_frame, menu_item_height, menu_item_height_comfortable,
    menu_item_metrics, popover_frame, submenu, ComboBox, ContextMenu, Menu, MenuItem,
    MenuItemMetrics, Popover, Select, SelectOption,
};
pub use overlays::{
    section_heading, shortcut_in, tooltip, tooltip_at_pointer, tooltip_at_pointer_ui, Dialog,
    DialogSize, Drawer, DrawerSide, CURSOR_CLEARANCE,
};
pub use selection::{Checkbox, Radio, RadioGroup, Slider, Switch, SwitchLabel};

/// The `Id` the installed [`Theme`] is parked under in [`egui::Context::data`].
fn theme_id() -> Id {
    Id::new("azur-egui-theme::theme")
}

/// The theme the components draw with.
///
/// [`crate::install`] stores it; this reads it back. An `Arc` rather than a
/// clone because a table asks for it once per row per frame, and `Fonts` holds
/// nine `FontId`s whose family names are themselves refcounted.
pub fn theme(ctx: &egui::Context) -> Arc<Theme> {
    if let Some(t) = ctx.data(|d| d.get_temp::<Arc<Theme>>(theme_id())) {
        return t;
    }
    // Nothing installed. Follow whichever side egui is on and remember it, so a
    // component in an otherwise unthemed app still draws in-palette.
    let t = Arc::new(Theme::of(ctx.theme()));
    ctx.data_mut(|d| d.insert_temp(theme_id(), t.clone()));
    t
}

/// Park a theme where [`theme`] will find it. Called by [`crate::install`].
pub(crate) fn set_theme(ctx: &egui::Context, t: &Theme) {
    ctx.data_mut(|d| d.insert_temp(theme_id(), Arc::new(t.clone())));
}

/// Shorthand for the components' first line.
pub(crate) fn theme_of(ui: &Ui) -> Arc<Theme> {
    theme(ui.ctx())
}

/// Azur's three control sizes.
///
/// One scale, not one per component: `Button`, `IconButton`, `TextField`,
/// `TextArea`, `NumberField`, `SearchBox` and `Select` all size themselves from
/// it, and all agree on 24 / 32 / 40px.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Size {
    /// 24px.
    Small,
    /// 32px — Azur's default.
    #[default]
    Medium,
    /// 40px.
    Large,
}

impl Size {
    /// The control's height.
    pub fn height(self) -> f32 {
        match self {
            Self::Small => control::SMALL,
            Self::Medium => control::MEDIUM,
            Self::Large => control::LARGE,
        }
    }

    /// Horizontal padding: `space-3` / `space-4` / `space-5`.
    pub fn padding_x(self) -> f32 {
        match self {
            Self::Small => space::S3,
            Self::Medium => space::S4,
            Self::Large => space::S5,
        }
    }

    /// The type step a control of this size labels itself with: caption / body /
    /// subtitle.
    pub fn font(self, f: &Fonts) -> FontId {
        match self {
            Self::Small => f.caption.clone(),
            Self::Medium => f.body.clone(),
            Self::Large => f.subtitle.clone(),
        }
    }

    /// Font size in px, which is also the icon box inside a `Button`
    /// (`.icon svg { width: 1em }`).
    pub fn font_size(self) -> f32 {
        match self {
            Self::Small => typography::SIZE_CAPTION,
            Self::Medium => typography::SIZE_BODY,
            Self::Large => typography::SIZE_SUBTITLE,
        }
    }

    /// An `IconButton`'s glyph, which runs one step of the ramp hotter than a
    /// `Button`'s: body / subtitle / title-small.
    pub fn icon_only(self) -> f32 {
        match self {
            Self::Small => typography::SIZE_BODY,
            Self::Medium => typography::SIZE_SUBTITLE,
            Self::Large => typography::SIZE_TITLE_SMALL,
        }
    }

    /// A checkbox box or a radio dot: 14 / 16 / 20px.
    pub fn box_size(self) -> f32 {
        match self {
            Self::Small => control::BOX_SMALL,
            Self::Medium => control::BOX_MEDIUM,
            Self::Large => control::BOX_LARGE,
        }
    }

    /// A switch track: `(width, height)`.
    pub fn switch(self) -> (f32, f32) {
        match self {
            Self::Small => control::SWITCH_SMALL,
            Self::Medium => control::SWITCH_MEDIUM,
            Self::Large => control::SWITCH_LARGE,
        }
    }
}

/// The five `Button` / `IconButton` variants.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Variant {
    /// Accent-filled. One per view.
    Primary,
    /// The control fill with a border — the everyday button.
    #[default]
    Secondary,
    /// No fill and no border until hovered.
    Subtle,
    /// A border, no fill.
    Outline,
    /// A destructive action.
    Danger,
}

/// The four status intents, shared by `MessageBar`, `Toast` and `Badge`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Intent {
    #[default]
    Info,
    Success,
    Warning,
    Danger,
}

impl Intent {
    /// The solid color: the icon, and the 3px bar down a `MessageBar`.
    pub fn color(self, t: &Theme) -> Color32 {
        match self {
            Self::Info => t.status.info,
            Self::Success => t.status.success,
            Self::Warning => t.status.warning,
            Self::Danger => t.status.danger,
        }
    }

    /// The translucent fill behind it.
    pub fn subtle(self, t: &Theme) -> Color32 {
        match self {
            Self::Info => t.status.info_subtle,
            Self::Success => t.status.success_subtle,
            Self::Warning => t.status.warning_subtle,
            Self::Danger => t.status.danger_subtle,
        }
    }

    /// The glyph Azur pairs with the intent.
    pub fn icon(self) -> crate::icons::Icon<'static> {
        match self {
            Self::Info => &crate::icons::info,
            Self::Success => &crate::icons::success,
            Self::Warning => &crate::icons::warning,
            Self::Danger => &crate::icons::error,
        }
    }
}

// ---------------------------------------------------------------------------
// Shared painting helpers
// ---------------------------------------------------------------------------

/// Multiply a color's channels without touching its alpha.
///
/// [`Color32::gamma_multiply`] scales the premultiplied components, alpha
/// included, so it fades rather than darkens. Azur's one use of `filter:
/// brightness()` — a pressed danger button — wants the darkening.
pub(crate) fn darken(c: Color32, f: f32) -> Color32 {
    let s = |v: u8| (v as f32 * f).clamp(0.0, 255.0) as u8;
    Color32::from_rgba_unmultiplied(s(c.r()), s(c.g()), s(c.b()), c.a())
}

/// The border color of a field-shaped control, from its state.
///
/// Shared by `TextField`, `TextArea`, `NumberField`, `SearchBox` and `Select`,
/// which all spell out the same four rules in CSS.
pub(crate) fn field_stroke(
    t: &Theme,
    enabled: bool,
    focused: bool,
    hovered: bool,
    error: bool,
) -> Color32 {
    if !enabled {
        t.stroke.subtle
    } else if error {
        t.status.danger
    } else if focused {
        t.stroke.focus
    } else if hovered {
        t.stroke.strong
    } else {
        t.stroke.control
    }
}

/// Lay a single line of text out, cut to `width` with an ellipsis.
pub(crate) fn truncated(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    width: f32,
) -> Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::single_section(
        text.to_owned(),
        egui::TextFormat::simple(font, color),
    );
    job.wrap = egui::text::TextWrapping::truncate_at_width(width.max(0.0));
    ui.painter().layout_job(job)
}

/// Paint a galley vertically centred in `rect`, `x` at its left edge.
///
/// Only for a row holding *one* piece of text and no glyph. Centring a galley's box
/// is the same thing as putting its own font's baseline in the middle, so the two
/// agree — but two galleys in different fonts, each centred that way, land on two
/// different baselines. A row with more than one text uses [`row_baseline`]; a row
/// with an icon in it uses [`ink_baseline`], because a line box centred beside a
/// glyph centred reads low.
pub(crate) fn galley_left_center(ui: &Ui, rect: Rect, galley: Arc<egui::Galley>) {
    let pos = egui::pos2(rect.left(), rect.center().y - galley.size().y * 0.5);
    ui.painter().galley(pos, galley, Color32::PLACEHOLDER);
}

/// Distance from a galley's top edge down to its first line's text baseline.
///
/// Which is the font's ascent, but read off the galley rather than asked for: egui
/// exposes ascent only through a laid-out glyph.
pub fn galley_baseline(galley: &egui::Galley) -> f32 {
    galley.rows.first().map_or(0.0, |r| {
        // `Glyph::pos` is the baseline position, relative to its row.
        r.pos.y + r.glyphs.first().map_or(r.size.y, |g| g.pos.y)
    })
}

/// The one baseline every text on a line sits on, for a line `height` tall starting
/// at `top`, whose principal font is `font`.
///
/// A line of text is rarely one font: a name in body beside a number in mono, a
/// label in caption beside a reading in body-strong, a menu entry beside its
/// shortcut. Each font has its own ascent and its own line height, so centring or
/// topping each galley's *box* — which is what `Align2` does — leaves their
/// baselines a pixel or two apart, and the text visibly steps up and down as the eye
/// crosses it. So a line commits to a single baseline instead: the one `font` gets
/// when its line box is centred. Text in the principal font looks exactly as it did;
/// everything else comes into line with it.
///
/// Pair with [`galley_on_baseline`], which is the other half of the rule.
/// `top` may be zero, which gives the baseline as an offset into a line — useful
/// where the line's position is only known per row, as in a scrolled table.
///
/// For a row that holds an **icon** as well as text, use [`ink_baseline`] instead:
/// centring the line box is what makes text hang low next to a glyph.
pub fn row_baseline(painter: &egui::Painter, font: &FontId, top: f32, height: f32) -> f32 {
    use egui::emath::GuiRounding as _;

    // Any string with an ascender and a descender: the probe is only measured, never
    // drawn, and egui caches the layout so this costs a hash lookup.
    let probe = painter.layout_no_wrap("Ay".to_owned(), font.clone(), Color32::PLACEHOLDER);
    let y = top + (height - probe.size().y) * 0.5 + galley_baseline(&probe);
    // Snapped to the pixel grid, because a shared baseline is not enough on its own:
    // epaint rounds each galley's *origin* to a whole physical pixel, so a baseline
    // landing halfway between two pixels is rounded down for a font whose ascent is
    // 14.0 and up for one whose ascent is 13.4, and the two miss each other by a
    // pixel after all. On the grid, both round back onto it.
    y.round_to_pixels(painter.ctx().pixels_per_point())
}

/// Paint a galley with its first line's baseline on `baseline` and its left edge at
/// `x`. The galley's own colour is used, so lay it out with the colour it wants.
pub fn galley_on_baseline(
    painter: &egui::Painter,
    x: f32,
    baseline: f32,
    galley: Arc<egui::Galley>,
) {
    let y = baseline - galley_baseline(&galley);
    painter.galley(egui::pos2(x, y), galley, Color32::PLACEHOLDER);
}

/// The one baseline for a row that also holds a **glyph**, and so has to be level
/// with something centred on its own ink.
///
/// [`row_baseline`] centres the line *box*, which is right for a row of nothing but
/// text and wrong the moment an icon appears beside it. A line box reserves room
/// under the baseline for descenders and room above the capitals for accents, and a
/// file name uses neither — so text centred that way hangs below the middle of the
/// row while the icon sits on it. Measured on Segoe UI in a 24px row, at
/// `pixels_per_point` 1:
///
/// | | 12px | 14px | 16px |
/// | --- | --- | --- | --- |
/// | line box height | 16.00 | 19.00 | 21.00 |
/// | ascent | 13.00 | 16.00 | 17.00 |
/// | ink above the baseline | 9.00 | 10.00 | 11.00 |
/// | **the ink sits this far low** | **0.50** | **1.50** | **1.00** |
///
/// So this centres the band from the top of the ascenders down to the baseline, and
/// lets the descenders hang out of it, which is what a typographer means by the
/// middle of a line and what the eye reads as level.
///
/// The application this was drawn out of found the same number three times by eye —
/// a listing's cells, a path segment, a tab's label — and wrote 2.0 in three
/// different files, having tried nothing and noticed nothing until a guide drawn
/// through an icon's centre came out above the x-height of every name in the window.
/// 1.5 is the measured figure at body size, and after the pixel snap below the two
/// land on the same row anyway.
///
/// Take it *per row, from the row's principal font*, exactly as with
/// [`row_baseline`]: every text on the line then shares one baseline, whatever font
/// or size each one is in. A 12px metadata column given its own ink baseline would
/// sit a point above the 14px name beside it, which is the fault this fixes reappearing
/// sideways.
pub fn ink_baseline(painter: &egui::Painter, font: &FontId, top: f32, height: f32) -> f32 {
    use egui::emath::GuiRounding as _;

    let probe = probe(painter, font);
    let y = top + height * 0.5 + ink_ascent(&probe) * 0.5;
    y.round_to_pixels(painter.ctx().pixels_per_point())
}

/// How far above a line box's middle the ink of a line of text in `font` actually
/// sits — the bottom row of the table in [`ink_baseline`].
///
/// [`ink_baseline`] is the better answer. This is for a caller that already centres
/// something in a rect and wants to keep doing that: lift the rect by this and every
/// text centred in it comes level with a glyph centred in the row. It is a property
/// of the font alone, so one value serves a whole row of columns.
pub fn ink_lift(painter: &egui::Painter, font: &FontId) -> f32 {
    let probe = probe(painter, font);
    galley_baseline(&probe) - probe.size().y * 0.5 - ink_ascent(&probe) * 0.5
}

/// The layout both baselines are measured from: a capital and a descender, never
/// drawn. egui caches it, so this costs a hash lookup after the first row.
fn probe(painter: &egui::Painter, font: &FontId) -> Arc<egui::Galley> {
    painter.layout_no_wrap("Ay".to_owned(), font.clone(), Color32::PLACEHOLDER)
}

/// How far the ink of a laid-out line reaches *above* its baseline.
///
/// Read off the rasterised glyphs rather than from the font's ascent, which on a UI
/// face is well above the capitals: Segoe UI at 14px has an ascent of 16.0 and puts
/// no ink higher than 10.0 above the baseline. The empty 6 points are the whole
/// reason line-box centring reads low.
fn ink_ascent(galley: &egui::Galley) -> f32 {
    let baseline = galley_baseline(galley);
    let top = galley
        .rows
        .first()
        .into_iter()
        .flat_map(|row| {
            row.glyphs
                .iter()
                .filter(|g| !g.uv_rect.is_nothing())
                .map(move |g| row.pos.y + g.pos.y + g.uv_rect.offset.y)
        })
        .fold(f32::INFINITY, f32::min);
    // Nothing rasterised — a font with no glyph for either probe character. Fall back
    // to the line box, which is what every other helper here would have used.
    if top.is_finite() {
        baseline - top
    } else {
        galley.size().y
    }
}

/// How deep an edge fade goes, at most. See [`scroll_fades`].
///
/// Sixteen points: deep enough that the row going under the edge is visibly on its way
/// out, and short enough that a 24px row is never wholly consumed by it.
pub const SCROLL_FADE: f32 = 16.0;

/// **A scrolling collection says it has more in it at the edge, where the eye already is.**
///
/// A list, table or log cut off mid-row looks like one that ends there, and the scrollbar
/// that says otherwise is a thin thing down the far side of a panel nobody is looking at
/// the far side of. So the surface comes back over the content at whichever edge there is
/// more past — a fade rather than a rule or a shadow, because the content stays legible
/// through the shallow end of it, which is what reads as *continues* instead of as *ends*.
///
/// This is the rule for every scrolling collection: [`ListView`], [`TreeView`] and
/// [`Table`] paint it themselves, and it is public for the ones an application paints by
/// hand — a file listing, a document, a console log.
///
/// - `surface` is the fill the region sits on. Not a new colour: this is that surface
///   returning over its own content, so it survives a theme switch and a moved role.
/// - `above` and `below` are how much is scrolled past each edge in points, and each fade
///   is that deep up to [`SCROLL_FADE`] — so it grows in as the first row goes under
///   rather than snapping on at full depth.
/// - `page` is what is readable, **scrollbars excluded**: a fade drawn over a scrollbar
///   makes the bar look like it is dissolving. Inside a `ScrollArea`'s closure that is
///   `ui.clip_rect()`; afterwards it is `ScrollAreaOutput::inner_rect`.
///
/// [`ListView`]: crate::components::ListView
/// [`TreeView`]: crate::components::TreeView
/// [`Table`]: crate::components::Table
pub fn scroll_fades(painter: &egui::Painter, page: Rect, surface: Color32, above: f32, below: f32) {
    if above > 0.5 {
        edge_fade(
            painter,
            Rect::from_min_max(
                page.left_top(),
                egui::pos2(page.right(), page.top() + above.min(SCROLL_FADE)),
            ),
            surface,
            true,
        );
    }
    if below > 0.5 {
        edge_fade(
            painter,
            Rect::from_min_max(
                egui::pos2(page.left(), page.bottom() - below.min(SCROLL_FADE)),
                page.right_bottom(),
            ),
            surface,
            false,
        );
    }
}

/// [`scroll_fades`] measured off a finished [`egui::ScrollArea`] — the whole of it for a
/// region whose content is all there is and which scrolls to the end of that content.
///
/// A caller that reserves trailing slack past its last row, or that measures its own
/// extent, wants [`scroll_fades`] and its own two numbers: a fade at the bottom for slack
/// nobody put anything in says there is more to read when there is not.
pub fn scroll_fades_of<T>(
    painter: &egui::Painter,
    out: &egui::scroll_area::ScrollAreaOutput<T>,
    surface: Color32,
) {
    let reach = (out.content_size.y - out.inner_rect.height()).max(0.0);
    let above = out.state.offset.y.clamp(0.0, reach);
    scroll_fades(painter, out.inner_rect, surface, above, reach - above);
}

/// One edge of [`scroll_fades`]: `surface` fading to nothing, down the rect or up it.
///
/// Four vertices and two triangles. [`egui::Painter`] has no gradient, and a stack of
/// sixteen one-point rects at stepped alphas is sixteen shapes and a visible banding for
/// the same effect.
fn edge_fade(painter: &egui::Painter, rect: Rect, surface: Color32, down: bool) {
    // Premultiplied, so fading to `TRANSPARENT` fades the colour *out* rather than towards
    // black — the interpolation the GPU does between these two is already the right one.
    let (from, to) = if down {
        (surface, Color32::TRANSPARENT)
    } else {
        (Color32::TRANSPARENT, surface)
    };
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), from);
    mesh.colored_vertex(rect.right_top(), from);
    mesh.colored_vertex(rect.left_bottom(), to);
    mesh.colored_vertex(rect.right_bottom(), to);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(2, 1, 3);
    painter.add(egui::Shape::mesh(mesh));
}

/// The spinning three-quarter ring Azur builds out of a bordered circle with one
/// side transparent.
///
/// `period` is a full turn in seconds — 0.8s for `Spinner`, 0.4s
/// (`duration-slower`) for the one inside a loading `Button`. Repaints itself,
/// since egui only redraws on demand.
pub(crate) fn spinner_arc(ui: &Ui, rect: Rect, color: Color32, period: f32) {
    let width = (rect.width() / 8.0).clamp(2.0, 3.0);
    let radius = (rect.width() - width) * 0.5;
    let time = ui.input(|i| i.time) as f32;
    let start = (time / period) * std::f32::consts::TAU;
    let sweep = std::f32::consts::TAU * 0.75;

    let steps = 24;
    let points: Vec<_> = (0..=steps)
        .map(|i| {
            let a = start + sweep * i as f32 / steps as f32;
            rect.center() + egui::Vec2::new(a.cos(), a.sin()) * radius
        })
        .collect();
    ui.painter()
        .add(egui::Shape::line(points, egui::Stroke::new(width, color)));
    ui.ctx().request_repaint();
}

/// The icon box Azur puts inside a control of this size: 14 / 16 / 20px.
pub(crate) fn control_icon(size: Size) -> f32 {
    match size {
        Size::Small => control::ICON_SMALL,
        Size::Medium => control::ICON_MEDIUM,
        Size::Large => control::ICON_LARGE,
    }
}

/// The corner radius of a *control*: a button, an icon button, a text field, a
/// `Select` or `ComboBox` trigger.
///
/// Azur says `radius-medium` (6px) for every one of them. This is `radius-small`
/// (4px), one step down the same scale, for the reason [`menu_frame`] is square:
/// 6px on a 24px-tall control reads as a pill, and a desktop control reads as a
/// rectangle with the corners taken off.
///
/// It stops at 4 rather than going to zero — which is the literal pre-11 Windows
/// control, and one line away here if you want it — because a small radius still
/// separates the things you can click from the flat surfaces they sit on, and the
/// menus are already carrying the square end of that contrast.
///
/// Surfaces are not controls and keep their own tokens: a `Card` and a `Dialog` stay
/// `radius-large`, a `MessageBar` and a `NavigationRail` entry stay `radius-medium`,
/// and the things that are round by definition — a `Switch`, a `Badge`, an `Avatar` —
/// are untouched.
pub fn control_radius() -> egui::CornerRadius {
    egui::CornerRadius::same(crate::tokens::radius::SMALL)
}

/// The square an icon of `size` occupies, vertically centred at `x`.
pub(crate) fn icon_rect(rect: Rect, x: f32, size: f32) -> Rect {
    Rect::from_min_size(
        egui::pos2(x, rect.center().y - size * 0.5),
        egui::Vec2::splat(size),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run one pass, so there are fonts to measure. [`egui::Context`] has none until
    /// it has run, and a painter's `layout_no_wrap` is what rasterises the probe.
    fn painted(check: impl FnOnce(&egui::Painter)) {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        // `run_ui` takes an `FnMut` and a check that moves is `FnOnce`, so it is taken
        // out of the option on the first pass. Its absence afterwards is also the
        // proof that the pass ran at all.
        let mut check = Some(check);
        let _ = ctx.run_ui(Default::default(), |ui| {
            if let Some(check) = check.take() {
                check(ui.painter());
            }
        });
        assert!(check.is_none(), "the pass never ran, so nothing was checked");
    }

    /// The alignment rule as arithmetic: the ink of a line sits below the middle of
    /// its own line box, by a distance that depends only on the font.
    #[test]
    fn a_line_of_text_sits_low_in_its_own_box() {
        painted(|painter| {
            for (size, expected) in [(12.0, 0.5), (14.0, 1.5), (16.0, 1.0)] {
                let font = FontId::proportional(size);
                let lift = ink_lift(painter, &font);
                // A system font might not be Segoe UI — on a machine where it is, these
                // are the measured figures; anywhere else, the sign and the magnitude
                // are still the claim being made.
                assert!(
                    (0.0..=3.0).contains(&lift),
                    "{size}px: the ink is {lift:.2} off the middle of the line box"
                );
                if cfg!(windows) {
                    assert!(
                        (lift - expected).abs() < 0.01,
                        "{size}px: measured {lift:.2}, Segoe UI gives {expected:.2}"
                    );
                }
            }
        });
    }

    /// And the baseline it produces puts that ink in the middle of the row, which is
    /// the whole point: it is what an icon centred in the same row is doing.
    #[test]
    fn an_ink_baseline_centres_the_ink_in_the_row() {
        painted(|painter| {
            let font = FontId::proportional(14.0);
            let row = Rect::from_min_size(egui::pos2(0.0, 100.0), egui::vec2(200.0, 24.0));
            let baseline = ink_baseline(painter, &font, row.top(), row.height());
            let probe = probe(painter, &font);
            let ink_middle = baseline - ink_ascent(&probe) * 0.5;
            assert!(
                (ink_middle - row.center().y).abs() <= 0.5,
                "the ink's middle is {:+.2} off the row's",
                ink_middle - row.center().y
            );
            // Above the line-box baseline by the lift, give or take the pixel snap both
            // of them end with.
            let boxed = row_baseline(painter, &font, row.top(), row.height());
            let lift = ink_lift(painter, &font);
            assert!(
                (boxed - baseline - lift).abs() <= 1.0,
                "box baseline {boxed}, ink baseline {baseline}, lift {lift}"
            );
        });
    }

    /// Every mesh a pass painted, with the rect it covers — which is all a fade is.
    fn meshes(above: f32, below: f32) -> Vec<Rect> {
        let ctx = egui::Context::default();
        let page = Rect::from_min_size(egui::pos2(0.0, 100.0), egui::vec2(200.0, 300.0));
        let out = ctx.run_ui(Default::default(), |ui| {
            scroll_fades(ui.painter(), page, Color32::from_gray(20), above, below);
        });
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Mesh(mesh) => Some(mesh.calc_bounds()),
                _ => None,
            })
            .collect()
    }

    /// A fade per edge there is something past, and none for an edge there is not: the whole of
    /// what the rule says, as rects.
    #[test]
    fn an_edge_fades_only_when_there_is_more_past_it() {
        assert!(
            meshes(0.0, 0.0).is_empty(),
            "a region with nothing to scroll wears nothing"
        );

        let top = meshes(40.0, 0.0);
        assert_eq!(top.len(), 1, "the top edge only");
        assert_eq!(top[0].top(), 100.0);
        assert_eq!(top[0].height(), SCROLL_FADE, "as deep as it goes");

        let both = meshes(40.0, 40.0);
        assert_eq!(both.len(), 2);
        assert_eq!(both[1].bottom(), 400.0, "and the second is at the foot");

        // Short of the full depth while there is less than that left to scroll, so it grows in as
        // the first row goes under rather than snapping on.
        let barely = meshes(4.0, 0.0);
        assert_eq!(barely[0].height(), 4.0);
    }
}
