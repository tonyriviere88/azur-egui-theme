//! `Button`, `IconButton`, `Card`, `Divider`.

use super::{darken, theme_of, Size, Variant};
use crate::icons::{focus_ring, Icon};
use crate::theme::Theme;
use crate::tokens::{radius, shadow, space};
use egui::{
    pos2, vec2, Color32, CornerRadius, InnerResponse, Rect, Response, Sense, Stroke, StrokeKind,
    Ui, Widget,
};
use std::borrow::Cow;

/// The fill, border and foreground of a button in one state.
struct Skin {
    fill: Color32,
    border: Color32,
    fg: Color32,
}

/// Resolve `Button.module.css` / `IconButton.module.css` for one state.
///
/// `pressed` is `IconButton`'s toggle state — a latched tool button — which Azur
/// paints in the accent-subtle ramp regardless of variant.
fn skin(
    t: &Theme,
    variant: Variant,
    enabled: bool,
    hovered: bool,
    down: bool,
    pressed: bool,
) -> Skin {
    if !enabled {
        return Skin {
            fill: t.bg.control_disabled,
            border: Color32::TRANSPARENT,
            fg: t.text.disabled,
        };
    }

    if pressed && variant != Variant::Danger {
        return Skin {
            fill: if down {
                t.accent.subtle_active
            } else if hovered {
                t.accent.subtle_hover
            } else {
                t.accent.subtle
            },
            border: Color32::TRANSPARENT,
            fg: t.accent.mark,
        };
    }

    match variant {
        Variant::Primary => Skin {
            fill: if down {
                t.accent.active
            } else if hovered {
                t.accent.hover
            } else {
                t.accent.default
            },
            border: Color32::TRANSPARENT,
            fg: t.text.on_accent,
        },
        Variant::Danger => Skin {
            // `:active { background: danger-hover; filter: brightness(.92) }` —
            // but taken off `danger_solid`, the rest fill, so pressing sinks
            // below it. Off `danger_hover` the pressed state would land back on
            // the rest color and the button would stop answering.
            fill: if down {
                darken(t.status.danger_solid, 0.92)
            } else if hovered {
                t.status.danger_hover
            } else {
                t.status.danger_solid
            },
            border: Color32::TRANSPARENT,
            fg: t.text.on_accent,
        },
        Variant::Secondary | Variant::Subtle | Variant::Outline => {
            let rest = if variant == Variant::Secondary {
                t.bg.control
            } else {
                Color32::TRANSPARENT
            };
            Skin {
                fill: if down {
                    t.bg.control_active
                } else if hovered {
                    t.bg.control_hover
                } else {
                    rest
                },
                border: if matches!(variant, Variant::Secondary | Variant::Outline) {
                    t.stroke.default
                } else {
                    Color32::TRANSPARENT
                },
                fg: t.text.primary,
            }
        }
    }
}

/// Azur's `Button`.
///
/// Five variants, three sizes, an optional icon either side of the label, and a
/// loading state that swaps the leading icon for a spinner.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::{components::{Button, Variant}, icons};
/// if ui.add(Button::new("Save").variant(Variant::Primary).icon(&icons::check)).clicked() {
///     // ...
/// }
/// # }
/// ```
pub struct Button<'a> {
    text: Cow<'a, str>,
    variant: Variant,
    size: Size,
    icon: Option<Icon<'a>>,
    trailing_icon: Option<Icon<'a>>,
    full_width: bool,
    loading: bool,
    min_width: f32,
}

impl<'a> Button<'a> {
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            text: text.into(),
            variant: Variant::default(),
            size: Size::default(),
            icon: None,
            trailing_icon: None,
            full_width: false,
            loading: false,
            min_width: 0.0,
        }
    }

    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// A leading glyph — see [`crate::icons`].
    pub fn icon(mut self, icon: Icon<'a>) -> Self {
        self.icon = Some(icon);
        self
    }

    /// A trailing glyph, for a button that opens something.
    pub fn trailing_icon(mut self, icon: Icon<'a>) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    /// `.fullWidth { width: 100% }`.
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    /// Replace the leading icon with a spinner. Azur also stops the button
    /// reacting to the pointer, which here means it stops reporting clicks.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// A floor for the button's width, so a row of them can be even.
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = min_width;
        self
    }
}

impl Widget for Button<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled() && !self.loading;
        let font = crate::style::medium(self.size.font_size());
        let icon_size = self.size.font_size();
        let gap = space::S2;

        let galley = ui
            .painter()
            .layout_no_wrap(self.text.to_string(), font, Color32::PLACEHOLDER);

        let leading = self.icon.is_some() || self.loading;
        let mut content_w = galley.size().x;
        if !self.text.is_empty() && (leading || self.trailing_icon.is_some()) {
            content_w += gap * (leading as u8 + self.trailing_icon.is_some() as u8) as f32;
        }
        content_w += (leading as u8 + self.trailing_icon.is_some() as u8) as f32 * icon_size;

        let height = self.size.height();
        let width = if self.full_width {
            ui.available_width()
        } else {
            (content_w + self.size.padding_x() * 2.0).max(self.min_width)
        };

        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let skin = skin(
            &t,
            self.variant,
            enabled,
            response.hovered(),
            response.is_pointer_button_down_on(),
            false,
        );
        let corner = super::control_radius();
        let p = ui.painter();
        p.rect(
            rect,
            corner,
            skin.fill,
            Stroke::new(1.0, skin.border),
            StrokeKind::Inside,
        );

        // `justify-content: center`, with the icons riding the same baseline box.
        let mut x = rect.center().x - content_w * 0.5;
        if leading {
            let r = super::icon_rect(rect, x, icon_size);
            if self.loading {
                super::spinner_arc(ui, r, skin.fg, 2.0);
            } else if let Some(icon) = self.icon {
                icon(p, r, skin.fg);
            }
            x += icon_size + if self.text.is_empty() { 0.0 } else { gap };
        }
        if !self.text.is_empty() {
            let galley = ui.painter().layout_no_wrap(
                self.text.to_string(),
                crate::style::medium(self.size.font_size()),
                skin.fg,
            );
            ui.painter().galley(
                pos2(x, rect.center().y - galley.size().y * 0.5),
                galley.clone(),
                skin.fg,
            );
            x += galley.size().x
                + if self.trailing_icon.is_some() {
                    gap
                } else {
                    0.0
                };
        }
        if let Some(icon) = self.trailing_icon {
            icon(ui.painter(), super::icon_rect(rect, x, icon_size), skin.fg);
        }

        if response.has_focus() {
            focus_ring(ui.painter(), rect, corner, t.stroke.focus);
        }
        response
    }
}

/// Azur's `IconButton`: a square button with a glyph and no label.
///
/// `pressed` is the latched state — Azur paints it in the accent-subtle ramp, so
/// a toolbar can show which tool is active.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::{components::{IconButton, Variant}, icons};
/// let mut grid = true;
/// if ui.add(IconButton::new(&icons::hamburger).variant(Variant::Subtle).pressed(grid)).clicked() {
///     grid = !grid;
/// }
/// # }
/// ```
pub struct IconButton<'a> {
    icon: Icon<'a>,
    variant: Variant,
    size: Size,
    pressed: bool,
    /// A tooltip, since an icon-only button has no label to read.
    tooltip: Option<Cow<'a, str>>,
}

impl<'a> IconButton<'a> {
    pub fn new(icon: Icon<'a>) -> Self {
        Self {
            icon,
            variant: Variant::Subtle,
            size: Size::default(),
            pressed: false,
            tooltip: None,
        }
    }

    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// The latched / toggled state.
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    pub fn tooltip(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(text.into());
        self
    }
}

impl Widget for IconButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let side = self.size.height();
        let (rect, response) = ui.allocate_exact_size(vec2(side, side), Sense::click());

        if ui.is_rect_visible(rect) {
            let skin = skin(
                &t,
                self.variant,
                enabled,
                response.hovered(),
                response.is_pointer_button_down_on(),
                self.pressed,
            );
            let corner = super::control_radius();
            ui.painter().rect(
                rect,
                corner,
                skin.fill,
                Stroke::new(1.0, skin.border),
                StrokeKind::Inside,
            );
            let glyph = self.size.icon_only();
            (self.icon)(
                ui.painter(),
                Rect::from_center_size(rect.center(), vec2(glyph, glyph)),
                skin.fg,
            );
            if response.has_focus() {
                focus_ring(ui.painter(), rect, corner, t.stroke.focus);
            }
        }

        match self.tooltip {
            Some(text) => super::tooltip(response, &text),
            None => response,
        }
    }
}

/// `Card`'s three variants.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CardVariant {
    /// A card that floats: `background-card` and `shadow-8`.
    #[default]
    Elevated,
    /// A card that is drawn: `background-layer` and a `stroke-default` border.
    Outline,
    /// A card that is only a tint: `background-card`, nothing else.
    Filled,
}

/// `Card`'s padding steps.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Padding {
    None,
    /// `space-4` — 12px.
    Small,
    /// `space-5` — 16px.
    #[default]
    Medium,
    /// `space-7` — 24px.
    Large,
}

impl Padding {
    fn px(self) -> i8 {
        match self {
            Self::None => 0,
            Self::Small => space::S4 as i8,
            Self::Medium => space::S5 as i8,
            Self::Large => space::S7 as i8,
        }
    }
}

/// Azur's `Card`: the container everything else sits in.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::Card;
/// Card::new().show(ui, |ui| {
///     ui.label("Contents");
/// });
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Card {
    variant: CardVariant,
    padding: Padding,
    interactive: bool,
    full_width: bool,
}

impl Card {
    pub fn new() -> Self {
        Self {
            full_width: true,
            ..Default::default()
        }
    }

    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// A clickable card: reacts to the pointer and reports clicks.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Stretch to the available width, which is what a `display: block` card
    /// does in CSS. On by default; turn it off to shrink to the content.
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    /// Draw the card and its contents.
    ///
    /// The returned response senses clicks when [`Self::interactive`] is set, and
    /// hover either way.
    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let t = theme_of(ui);
        let mut frame = egui::Frame::new()
            .inner_margin(egui::Margin::same(self.padding.px()))
            .corner_radius(CornerRadius::same(radius::LARGE))
            .begin(ui);

        if self.full_width {
            let w = frame.content_ui.available_width();
            frame.content_ui.set_min_width(w);
        }
        let inner = add(&mut frame.content_ui);

        // The fill depends on hover, so the frame is painted after the content has
        // been measured and the space allocated -- `Prepared` keeps a slot for the
        // background behind everything drawn since.
        let hover = frame.allocate_space(ui);
        let response = if self.interactive {
            ui.interact(hover.rect, hover.id, Sense::click())
        } else {
            hover
        };
        let hovered = self.interactive && response.hovered();
        let down = self.interactive && response.is_pointer_button_down_on();

        let (fill, border, elevation) = match self.variant {
            CardVariant::Elevated => (
                t.bg.card,
                Color32::TRANSPARENT,
                if hovered { shadow::S16 } else { shadow::S8 },
            ),
            CardVariant::Outline => (
                t.bg.layer,
                if hovered {
                    t.stroke.strong
                } else {
                    t.stroke.default
                },
                egui::Shadow::NONE,
            ),
            CardVariant::Filled => (t.bg.card, Color32::TRANSPARENT, egui::Shadow::NONE),
        };
        frame.frame.fill = if down {
            t.bg.control_active
        } else if hovered {
            t.bg.card_hover
        } else {
            fill
        };
        frame.frame.stroke = Stroke::new(1.0, border);
        frame.frame.shadow = elevation;
        frame.paint(ui);

        if self.interactive && response.has_focus() {
            focus_ring(
                ui.painter(),
                response.rect,
                CornerRadius::same(radius::LARGE),
                t.stroke.focus,
            );
        }
        InnerResponse::new(inner, response)
    }
}

/// Azur's `Divider`: a 1px `stroke-subtle` rule.
#[derive(Clone, Copy, Debug)]
pub struct Divider {
    vertical: bool,
    inset: bool,
    length: Option<f32>,
}

impl Default for Divider {
    fn default() -> Self {
        Self::horizontal()
    }
}

impl Divider {
    pub fn horizontal() -> Self {
        Self {
            vertical: false,
            inset: false,
            length: None,
        }
    }

    pub fn vertical() -> Self {
        Self {
            vertical: true,
            inset: false,
            length: None,
        }
    }

    /// Pull the rule in from both ends: `space-5` horizontally, `space-3`
    /// vertically, as Azur's `.inset` does.
    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    /// Fix the length instead of filling the available space.
    pub fn length(mut self, length: f32) -> Self {
        self.length = Some(length);
        self
    }
}

impl Widget for Divider {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let inset = if self.inset {
            if self.vertical {
                space::S3
            } else {
                space::S5
            }
        } else {
            0.0
        };

        let (rect, response) = if self.vertical {
            let h = self.length.unwrap_or_else(|| ui.available_height());
            ui.allocate_exact_size(vec2(1.0, h), Sense::hover())
        } else {
            let w = self.length.unwrap_or_else(|| ui.available_width());
            ui.allocate_exact_size(vec2(w, 1.0), Sense::hover())
        };

        if ui.is_rect_visible(rect) {
            let line = if self.vertical {
                Rect::from_min_max(
                    pos2(rect.center().x.round(), rect.top() + inset),
                    pos2(rect.center().x.round() + 1.0, rect.bottom() - inset),
                )
            } else {
                Rect::from_min_max(
                    pos2(rect.left() + inset, rect.center().y.round()),
                    pos2(rect.right() - inset, rect.center().y.round() + 1.0),
                )
            };
            ui.painter()
                .rect_filled(line, CornerRadius::ZERO, t.stroke.subtle);
        }
        response
    }
}
