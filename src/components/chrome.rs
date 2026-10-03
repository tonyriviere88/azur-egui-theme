//! `Text`, `Avatar`, `Breadcrumb`, `CommandBar`, `NavigationRail`, `TitleBar`.
//!
//! The furniture of a desktop window. None of it has an egui counterpart, and all
//! of it is what makes an app read as Azur before a single control is drawn.

use super::{theme_of, truncated, Size};
use crate::icons::{
    chevron_right, focus_ring, focus_ring_inset, window_close, window_maximize, window_minimize,
    window_restore, Icon,
};
use crate::theme::Theme;
use crate::tokens::{palette, radius, row, space, typography};
use egui::{
    pos2, vec2, Color32, CornerRadius, FontId, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    Widget,
};
use std::borrow::Cow;

/// Azur's type ramp, as the `Text` component names it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TextVariant {
    Display,
    TitleLarge,
    Title,
    TitleSmall,
    Subtitle,
    #[default]
    Body,
    BodyStrong,
    Caption,
}

impl TextVariant {
    /// The font, and the line height Azur pairs with it.
    pub fn font(self, t: &Theme) -> (FontId, f32) {
        use typography::*;
        let f = &t.fonts;
        match self {
            Self::Display => (f.display.clone(), LINE_DISPLAY),
            Self::TitleLarge => (f.title_large.clone(), LINE_TITLE_LARGE),
            Self::Title => (f.title.clone(), LINE_TITLE),
            Self::TitleSmall => (f.title_small.clone(), LINE_TITLE_SMALL),
            Self::Subtitle => (f.subtitle.clone(), LINE_SUBTITLE),
            Self::Body => (f.body.clone(), LINE_BODY),
            Self::BodyStrong => (f.body_strong.clone(), LINE_BODY),
            Self::Caption => (f.caption.clone(), LINE_CAPTION),
        }
    }
}

/// `Text`'s semantic colors.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TextColor {
    #[default]
    Primary,
    Secondary,
    Tertiary,
    Disabled,
    Accent,
    Danger,
}

impl TextColor {
    pub fn color(self, t: &Theme) -> Color32 {
        match self {
            Self::Primary => t.text.primary,
            Self::Secondary => t.text.secondary,
            Self::Tertiary => t.text.tertiary,
            Self::Disabled => t.text.disabled,
            Self::Accent => t.accent.mark,
            Self::Danger => t.status.danger,
        }
    }
}

/// Azur's `Text`: the typography primitive.
///
/// egui's `Label` would do, but every call site would have to name a font *and* a
/// color, and that is how a type ramp drifts. This names the variant instead.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::{Text, TextColor, TextVariant};
/// ui.add(Text::new("Registration").variant(TextVariant::Title));
/// ui.add(Text::new("4 of 12 scans").color(TextColor::Secondary));
/// # }
/// ```
pub struct Text<'a> {
    text: Cow<'a, str>,
    variant: TextVariant,
    color: TextColor,
    truncate: bool,
    /// Override the color outright, for the handful of places a status hue is
    /// computed rather than named.
    explicit: Option<Color32>,
}

impl<'a> Text<'a> {
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            text: text.into(),
            variant: TextVariant::default(),
            color: TextColor::default(),
            truncate: false,
            explicit: None,
        }
    }

    pub fn variant(mut self, variant: TextVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn color(mut self, color: TextColor) -> Self {
        self.color = color;
        self
    }

    /// A color from outside the six semantic ones — e.g.
    /// [`Theme::threshold_text`](crate::Theme::threshold_text).
    pub fn explicit_color(mut self, color: Color32) -> Self {
        self.explicit = Some(color);
        self
    }

    /// One line, cut with an ellipsis. Azur's `truncate`.
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }
}

impl Widget for Text<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let (font, line) = self.variant.font(&t);
        let color = self
            .explicit
            .unwrap_or_else(|| self.color.color(&t))
            .to_owned();

        if self.truncate {
            let width = ui.available_width();
            let galley = truncated(ui, &self.text, font, color, width);
            let (rect, response) = ui.allocate_exact_size(vec2(width, line), Sense::hover());
            super::galley_left_center(ui, rect, galley);
            response
        } else {
            ui.add(egui::Label::new(
                egui::RichText::new(self.text.as_ref())
                    .font(font)
                    .color(color),
            ))
        }
    }
}

/// `Avatar`'s shape.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AvatarShape {
    #[default]
    Circle,
    /// `radius-medium`.
    Square,
}

/// The presence dot on an `Avatar`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AvatarStatus {
    Online,
    Away,
    Busy,
    Offline,
}

impl AvatarStatus {
    fn color(self, t: &Theme) -> Color32 {
        match self {
            Self::Online => t.status.success,
            Self::Away => t.status.warning,
            Self::Busy => t.status.danger,
            Self::Offline => t.text.tertiary,
        }
    }
}

/// Azur's `Avatar`: initials on a tone picked from the name, with an optional
/// presence dot.
///
/// The five tones and the way they are chosen — sum the name's chars, modulo five
/// — are `Avatar.tsx`'s, so the same name lands on the same color here as it does
/// on the web.
pub struct Avatar<'a> {
    name: Cow<'a, str>,
    size: Size,
    shape: AvatarShape,
    status: Option<AvatarStatus>,
}

impl<'a> Avatar<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            size: Size::default(),
            shape: AvatarShape::default(),
            status: None,
        }
    }

    /// 24 / 32 / 40px.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn shape(mut self, shape: AvatarShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn status(mut self, status: AvatarStatus) -> Self {
        self.status = Some(status);
        self
    }

    fn diameter(&self) -> f32 {
        match self.size {
            Size::Small => 24.0,
            Size::Medium => 32.0,
            Size::Large => 40.0,
        }
    }

    /// First letters of the first two words, uppercased — `getInitials`.
    fn initials(&self) -> String {
        let mut words = self.name.split_whitespace();
        let mut out = String::new();
        if let Some(first) = words.next().and_then(|w| w.chars().next()) {
            out.extend(first.to_uppercase());
        }
        if let Some(second) = words.next().and_then(|w| w.chars().next()) {
            out.extend(second.to_uppercase());
        }
        out
    }

    /// `getColorToken`: the char-code sum, modulo the five tones.
    ///
    /// The first tone is `accent.mark` rather than `accent.default`, because these
    /// five have to stay five: the CSS's list is `accent-default` followed by
    /// `azure-60`, and the dark theme's retuned accent fill *is* `azure-60` — which
    /// would collapse two of the tones into one. `mark` is the CSS's
    /// `accent-default` in both palettes, so this is the faithful reading as well
    /// as the distinct one.
    fn tone(&self, t: &Theme) -> Color32 {
        let tones = [
            t.accent.mark,
            palette::AZURE_60,
            palette::GREEN_60,
            palette::AMBER_60,
            palette::RED_60,
        ];
        if self.name.is_empty() {
            return tones[0];
        }
        let sum: u32 = self.name.chars().map(|c| c as u32).sum();
        tones[(sum % tones.len() as u32) as usize]
    }
}

impl Widget for Avatar<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let d = self.diameter();
        let (rect, response) = ui.allocate_exact_size(vec2(d, d), Sense::hover());
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let corner = match self.shape {
            AvatarShape::Circle => CornerRadius::same(radius::CIRCULAR),
            AvatarShape::Square => CornerRadius::same(radius::MEDIUM),
        };
        ui.painter().rect_filled(rect, corner, self.tone(&t));

        let font = crate::style::strong(if self.size == Size::Large {
            typography::SIZE_SUBTITLE
        } else {
            typography::SIZE_CAPTION
        });
        let galley = ui
            .painter()
            .layout_no_wrap(self.initials(), font, t.text.on_accent);
        ui.painter().galley(
            pos2(
                rect.center().x - galley.size().x * 0.5,
                rect.center().y - galley.size().y * 0.5,
            ),
            galley,
            t.text.on_accent,
        );

        if let Some(status) = self.status {
            // 6 / 8 / 10px, ringed in `background-layer` so it reads on any tone.
            let dot = match self.size {
                Size::Small => 6.0,
                Size::Medium => 8.0,
                Size::Large => 10.0,
            };
            let center = pos2(rect.right() - dot * 0.5, rect.bottom() - dot * 0.5);
            ui.painter()
                .circle_filled(center, dot * 0.5 + 2.0, t.bg.layer);
            ui.painter()
                .circle_filled(center, dot * 0.5, status.color(&t));
        }
        response
    }
}

/// Azur's `Breadcrumb`: a trail of links ending in the current place.
///
/// Returns the index of a clicked ancestor, if one was clicked.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::Breadcrumb;
/// if let Some(i) = Breadcrumb::new(["Projects", "Site survey", "Scan 4"]).show(ui) {
///     let _ = i;
/// }
/// # }
/// ```
pub struct Breadcrumb<'a> {
    items: Vec<Cow<'a, str>>,
    max_item_width: f32,
}

impl<'a> Breadcrumb<'a> {
    pub fn new<I, S>(items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Cow<'a, str>>,
    {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            // `.link { max-width: 240px }`.
            max_item_width: 240.0,
        }
    }

    pub fn max_item_width(mut self, width: f32) -> Self {
        self.max_item_width = width;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Option<usize> {
        let t = theme_of(ui);
        let last = self.items.len().saturating_sub(1);
        let mut clicked = None;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::S2;
            for (i, item) in self.items.iter().enumerate() {
                let current = i == last;
                let (font, color) = if current {
                    (t.fonts.body_strong.clone(), t.text.primary)
                } else {
                    (t.fonts.body.clone(), t.text.secondary)
                };
                let galley = truncated(ui, item, font.clone(), color, self.max_item_width);
                let (rect, response) = ui.allocate_exact_size(
                    vec2(galley.size().x, typography::LINE_BODY),
                    if current {
                        Sense::hover()
                    } else {
                        Sense::click()
                    },
                );

                if !current && response.hovered() {
                    // `:hover { color: text-primary; text-decoration: underline }`.
                    let galley = truncated(ui, item, font, t.text.primary, self.max_item_width);
                    let w = galley.size().x;
                    super::galley_left_center(ui, rect, galley);
                    let y = rect.center().y + typography::LINE_BODY * 0.3;
                    ui.painter().line_segment(
                        [pos2(rect.left(), y), pos2(rect.left() + w, y)],
                        Stroke::new(1.0, t.text.primary),
                    );
                } else {
                    super::galley_left_center(ui, rect, galley);
                }
                if response.clicked() {
                    clicked = Some(i);
                }
                if response.has_focus() {
                    focus_ring(
                        ui.painter(),
                        rect,
                        CornerRadius::same(radius::SMALL),
                        t.stroke.focus,
                    );
                }

                if !current {
                    let (sep, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                    chevron_right(ui.painter(), sep, t.text.tertiary);
                }
            }
        });
        clicked
    }
}

/// One slot of a [`CommandBar`].
pub enum CommandItem<'a> {
    /// A 32px icon button. `active` is the latched state.
    Button {
        icon: Icon<'a>,
        tooltip: Cow<'a, str>,
        active: bool,
        enabled: bool,
    },
    /// A 1×20px rule.
    Divider,
}

impl<'a> CommandItem<'a> {
    pub fn button(icon: Icon<'a>, tooltip: impl Into<Cow<'a, str>>) -> Self {
        Self::Button {
            icon,
            tooltip: tooltip.into(),
            active: false,
            enabled: true,
        }
    }

    pub fn active(mut self, value: bool) -> Self {
        if let Self::Button { active, .. } = &mut self {
            *active = value;
        }
        self
    }

    pub fn enabled(mut self, value: bool) -> Self {
        if let Self::Button { enabled, .. } = &mut self {
            *enabled = value;
        }
        self
    }
}

/// Azur's `CommandBar`: a strip of icon buttons on `background-layer-alt`.
///
/// Returns the index of the item that was clicked, counting dividers.
pub struct CommandBar<'a> {
    items: Vec<CommandItem<'a>>,
    full_width: bool,
}

impl<'a> CommandBar<'a> {
    pub fn new<I: IntoIterator<Item = CommandItem<'a>>>(items: I) -> Self {
        Self {
            items: items.into_iter().collect(),
            full_width: false,
        }
    }

    /// Stretch across the window, as a toolbar under a title bar does. Azur's own
    /// is `width: fit-content`.
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Option<usize> {
        let t = theme_of(ui);
        let mut clicked = None;
        let frame = egui::Frame::new()
            .fill(t.bg.layer_alt)
            .corner_radius(CornerRadius::same(radius::MEDIUM))
            .inner_margin(egui::Margin::same(space::S1 as i8));

        let response = frame.show(ui, |ui| {
            if self.full_width {
                let w = ui.available_width();
                ui.set_min_width(w);
            }
            ui.spacing_mut().item_spacing.x = space::S1;
            ui.horizontal(|ui| {
                for (i, item) in self.items.iter().enumerate() {
                    match item {
                        CommandItem::Divider => {
                            ui.add_space(space::S1);
                            let (rect, _) = ui.allocate_exact_size(vec2(1.0, 20.0), Sense::hover());
                            ui.painter()
                                .rect_filled(rect, CornerRadius::ZERO, t.stroke.subtle);
                            ui.add_space(space::S1);
                        }
                        CommandItem::Button {
                            icon,
                            tooltip,
                            active,
                            enabled,
                        } => {
                            let (rect, response) = ui.allocate_exact_size(
                                vec2(32.0, 32.0),
                                if *enabled {
                                    Sense::click()
                                } else {
                                    Sense::hover()
                                },
                            );
                            let corner = CornerRadius::same(radius::SMALL);
                            let fill = if !*enabled {
                                None
                            } else if *active {
                                Some(if response.hovered() {
                                    t.accent.subtle_hover
                                } else {
                                    t.accent.subtle
                                })
                            } else if response.is_pointer_button_down_on() {
                                Some(t.bg.control_active)
                            } else if response.hovered() {
                                Some(t.bg.control_hover)
                            } else {
                                None
                            };
                            if let Some(fill) = fill {
                                ui.painter().rect_filled(rect, corner, fill);
                            }
                            let color = if !*enabled {
                                t.text.disabled
                            } else if *active {
                                t.accent.mark
                            } else {
                                t.text.primary
                            };
                            icon(
                                ui.painter(),
                                Rect::from_center_size(rect.center(), vec2(16.0, 16.0)),
                                color,
                            );
                            if response.has_focus() {
                                focus_ring(ui.painter(), rect, corner, t.stroke.focus);
                            }
                            if response.clicked() {
                                clicked = Some(i);
                            }
                            if !tooltip.is_empty() {
                                super::tooltip(response, tooltip);
                            }
                        }
                    }
                }
            });
        });

        // `border-bottom: 1px stroke-subtle`.
        let rect = response.response.rect;
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, t.stroke.subtle),
        );
        clicked
    }
}

/// One entry of a [`NavigationRail`].
pub struct NavItem<'a> {
    pub label: Cow<'a, str>,
    pub icon: Option<Icon<'a>>,
    /// A count in an accent pill, or `""` for none. `"•"` gets the dot form.
    pub badge: Cow<'a, str>,
}

impl<'a> NavItem<'a> {
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            badge: Cow::Borrowed(""),
        }
    }

    pub fn icon(mut self, icon: Icon<'a>) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn badge(mut self, badge: impl Into<Cow<'a, str>>) -> Self {
        self.badge = badge.into();
        self
    }
}

/// The collapsed rail, which is also the width of the icon column in both states.
const RAIL_COLLAPSED_W: f32 = 48.0;
/// The rail with its labels showing.
const RAIL_EXPANDED_W: f32 = 220.0;

/// Azur's `NavigationRail`: the 220px sidebar, or 48px collapsed.
///
/// Meant for a left [`egui::Panel`], which is what sizes it:
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::{NavItem, NavigationRail};
/// let mut section = 0;
/// let rail = NavigationRail::new(&mut section);
/// egui::Panel::left("nav").exact_size(rail.width()).show(ui, |ui| {
///     ui.add(rail.items([
///         NavItem::new("Scans"),
///         NavItem::new("Registration").badge("2"),
///     ]));
/// });
/// # }
/// ```
pub struct NavigationRail<'a> {
    selected: &'a mut usize,
    items: Vec<NavItem<'a>>,
    collapsed: bool,
    toggle: Option<(Icon<'a>, Cow<'a, str>, &'a mut bool)>,
}

impl<'a> NavigationRail<'a> {
    pub fn new(selected: &'a mut usize) -> Self {
        Self {
            selected,
            items: Vec::new(),
            collapsed: false,
            toggle: None,
        }
    }

    pub fn items<I: IntoIterator<Item = NavItem<'a>>>(mut self, items: I) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    /// Icons only, at 48px wide.
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// Give the rail its own collapse control, as the row above its entries.
    ///
    /// `clicked` is set for the frame the row is pressed; the caller owns the
    /// collapsed flag and flips it. The row is drawn like an entry but is never
    /// selected — it does not go anywhere, so nothing about it should suggest a
    /// destination.
    ///
    /// The control belongs to the rail rather than to a toolbar beside it: it is
    /// the rail that changes, and a reader looking for it looks at the thing it
    /// acts on.
    ///
    /// `tooltip` is not optional, for the reason [`CommandItem::button`]'s is not:
    /// this row is a glyph and nothing else in *both* states, so the tooltip is the
    /// only place it says what it does. The caller writes it because only the caller
    /// knows which way it will go — "Collapse the sidebar" while the labels show,
    /// "Expand" once they are gone.
    pub fn toggle(
        mut self,
        icon: Icon<'a>,
        tooltip: impl Into<Cow<'a, str>>,
        clicked: &'a mut bool,
    ) -> Self {
        self.toggle = Some((icon, tooltip.into(), clicked));
        self
    }

    /// The width Azur gives the rail in this state: 220px, or 48px collapsed.
    pub fn width(&self) -> f32 {
        if self.collapsed {
            RAIL_COLLAPSED_W
        } else {
            RAIL_EXPANDED_W
        }
    }
}

/// Where a rail entry's 20px glyph goes, measured from the row's left edge.
///
/// The same column in both states, which is the point of it. A rail is toggled in
/// front of the reader, and an icon that jumps sideways as the labels arrive reads
/// as the whole column being redrawn rather than as one panel getting wider.
///
/// The offset is the one that centres a glyph in the *collapsed* rail, so the
/// expanded rail carries its icons two pixels further from the edge than
/// `.item { padding-left: space-3 }` asks for. That is the right way round: the
/// collapsed rail is nothing but this column, so it is the state where centring is
/// something a reader can actually see, while in the expanded rail the icon is read
/// against the label beside it and 12px from the edge looks no different from 14.
fn rail_icon_x(row: Rect, pad_x: f32, icon_box: f32) -> f32 {
    row.left() + (RAIL_COLLAPSED_W - pad_x * 2.0 - icon_box) * 0.5
}

impl Widget for NavigationRail<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let collapsed = self.collapsed;
        // `padding: space-3 space-2`, `gap: space-1`. One horizontal padding for
        // both states rather than the tighter `space-1` collapsed: the selected
        // entry's indicator is drawn on the row's own left edge, so a padding that
        // narrows with the rail slides the indicator sideways every time the rail
        // is toggled.
        let pad_x = space::S2;

        let response = ui
            .vertical(|ui| {
                ui.add_space(space::S3);
                ui.spacing_mut().item_spacing.y = space::S1;
                let width = ui.available_width() - pad_x * 2.0;
                // `NavigationRail.module.css` puts `radius-medium` on an entry. A row
                // in a panel takes `radius-small` here, the same as a `ListRow` and a
                // `TreeView` node, and for the reason
                // [`super::control_radius`] gives: 6px on a 36px row that runs the
                // width of the rail reads as a floating pill rather than a band of the
                // panel. It also stops the 3px selection indicator meeting a curve.
                let corner = CornerRadius::same(radius::SMALL);

                // The collapse control, above the destinations. Shaped like an
                // entry so the icons line up in one column, but drawn in
                // `text-secondary` and never selected: it is a control, not a
                // place, and the extra air below it says where the list starts.
                if let Some((icon, tip, clicked)) = self.toggle.take() {
                    ui.horizontal(|ui| {
                        ui.add_space(pad_x);
                        let (rect, response) =
                            ui.allocate_exact_size(vec2(width, row::NAV), Sense::click());
                        if response.clicked() {
                            *clicked = true;
                        }
                        if response.is_pointer_button_down_on() {
                            ui.painter().rect_filled(rect, corner, t.bg.control_active);
                        } else if response.hovered() {
                            ui.painter().rect_filled(rect, corner, t.bg.control_hover);
                        }
                        let icon_box = 20.0;
                        icon(
                            ui.painter(),
                            super::icon_rect(rect, rail_icon_x(rect, pad_x, icon_box), icon_box),
                            t.text.secondary,
                        );
                        if response.has_focus() {
                            focus_ring_inset(ui.painter(), rect, corner, t.stroke.focus);
                        }
                        super::tooltip(response, &tip);
                    });
                    ui.add_space(space::S2);
                }

                for (i, item) in self.items.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add_space(pad_x);
                        let (rect, response) =
                            ui.allocate_exact_size(vec2(width, row::NAV), Sense::click());
                        let selected = *self.selected == i;
                        if response.clicked() {
                            *self.selected = i;
                        }
                        if !ui.is_rect_visible(rect) {
                            return;
                        }

                        if selected {
                            ui.painter().rect_filled(rect, corner, t.accent.subtle);
                        } else if response.is_pointer_button_down_on() {
                            ui.painter().rect_filled(rect, corner, t.bg.control_active);
                        } else if response.hovered() {
                            ui.painter().rect_filled(rect, corner, t.bg.control_hover);
                        }
                        // `.indicator { left: 0; top/bottom: 6px; width: 3px }`.
                        if selected {
                            ui.painter().rect_filled(
                                Rect::from_min_max(
                                    pos2(rect.left(), rect.top() + 6.0),
                                    pos2(rect.left() + 3.0, rect.bottom() - 6.0),
                                ),
                                CornerRadius::same(radius::CIRCULAR),
                                t.accent.mark,
                            );
                        }

                        let fg = if selected {
                            t.accent.mark
                        } else {
                            t.text.primary
                        };
                        let icon_box = 20.0;
                        let icon_x = rail_icon_x(rect, pad_x, icon_box);
                        let mut x = rect.left() + space::S3;
                        if collapsed {
                            if let Some(icon) = item.icon {
                                icon(ui.painter(), super::icon_rect(rect, icon_x, icon_box), fg);
                            }
                        } else {
                            if let Some(icon) = item.icon {
                                icon(ui.painter(), super::icon_rect(rect, icon_x, icon_box), fg);
                                x = icon_x + icon_box + space::S3;
                            }
                            let mut right = rect.right() - space::S3;
                            if !item.badge.is_empty() {
                                right -= badge_pill(ui, &t, rect, right, &item.badge);
                                right -= space::S2;
                            }
                            let galley = truncated(
                                ui,
                                &item.label,
                                t.fonts.body.clone(),
                                fg,
                                (right - x).max(0.0),
                            );
                            super::galley_left_center(
                                ui,
                                Rect::from_min_max(pos2(x, rect.top()), pos2(right, rect.bottom())),
                                galley,
                            );
                        }
                        if response.has_focus() {
                            focus_ring_inset(ui.painter(), rect, corner, t.stroke.focus);
                        }
                        // Collapsed, the row is a bare glyph: the label it would
                        // have carried is the tooltip, so a rail whose icons a
                        // reader does not yet recognise is still readable. Not in
                        // the expanded state -- repeating a label the pointer is
                        // already resting on is noise, and it would cover the entry
                        // below it.
                        if collapsed {
                            super::tooltip(response, &item.label);
                        }
                    });
                }
                ui.add_space(space::S3);
            })
            .response;

        // `border-right: 1px stroke-subtle`.
        let rect = ui.max_rect();
        ui.painter().line_segment(
            [rect.right_top(), rect.right_bottom()],
            Stroke::new(1.0, t.stroke.subtle),
        );
        response
    }
}

/// The count pill on a nav entry. Returns its width.
fn badge_pill(ui: &Ui, t: &Theme, row: Rect, right: f32, text: &str) -> f32 {
    if text == "•" {
        ui.painter()
            .circle_filled(pos2(right - 4.0, row.center().y), 4.0, t.accent.mark);
        return 8.0;
    }
    let font = crate::style::strong(typography::SIZE_CAPTION);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font, t.text.on_accent);
    // `min-width: 18px; height: 18px; padding: 0 space-2`.
    let w = (galley.size().x + space::S2 * 2.0).max(18.0);
    let rect = Rect::from_min_size(pos2(right - w, row.center().y - 9.0), vec2(w, 18.0));
    ui.painter()
        .rect_filled(rect, CornerRadius::same(radius::CIRCULAR), t.accent.default);
    ui.painter().galley(
        pos2(
            rect.center().x - galley.size().x * 0.5,
            rect.center().y - galley.size().y * 0.5,
        ),
        galley,
        t.text.on_accent,
    );
    w
}

/// What a [`TitleBar`] reports back.
///
/// `Maximize` and `Restore` come from the middle button *and* from a double-click
/// on the caption — the two mean the same thing, and which one you get depends on
/// [`TitleBar::maximized`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TitleBarAction {
    Minimize,
    Maximize,
    Restore,
    Close,
    /// The caption was dragged — move the window.
    Drag,
}

/// Azur's `TitleBar`: 32px of chrome with the window buttons at the end.
///
/// Meant for a top [`egui::Panel`] with no frame of its own, in a viewport created
/// with `with_decorations(false)`. It *reports* what the user asked for and does
/// none of it — moving, maximizing and closing a window are the caller's to send,
/// since only the caller knows which viewport it means.
///
/// All five actions have to be wired or the window loses behaviour the platform
/// used to provide for free. Whole thing, including the resize borders that go with
/// a decorationless window:
///
/// ```no_run
/// use azur_egui_theme::components::{resize_handles, TitleBar, TitleBarAction};
/// use egui::ViewportCommand as Cmd;
/// # fn demo(ui: &mut egui::Ui) {
/// let ctx = ui.ctx().clone();
/// // The window manager owns this, not you: it also changes under Win+Up or a
/// // snap, so a local flag would drift out of sync.
/// let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
///
/// egui::Panel::top("title")
///     .exact_size(32.0)
///     .frame(egui::Frame::NONE)
///     .show(ui, |ui| {
///         match TitleBar::new("Untitled — Azur").maximized(maximized).show(ui) {
///             Some(TitleBarAction::Close) => ctx.send_viewport_cmd(Cmd::Close),
///             Some(TitleBarAction::Minimize) => ctx.send_viewport_cmd(Cmd::Minimized(true)),
///             Some(TitleBarAction::Maximize) => ctx.send_viewport_cmd(Cmd::Maximized(true)),
///             Some(TitleBarAction::Restore) => ctx.send_viewport_cmd(Cmd::Maximized(false)),
///             Some(TitleBarAction::Drag) => {
///                 // Dragging a maximized window restores it first, as the platform
///                 // does — otherwise the drag has nowhere to go.
///                 if maximized {
///                     ctx.send_viewport_cmd(Cmd::Maximized(false));
///                 }
///                 ctx.send_viewport_cmd(Cmd::StartDrag);
///             }
///             None => {}
///         }
///     });
///
/// resize_handles(&ctx); // after the panels
/// # }
/// ```
pub struct TitleBar<'a> {
    title: Cow<'a, str>,
    icon: Option<Icon<'a>>,
    maximized: bool,
    controls: bool,
}

impl<'a> TitleBar<'a> {
    pub fn new(title: impl Into<Cow<'a, str>>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            maximized: false,
            controls: true,
        }
    }

    /// A 16px app icon before the title.
    pub fn icon(mut self, icon: Icon<'a>) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Swap the maximize button for restore.
    pub fn maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    /// Draw the minimize / maximize / close buttons. Off for a window whose
    /// decorations the platform is drawing.
    pub fn controls(mut self, controls: bool) -> Self {
        self.controls = controls;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Option<TitleBarAction> {
        let t = theme_of(ui);
        let height = row::TITLE_BAR;
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let mut action = None;

        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, t.bg.layer);
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, t.stroke.subtle),
        );

        // `.leading { padding: 0 space-3; gap: space-2 }`.
        let mut x = rect.left() + space::S3;
        if let Some(icon) = self.icon {
            icon(
                ui.painter(),
                super::icon_rect(rect, x, 16.0),
                t.text.secondary,
            );
            x += 16.0 + space::S2;
        }

        // The window buttons, from the right: 46 × 32 each, with a 10px glyph —
        // the size the platform draws its own caption buttons at, so these read
        // the same weight as every other window on the desktop.
        const CONTROL_WIDTH: f32 = 46.0;
        const GLYPH: f32 = 10.0;

        let mut controls_left = rect.right();
        if self.controls {
            let buttons: [(TitleBarAction, Icon<'_>); 3] = [
                (TitleBarAction::Close, &window_close),
                (
                    if self.maximized {
                        TitleBarAction::Restore
                    } else {
                        TitleBarAction::Maximize
                    },
                    if self.maximized {
                        &window_restore
                    } else {
                        &window_maximize
                    },
                ),
                (TitleBarAction::Minimize, &window_minimize),
            ];
            for (which, icon) in buttons {
                controls_left -= CONTROL_WIDTH;
                let button = Rect::from_min_size(
                    pos2(controls_left.round(), rect.top()),
                    vec2(CONTROL_WIDTH, height),
                );
                let response = ui.interact(
                    button,
                    ui.id().with(("azur-titlebar", which as u8)),
                    Sense::click(),
                );
                let danger = which == TitleBarAction::Close;
                let (fill, fg) = if response.hovered() {
                    if danger {
                        (Some(crate::desktop::CLOSE_FILL), crate::desktop::CLOSE_INK)
                    } else {
                        (Some(t.bg.control_hover), t.text.primary)
                    }
                } else {
                    (None, t.text.secondary)
                };
                if let Some(fill) = fill {
                    ui.painter().rect_filled(button, CornerRadius::ZERO, fill);
                }
                icon(
                    ui.painter(),
                    Rect::from_center_size(button.center(), vec2(GLYPH, GLYPH)),
                    fg,
                );
                if response.has_focus() {
                    focus_ring_inset(ui.painter(), button, CornerRadius::ZERO, t.stroke.focus);
                }
                if response.clicked() {
                    action = Some(which);
                }
            }
        }

        let title_right = controls_left - space::S3;
        let galley = truncated(
            ui,
            &self.title,
            t.fonts.body.clone(),
            t.text.secondary,
            (title_right - x).max(0.0),
        );
        super::galley_left_center(
            ui,
            Rect::from_min_max(pos2(x, rect.top()), pos2(title_right, rect.bottom())),
            galley,
        );

        // The whole caption drags the window — the title text included, which is
        // what every other window on the desktop does. Only the buttons are
        // excluded, and they were interacted with first so they win the pointer.
        let caption = Rect::from_min_max(rect.min, pos2(controls_left, rect.bottom()));
        if caption.width() > 0.0 && action.is_none() {
            let response = ui.interact(
                caption,
                ui.id().with("azur-titlebar-caption"),
                Sense::click_and_drag(),
            );
            // Double-click before drag: a double-click also reports a drag start,
            // and reading them in the other order turns every double-click into a
            // one-pixel window move.
            if response.double_clicked() {
                action = Some(if self.maximized {
                    TitleBarAction::Restore
                } else {
                    TitleBarAction::Maximize
                });
            } else if response.drag_started() {
                action = Some(TitleBarAction::Drag);
            }
        }
        action
    }
}

/// Give a borderless window its resize borders.
///
/// A viewport created with `with_decorations(false)` — which is what using
/// [`TitleBar`] implies — loses the frame the platform would have given it, and
/// with it the edges you drag to resize. This puts them back: eight invisible
/// strips around the viewport that show the right cursor and hand the drag to the
/// window manager.
///
/// Call it once per frame, after your panels. Nothing to wire up:
///
/// ```no_run
/// # fn demo(ctx: &egui::Context) {
/// azur_egui_theme::components::resize_handles(ctx);
/// # }
/// ```
///
/// It does nothing while the window is maximized, since there is nothing to drag
/// an edge into.
pub fn resize_handles(ctx: &egui::Context) {
    use egui::{CursorIcon, ResizeDirection as Dir};

    if ctx.input(|i| i.viewport().maximized == Some(true) || i.viewport().fullscreen == Some(true))
    {
        return;
    }

    // Windows' own invisible resize border is about this wide, and the corners are
    // a larger square so that "resize both axes" is reachable.
    const EDGE: f32 = 6.0;
    const CORNER: f32 = 16.0;

    let r = ctx.viewport_rect();
    let (l, t, rt, b) = (r.left(), r.top(), r.right(), r.bottom());

    // The corners own the first `CORNER` of each side and the edges pick up from
    // there, so no two zones overlap and none has to win a fight for the pointer.
    let zones: [(Rect, Dir, CursorIcon); 8] = [
        (
            Rect::from_min_size(pos2(l, t), vec2(CORNER, CORNER)),
            Dir::NorthWest,
            CursorIcon::ResizeNorthWest,
        ),
        (
            Rect::from_min_size(pos2(rt - CORNER, t), vec2(CORNER, CORNER)),
            Dir::NorthEast,
            CursorIcon::ResizeNorthEast,
        ),
        (
            Rect::from_min_size(pos2(l, b - CORNER), vec2(CORNER, CORNER)),
            Dir::SouthWest,
            CursorIcon::ResizeSouthWest,
        ),
        (
            Rect::from_min_size(pos2(rt - CORNER, b - CORNER), vec2(CORNER, CORNER)),
            Dir::SouthEast,
            CursorIcon::ResizeSouthEast,
        ),
        (
            Rect::from_min_max(pos2(l + CORNER, t), pos2(rt - CORNER, t + EDGE)),
            Dir::North,
            CursorIcon::ResizeNorth,
        ),
        (
            Rect::from_min_max(pos2(l + CORNER, b - EDGE), pos2(rt - CORNER, b)),
            Dir::South,
            CursorIcon::ResizeSouth,
        ),
        (
            Rect::from_min_max(pos2(l, t + CORNER), pos2(l + EDGE, b - CORNER)),
            Dir::West,
            CursorIcon::ResizeWest,
        ),
        (
            Rect::from_min_max(pos2(rt - EDGE, t + CORNER), pos2(rt, b - CORNER)),
            Dir::East,
            CursorIcon::ResizeEast,
        ),
    ];

    for (i, (zone, direction, cursor)) in zones.into_iter().enumerate() {
        if zone.width() <= 0.0 || zone.height() <= 0.0 {
            continue;
        }
        // `Middle` puts the strips over the panels but under the popups, so an edge
        // never steals the pointer from a menu that reaches the window border.
        let response = egui::Area::new(egui::Id::new(("azur-resize", i)))
            .order(egui::Order::Middle)
            .fixed_pos(zone.min)
            .interactable(true)
            .show(ctx, |ui| {
                ui.allocate_response(zone.size(), Sense::click_and_drag())
            })
            .inner;

        if response.hovered() || response.dragged() {
            ctx.set_cursor_icon(cursor);
        }
        if response.drag_started() {
            ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(direction));
        }
    }
}

/// Paint a `stroke-subtle` bottom edge under a panel — what Azur gives its
/// `TitleBar` and `CommandBar`, for chrome you build yourself.
pub fn chrome_edge(ui: &Ui, rect: Rect) {
    let t = theme_of(ui);
    ui.painter().rect_stroke(
        Rect::from_min_max(rect.left_bottom() - vec2(0.0, 1.0), rect.right_bottom()),
        CornerRadius::ZERO,
        Stroke::new(1.0, t.stroke.subtle),
        StrokeKind::Inside,
    );
}
