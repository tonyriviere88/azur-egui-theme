//! `TextField`, `TextArea`, `NumberField`, `SearchBox`.
//!
//! All four are the same object in Azur — a bordered wrapper at one of the three
//! control heights, holding a frameless input — with a different set of affixes.
//! The label above and the message below are part of the component, not the
//! caller's job, which is why these are widgets rather than styling helpers.

use super::{field_stroke, theme_of, Size};
use crate::icons::{close, focus_ring, minus, plus, search, Icon};
use crate::theme::Theme;
use crate::tokens::{radius, space, typography};
use egui::{
    pos2, vec2, Align, Color32, CornerRadius, Layout, Rect, Response, Sense, Shape, Stroke,
    StrokeKind, TextEdit, Ui, UiBuilder, Widget,
};
use std::borrow::Cow;

/// Draw the `.label` row: body copy in the heavier weight, with a danger-colored
/// asterisk when required, `space-2` above the field.
fn label_row(ui: &mut Ui, t: &Theme, label: &str, required: bool, enabled: bool) {
    if label.is_empty() {
        return;
    }
    let mut job = egui::text::LayoutJob::default();
    job.append(
        label,
        0.0,
        egui::TextFormat::simple(t.fonts.body.clone(), t.text_for(enabled)),
    );
    if required {
        job.append(
            " *",
            0.0,
            egui::TextFormat::simple(t.fonts.body.clone(), t.status.danger),
        );
    }
    let galley = ui.painter().layout_job(job);
    let (rect, _) = ui.allocate_exact_size(
        vec2(ui.available_width(), typography::LINE_BODY),
        Sense::hover(),
    );
    super::galley_left_center(ui, rect, galley);
    ui.add_space(space::S2 - ui.spacing().item_spacing.y);
}

/// Draw the `.message` row: caption, `text-secondary` or `danger`.
fn message_row(ui: &mut Ui, t: &Theme, message: &str, error: bool, enabled: bool) {
    if message.is_empty() {
        return;
    }
    ui.add_space(space::S2 - ui.spacing().item_spacing.y);
    let color = if !enabled {
        t.text.disabled
    } else if error {
        t.status.danger
    } else {
        t.text.secondary
    };
    let galley = super::truncated(
        ui,
        message,
        t.fonts.caption.clone(),
        color,
        ui.available_width(),
    );
    let (rect, _) = ui.allocate_exact_size(
        vec2(ui.available_width(), typography::LINE_CAPTION),
        Sense::hover(),
    );
    super::galley_left_center(ui, rect, galley);
}

/// Which of the four rules a field's border is following.
///
/// Passed in rather than worked out, because the two interesting members are only known after the
/// input inside the field has run — see [`field_frame`].
#[derive(Clone, Copy)]
pub struct FieldLook {
    pub enabled: bool,
    /// The input's own `has_focus()`.
    pub focused: bool,
    /// [`field_hovered`], and not a response's `hovered()` — the doc there says why.
    pub hovered: bool,
    pub error: bool,
}

/// Whether the pointer is over the field, for the border's hover state.
///
/// **Geometric, and not the wrapper response's `hovered()`.** A field's wrapper is allocated
/// before the `TextEdit` that goes inside it, so the edit is the topmost widget over the text
/// area and takes the hover from it — which left the border lighting up only where the edit is
/// *not*: over the prefix icon and the padding either side. Hovering the part of the field you
/// actually type in did nothing, which reads as a field that does not respond.
///
/// "Is the pointer inside this rect" is a question about the rect, and `rect_contains_pointer`
/// answers it while still respecting the layer — so a menu or a popup over the field suppresses
/// it, which a bare geometric test of its own would not.
pub fn field_hovered(ui: &Ui, rect: Rect, enabled: bool) -> bool {
    enabled && ui.rect_contains_pointer(rect)
}

/// A field's own fill and border, painted into a slot reserved before the input was drawn — the
/// border depends on focus, which is only known afterwards.
///
/// Public because a *composite* field is an application's to lay out, and this is the part of one
/// that must not be re-derived. [`TextField`] and the rest are whole controls — label, wrapper,
/// input, message — and the set has no component for a field with several controls inside it: an
/// editor's find bar, with match-case, whole-word and regex toggles in the right-hand end of the
/// field, is a composition rather than a component. What it still has to get exactly right is
/// this: one fill, one radius, and the four-state border every other field in the window wears.
/// Reserve the slot with `ui.painter().add(Shape::Noop)` before the input and call this after it,
/// with the input's own `has_focus()`.
///
/// The corner radius comes from the **installed style** rather than from
/// [`super::control_radius`] directly, which is where [`crate::install`] puts that same value.
/// The two are therefore identical by default and this is not a change of appearance; what it
/// buys is that a caller can scope the radius the way egui's own widgets already allow, with
/// `ui.visuals_mut()` around the call. `TextEdit` reads
/// `Style::interact(..).corner_radius`; before this, a `TextField` beside one could not be made
/// to agree with it.
///
/// `widgets.inactive` specifically, and not the state this field is in: a control's corners do
/// not change when the pointer arrives.
pub fn field_frame(ui: &Ui, slot: egui::layers::ShapeIdx, rect: Rect, t: &Theme, state: FieldLook) {
    let FieldLook {
        enabled,
        focused,
        hovered,
        error,
    } = state;
    let fill = if enabled {
        t.bg.control
    } else {
        t.bg.control_disabled
    };
    let stroke = field_stroke(t, enabled, focused, hovered, error);
    ui.painter().set(
        slot,
        Shape::Rect(egui::epaint::RectShape::new(
            rect,
            ui.visuals().widgets.inactive.corner_radius,
            fill,
            Stroke::new(1.0, stroke),
            StrokeKind::Inside,
        )),
    );
}

/// Whether focus has just arrived at `id`, by any route.
///
/// Not `Response::gained_focus`, which misses one of them. A caller that hands a field focus with
/// `request_focus()` *after* the widget has run — which is how a field that opens already focused
/// is written — has that focus granted at the end of the frame, so the widget never sees the
/// transition: on the frame it asked it had no focus, and on the next one it had simply always had
/// it. Remembering last frame's answer per id catches a click, a tab and a request alike.
fn focus_arrived(ctx: &egui::Context, id: egui::Id, has_focus: bool) -> bool {
    let key = id.with("azur-had-focus");
    ctx.data_mut(|d| {
        let had = d.get_temp::<bool>(key).unwrap_or(false);
        d.insert_temp(key, has_focus);
        has_focus && !had
    })
}

/// Select everything in a field, so the next keystroke replaces it.
fn select_all(ctx: &egui::Context, id: egui::Id, text: &str) {
    use egui::text::{CCursor, CCursorRange};

    let Some(mut state) = TextEdit::load_state(ctx, id) else {
        return;
    };
    let end = CCursor::new(text.chars().count());
    state
        .cursor
        .set_char_range(Some(CCursorRange::two(CCursor::new(0), end)));
    state.store(ctx, id);
}

/// A frameless [`TextEdit`] filling `rect`, vertically centred.
///
/// **Gaining focus selects the contents.** A field you have just arrived in is usually a field
/// whose value you mean to replace — a path, a filter, a name — and selecting it makes that one
/// keystroke instead of `Ctrl+A` or a drag. Every way in is covered: see [`focus_arrived`], which
/// is not the obvious `Response::gained_focus` for a reason worth reading.
///
/// It costs nothing to override, which is why it is the default rather than an option: the
/// selection is set once, on the frame focus arrives, so a drag started by that same click still
/// ends up wherever the pointer took it, and every later click behaves normally.
fn inner_edit(
    ui: &mut Ui,
    rect: Rect,
    t: &Theme,
    text: &mut String,
    placeholder: &str,
    password: bool,
    enabled: bool,
) -> Response {
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    let response = child.add_enabled(
        enabled,
        TextEdit::singleline(text)
            .frame(egui::Frame::NONE)
            .password(password)
            .margin(egui::Margin::ZERO)
            .desired_width(rect.width())
            .font(egui::FontSelection::FontId(t.fonts.body.clone()))
            .text_color(t.text_for(enabled)),
    );
    if focus_arrived(ui.ctx(), response.id, response.has_focus()) {
        select_all(ui.ctx(), response.id, text);
    }
    // Azur's placeholder is `text-tertiary`; egui's own hint text is drawn in the
    // weak text color, which this theme points at `text-secondary`. Painting it
    // here keeps the token exact.
    if text.is_empty() && !placeholder.is_empty() && !response.has_focus() {
        let galley = super::truncated(
            ui,
            placeholder,
            t.fonts.body.clone(),
            if enabled {
                t.text.tertiary
            } else {
                t.text.disabled
            },
            rect.width(),
        );
        super::galley_left_center(ui, rect, galley);
    }
    response
}

/// An 18px square button inside a field — Azur's `.clearButton`.
///
/// Takes its id from the wrapper's rather than from the auto-id sequence: it is
/// drawn after the input, which has already moved that sequence on.
fn affix_button(
    ui: &mut Ui,
    rect: Rect,
    t: &Theme,
    icon: Icon<'_>,
    enabled: bool,
    id: egui::Id,
) -> Response {
    let response = ui.interact(rect, id, Sense::click());
    let hovered = enabled && response.hovered();
    if hovered {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(radius::SMALL), t.bg.control_hover);
    }
    let color = if !enabled {
        t.text.disabled
    } else if hovered {
        t.text.primary
    } else {
        t.text.secondary
    };
    icon(ui.painter(), rect.shrink(3.0), color);
    if response.has_focus() {
        focus_ring(
            ui.painter(),
            rect,
            CornerRadius::same(radius::SMALL),
            t.stroke.focus,
        );
    }
    response
}

/// Azur's `TextField`.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::TextField;
/// let mut name = String::new();
/// ui.add(
///     TextField::new(&mut name)
///         .label("Project name")
///         .required(true)
///         .placeholder("Untitled")
///         .message("Shown in the title bar"),
/// );
/// # }
/// ```
pub struct TextField<'a> {
    text: &'a mut String,
    label: Cow<'a, str>,
    placeholder: Cow<'a, str>,
    message: Cow<'a, str>,
    required: bool,
    error: bool,
    password: bool,
    clearable: bool,
    size: Size,
    width: Option<f32>,
    prefix: Option<Icon<'a>>,
    suffix: Option<Icon<'a>>,
    text_lift: f32,
}

impl<'a> TextField<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            label: Cow::Borrowed(""),
            placeholder: Cow::Borrowed(""),
            message: Cow::Borrowed(""),
            required: false,
            error: false,
            password: false,
            clearable: false,
            size: Size::default(),
            width: None,
            prefix: None,
            suffix: None,
            text_lift: 0.0,
        }
    }

    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = label.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// The caption under the field. Turns `danger` with [`Self::error`].
    pub fn message(mut self, message: impl Into<Cow<'a, str>>) -> Self {
        self.message = message.into();
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// Show an ✕ that empties the field once there is something in it.
    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// A leading glyph inside the field.
    pub fn prefix(mut self, icon: Icon<'a>) -> Self {
        self.prefix = Some(icon);
        self
    }

    /// A trailing glyph inside the field.
    pub fn suffix(mut self, icon: Icon<'a>) -> Self {
        self.suffix = Some(icon);
        self
    }

    /// Take `points` off the top of the text and the placeholder, leaving the box where it is.
    ///
    /// **The line-box correction, for a field sitting in a row of things centred on their own ink.**
    /// Text centred in a box is centred on its *line box*, which reserves room under the baseline for
    /// descenders and above the capitals for accents — so a field of mostly-x-height text reads a
    /// point or two low beside a glyph centred on its ink. It is the same correction
    /// [`super::ink_baseline`] makes for text a caller paints itself, and the reason a dense desktop
    /// listing lifts every cell by two points.
    ///
    /// Zero by default, and it stays zero for a field in a form: there the box *is* what the field is
    /// aligned to, and the platform's own fields centre on the line box too. It is worth setting when
    /// the field is one item in a toolbar of icons — where the icons are the thing the text has to
    /// come level with, and they do not care about line boxes.
    ///
    /// The frame, the prefix, the suffix and the ✕ do not move: they are boxes and glyphs, already
    /// centred on the rect, and moving them would be correcting a mistake they are not making.
    pub fn text_lift(mut self, points: f32) -> Self {
        self.text_lift = points;
        self
    }
}

impl Widget for TextField<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let width = self.width.unwrap_or_else(|| ui.available_width());

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            label_row(ui, &t, &self.label, self.required, enabled);

            let slot = ui.painter().add(Shape::Noop);
            let (rect, wrapper) =
                ui.allocate_exact_size(vec2(width, self.size.height()), Sense::hover());

            let pad = self.size.padding_x();
            let icon = super::control_icon(self.size);
            let gap = space::S2;
            let mut left = rect.left() + pad;
            let mut right = rect.right() - pad;

            if let Some(prefix) = self.prefix {
                let r = super::icon_rect(rect, left, icon);
                prefix(ui.painter(), r, t.text.secondary);
                left += icon + gap;
            }
            let show_clear = self.clearable && !self.text.is_empty() && enabled;
            if show_clear {
                right -= 18.0;
            }
            if let Some(suffix) = self.suffix {
                let r = super::icon_rect(rect, right - icon, icon);
                suffix(ui.painter(), r, t.text.secondary);
                right -= icon + gap;
            }

            // The text's rect, which is the field's own minus the padding and the affixes — and
            // lifted, if the caller asked for it. See [`TextField::text_lift`]: the box does not move.
            let inner = Rect::from_min_max(pos2(left, rect.top()), pos2(right, rect.bottom()))
                .translate(vec2(0.0, -self.text_lift));
            let edit = inner_edit(
                ui,
                inner,
                &t,
                self.text,
                &self.placeholder,
                self.password,
                enabled,
            );

            let mut response = edit.clone();
            if show_clear {
                let r = Rect::from_center_size(
                    pos2(rect.right() - pad - 9.0, rect.center().y),
                    vec2(18.0, 18.0),
                );
                if affix_button(ui, r, &t, &close, enabled, wrapper.id.with("azur-clear")).clicked()
                {
                    self.text.clear();
                    response.mark_changed();
                }
            }

            field_frame(
                ui,
                slot,
                rect,
                &t,
                FieldLook {
                    enabled,
                    focused: edit.has_focus(),
                    hovered: field_hovered(ui, rect, enabled),
                    error: self.error,
                },
            );
            message_row(ui, &t, &self.message, self.error, enabled);
            response
        })
        .inner
    }
}

/// Azur's `TextArea`: the same field, several lines tall.
pub struct TextArea<'a> {
    text: &'a mut String,
    label: Cow<'a, str>,
    placeholder: Cow<'a, str>,
    message: Cow<'a, str>,
    required: bool,
    error: bool,
    rows: usize,
    width: Option<f32>,
}

impl<'a> TextArea<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            label: Cow::Borrowed(""),
            placeholder: Cow::Borrowed(""),
            message: Cow::Borrowed(""),
            required: false,
            error: false,
            rows: 3,
            width: None,
        }
    }

    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = label.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn message(mut self, message: impl Into<Cow<'a, str>>) -> Self {
        self.message = message.into();
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// Visible rows, at Azur's body line height.
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for TextArea<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let width = self.width.unwrap_or_else(|| ui.available_width());
        // `padding: space-3 space-4`.
        let (pad_x, pad_y) = (space::S4, space::S3);
        let height = typography::LINE_BODY * self.rows as f32 + pad_y * 2.0;

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            label_row(ui, &t, &self.label, self.required, enabled);

            let slot = ui.painter().add(Shape::Noop);
            let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
            let inner = rect.shrink2(vec2(pad_x, pad_y));

            let mut child = ui.new_child(UiBuilder::new().max_rect(inner));
            let edit = child.add_enabled(
                enabled,
                TextEdit::multiline(self.text)
                    .frame(egui::Frame::NONE)
                    .margin(egui::Margin::ZERO)
                    .desired_width(inner.width())
                    .desired_rows(self.rows)
                    .font(egui::FontSelection::FontId(t.fonts.body.clone()))
                    .text_color(t.text_for(enabled)),
            );
            if self.text.is_empty() && !self.placeholder.is_empty() && !edit.has_focus() {
                let galley = super::truncated(
                    ui,
                    &self.placeholder,
                    t.fonts.body.clone(),
                    if enabled {
                        t.text.tertiary
                    } else {
                        t.text.disabled
                    },
                    inner.width(),
                );
                ui.painter().galley(inner.min, galley, Color32::PLACEHOLDER);
            }

            field_frame(
                ui,
                slot,
                rect,
                &t,
                FieldLook {
                    enabled,
                    focused: edit.has_focus(),
                    hovered: field_hovered(ui, rect, enabled),
                    error: self.error,
                },
            );
            message_row(ui, &t, &self.message, self.error, enabled);
            edit
        })
        .inner
    }
}

/// Azur's `NumberField`: a numeric field with a two-button stepper.
///
/// The text being edited lives in egui's per-widget memory rather than in the
/// caller's `f64`, so a half-typed `"-"` or `"1."` survives until it parses —
/// which is the behaviour of the `<input type="number">` this ports.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::NumberField;
/// let mut tolerance = 0.5;
/// ui.add(NumberField::new(&mut tolerance).label("Tolerance").step(0.1).decimals(2));
/// # }
/// ```
pub struct NumberField<'a> {
    value: &'a mut f64,
    label: Cow<'a, str>,
    message: Cow<'a, str>,
    required: bool,
    error: bool,
    range: std::ops::RangeInclusive<f64>,
    step: f64,
    decimals: usize,
    size: Size,
    width: Option<f32>,
}

impl<'a> NumberField<'a> {
    pub fn new(value: &'a mut f64) -> Self {
        Self {
            value,
            label: Cow::Borrowed(""),
            message: Cow::Borrowed(""),
            required: false,
            error: false,
            range: f64::NEG_INFINITY..=f64::INFINITY,
            step: 1.0,
            decimals: 0,
            size: Size::default(),
            width: None,
        }
    }

    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = label.into();
        self
    }

    pub fn message(mut self, message: impl Into<Cow<'a, str>>) -> Self {
        self.message = message.into();
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    pub fn range(mut self, range: std::ops::RangeInclusive<f64>) -> Self {
        self.range = range;
        self
    }

    /// What one press of the stepper adds.
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for NumberField<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let id = ui.next_auto_id();
        let clamp = |v: f64| v.clamp(*self.range.start(), *self.range.end());
        let format = |v: f64| format!("{:.*}", self.decimals, v);

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            label_row(ui, &t, &self.label, self.required, enabled);

            let slot = ui.painter().add(Shape::Noop);
            let (rect, _) =
                ui.allocate_exact_size(vec2(width, self.size.height()), Sense::hover());

            // `.stepper { width: 26px; border-left: 1px stroke-subtle }`.
            let stepper = Rect::from_min_max(pos2(rect.right() - 26.0, rect.top()), rect.max);
            let field = Rect::from_min_max(rect.min, pos2(stepper.left(), rect.bottom()))
                .shrink2(vec2(self.size.padding_x(), 0.0));

            let mut buffer = ui
                .data(|d| d.get_temp::<String>(id))
                .unwrap_or_else(|| format(*self.value));
            let focused = ui.memory(|m| m.has_focus(id.with("edit")));
            if !focused {
                // Not being typed in: the caller's value is the truth.
                buffer = format(*self.value);
            }

            let mut child = ui.new_child(
                UiBuilder::new()
                    .max_rect(field)
                    .layout(Layout::left_to_right(Align::Center)),
            );
            let edit = child.add_enabled(
                enabled,
                TextEdit::singleline(&mut buffer)
                    .id(id.with("edit"))
                    .frame(egui::Frame::NONE)
                    .margin(egui::Margin::ZERO)
                    .desired_width(field.width())
                    .font(egui::FontSelection::FontId(t.fonts.body.clone()))
                    .text_color(t.text_for(enabled)),
            );

            let mut response = edit.clone();
            if edit.changed() {
                if let Ok(v) = buffer.trim().parse::<f64>() {
                    *self.value = clamp(v);
                    response.mark_changed();
                }
            }
            ui.data_mut(|d| d.insert_temp(id, buffer));

            // Two half-height buttons, `stroke-subtle` between them.
            let half = stepper.height() * 0.5;
            for (i, (icon, delta)) in [(&plus as Icon, self.step), (&minus as Icon, -self.step)]
                .into_iter()
                .enumerate()
            {
                let r = Rect::from_min_size(
                    pos2(stepper.left(), stepper.top() + half * i as f32),
                    vec2(stepper.width(), half),
                );
                let step_response = ui.interact(r, id.with(("step", i)), Sense::click());
                let hovered = enabled && step_response.hovered();
                if hovered || (enabled && step_response.is_pointer_button_down_on()) {
                    ui.painter().rect_filled(
                        r,
                        CornerRadius::ZERO,
                        if step_response.is_pointer_button_down_on() {
                            t.bg.control_active
                        } else {
                            t.bg.control_hover
                        },
                    );
                }
                let color = if !enabled {
                    t.text.disabled
                } else if hovered {
                    t.text.primary
                } else {
                    t.text.secondary
                };
                icon(
                    ui.painter(),
                    Rect::from_center_size(r.center(), vec2(12.0, 12.0)),
                    color,
                );
                if step_response.clicked() {
                    *self.value = clamp(*self.value + delta);
                    response.mark_changed();
                }
            }
            let p = ui.painter();
            p.line_segment(
                [stepper.left_top(), stepper.left_bottom()],
                Stroke::new(1.0, t.stroke.subtle),
            );
            p.line_segment(
                [
                    pos2(stepper.left(), stepper.center().y),
                    pos2(stepper.right(), stepper.center().y),
                ],
                Stroke::new(1.0, t.stroke.subtle),
            );

            field_frame(
                ui,
                slot,
                rect,
                &t,
                FieldLook {
                    enabled,
                    focused: edit.has_focus(),
                    hovered: field_hovered(ui, rect, enabled),
                    error: self.error,
                },
            );
            message_row(ui, &t, &self.message, self.error, enabled);
            response
        })
        .inner
    }
}

/// Azur's `SearchBox`: a field with a magnifier in front and a clear button
/// behind.
pub struct SearchBox<'a> {
    text: &'a mut String,
    placeholder: Cow<'a, str>,
    size: Size,
    width: Option<f32>,
}

impl<'a> SearchBox<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            placeholder: Cow::Borrowed("Search"),
            size: Size::default(),
            width: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for SearchBox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let pad = self.size.padding_x();
        let icon = super::control_icon(self.size);

        let slot = ui.painter().add(Shape::Noop);
        let (rect, wrapper) =
            ui.allocate_exact_size(vec2(width, self.size.height()), Sense::hover());

        search(
            ui.painter(),
            super::icon_rect(rect, rect.left() + pad, icon),
            if enabled {
                t.text.secondary
            } else {
                t.text.disabled
            },
        );
        let show_clear = !self.text.is_empty() && enabled;
        let right = rect.right() - pad - if show_clear { 18.0 + space::S2 } else { 0.0 };

        let edit = inner_edit(
            ui,
            Rect::from_min_max(
                pos2(rect.left() + pad + icon + space::S2, rect.top()),
                pos2(right, rect.bottom()),
            ),
            &t,
            self.text,
            &self.placeholder,
            false,
            enabled,
        );

        let mut response = edit.clone();
        if show_clear {
            let r = Rect::from_center_size(
                pos2(rect.right() - pad - 9.0, rect.center().y),
                vec2(18.0, 18.0),
            );
            if affix_button(ui, r, &t, &close, enabled, wrapper.id.with("azur-clear")).clicked() {
                self.text.clear();
                response.mark_changed();
            }
        }

        field_frame(
            ui,
            slot,
            rect,
            &t,
            FieldLook {
                enabled,
                focused: edit.has_focus(),
                hovered: field_hovered(ui, rect, enabled),
                error: false,
            },
        );
        response
    }
}
