//! `Popover`, `Menu` / `MenuItem`, `ContextMenu`, `Select`, `ComboBox`.
//!
//! These float on one paint — `background-layer-alt`, a `stroke-subtle` border,
//! `shadow-16` — in two shapes: [`popover_frame`] is Azur's rounded, padded surface,
//! and [`menu_frame`] is the square, edge-to-edge one a desktop menu wants. Both are
//! what [`crate::style`] installs as egui's `window_fill` / `window_stroke` /
//! `menu_corner_radius` / `menu_margin`, so `ui.menu_button`, a nested [`submenu`]
//! and a `Popover` all agree without anything extra.

use super::{field_stroke, theme_of, truncated, Size, Variant};
use crate::icons::{caret_right, check, chevron_down, chevron_up, focus_ring, Icon};
use crate::theme::Theme;
use crate::tokens::{radius, row, shadow, space, typography};
use egui::{
    pos2, vec2, Color32, CornerRadius, Frame, InnerResponse, Margin, Rect, Response, Sense, Shape,
    Stroke, StrokeKind, Ui, Widget,
};
use std::borrow::Cow;

/// The frame every Azur overlay that floats *near* something uses: popover, menu,
/// listbox, context menu.
///
/// `Dialog`, `Drawer` and `Toast` are panels rather than popovers and carry
/// different tokens; they build their own.
pub fn popover_frame(t: &Theme) -> Frame {
    Frame::new()
        .fill(t.bg.layer_alt)
        .stroke(Stroke::new(1.0, t.stroke.subtle))
        .corner_radius(CornerRadius::same(radius::LARGE))
        .shadow(shadow::S16)
        // `.list { padding: space-2 }`.
        .inner_margin(Margin::same(space::S2 as i8))
}

/// The surface a *menu* uses: `Menu`, `ContextMenu`, a submenu, and the listbox of a
/// `Select` or `ComboBox`.
///
/// Same paint as [`popover_frame`] — `background-layer-alt`, a `stroke-subtle`
/// border, `shadow-16` — and a different shape, because a menu is a desktop idiom
/// rather than a web one:
///
/// - **Square corners.** Azur's `radius-large` is the Windows 11 look; a menu before
///   it was a rectangle, and the corner is the single loudest signal of which era a
///   menu belongs to.
/// - **No horizontal padding.** Rows run from border to border, so a highlighted
///   entry is a full-width band rather than a floating pill. This is what
///   [`MenuItem`] squares its own corners to match, and what
///   [`menu_divider`] spans.
///
/// The 2px left is vertical only: the breathing room a classic menu keeps between
/// its frame and its first and last entry.
///
/// [`mod@crate::style`] gives egui's own `Frame::menu` the same shape, so a
/// [`submenu`] — which is egui's machinery, not this crate's — comes out identical to
/// the menu it grew from.
pub fn menu_frame(t: &Theme) -> Frame {
    Frame::new()
        .fill(t.bg.layer_alt)
        .stroke(Stroke::new(1.0, t.stroke.subtle))
        .corner_radius(crate::desktop::menu_radius(t).0)
        .shadow(shadow::S16)
        .inner_margin(Margin::symmetric(0, space::S1 as i8))
}

/// `.listbox { max-height: 280px }` — how far a `Select` or `ComboBox` list scrolls before it
/// scrolls.
///
/// Named because it is now needed twice at each call site: once as the scroll area's ceiling, and
/// once as the room the surrounding `Ui` has to be told it may use. The two drifting apart is the
/// bug — a scroll area given less room than its own `max_height` is a list that cannot grow. See
/// [`Menu::show`].
const LISTBOX_HEIGHT: f32 = 280.0;

/// The row height of a menu entry or a listbox option: 28px.
///
/// This is the one deliberate density change against `Menu.module.css`, which puts
/// `space-3` either side of a body line for a 36px row. 36px is comfortable for a
/// pointer on a page; on a desktop menu — where entries come in tens, the pointer
/// is already resting on them, and the whole list has to fit under a toolbar — it
/// reads as padded out.
///
/// The number is still the design system's: dropping the vertical padding one step
/// to `space-2` gives exactly [`row::COMPACT`], the height Azur's own `ListView`
/// uses at `density="compact"`. Set [`MenuItem::height`] to put the web value back.
pub fn menu_item_height() -> f32 {
    debug_assert_eq!(row::COMPACT, typography::LINE_BODY + space::S2 * 2.0);
    row::COMPACT
}

/// The 36px row `Menu.module.css` specifies, for a menu that wants the web's
/// density — a touch-friendly kiosk, or a view that mirrors the web app.
pub fn menu_item_height_comfortable() -> f32 {
    typography::LINE_BODY + space::S3 * 2.0
}

/// What [`menu_divider`] allocates: a one-point rule with `space-1` of air either side.
///
/// The companion to [`menu_item_height`], and here for the same reason. An application
/// that positions its own menu — anything hung off a right click, which has to be placed
/// against a screen edge before a single row is drawn — has to add up the height of the
/// menu it is about to draw. Deriving that from the tokens works right up until this
/// function's padding changes, and then every menu with a divider in it is placed a few
/// points out, the error growing with the number of dividers. So the height is stated here,
/// beside the code that allocates it.
pub fn menu_divider_height() -> f32 {
    space::S1 * 2.0 + 1.0
}

/// Everything a [`MenuItem`] puts around its text, for a caller that has to size a menu
/// before drawing one.
///
/// The width counterpart of [`menu_item_height`], and here for the same reason it is: an
/// application that places its own popup has to know how wide the menu will be before the
/// first entry is laid out, and [`MenuItem`]'s own report — see `report_width` — arrives a
/// frame too late for that.
///
/// The trap this closes is `gutter`. Within a menu that has it, the leading slot is reserved
/// whether or not a given entry fills it — so a caller that adds it only for the entries with
/// an icon comes out 24 points short on any menu whose widest entry has none, and a menu 24
/// points short does not look misplaced, it looks like `Copy pa…`. Use [`Self::width`] for a
/// menu that reserves the column and [`Self::width_without_icons`] for one that does not.
pub struct MenuItemMetrics {
    /// Inside either edge.
    pub padding: f32,
    /// The leading icon-or-tick column, gap included.
    ///
    /// Reserved for every entry of a menu that **may** show an icon or a tick, and for no
    /// entry of one that may not — see [`MenuItem::gutter`] for the rule and why it is
    /// per-menu rather than per-entry.
    pub gutter: f32,
    /// Between a label and its shortcut.
    pub gap: f32,
    /// The submenu arrow, gap included, on the entries that have one.
    pub arrow: f32,
}

impl MenuItemMetrics {
    /// How wide an entry with this text wants to be, in a menu that reserves the icon column.
    ///
    /// `label` and `shortcut` are the widths of the laid-out text, which only the caller
    /// can measure — it owns the fonts the menu is drawn with.
    pub fn width(&self, label: f32, shortcut: f32, submenu: bool) -> f32 {
        self.gutter + self.width_without_icons(label, shortcut, submenu)
    }

    /// And in a menu that does not: no entry in it can carry an icon or a tick, so the column
    /// is 24 points of nothing down the left-hand side. See [`MenuItem::gutter`].
    pub fn width_without_icons(&self, label: f32, shortcut: f32, submenu: bool) -> f32 {
        self.padding * 2.0
            + label
            + if shortcut > 0.0 { self.gap + shortcut } else { 0.0 }
            + if submenu { self.arrow } else { 0.0 }
    }
}

pub fn menu_item_metrics() -> MenuItemMetrics {
    MenuItemMetrics {
        padding: space::S3,
        gutter: 16.0 + space::S3,
        gap: space::S3,
        arrow: 12.0 + space::S3,
    }
}

/// Whether the menu currently being drawn reserves the leading icon column.
///
/// Set by [`Menu`], [`ContextMenu`], [`Select`] and [`ComboBox`] around their content and read
/// by every [`MenuItem`] inside it, because **the answer belongs to the menu and not to the
/// entry**: the whole point of the column is that every label in one menu starts at the same x,
/// and a decision made per entry cannot promise that. An entry may still override it — see
/// [`MenuItem::gutter`] — which is what a caller building a menu by hand has.
///
/// Absent means reserved, so a `MenuItem` added outside any of those containers looks the way it
/// always did.
fn gutter_flag() -> egui::Id {
    egui::Id::new("azur-menu-gutter")
}

/// Run `content` with the icon column reserved, or not, and put the previous answer back.
///
/// Saved and restored rather than simply set, because menus nest: a submenu that shows icons
/// inside a parent that does not must not leave the parent's remaining entries reserving a
/// column the parent does not have.
fn with_gutter<R>(ctx: &egui::Context, reserve: bool, content: impl FnOnce() -> R) -> R {
    let was = ctx.data(|d| d.get_temp::<bool>(gutter_flag()));
    ctx.data_mut(|d| d.insert_temp(gutter_flag(), reserve));
    let out = content();
    ctx.data_mut(|d| match was {
        Some(before) => {
            d.insert_temp(gutter_flag(), before);
        }
        None => {
            d.remove::<bool>(gutter_flag());
        }
    });
    out
}

/// Where a [`MenuItem`] leaves the width it would like to be.
///
/// Azur's menu is `min-width: 180px` and grows to its content, which CSS can do
/// in one pass and egui cannot: a popup has to be sized before the entries inside
/// it are laid out. So the entries record their natural width here, and the menu
/// reads it back on the next frame — which is one frame late only when the
/// contents change, and menus are opened far more often than they are edited.
fn width_probe() -> egui::Id {
    egui::Id::new("azur-menu-width-probe")
}

fn report_width(ui: &Ui, natural: f32) {
    if let Some(target) = ui.data(|d| d.get_temp::<egui::Id>(width_probe())) {
        let widest = ui
            .data(|d| d.get_temp::<f32>(target))
            .unwrap_or(0.0)
            .max(natural);
        ui.data_mut(|d| d.insert_temp(target, widest));
    }
}

/// Size a menu popup from what its entries asked for last frame, and arm the probe
/// for this frame's entries.
fn measured_width(ctx: &egui::Context, id: egui::Id, min: f32, max: f32) -> f32 {
    let measured = ctx.data(|d| d.get_temp::<f32>(id)).unwrap_or(0.0);
    ctx.data_mut(|d| {
        d.insert_temp(id, 0.0f32);
        d.insert_temp(width_probe(), id);
    });
    // Rounded up to a whole point: handing the entries back *exactly* the width they
    // asked for leaves the widest one precisely its own galley width, and any
    // fraction lost on the way in makes it one point short — so it ellipsizes, and a
    // menu with an inexplicable `Paste spe…` in it is worse than one a point wider.
    measured.ceil().clamp(min, max)
}

/// What a menu's frame adds around its content: `menu_margin`, plus the 1px stroke
/// that `Frame::total_margin` counts and `menu_margin` alone does not.
fn menu_chrome_width(ctx: &egui::Context) -> f32 {
    let style = ctx.global_style();
    style.spacing.menu_margin.sum().x + style.visuals.window_stroke.width * 2.0
}

fn disarm_probe(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<egui::Id>(width_probe()));
}

/// Which menu the probe is currently collecting widths for, so a nested one can
/// take it over and give it back. There is a single probe, and the entries after a
/// submenu belong to the parent's width rather than to the child's.
fn probe_target(ctx: &egui::Context) -> Option<egui::Id> {
    ctx.data(|d| d.get_temp::<egui::Id>(width_probe()))
}

fn arm_probe(ctx: &egui::Context, target: Option<egui::Id>) {
    ctx.data_mut(|d| match target {
        Some(t) => {
            d.insert_temp(width_probe(), t);
        }
        None => d.remove::<egui::Id>(width_probe()),
    });
}

/// Azur's `Popover`: an anchored surface holding whatever you put in it.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::Popover;
/// let anchor = ui.button("Details");
/// Popover::new(&anchor).show(ui.ctx(), |ui| {
///     ui.label("Anything at all");
/// });
/// # }
/// ```
pub struct Popover<'a> {
    anchor: &'a Response,
    open: Option<&'a mut bool>,
    width: Option<f32>,
    /// Prefer to open above rather than below.
    above: bool,
}

impl<'a> Popover<'a> {
    /// Toggled by clicking `anchor`, closed by clicking away.
    pub fn new(anchor: &'a Response) -> Self {
        Self {
            anchor,
            open: None,
            width: None,
            above: false,
        }
    }

    /// Drive the open state from your own `bool` instead of egui's memory.
    pub fn open(mut self, open: &'a mut bool) -> Self {
        self.open = Some(open);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Open upwards. egui flips either way when there is no room, this only sets
    /// the preference.
    pub fn above(mut self, above: bool) -> Self {
        self.above = above;
        self
    }

    pub fn show<R>(
        self,
        ctx: &egui::Context,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        let t = theme_of_ctx(ctx);
        let mut popup = match self.open {
            Some(open) => egui::Popup::from_response(self.anchor).open_bool(open),
            None => egui::Popup::from_toggle_button_response(self.anchor),
        }
        .frame(popover_frame(&t))
        .gap(space::S1)
        .align(if self.above {
            egui::RectAlign::TOP_START
        } else {
            egui::RectAlign::BOTTOM_START
        });
        if let Some(w) = self.width {
            popup = popup.width(w);
        }
        popup.show(content)
    }
}

fn theme_of_ctx(ctx: &egui::Context) -> std::sync::Arc<Theme> {
    super::theme(ctx)
}

/// One entry in a `Menu` or a `Select` listbox.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::{components::MenuItem, icons};
/// if ui.add(MenuItem::new("Rename").icon(&icons::check).shortcut("F2")).clicked() {
///     // ...
/// }
/// # }
/// ```
pub struct MenuItem<'a> {
    label: Cow<'a, str>,
    icon: Option<Icon<'a>>,
    shortcut: Cow<'a, str>,
    selected: bool,
    submenu: bool,
    danger: bool,
    secondary: bool,
    height: f32,
    gutter: Option<bool>,
}

impl<'a> MenuItem<'a> {
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            shortcut: Cow::Borrowed(""),
            selected: false,
            submenu: false,
            danger: false,
            secondary: false,
            height: menu_item_height(),
            gutter: None,
        }
    }

    /// Override the row height. [`menu_item_height`] (28px, desktop density) is the
    /// default; [`menu_item_height_comfortable`] is the 36px the web spec asks for.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// A 16px leading glyph, in `text-secondary` as Azur's `.itemIcon` is.
    pub fn icon(mut self, icon: Icon<'a>) -> Self {
        self.icon = Some(icon);
        self
    }

    /// A keyboard shortcut, set right in `text-tertiary`.
    ///
    /// Azur's `MenuItem` has no shortcut slot; a desktop menu needs one, and this
    /// is the only addition here that its CSS does not describe.
    pub fn shortcut(mut self, shortcut: impl Into<Cow<'a, str>>) -> Self {
        self.shortcut = shortcut.into();
        self
    }

    /// Show a tick in front of the label — a checked menu entry, or the chosen
    /// option of a `Select`.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Draw a trailing chevron: this entry opens a submenu.
    pub fn submenu(mut self, submenu: bool) -> Self {
        self.submenu = submenu;
        self
    }

    /// A destructive entry — the label goes `danger`.
    /// Whether this entry reserves the leading icon-or-tick column.
    ///
    /// **Normally not called.** The answer belongs to the *menu* — every label in one menu has to
    /// start at the same x, which is what the column is for — so it is [`Menu::icons`],
    /// [`ContextMenu::icons`], [`Select::icons`] or [`ComboBox::icons`] that decides, and every
    /// entry inside reads it. This is the override for a caller assembling a popup by hand.
    ///
    /// The rule those four express: **the column is reserved only if the menu may show an icon or
    /// a tick.** A menu of plain values — a zoom, a font size, a list of encodings — reserves 24
    /// points down its left edge for something that can never appear there, and the labels read as
    /// indented for no reason.
    ///
    /// An entry that *does* have an icon or a tick reserves the column whatever this says: dropping
    /// it would draw the glyph over the label, and a caller who has said "no icons" and then passed
    /// one is better served by a ragged menu they can see than by a lost glyph they cannot.
    pub fn gutter(mut self, gutter: bool) -> Self {
        self.gutter = Some(gutter);
        self
    }

    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }

    /// A row whose label is not the point — the label goes `text-secondary`.
    ///
    /// For an entry that names a container rather than a thing to run: a group the app itself
    /// assembled and had to invent a name for, `More…`, `Recent`. One tier down says "this is
    /// scaffolding" without a second font or an italic, and it holds in both palettes because
    /// `text-secondary` is a role and not a shade.
    ///
    /// Not for a disabled row, which is `text-disabled` and reads as unavailable rather than as
    /// quieter; `danger` wins over this, since a destructive entry has to stay loud.
    pub fn secondary(mut self, secondary: bool) -> Self {
        self.secondary = secondary;
        self
    }
}

impl Widget for MenuItem<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let height = self.height;
        // The one statement of this entry's geometry, so a caller sizing a menu from
        // `menu_item_metrics` and the entry drawing itself cannot disagree.
        let metrics = menu_item_metrics();
        let pad = metrics.padding;
        let icon_box = 16.0;
        let gap = metrics.gap;
        // Reserved because the menu says so, or because this entry has something to put there.
        // See `MenuItem::gutter`.
        let leading = self.selected
            || self.icon.is_some()
            || self
                .gutter
                .unwrap_or_else(|| ui.data(|d| d.get_temp::<bool>(gutter_flag())).unwrap_or(true));

        // Report what this entry would like to be, so the menu around it can size
        // itself to its widest one. See `report_width`.
        let label_w = ui
            .painter()
            .layout_no_wrap(
                self.label.to_string(),
                t.fonts.body.clone(),
                Color32::PLACEHOLDER,
            )
            .size()
            .x;
        let shortcut_w = if self.shortcut.is_empty() {
            0.0
        } else {
            ui.painter()
                .layout_no_wrap(
                    self.shortcut.to_string(),
                    t.fonts.caption.clone(),
                    Color32::PLACEHOLDER,
                )
                .size()
                .x
        };
        // In a menu that has the column, it is reserved whether or not *this* entry fills it, so
        // every label starts at the same x — that is what lets the eye run down the labels
        // instead of following a ragged left edge as icons come and go. In a menu that cannot
        // have one, there is nothing to line up with.
        report_width(
            ui,
            if leading {
                metrics.width(label_w, shortcut_w, self.submenu)
            } else {
                metrics.width_without_icons(label_w, shortcut_w, self.submenu)
            },
        );

        // Entries fill the menu, so a hover fill is flush with its edges.
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
        if !ui.is_rect_visible(rect) {
            return response;
        }

        // Square, and the full width of the menu: the highlight is a band across the
        // row, not a pill floating inside it. See [`menu_frame`]. A Fluent window rounds
        // it, as WinUI does, on the same rect.
        let corner = crate::desktop::menu_radius(&t).1;
        if enabled && (response.hovered() || response.has_focus()) {
            ui.painter().rect_filled(rect, corner, t.bg.control_hover);
        }

        let fg = if !enabled {
            t.text.disabled
        } else if self.danger {
            t.status.danger
        } else if self.secondary {
            t.text.secondary
        } else {
            t.text.primary
        };
        let mut x = rect.left() + pad;

        // The tick and the icon share the leading slot, as they do in Azur: a
        // `Select` option ticks, a `Menu` entry may carry a glyph. The slot is
        // advanced past either way — see `leading_w`.
        if self.selected {
            // Lowered to the label's ink. The label is on `row_baseline`, which centres its line
            // box and so leaves the words hanging below the middle of the row by the descender room
            // they do not use; a tick centred on the row then sits that far above the text beside
            // it — 1.5px at body size, measured off a capture. Moving the tick rather than the
            // label keeps every menu's text exactly where it was. See [`super::ink_lift`].
            let lift = super::ink_lift(ui.painter(), &t.fonts.body);
            check(
                ui.painter(),
                super::icon_rect(rect, x, 14.0).translate(vec2(0.0, lift)),
                if enabled {
                    t.accent.mark
                } else {
                    t.text.disabled
                },
            );
        } else if let Some(icon) = self.icon {
            icon(
                ui.painter(),
                super::icon_rect(rect, x, icon_box),
                if enabled {
                    t.text.secondary
                } else {
                    t.text.disabled
                },
            );
        }
        if leading {
            x += icon_box + gap;
        }

        // The label is body and the shortcut is caption, so they get one baseline
        // between them rather than each centring its own box — see
        // [`super::row_baseline`].
        let baseline = super::row_baseline(ui.painter(), &t.fonts.body, rect.top(), height);

        let mut right = rect.right() - pad;
        if self.submenu {
            caret_right(
                ui.painter(),
                super::icon_rect(rect, right - 12.0, 12.0),
                if enabled {
                    t.text.secondary
                } else {
                    t.text.disabled
                },
            );
            right -= 12.0 + gap;
        }
        if !self.shortcut.is_empty() {
            let galley = ui.painter().layout_no_wrap(
                self.shortcut.to_string(),
                t.fonts.caption.clone(),
                if enabled {
                    t.text.tertiary
                } else {
                    t.text.disabled
                },
            );
            let w = galley.size().x;
            super::galley_on_baseline(ui.painter(), right - w, baseline, galley);
            right -= w + gap;
        }

        let galley = truncated(
            ui,
            &self.label,
            t.fonts.body.clone(),
            fg,
            (right - x).max(0.0),
        );
        super::galley_on_baseline(ui.painter(), x, baseline, galley);

        if response.has_focus() {
            crate::icons::focus_ring_inset(ui.painter(), rect, corner, t.stroke.focus);
        }
        response
    }
}

/// Azur's `MenuDivider`: a `stroke-subtle` rule with `space-2` either side.
pub fn menu_divider(ui: &mut Ui) {
    let t = theme_of(ui);
    ui.add_space(space::S1);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    // Border to border, like the rows it separates — a classic menu's rule is a full
    // cut across the popup, where Azur insets it the way a web list does.
    ui.painter().rect_filled(
        Rect::from_min_max(
            pos2(rect.left(), rect.center().y.round()),
            pos2(rect.right(), rect.center().y.round() + 1.0),
        ),
        CornerRadius::ZERO,
        t.stroke.subtle,
    );
    ui.add_space(space::S1);
}

/// What [`menu_header`] allocates: a caption line with `space-2` of air above and below it.
///
/// Stated beside the code that allocates it for the reason [`menu_divider_height`] is.
pub fn menu_header_height() -> f32 {
    typography::LINE_CAPTION + space::S2 * 2.0
}

/// A heading over a group of entries in a menu: `caption` in `text-secondary`, and nothing to
/// click.
///
/// **On the labels' own column**, so the heading reads as the title of the entries under it
/// rather than as one more of them: its text starts where theirs does — past the icon column in a
/// menu that reserves one, see [`MenuItem::gutter`] — and on the caption baseline, as a shortcut
/// is. Quieter than an entry by type and by ink, and never hovered, because it is not a place you
/// can go.
pub fn menu_header(ui: &mut Ui, text: &str) {
    let t = theme_of(ui);
    let metrics = menu_item_metrics();
    let leading = ui
        .data(|d| d.get_temp::<bool>(gutter_flag()))
        .unwrap_or(true);
    let x_pad = metrics.padding + if leading { metrics.gutter } else { 0.0 };

    let natural = ui
        .painter()
        .layout_no_wrap(text.to_owned(), t.fonts.caption.clone(), Color32::PLACEHOLDER)
        .size()
        .x;
    report_width(ui, x_pad + natural + metrics.padding);

    let height = menu_header_height();
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let x = rect.left() + x_pad;
    let baseline = super::row_baseline(ui.painter(), &t.fonts.caption, rect.top(), height);
    let galley = truncated(
        ui,
        text,
        t.fonts.caption.clone(),
        t.text.secondary,
        (rect.right() - metrics.padding - x).max(0.0),
    );
    super::galley_on_baseline(ui.painter(), x, baseline, galley);
}

/// Azur's `Menu`: a popover of [`MenuItem`]s hung off a button.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::{Button, Menu, MenuItem};
/// let trigger = ui.add(Button::new("Edit"));
/// Menu::new(&trigger).show(ui.ctx(), |ui| {
///     let _ = ui.add(MenuItem::new("Undo").shortcut("Ctrl+Z"));
///     let _ = ui.add(MenuItem::new("Redo").shortcut("Ctrl+Y"));
/// });
/// # }
/// ```
pub struct Menu<'a> {
    trigger: &'a Response,
    open: Option<&'a mut bool>,
    min_width: f32,
    max_width: f32,
    max_height: f32,
    icons: bool,
}

impl<'a> Menu<'a> {
    pub fn new(trigger: &'a Response) -> Self {
        Self {
            trigger,
            open: None,
            icons: true,
            // `.list { min-width: 180px; max-height: 320px }`.
            min_width: 180.0,
            // No token: past this, a label truncates rather than pushing the menu
            // across the window.
            max_width: 420.0,
            max_height: 320.0,
        }
    }

    /// Drive the open state from your own `bool` instead of egui's memory.
    ///
    /// Left alone, a menu belongs to its own trigger: a click on the button toggles it, and
    /// nothing else can. That is right for a button and cannot express **a row of triggers
    /// that behave as one control once any of them is open** — a menu bar, or a breadcrumb,
    /// where moving the pointer along the bar has to bring the open menu with it. The state
    /// there is "which one of these is showing", which is one value for the whole row and so
    /// belongs to the row.
    ///
    /// Everything about *closing* still happens here: a click on an entry, a click outside and
    /// `Escape` all set the `bool` back to `false`, so the caller learns the menu has gone
    /// without having to work out why.
    pub fn open(mut self, open: &'a mut bool) -> Self {
        self.open = Some(open);
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    /// The width a long label truncates at. Not a token — Azur's menu grows with
    /// its content and the page scrolls; a desktop menu cannot.
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }

    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }

    /// Whether any entry in this menu may carry an icon or a tick. On by default.
    ///
    /// Off drops the 24-point leading column from every entry, which is right for a menu of plain
    /// values and wrong for one where an icon may appear — see [`MenuItem::gutter`] for the rule.
    /// It applies to the entries the content closure adds, however they were built, so the
    /// decision is made once for the menu rather than repeated on every row.
    pub fn icons(mut self, icons: bool) -> Self {
        self.icons = icons;
        self
    }

    pub fn show<R>(
        self,
        ctx: &egui::Context,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        let t = theme_of_ctx(ctx);
        let max_height = self.max_height;
        let width = measured_width(
            ctx,
            egui::Popup::default_response_id(self.trigger).with("azur-width"),
            self.min_width,
            self.max_width,
        );
        let icons = self.icons;
        let mut popup = egui::Popup::menu(self.trigger);
        if let Some(open) = self.open {
            // Replaces the toggle-on-click that `Popup::menu` set up, and nothing else about
            // it: the frame, the layout, the order and the close behaviour are the same menu.
            popup = popup.open_bool(open);
        }
        let shown = popup
            .frame(menu_frame(&t))
            .gap(space::S1)
            // Only a seed: `Popup::width` feeds `Area::default_width`, which the area
            // uses on the frame it is created and never again. The probe's answer
            // arrives a frame later, so the width has to be applied from *inside* as
            // well or every menu keeps the fallback it opened at.
            .width(width + menu_chrome_width(ctx))
            .show(move |ui| {
                ui.set_width(width);
                // **The room the scroll area is allowed to grow into, taken rather than inherited.**
                //
                // An `Area` hands its content *last frame's size* as this frame's `max_rect` — see
                // `egui::Area::content_ui` — and a `ScrollArea` fits itself into whatever room it is
                // given without ever asking for more. Put those two together and a menu is stuck at
                // the size of the first list it ever showed: a dropdown whose rows arrive a frame
                // after it opens measures itself on `Reading…`, and the rows that land next frame
                // then shrink to fit the one row of room left over. Two rows and a scrollbar, for
                // ever, because nothing ever asks for more.
                //
                // `set_max_height` is a ceiling and not a size — it moves `max_rect`, never
                // `min_rect` — so the scroll area still shrinks to its content and the popup is
                // still only as tall as what is in it. What it stops is the menu being unable to
                // *ask*.
                ui.set_max_height(max_height);
                // `gap: space-1` between entries.
                ui.spacing_mut().item_spacing.y = 0.0;
                // A menu is not a panel. A non-floating scroll bar allocates its
                // width whether or not it is showing, so every menu would carry a
                // 14pt gutter of nothing down its right edge -- and that gutter is
                // what clipped the widest entry into `Paste spe...`. Floating means
                // the bar overlays while scrolling and takes no space otherwise,
                // which is also nearer what a desktop menu does when it overflows.
                ui.style_mut().spacing.scroll.floating = true;
                let ctx = ui.ctx().clone();
                with_gutter(&ctx, icons, || {
                    egui::ScrollArea::vertical()
                        .id_salt("azur-menu")
                        .max_height(max_height)
                        .show(ui, content)
                        .inner
                })
            });
        disarm_probe(ctx);
        shown
    }
}

/// Azur's `ContextMenuTarget`: the same menu, opened by right-clicking a widget.
pub struct ContextMenu<'a> {
    target: &'a Response,
    min_width: f32,
    max_width: f32,
    sticky: bool,
    icons: bool,
}

impl<'a> ContextMenu<'a> {
    pub fn new(target: &'a Response) -> Self {
        Self {
            target,
            min_width: 180.0,
            max_width: 420.0,
            sticky: false,
            icons: true,
        }
    }

    /// Stay open when something inside is clicked; close on a click outside, or on
    /// Escape.
    ///
    /// For a menu whose entries are *settings* rather than commands — a list of
    /// checkboxes — where closing after each tick means reopening it for the next
    /// one. A menu of commands should not do this: dismissal is how a reader knows
    /// the command was taken.
    ///
    /// Entries that really are commands still have to call [`Ui::close`]
    /// themselves once this is set.
    pub fn sticky(mut self, sticky: bool) -> Self {
        self.sticky = sticky;
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }

    /// Whether any entry in this menu may carry an icon or a tick. On by default, and the same
    /// rule as [`Menu::icons`].
    pub fn icons(mut self, icons: bool) -> Self {
        self.icons = icons;
        self
    }

    pub fn show<R>(
        self,
        ctx: &egui::Context,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        let t = theme_of_ctx(ctx);
        let icons = self.icons;
        let width = measured_width(
            ctx,
            self.target.id.with("azur-context-width"),
            self.min_width,
            self.max_width,
        );
        let mut popup = egui::Popup::context_menu(self.target)
            .frame(menu_frame(&t))
            // A seed only — see `Menu::show`.
            .width(width + menu_chrome_width(ctx));
        if self.sticky {
            popup = popup.close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
        }
        let shown = popup.show(move |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing.y = 0.0;
            let ctx = ui.ctx().clone();
            with_gutter(&ctx, icons, || content(ui))
        });
        disarm_probe(ctx);
        shown
    }
}

/// A menu entry that opens a nested menu beside it.
///
/// ```no_run
/// # use azur_egui_theme::components::{MenuItem, submenu};
/// # fn f(ui: &mut egui::Ui) {
/// submenu(ui, MenuItem::new("Set priority"), |ui| {
///     for name in ["High", "Normal", "Low"] {
///         if ui.add(MenuItem::new(name)).clicked() {
///             ui.close();
///         }
///     }
/// });
/// # }
/// ```
///
/// The entry itself is a [`MenuItem`] — pass one with an icon or a `danger` flag if
/// it needs them; the trailing chevron is applied here so no caller can open a
/// submenu without advertising it. Everything about *when* it opens is egui's own
/// submenu machinery, which is worth deferring to: open on hover, close when a
/// sibling opens, keep the parent open while the child is up, close the whole stack
/// on a click outside, and walk in and out with the arrow keys. Behaviour a reader
/// already knows is not worth reimplementing differently.
///
/// The nested surface is `Frame::menu`, which [`mod@crate::style`] has already set to
/// the same fill, stroke, radius, shadow and margin as [`popover_frame`] — so it
/// matches the menu it grew out of without being told to.
pub fn submenu<'a, R>(
    ui: &mut Ui,
    item: MenuItem<'a>,
    content: impl FnOnce(&mut Ui) -> R,
) -> Option<InnerResponse<R>> {
    let response = ui.add(item.submenu(true));
    let ctx = ui.ctx().clone();

    // Sized the same way a menu is, and for a sharper reason than tidiness: a
    // `MenuItem` fills the width it is handed, and a submenu popup is handed the
    // whole screen unless it is told otherwise. Left alone it comes out hundreds of
    // pixels wide, which leaves no room beside the parent — so egui, finding the
    // panel cannot fit to the right, drops it underneath instead. Giving it the
    // width of its own widest entry is what keeps it where a submenu belongs.
    let parent_probe = probe_target(&ctx);
    let width = measured_width(&ctx, response.id.with("azur-submenu-width"), 180.0, 420.0);

    let shown = egui::containers::menu::SubMenu::new().show(ui, &response, |ui| {
        ui.set_width(width);
        // The row rhythm the parent menu sets in `ContextMenu::show`. A popup gets
        // a fresh style, so it does not inherit it.
        ui.spacing_mut().item_spacing.y = 0.0;
        content(ui)
    });

    arm_probe(&ctx, parent_probe);
    shown
}

/// One option of a [`Select`] or a [`ComboBox`].
#[derive(Clone, Debug)]
pub struct SelectOption<'a> {
    pub label: Cow<'a, str>,
    pub disabled: bool,
}

impl<'a> SelectOption<'a> {
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<'a> From<&'a str> for SelectOption<'a> {
    fn from(label: &'a str) -> Self {
        Self::new(label)
    }
}

impl From<String> for SelectOption<'_> {
    fn from(label: String) -> Self {
        Self::new(label)
    }
}

/// A field-shaped trigger — the shared body of `Select` and `ComboBox`.
///
/// Allocated in one call and painted in another, because the border color depends
/// on whether the popup is open and that is keyed off the trigger's own response
/// id. The `Noop` shape reserves the slot so the fill still lands behind
/// everything drawn in between.
struct Trigger {
    rect: Rect,
    /// Where the value or the input goes.
    content: Rect,
}

/// The trigger's rect, and the slot its background will go in.
#[derive(Clone, Copy)]
struct TriggerSlot {
    rect: Rect,
    background: egui::layers::ShapeIdx,
}

fn trigger_allocate(ui: &mut Ui, size: Size, width: f32, sense: Sense) -> (TriggerSlot, Response) {
    let background = ui.painter().add(Shape::Noop);
    let (rect, response) = ui.allocate_exact_size(vec2(width, size.height()), sense);
    (TriggerSlot { rect, background }, response)
}

#[allow(clippy::too_many_arguments)]
fn trigger_paint(
    ui: &mut Ui,
    t: &Theme,
    size: Size,
    slot: TriggerSlot,
    response: &Response,
    enabled: bool,
    open: bool,
    variant: Variant,
    leading: Option<Icon<'_>>,
) -> Trigger {
    let TriggerSlot { rect, background } = slot;
    let pad = size.padding_x();
    // `Variant::Subtle` gives up the fill and the border while the trigger is at
    // rest, exactly as a subtle `Button` does; every other state is the field's.
    // Anything but `Subtle` is the field look — a trigger has no accent or
    // destructive form to take.
    let subtle = variant == Variant::Subtle;

    let stroke = if !enabled || (subtle && !open) {
        Color32::TRANSPARENT
    } else {
        // `.triggerOpen { border-color: stroke-focus }`.
        field_stroke(t, enabled, open, response.hovered(), false)
    };
    let fill = if !enabled {
        t.bg.control_disabled
    } else if response.hovered() {
        t.bg.control_hover
    } else if subtle && !open {
        Color32::TRANSPARENT
    } else {
        t.bg.control
    };
    ui.painter().set(
        background,
        Shape::Rect(egui::epaint::RectShape::new(
            rect,
            super::control_radius(),
            fill,
            Stroke::new(1.0, stroke),
            StrokeKind::Inside,
        )),
    );

    // `.chevron { width: 12px }`, `space-3` from the value.
    let chevron_box = super::icon_rect(rect, rect.right() - pad - 12.0, 12.0);
    let affix_color = if enabled {
        t.text.secondary
    } else {
        t.text.disabled
    };
    if open {
        chevron_up(ui.painter(), chevron_box, affix_color);
    } else {
        chevron_down(ui.painter(), chevron_box, affix_color);
    }

    // A leading glyph, laid out exactly as `SearchBox` lays out its magnifier:
    // `control-icon` square at the padding edge, `space-2` before the value. The
    // two are the same shape of control, so they say what they are the same way.
    let mut content_left = rect.left() + pad;
    if let Some(icon) = leading {
        let box_size = super::control_icon(size);
        icon(
            ui.painter(),
            super::icon_rect(rect, content_left, box_size),
            affix_color,
        );
        content_left += box_size + space::S2;
    }

    if response.has_focus() {
        focus_ring(ui.painter(), rect, super::control_radius(), t.stroke.focus);
    }

    Trigger {
        rect,
        content: Rect::from_min_max(
            pos2(content_left, rect.top()),
            pos2(chevron_box.left() - space::S3, rect.bottom()),
        ),
    }
}

/// Azur's `Select`: a closed list of options behind a field-shaped trigger.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::Select;
/// let mut units: Option<usize> = Some(0);
/// ui.add(Select::new(&mut units).options(["Millimetres", "Inches"]).placeholder("Pick units"));
/// # }
/// ```
pub struct Select<'a> {
    selected: &'a mut Option<usize>,
    options: Vec<SelectOption<'a>>,
    placeholder: Cow<'a, str>,
    size: Size,
    variant: Variant,
    width: Option<f32>,
    icon: Option<Icon<'a>>,
    icons: bool,
}

impl<'a> Select<'a> {
    pub fn new(selected: &'a mut Option<usize>) -> Self {
        Self {
            selected,
            options: Vec::new(),
            placeholder: Cow::Borrowed("Select…"),
            size: Size::default(),
            variant: Variant::default(),
            icons: true,
            width: None,
            icon: None,
        }
    }

    pub fn options<I>(mut self, options: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<SelectOption<'a>>,
    {
        self.options = options.into_iter().map(Into::into).collect();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// [`Variant::Secondary`], the field look, or [`Variant::Subtle`] — no fill and
    /// no border until the pointer is on it.
    ///
    /// Subtle is for a select that sits in a toolbar beside subtle icon buttons
    /// rather than in a form beside other fields: there is nothing to line its
    /// border up with there, and one more box on a strip of bare glyphs reads as
    /// the loudest thing on it. The field look is the default, and the two accent
    /// variants have no meaning on a trigger.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Whether the list may show a **tick** beside the option that is current. On by default, and
    /// the same rule and the same reasoning as [`ComboBox::icons`].
    pub fn icons(mut self, icons: bool) -> Self {
        self.icons = icons;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// A leading glyph — see [`crate::icons`].
    ///
    /// For a select whose subject is not obvious from the value alone: a clock in
    /// front of "Normal" says the field is about time, which saves the label
    /// beside it. Not a substitute for a label when the options themselves are
    /// the subject ("Millimetres", "Inches" need no ruler).
    pub fn icon(mut self, icon: Icon<'a>) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl Widget for Select<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let ticks = self.icons;
        let width = self.width.unwrap_or_else(|| ui.available_width());

        let (slot, mut response) = trigger_allocate(ui, self.size, width, Sense::click());
        let open = egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&response));
        let trigger_rects = trigger_paint(
            ui,
            &t,
            self.size,
            slot,
            &response,
            enabled,
            open,
            self.variant,
            self.icon,
        );

        // The value, or the placeholder in `text-tertiary`.
        let (text, color): (&str, Color32) = match self.selected.and_then(|i| self.options.get(i)) {
            Some(option) => (
                &option.label,
                if enabled {
                    t.text.primary
                } else {
                    t.text.disabled
                },
            ),
            None => (
                &self.placeholder,
                if enabled {
                    t.text.tertiary
                } else {
                    t.text.disabled
                },
            ),
        };
        let galley = truncated(
            ui,
            text,
            self.size.font(&t.fonts),
            color,
            trigger_rects.content.width(),
        );
        super::galley_left_center(ui, trigger_rects.content, galley);

        let picked = egui::Popup::from_toggle_button_response(&response)
            .frame(menu_frame(&t))
            .gap(space::S1)
            .width(trigger_rects.rect.width())
            .show(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                // See `Menu::show`: a floating bar keeps the listbox from carrying an
                // empty scroll gutter, and the height is a ceiling the scroll area has to be
                // handed or it can never grow past the shortest list it ever showed.
                ui.style_mut().spacing.scroll.floating = true;
                ui.set_max_height(LISTBOX_HEIGHT);
                let mut picked = None;
                egui::ScrollArea::vertical()
                    .id_salt("azur-listbox")
                    .max_height(LISTBOX_HEIGHT)
                    .show(ui, |ui| {
                        for (i, option) in self.options.iter().enumerate() {
                            let item = ui.add_enabled(
                                !option.disabled,
                                MenuItem::new(option.label.clone())
                                    .selected(ticks && *self.selected == Some(i))
                                    .gutter(ticks),
                            );
                            if item.clicked() {
                                picked = Some(i);
                            }
                        }
                    });
                picked
            })
            .and_then(|r| r.inner);

        if let Some(i) = picked {
            *self.selected = Some(i);
            response.mark_changed();
            egui::Popup::close_id(ui.ctx(), egui::Popup::default_response_id(&response));
        }
        response
    }
}

/// Azur's `ComboBox`: a `Select` whose trigger is an editable input.
///
/// The text is the caller's, so it survives whatever the list does with it; the
/// index is set only when an option is actually chosen.
///
/// **Two kinds of combo box, and [`ComboBox::filter`] is which one this is.** Filtering is right
/// for a *picker*: a long list of names, where typing narrows it and the text is a search. It is
/// wrong for a *value with presets* — a zoom, a font size, a line height — where the field already
/// holds the current value, so filtering by it narrows the list to the one option you are already
/// on, and a value nobody offered narrows it to nothing at all and the list stops opening. That
/// second case wants every option every time, which is `filter(false)`.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::ComboBox;
/// let (mut text, mut picked) = (String::new(), None);
/// ui.add(ComboBox::new(&mut text, &mut picked).options(["Bilinear", "Bicubic", "Nearest"]));
/// # }
/// ```
pub struct ComboBox<'a> {
    text: &'a mut String,
    selected: &'a mut Option<usize>,
    options: Vec<SelectOption<'a>>,
    placeholder: Cow<'a, str>,
    size: Size,
    variant: Variant,
    filter: bool,
    icons: bool,
    width: Option<f32>,
}

impl<'a> ComboBox<'a> {
    pub fn new(text: &'a mut String, selected: &'a mut Option<usize>) -> Self {
        Self {
            text,
            selected,
            options: Vec::new(),
            placeholder: Cow::Borrowed(""),
            size: Size::default(),
            variant: Variant::default(),
            filter: true,
            icons: true,
            width: None,
        }
    }

    pub fn options<I>(mut self, options: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<SelectOption<'a>>,
    {
        self.options = options.into_iter().map(Into::into).collect();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// [`Variant::Subtle`] gives up the fill and the border until the pointer is over it, and
    /// everything else is the field look — the same two states a [`Select`] takes, for the same
    /// reason: the accent and destructive forms have no meaning on a trigger.
    ///
    /// Subtle is what a combo box wants **on a crowded bar**, where a field's border round every
    /// control is more lines than there is information. The value still reads as a value, and the
    /// affordance arrives when the pointer does.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Whether typing narrows the list. On by default; see the type's own documentation for the
    /// case that wants it off, which is any field whose text is a *value* rather than a search.
    ///
    /// Off, the list always offers everything and the text is free-form — the field and the list
    /// are two ways to say the same thing rather than one filtering the other.
    pub fn filter(mut self, filter: bool) -> Self {
        self.filter = filter;
        self
    }

    /// Whether the list may show a **tick** beside the option that is current. On by default.
    ///
    /// Off drops it, and with it the 24-point leading column every entry reserves for it — see
    /// [`MenuItem::gutter`] for the rule. The two go together and cannot be separated: a tick with
    /// no column to sit in would be drawn over a label, and a column with nothing that can ever
    /// appear in it is an indent for no reason.
    ///
    /// Right for a list of **plain values** — a zoom, a font size, a line height — where the
    /// trigger already shows which one is current, in a field you can read without opening
    /// anything. A tick in a list you have to open in order to see is the weaker of the two places
    /// to say the same thing.
    pub fn icons(mut self, icons: bool) -> Self {
        self.icons = icons;
        self
    }


    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for ComboBox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let id = ui.next_auto_id();
        let open_id = id.with("azur-combo-open");
        let mut open = ui.data(|d| d.get_temp::<bool>(open_id)).unwrap_or(false);

        let (slot, trigger_response) = trigger_allocate(ui, self.size, width, Sense::click());
        let rects = trigger_paint(
            ui,
            &t,
            self.size,
            slot,
            &trigger_response,
            enabled,
            open,
            self.variant,
            None,
        );

        // The input sits where a `Select` puts its value.
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rects.content)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        let edit = child.add_enabled(
            enabled,
            egui::TextEdit::singleline(self.text)
                .id(id.with("edit"))
                .frame(egui::Frame::NONE)
                .margin(Margin::ZERO)
                .desired_width(rects.content.width())
                .font(egui::FontSelection::FontId(self.size.font(&t.fonts)))
                .text_color(t.text_for(enabled)),
        );
        if self.text.is_empty() && !self.placeholder.is_empty() && !edit.has_focus() {
            let galley = truncated(
                ui,
                &self.placeholder,
                self.size.font(&t.fonts),
                if enabled {
                    t.text.tertiary
                } else {
                    t.text.disabled
                },
                rects.content.width(),
            );
            super::galley_left_center(ui, rects.content, galley);
        }

        let mut response = edit.clone();
        // Typing opens the list; a click on the field or the chevron toggles it.
        //
        // **`clicked` and not `gained_focus`, and the difference is one frame.** Focus is granted on
        // the button *press*; egui's own guard against a popup being closed by the very click that
        // opened it is `was_open_last_frame`, which is false on that frame — so the *release* a
        // frame later counted as a click outside the list and shut it again. The list appeared for
        // exactly one frame, which from the outside is a combo box with no list at all.
        if edit.changed() || (edit.clicked() && !open) {
            open = true;
        }
        if trigger_response.clicked() && !edit.has_focus() {
            open = !open;
        }

        // **How wide the list has to be**, before it is opened.
        //
        // A popup's `width` is the area's, and `MenuItem` elides its label to whatever it is given
        // — so a combo box on a crowded bar, sixty points wide, turned every entry into `Fi…`. It
        // is anchored to the trigger and may not be narrower than it, and it has to be at least as
        // wide as its widest entry wants.
        //
        // Measured here rather than probed the way [`Menu`] does it. `Menu` cannot know its content
        // — a caller fills it — so it reports a width from inside and reads it back a frame later;
        // this one knows every option up front, and a galley is cached, so there is no reason to
        // spend that frame. Which matters: the frame `Menu` spends is a frame this list would be
        // showing `Fi…` for, and a dropdown is looked at for about three of them.
        let list = {
            let metrics = menu_item_metrics();
            let widest = self
                .options
                .iter()
                .map(|option| {
                    ui.painter()
                        .layout_no_wrap(
                            option.label.to_string(),
                            t.fonts.body.clone(),
                            Color32::PLACEHOLDER,
                        )
                        .size()
                        .x
                })
                .fold(0.0f32, f32::max);
            // No shortcut and no submenu: a `Select`'s list has neither. And no leading column
            // either, when the list cannot tick anything — see `ComboBox::icons`.
            let bare = if self.icons {
                metrics.width(widest, 0.0, false)
            } else {
                metrics.width_without_icons(widest, 0.0, false)
            };
            bare
                .max(rects.rect.width())
                // The same ceiling `Menu` puts on a label — past this it truncates rather than
                // pushing the list across the window.
                .min(420.0)
        };

        let icons = self.icons;
        // What has been typed, read as a filter — [`crate::filter`], the same rule as any
        // other field with a funnel in it. For the single word this is nearly always given
        // that is a case-insensitive substring and nothing more; what it adds is that two
        // words narrow in either order, which is the difference between typing `box com`
        // and finding nothing.
        let needle = if self.filter {
            crate::filter::Query::parse(self.text)
        } else {
            crate::filter::Query::default()
        };
        let matches: Vec<usize> = self
            .options
            .iter()
            .enumerate()
            .filter(|(_, o)| needle.matches(&o.label))
            .map(|(i, _)| i)
            .collect();

        if open && !matches.is_empty() {
            // `open_bool` hands egui the flag, so a click outside closes the list
            // without this having to hunt for one.
            let picked = egui::Popup::from_response(&trigger_response)
                .open_bool(&mut open)
                .frame(menu_frame(&t))
                .gap(space::S1)
                .width(list + menu_chrome_width(ui.ctx()))
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .show(|ui| {
                    ui.set_width(list);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.style_mut().spacing.scroll.floating = true;
                    // The list here is *filtered* as the field is typed into, which is the one
                    // shape this bites hardest: one match makes the popup one row tall, and
                    // rubbing the letter out again leaves fourteen matches in that one row of
                    // room. See `Menu::show` for why the ceiling has to be handed over.
                    ui.set_max_height(LISTBOX_HEIGHT);
                    let mut picked = None;
                    egui::ScrollArea::vertical()
                        .id_salt("azur-combo-listbox")
                        .max_height(LISTBOX_HEIGHT)
                        .show(ui, |ui| {
                            for i in &matches {
                                let option = &self.options[*i];
                                if ui
                                    .add_enabled(
                                        !option.disabled,
                                        MenuItem::new(option.label.clone())
                                            .selected(icons && *self.selected == Some(*i))
                                            .gutter(icons),
                                    )
                                    .clicked()
                                {
                                    picked = Some(*i);
                                }
                            }
                        });
                    picked
                })
                .and_then(|r| r.inner);

            if let Some(i) = picked {
                *self.selected = Some(i);
                *self.text = self.options[i].label.to_string();
                open = false;
                response.mark_changed();
            }
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            open = false;
        }
        ui.data_mut(|d| d.insert_temp(open_id, open));
        response
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /// [`menu_item_metrics`] has to be what a [`MenuItem`] actually asks for.
    ///
    /// The companion of the height check: an application that places its own popup sizes it
    /// from these numbers before a row is drawn, so a drift here is a menu that is a little
    /// too narrow -- and a menu a little too narrow does not look misplaced, it looks like
    /// somebody shipped `Copy pa...`. The trap the assertion below is really watching is the
    /// leading gutter, which the item reserves for every entry and a caller is tempted to
    /// add only for the ones with an icon.
    #[test]
    fn the_stated_item_width_is_the_width_an_item_reports() {
        let ctx = egui::Context::default();
        let t = crate::install_dark(&ctx);

        let cases: [(&str, &str, bool); 4] = [
            ("Refresh", "F5", false),
            ("Open terminal here", "", false),
            ("Send to", "", true),
            ("New folder", "Ctrl+Shift+N", false),
        ];

        for (label, shortcut, submenu) in cases {
            let mut reported = 0.0f32;
            let target = egui::Id::new(("width-probe-test", label));
            let _ = ctx.run_ui(Default::default(), |ctx| {
                // Arm the probe the same way `measured_width` does, then read back what
                // the entry left in it.
                ctx.data_mut(|d| {
                    d.insert_temp(target, 0.0f32);
                    d.insert_temp(width_probe(), target);
                });
                egui::Area::new(egui::Id::new(("probe", label))).show(ctx, |ui| {
                    ui.set_width(600.0);
                    let mut item = MenuItem::new(label).shortcut(shortcut);
                    if submenu {
                        item = item.submenu(true);
                    }
                    ui.add(item);
                });
                reported = ctx.data(|d| d.get_temp::<f32>(target)).unwrap_or(0.0);
            });
            assert!(reported > 0.0, "`{label}` reported nothing");

            // The same arithmetic a caller does, from the stated metrics and its own fonts.
            let (label_w, shortcut_w) = ctx.fonts_mut(|fonts| {
                (
                    fonts
                        .layout_no_wrap(label.to_owned(), t.fonts.body.clone(), Color32::WHITE)
                        .size()
                        .x,
                    if shortcut.is_empty() {
                        0.0
                    } else {
                        fonts
                            .layout_no_wrap(
                                shortcut.to_owned(),
                                t.fonts.caption.clone(),
                                Color32::WHITE,
                            )
                            .size()
                            .x
                    },
                )
            });
            let stated = menu_item_metrics().width(label_w, shortcut_w, submenu);
            assert_eq!(
                stated, reported,
                "`{label}`: the metrics say {stated} and the item asked for {reported}"
            );
        }
    }

    /// **A menu that cannot show an icon does not reserve a column for one** — in its width and in
    /// where it puts its labels.
    ///
    /// Both, because the two have to agree or the menu is mis-sized in the direction that
    /// truncates: an entry that drops the 24-point column while its container still reserves it
    /// leaves the menu 24 points too wide, and one that keeps it while the container has dropped it
    /// leaves the widest label 24 points short. So the reported width is checked against
    /// [`MenuItemMetrics::width_without_icons`] — the arithmetic a caller sizing its own popup does
    /// — and the label's own x is checked against the padding, which is what somebody looking at an
    /// indented list of percentages is actually complaining about.
    #[test]
    fn a_menu_with_no_icons_reserves_no_room_for_them() {
        let ctx = egui::Context::default();
        let t = crate::install_dark(&ctx);
        const LABEL: &str = "400%";

        let mut seen: Vec<(f32, f32)> = Vec::new();
        for icons in [true, false] {
            let target = egui::Id::new(("gutter-probe", icons));
            let mut reported = 0.0f32;
            // Twice: a fresh context has no glyph atlas until a pass has asked for one, so the
            // first frame lays the label out to nothing and paints no text at all.
            let mut out = Default::default();
            for _ in 0..2 {
                out = ctx.run_ui(Default::default(), |ctx| {
                    ctx.data_mut(|d| {
                        d.insert_temp(target, 0.0f32);
                        d.insert_temp(width_probe(), target);
                    });
                    egui::Area::new(egui::Id::new(("gutter", icons)))
                        .fixed_pos(egui::pos2(0.0, 0.0))
                        .show(ctx, |ui| {
                            ui.set_width(600.0);
                            ui.add(MenuItem::new(LABEL).gutter(icons));
                        });
                    reported = ctx.data(|d| d.get_temp::<f32>(target)).unwrap_or(0.0);
                });
            }
            assert!(reported > 0.0, "icons={icons}: the entry reported nothing");

            // Where the label actually landed, out of what the frame painted.
            fn find(shape: &Shape, into: &mut Option<f32>) {
                match shape {
                    Shape::Text(text) if text.galley.text() == LABEL => *into = Some(text.pos.x),
                    Shape::Vec(shapes) => {
                        for shape in shapes {
                            find(shape, into);
                        }
                    }
                    _ => {}
                }
            }
            let mut at = None;
            for clipped in &out.shapes {
                find(&clipped.shape, &mut at);
            }
            seen.push((reported, at.expect("the label was not drawn")));
        }

        let metrics = menu_item_metrics();
        let width = ctx.fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(LABEL.to_owned(), t.fonts.body.clone(), Color32::WHITE)
                .size()
                .x
        });
        // What each one asked to be, against the arithmetic a caller would do.
        assert_eq!(seen[0].0, metrics.width(width, 0.0, false), "with the column");
        assert_eq!(
            seen[1].0,
            metrics.width_without_icons(width, 0.0, false),
            "without it"
        );
        // And what each one drew: the padding alone, or the padding and the column.
        assert_eq!(seen[1].1, metrics.padding, "the label is still indented");
        assert_eq!(
            seen[0].1 - seen[1].1,
            metrics.gutter,
            "the two do not differ by exactly the column: {seen:?}"
        );
    }

    /// A menu sized from the metrics is wide enough to draw its widest entry without
    /// ellipsizing it -- including one with no icon, which is the case that was wrong.
    #[test]
    fn an_entry_with_no_icon_still_gets_its_gutter() {
        let metrics = menu_item_metrics();
        assert!(
            metrics.gutter > 0.0,
            "the leading column is reserved for every entry, icon or not"
        );
        // Two entries, same text, one with a submenu arrow: the arrow costs extra and the
        // gutter costs the same.
        assert_eq!(
            metrics.width(100.0, 0.0, true) - metrics.width(100.0, 0.0, false),
            metrics.arrow
        );
        assert_eq!(
            metrics.width(100.0, 40.0, false) - metrics.width(100.0, 0.0, false),
            metrics.gap + 40.0
        );
    }
}
