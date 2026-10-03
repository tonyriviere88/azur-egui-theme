//! `ProgressBar`, `Spinner`, `Badge`, `MessageBar`, `Toast` / `Toaster`.

use super::{theme_of, truncated, Intent, Size};
use crate::icons::close;
use crate::theme::Theme;
use crate::tokens::{motion, radius, shadow, space, typography};
use egui::{
    pos2, vec2, Align, Color32, CornerRadius, Layout, Rect, Response, Sense, Stroke, Ui, Widget,
};
use std::borrow::Cow;

/// Which token a `ProgressBar` fills with.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ProgressColor {
    #[default]
    Accent,
    Success,
    Warning,
    Danger,
}

impl ProgressColor {
    fn color(self, t: &Theme) -> Color32 {
        match self {
            Self::Accent => t.accent.default,
            Self::Success => t.status.success,
            Self::Warning => t.status.warning,
            Self::Danger => t.status.danger,
        }
    }
}

/// Azur's `ProgressBar`: a pill track with a pill fill.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::ProgressBar;
/// ui.add(ProgressBar::new(0.4));
/// ui.add(ProgressBar::indeterminate());
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ProgressBar {
    /// `0..=1`, or `None` for the indeterminate sweep.
    value: Option<f32>,
    color: ProgressColor,
    /// 4px (`small`) or 6px (`medium`).
    large: bool,
    width: Option<f32>,
}

impl ProgressBar {
    pub fn new(value: f32) -> Self {
        Self {
            value: Some(value),
            color: ProgressColor::default(),
            large: true,
            width: None,
        }
    }

    /// The 35%-wide segment that sweeps across on a 1.4s cycle.
    pub fn indeterminate() -> Self {
        Self {
            value: None,
            color: ProgressColor::default(),
            large: true,
            width: None,
        }
    }

    pub fn color(mut self, color: ProgressColor) -> Self {
        self.color = color;
        self
    }

    /// 4px instead of 6px — Azur's `size="small"`.
    pub fn small(mut self, small: bool) -> Self {
        self.large = !small;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for ProgressBar {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let height = if self.large { 6.0 } else { 4.0 };
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let corner = CornerRadius::same(radius::CIRCULAR);
        ui.painter().rect_filled(rect, corner, t.bg.layer_alt);

        let fill = self.color.color(&t);
        match self.value {
            Some(v) => {
                let w = rect.width() * v.clamp(0.0, 1.0);
                if w > 0.0 {
                    ui.painter().rect_filled(
                        Rect::from_min_size(rect.min, vec2(w, rect.height())),
                        corner,
                        fill,
                    );
                }
            }
            None => {
                // `left: -35% -> 100%` over 1.4s.
                let phase = (ui.input(|i| i.time) as f32 / 1.4).fract();
                let seg = rect.width() * 0.35;
                let left = rect.left() - seg + (rect.width() + seg) * phase;
                let clipped = Rect::from_min_size(pos2(left, rect.top()), vec2(seg, rect.height()))
                    .intersect(rect);
                if clipped.width() > 0.0 {
                    ui.painter().rect_filled(clipped, corner, fill);
                }
                ui.ctx().request_repaint();
            }
        }
        response
    }
}

/// Azur's `Spinner`: a `stroke-subtle` ring with one accent quarter turning.
///
/// egui has a spinner, but it grows and shrinks its arc; Azur's is a fixed
/// three-quarter ring at constant speed.
#[derive(Clone, Copy, Debug, Default)]
pub struct Spinner {
    size: Size,
}

impl Spinner {
    pub fn new() -> Self {
        Self::default()
    }

    /// 16 / 24 / 32px.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    fn diameter(self) -> f32 {
        match self.size {
            Size::Small => 16.0,
            Size::Medium => 24.0,
            Size::Large => 32.0,
        }
    }
}

impl Widget for Spinner {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let d = self.diameter();
        let (rect, response) = ui.allocate_exact_size(vec2(d, d), Sense::hover());
        if ui.is_rect_visible(rect) {
            let width = if self.size == Size::Large { 3.0 } else { 2.0 };
            ui.painter().circle_stroke(
                rect.center(),
                (d - width) * 0.5,
                Stroke::new(width, t.stroke.subtle),
            );
            // 0.8s a turn, per `azur-spinner-spin`.
            super::spinner_arc(ui, rect, t.accent.mark, 0.8);
        }
        response
    }
}

/// `Badge`'s variants: a subtle fill with the solid color as its text.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BadgeVariant {
    /// `background-layer-alt` / `text-secondary`.
    #[default]
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
}

impl BadgeVariant {
    fn colors(self, t: &Theme) -> (Color32, Color32) {
        match self {
            Self::Neutral => (t.bg.layer_alt, t.text.secondary),
            Self::Accent => (t.accent.subtle, t.accent.mark),
            Self::Success => (t.status.success_subtle, t.status.success),
            Self::Warning => (t.status.warning_subtle, t.status.warning),
            Self::Danger => (t.status.danger_subtle, t.status.danger),
        }
    }

    /// The solid color, for the dot form.
    fn solid(self, t: &Theme) -> Color32 {
        match self {
            Self::Neutral => t.text.tertiary,
            _ => self.colors(t).1,
        }
    }
}

/// Azur's `Badge`: a caption-sized pill, or a bare dot.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::{Badge, BadgeVariant};
/// ui.add(Badge::new("Draft"));
/// ui.add(Badge::new("3 errors").variant(BadgeVariant::Danger));
/// ui.add(Badge::dot().variant(BadgeVariant::Success));
/// # }
/// ```
pub struct Badge<'a> {
    text: Cow<'a, str>,
    variant: BadgeVariant,
    /// 16px (`small`) or 20px (`medium`).
    small: bool,
    dot: bool,
}

impl<'a> Badge<'a> {
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::default(),
            small: false,
            dot: false,
        }
    }

    /// The dot form: 6px or 8px of solid color, no label.
    pub fn dot() -> Self {
        Self {
            text: Cow::Borrowed(""),
            variant: BadgeVariant::default(),
            small: false,
            dot: true,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn small(mut self, small: bool) -> Self {
        self.small = small;
        self
    }
}

impl Widget for Badge<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let corner = CornerRadius::same(radius::CIRCULAR);

        if self.dot {
            let d = if self.small { 6.0 } else { 8.0 };
            let (rect, response) = ui.allocate_exact_size(vec2(d, d), Sense::hover());
            if ui.is_rect_visible(rect) {
                ui.painter()
                    .circle_filled(rect.center(), d * 0.5, self.variant.solid(&t));
            }
            return response;
        }

        // `height: space-5 | space-6`, `padding: 0 space-3 | space-4`.
        let (height, pad) = if self.small {
            (space::S5, space::S3)
        } else {
            (space::S6, space::S4)
        };
        let (fill, fg) = self.variant.colors(&t);
        let font = crate::style::medium(typography::SIZE_CAPTION);
        let galley = ui.painter().layout_no_wrap(self.text.to_string(), font, fg);
        let (rect, response) =
            ui.allocate_exact_size(vec2(galley.size().x + pad * 2.0, height), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().rect_filled(rect, corner, fill);
            ui.painter().galley(
                pos2(
                    rect.center().x - galley.size().x * 0.5,
                    rect.center().y - galley.size().y * 0.5,
                ),
                galley,
                fg,
            );
        }
        response
    }
}

/// Azur's `MessageBar`: an inline status block with a 3px intent bar.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::{Intent, MessageBar};
/// ui.add(
///     MessageBar::new("Two scans could not be registered.")
///         .title("Registration incomplete")
///         .intent(Intent::Warning),
/// );
/// # }
/// ```
pub struct MessageBar<'a> {
    description: Cow<'a, str>,
    title: Cow<'a, str>,
    intent: Intent,
    dismissible: bool,
    width: Option<f32>,
}

impl<'a> MessageBar<'a> {
    pub fn new(description: impl Into<Cow<'a, str>>) -> Self {
        Self {
            description: description.into(),
            title: Cow::Borrowed(""),
            intent: Intent::default(),
            dismissible: false,
            width: None,
        }
    }

    /// A `body-strong` line above the description.
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = title.into();
        self
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    /// Show the ✕. The response's `clicked()` is the dismissal.
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for MessageBar<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let width = self.width.unwrap_or_else(|| ui.available_width());
        // `padding: space-4 space-5`, `gap: space-3`, 16px icon.
        let (pad_x, pad_y, gap, icon_box) = (space::S5, space::S4, space::S3, 16.0);
        let close_box = if self.dismissible { 24.0 + gap } else { 0.0 };
        let text_w = (width - pad_x * 2.0 - icon_box - gap - close_box).max(0.0);

        let title = (!self.title.is_empty()).then(|| {
            truncated(
                ui,
                &self.title,
                t.fonts.body_strong.clone(),
                t.text.primary,
                text_w,
            )
        });
        let description = {
            let mut job = egui::text::LayoutJob::single_section(
                self.description.to_string(),
                egui::TextFormat::simple(t.fonts.body.clone(), t.text.secondary),
            );
            job.wrap.max_width = text_w;
            ui.painter().layout_job(job)
        };

        let content_h =
            title.as_ref().map_or(0.0, |g| g.size().y + space::S1) + description.size().y;
        let height = content_h.max(typography::LINE_BODY) + pad_y * 2.0;
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let color = self.intent.color(&t);
        let corner = CornerRadius::same(radius::MEDIUM);
        ui.painter()
            .rect_filled(rect, corner, self.intent.subtle(&t));
        // `border-left: 3px solid <intent>`, inside the rounded corner.
        ui.painter().rect_filled(
            Rect::from_min_size(rect.min, vec2(3.0, rect.height())),
            CornerRadius {
                nw: radius::MEDIUM,
                sw: radius::MEDIUM,
                ne: 0,
                se: 0,
            },
            color,
        );

        let icon_top = rect.top() + pad_y + 1.0; // `.icon { margin-top: 1px }`
        (self.intent.icon())(
            ui.painter(),
            Rect::from_min_size(
                pos2(rect.left() + pad_x, icon_top),
                vec2(icon_box, icon_box),
            ),
            color,
        );

        let x = rect.left() + pad_x + icon_box + gap;
        let mut y = rect.top() + pad_y;
        if let Some(title) = title {
            let h = title.size().y;
            ui.painter().galley(pos2(x, y), title, Color32::PLACEHOLDER);
            y += h + space::S1;
        }
        ui.painter()
            .galley(pos2(x, y), description, Color32::PLACEHOLDER);

        // The ✕ is the only thing here that can be clicked, so it *is* the
        // response when there is one: `clicked()` on a dismissible message bar
        // means dismissed.
        if self.dismissible {
            let r = Rect::from_min_size(
                pos2(rect.right() - pad_x - 24.0, rect.top() + pad_y - 2.0),
                vec2(24.0, 24.0),
            );
            let close_response = ui.interact(r, response.id.with("azur-dismiss"), Sense::click());
            if close_response.hovered() {
                ui.painter().rect_filled(
                    r,
                    CornerRadius::same(radius::SMALL),
                    if close_response.is_pointer_button_down_on() {
                        t.bg.control_active
                    } else {
                        t.bg.control_hover
                    },
                );
            }
            close(
                ui.painter(),
                r.shrink(6.0),
                if close_response.hovered() {
                    t.text.primary
                } else {
                    t.text.secondary
                },
            );
            return close_response;
        }
        response
    }
}

/// Which corner a [`Toaster`] stacks in.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ToasterPosition {
    #[default]
    TopEnd,
    TopStart,
    BottomEnd,
    BottomStart,
}

/// One toast, as handed to [`Toaster::push`].
#[derive(Clone, Debug)]
pub struct Toast {
    pub title: String,
    pub description: String,
    pub intent: Intent,
    /// Seconds on screen, or `None` to stay until dismissed.
    pub duration: Option<f32>,
}

impl Toast {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            title: String::new(),
            description: description.into(),
            intent: Intent::default(),
            duration: Some(5.0),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    /// How long it stays. `None` means until it is closed.
    pub fn duration(mut self, seconds: Option<f32>) -> Self {
        self.duration = seconds;
        self
    }
}

struct Live {
    toast: Toast,
    /// Set the first time it is drawn — `Toaster::push` has no `Context` to ask
    /// for the time.
    shown_at: Option<f64>,
    key: u64,
}

/// Azur's `ToasterProvider`, as something an app owns.
///
/// Keep one beside your state, [`push`](Self::push) into it, and call
/// [`show`](Self::show) once per frame:
///
/// ```no_run
/// # struct App { toaster: azur_egui_theme::components::Toaster }
/// # impl App { fn update(&mut self, ctx: &egui::Context) {
/// use azur_egui_theme::components::{Intent, Toast};
/// self.toaster.push(Toast::new("Export finished").intent(Intent::Success));
/// self.toaster.show(ctx);
/// # } }
/// ```
#[derive(Default)]
pub struct Toaster {
    toasts: Vec<Live>,
    position: ToasterPosition,
    next_key: u64,
}

impl Toaster {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn position(mut self, position: ToasterPosition) -> Self {
        self.position = position;
        self
    }

    pub fn push(&mut self, toast: Toast) {
        self.next_key += 1;
        self.toasts.push(Live {
            toast,
            shown_at: None,
            key: self.next_key,
        });
    }

    /// Drop everything on screen.
    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }

    /// Draw the stack and retire whatever has timed out.
    pub fn show(&mut self, ctx: &egui::Context) {
        if self.toasts.is_empty() {
            return;
        }
        let t = super::theme(ctx);
        let now = ctx.input(|i| i.time);
        let screen = ctx.content_rect();
        // `.viewport { padding: space-5; gap: space-3; width: 360px }`.
        let (pad, gap, card_w) = (space::S5, space::S3, 360.0);

        let (top, right) = match self.position {
            ToasterPosition::TopEnd => (true, true),
            ToasterPosition::TopStart => (true, false),
            ToasterPosition::BottomEnd => (false, true),
            ToasterPosition::BottomStart => (false, false),
        };
        let x = if right {
            screen.right() - pad - card_w
        } else {
            screen.left() + pad
        };

        let mut offset = pad;
        let mut dismissed = Vec::new();
        for live in &mut self.toasts {
            let shown_at = *live.shown_at.get_or_insert(now);
            if let Some(duration) = live.toast.duration {
                if now - shown_at > duration as f64 {
                    dismissed.push(live.key);
                    continue;
                }
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }

            // Azur slides a toast in over `duration-normal`; this fades and lifts
            // it over the same window, which is as close as a painted card gets.
            let age = (now - shown_at) as f32;
            let enter = (age / motion::NORMAL).clamp(0.0, 1.0);
            let eased = motion::cubic_bezier(motion::DECELERATE, enter);
            let rise = (1.0 - eased) * 8.0;

            let pos = if top {
                pos2(x, screen.top() + offset + rise)
            } else {
                pos2(x, screen.bottom() - offset - rise)
            };
            let anchor = if top {
                egui::Align2::LEFT_TOP
            } else {
                egui::Align2::LEFT_BOTTOM
            };

            let area = egui::Area::new(egui::Id::new(("azur-toast", live.key)))
                .order(crate::tokens::layer::TOAST)
                .fixed_pos(pos)
                .pivot(anchor)
                .interactable(true)
                .show(ctx, |ui| {
                    ui.set_width(card_w);
                    ui.multiply_opacity(eased.max(0.05));
                    toast_card(ui, &t, &live.toast)
                });
            if area.inner {
                dismissed.push(live.key);
            }
            offset += area.response.rect.height() + gap;
        }
        self.toasts.retain(|l| !dismissed.contains(&l.key));
    }
}

/// One toast card. Returns whether its ✕ was clicked.
fn toast_card(ui: &mut Ui, t: &Theme, toast: &Toast) -> bool {
    let mut closed = false;
    egui::Frame::new()
        .fill(t.bg.card)
        .stroke(Stroke::new(1.0, t.stroke.subtle))
        .corner_radius(CornerRadius::same(radius::LARGE))
        .shadow(shadow::S16)
        .inner_margin(egui::Margin::symmetric(space::S5 as i8, space::S4 as i8))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(space::S3, space::S1);
            ui.horizontal_top(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                (toast.intent.icon())(ui.painter(), icon_rect, toast.intent.color(t));

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = space::S1;
                    if !toast.title.is_empty() {
                        ui.label(
                            egui::RichText::new(&toast.title)
                                .font(t.fonts.body_strong.clone())
                                .color(t.text.primary),
                        );
                    }
                    if !toast.description.is_empty() {
                        ui.label(
                            egui::RichText::new(&toast.description)
                                .font(t.fonts.body.clone())
                                .color(t.text.secondary),
                        );
                    }
                });

                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    closed = ui
                        .add(
                            super::IconButton::new(&close)
                                .size(Size::Small)
                                .variant(super::Variant::Subtle),
                        )
                        .clicked();
                });
            });
        });
    closed
}
