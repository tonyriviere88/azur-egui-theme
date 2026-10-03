//! `Checkbox`, `Radio` / `RadioGroup`, `Switch`, `Slider`.
//!
//! egui has all four, and none of them look like Azur's: a checked egui checkbox
//! is a mark on the control fill where Azur's is an accent-filled box, egui's
//! radio fills the whole circle, its switch does not exist, and its slider has no
//! label row. So these paint themselves.

use super::{theme_of, truncated, Size};
use crate::icons::{check, dash, focus_ring};
use crate::theme::Theme;
use crate::tokens::{motion, radius, shadow, space, typography};
use egui::{
    pos2, vec2, Color32, CornerRadius, Rect, Response, Sense, Stroke, StrokeKind, Ui, Widget,
};
use std::borrow::Cow;

/// The row a labelled control occupies: the taller of its box and one body line.
fn row_height(box_size: f32) -> f32 {
    box_size.max(typography::LINE_BODY)
}

/// Lay out `box + space-3 + label`, and return the two rects.
fn labelled(
    ui: &mut Ui,
    box_size: (f32, f32),
    label: &str,
    label_first: bool,
) -> (Rect, Rect, Response) {
    let gap = if label.is_empty() { 0.0 } else { space::S3 };
    let font = theme_of(ui).fonts.body.clone();
    let label_w = if label.is_empty() {
        0.0
    } else {
        ui.painter()
            .layout_no_wrap(label.to_owned(), font, Color32::PLACEHOLDER)
            .size()
            .x
    };

    let (w, h) = (box_size.0 + gap + label_w, row_height(box_size.1));
    let (rect, response) = ui.allocate_exact_size(vec2(w, h), Sense::click());

    let (box_x, label_x) = if label_first {
        (rect.right() - box_size.0, rect.left())
    } else {
        (rect.left(), rect.left() + box_size.0 + gap)
    };
    let control = Rect::from_min_size(
        pos2(box_x, rect.center().y - box_size.1 * 0.5),
        vec2(box_size.0, box_size.1),
    );
    let text = Rect::from_min_max(
        pos2(label_x, rect.top()),
        pos2(label_x + label_w, rect.bottom()),
    );
    (control, text, response)
}

fn paint_label(ui: &Ui, rect: Rect, label: &str, t: &Theme, enabled: bool) {
    if label.is_empty() {
        return;
    }
    let color = t.text_for(enabled);
    let galley = truncated(ui, label, t.fonts.body.clone(), color, rect.width() + 1.0);
    super::galley_left_center(ui, rect, galley);
}

/// Azur's `Checkbox`.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::Checkbox;
/// let mut on = false;
/// ui.add(Checkbox::new(&mut on, "Show hidden files"));
/// # }
/// ```
pub struct Checkbox<'a> {
    checked: &'a mut bool,
    label: Cow<'a, str>,
    size: Size,
    indeterminate: bool,
}

impl<'a> Checkbox<'a> {
    pub fn new(checked: &'a mut bool, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            checked,
            label: label.into(),
            size: Size::default(),
            indeterminate: false,
        }
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// The mixed state: a dash rather than a tick, on the same accent fill.
    /// Azur's `aria-checked="mixed"`.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }
}

impl Widget for Checkbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let side = self.size.box_size();
        let (box_rect, text_rect, mut response) = labelled(ui, (side, side), &self.label, false);

        if response.clicked() {
            *self.checked = !*self.checked;
            response.mark_changed();
        }
        if !ui.is_rect_visible(response.rect) {
            return response;
        }

        let on = *self.checked || self.indeterminate;
        let hovered = enabled && response.hovered();
        let down = enabled && response.is_pointer_button_down_on();

        let (fill, border) = if !enabled {
            if on {
                (t.accent.disabled, t.accent.disabled)
            } else {
                (t.bg.control_disabled, t.stroke.subtle)
            }
        } else if on {
            let c = if down {
                t.accent.active
            } else if hovered {
                t.accent.hover
            } else {
                t.accent.default
            };
            (c, c)
        } else {
            (
                if down {
                    t.bg.control_active
                } else {
                    t.bg.control
                },
                if hovered {
                    t.stroke.strong
                } else {
                    t.stroke.control
                },
            )
        };

        let corner = CornerRadius::same(radius::SMALL);
        let p = ui.painter();
        p.rect(
            box_rect,
            corner,
            fill,
            Stroke::new(1.0, border),
            StrokeKind::Inside,
        );

        if on {
            // `.check, .dash { width: 80%; height: 80% }`.
            let glyph = box_rect.shrink(side * 0.1);
            let color = if enabled {
                t.text.on_accent
            } else {
                t.text.disabled
            };
            if self.indeterminate {
                dash(p, glyph, color);
            } else {
                check(p, glyph, color);
            }
        }

        if response.has_focus() {
            focus_ring(p, box_rect, corner, t.stroke.focus);
        }
        paint_label(ui, text_rect, &self.label, &t, enabled);
        response
    }
}

/// Azur's `Radio`: one option of a group.
///
/// The whole group shares a value, so this takes the value it stands for and the
/// one currently chosen.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::{Radio, RadioGroup};
/// #[derive(PartialEq, Clone)] enum Fit { Width, Page }
/// let mut fit = Fit::Width;
/// RadioGroup::vertical().show(ui, |ui| {
///     ui.add(Radio::new(&mut fit, Fit::Width, "Fit width"));
///     ui.add(Radio::new(&mut fit, Fit::Page, "Fit page"));
/// });
/// # }
/// ```
pub struct Radio<'a, T> {
    current: &'a mut T,
    value: T,
    label: Cow<'a, str>,
    size: Size,
}

impl<'a, T: PartialEq> Radio<'a, T> {
    pub fn new(current: &'a mut T, value: T, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            current,
            value,
            label: label.into(),
            size: Size::default(),
        }
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

impl<T: PartialEq> Widget for Radio<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let side = self.size.box_size();
        let (box_rect, text_rect, mut response) = labelled(ui, (side, side), &self.label, false);

        let selected = *self.current == self.value;
        if response.clicked() && !selected {
            *self.current = self.value;
            response.mark_changed();
        }
        if !ui.is_rect_visible(response.rect) {
            return response;
        }

        let hovered = enabled && response.hovered();
        let down = enabled && response.is_pointer_button_down_on();
        // Unlike the checkbox, Azur's radio keeps the control fill when selected
        // and only moves its border and its dot onto the accent.
        let accent = if !enabled {
            t.accent.disabled
        } else if down {
            t.accent.active
        } else if hovered {
            t.accent.hover
        } else {
            t.accent.default
        };
        let (fill, border) = if !enabled {
            (
                t.bg.control_disabled,
                if selected { accent } else { t.stroke.subtle },
            )
        } else if selected {
            (
                if down {
                    t.bg.control_active
                } else {
                    t.bg.control
                },
                accent,
            )
        } else {
            (
                if down {
                    t.bg.control_active
                } else {
                    t.bg.control
                },
                if hovered {
                    t.stroke.strong
                } else {
                    t.stroke.control
                },
            )
        };

        let p = ui.painter();
        let center = box_rect.center();
        let r = side * 0.5;
        p.circle_filled(center, r, fill);
        p.circle_stroke(center, r - 0.5, Stroke::new(1.0, border));
        if selected {
            // 6 / 7 / 9px inner dot for a 14 / 16 / 20px control.
            let dot = match self.size {
                Size::Small => 6.0,
                Size::Medium => 7.0,
                Size::Large => 9.0,
            };
            p.circle_filled(center, dot * 0.5, accent);
        }
        if response.has_focus() {
            focus_ring(
                p,
                box_rect,
                CornerRadius::same(radius::CIRCULAR),
                t.stroke.focus,
            );
        }
        paint_label(ui, text_rect, &self.label, &t, enabled);
        response
    }
}

/// The layout Azur's `RadioGroup` provides: its options, `space-4` apart.
#[derive(Clone, Copy, Debug, Default)]
pub struct RadioGroup {
    vertical: bool,
}

impl RadioGroup {
    /// `flex-direction: column`.
    pub fn vertical() -> Self {
        Self { vertical: true }
    }

    /// `flex-direction: row; flex-wrap: wrap`.
    pub fn horizontal() -> Self {
        Self { vertical: false }
    }

    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(space::S4, space::S4);
            // `horizontal_wrapped` rather than a `Layout` with `main_wrap` set:
            // a wrapping layout claims the whole available height up front, which
            // in a card leaves a hole under the options.
            if self.vertical {
                ui.vertical(add).inner
            } else {
                ui.horizontal_wrapped(add).inner
            }
        })
    }
}

/// Where a `Switch` puts its label.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SwitchLabel {
    /// After the track, reading order. Azur's default.
    #[default]
    End,
    /// Before the track, for a settings list where the labels align left.
    Start,
}

/// Azur's `Switch`: a pill track with a sliding thumb.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::Switch;
/// let mut on = true;
/// ui.add(Switch::new(&mut on, "Snap to grid"));
/// # }
/// ```
pub struct Switch<'a> {
    checked: &'a mut bool,
    label: Cow<'a, str>,
    size: Size,
    label_position: SwitchLabel,
}

impl<'a> Switch<'a> {
    pub fn new(checked: &'a mut bool, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            checked,
            label: label.into(),
            size: Size::default(),
            label_position: SwitchLabel::default(),
        }
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn label_position(mut self, position: SwitchLabel) -> Self {
        self.label_position = position;
        self
    }
}

impl Widget for Switch<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let (tw, th) = self.size.switch();
        let (track, text_rect, mut response) = labelled(
            ui,
            (tw, th),
            &self.label,
            self.label_position == SwitchLabel::Start,
        );

        if response.clicked() {
            *self.checked = !*self.checked;
            response.mark_changed();
        }
        if !ui.is_rect_visible(response.rect) {
            return response;
        }

        let on = *self.checked;
        let hovered = enabled && response.hovered();
        let down = enabled && response.is_pointer_button_down_on();

        let (fill, border) = if !enabled {
            if on {
                (t.accent.disabled, t.accent.disabled)
            } else {
                (t.bg.control_disabled, t.stroke.subtle)
            }
        } else if on {
            let c = if down {
                t.accent.active
            } else if hovered {
                t.accent.hover
            } else {
                t.accent.default
            };
            (c, c)
        } else {
            (
                if down {
                    t.bg.control_active
                } else {
                    t.bg.control
                },
                if hovered {
                    t.stroke.strong
                } else {
                    t.stroke.control
                },
            )
        };

        let corner = CornerRadius::same(radius::CIRCULAR);
        let p = ui.painter();
        p.rect(
            track,
            corner,
            fill,
            Stroke::new(1.0, border),
            StrokeKind::Inside,
        );

        // `.thumb { margin-left: 2px }`, and a `translateX` of the thumb's own
        // width when checked.
        let thumb_d = th - 4.0;
        let travel = tw - thumb_d - 4.0;
        let anim =
            ui.ctx()
                .animate_bool_with_time(response.id.with("azur-switch"), on, motion::FAST);
        let cx = track.left() + 2.0 + thumb_d * 0.5 + travel * anim;
        let thumb_color = if !enabled && !on {
            t.text.disabled
        } else if on {
            t.text.on_accent
        } else {
            t.text.secondary
        };
        p.circle_filled(pos2(cx, track.center().y), thumb_d * 0.5, thumb_color);

        if response.has_focus() {
            focus_ring(p, track, corner, t.stroke.focus);
        }
        paint_label(ui, text_rect, &self.label, &t, enabled);
        response
    }
}

/// Azur's `Slider`: an optional label row over a filled track with a round
/// handle.
///
/// egui's own slider is close after [`mod@crate::style`] turns on the trailing fill
/// and the circular handle, but it has no label row, its handle carries no ring,
/// and its rail is one token off. This is the Azur one.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::Slider;
/// let mut opacity = 0.6;
/// ui.add(Slider::new(&mut opacity, 0.0..=1.0).label("Opacity").decimals(2));
/// # }
/// ```
pub struct Slider<'a> {
    value: &'a mut f32,
    range: std::ops::RangeInclusive<f32>,
    label: Option<Cow<'a, str>>,
    size: Size,
    show_value: bool,
    decimals: usize,
    suffix: Cow<'a, str>,
    step: Option<f32>,
}

impl<'a> Slider<'a> {
    pub fn new(value: &'a mut f32, range: std::ops::RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            label: None,
            size: Size::default(),
            show_value: true,
            decimals: 0,
            suffix: Cow::Borrowed(""),
            step: None,
        }
    }

    /// The label on the left of the header row.
    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Show the value on the right of the header row. On by default, but only
    /// drawn when there is a header — that is, when there is a label.
    pub fn show_value(mut self, show_value: bool) -> Self {
        self.show_value = show_value;
        self
    }

    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    /// A unit to print after the value: `"%"`, `" px"`.
    pub fn suffix(mut self, suffix: impl Into<Cow<'a, str>>) -> Self {
        self.suffix = suffix.into();
        self
    }

    /// Quantise to a step.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Track height and handle diameter for each size.
    fn metrics(&self) -> (f32, f32) {
        match self.size {
            Size::Small => (4.0, 14.0),
            Size::Medium => (6.0, 18.0),
            Size::Large => (8.0, 22.0),
        }
    }
}

impl Widget for Slider<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let (rail_h, handle_d) = self.metrics();
        let width = ui.available_width();

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = space::S2;

            if let Some(label) = &self.label {
                let (header, _) =
                    ui.allocate_exact_size(vec2(width, typography::LINE_BODY), Sense::hover());
                if ui.is_rect_visible(header) {
                    let value_text = if self.show_value {
                        format!("{:.*}{}", self.decimals, *self.value, self.suffix)
                    } else {
                        String::new()
                    };
                    let value_galley = ui.painter().layout_no_wrap(
                        value_text,
                        t.fonts.caption.clone(),
                        if enabled {
                            t.text.secondary
                        } else {
                            t.text.disabled
                        },
                    );
                    let label_galley = truncated(
                        ui,
                        label,
                        t.fonts.body.clone(),
                        if enabled {
                            t.text.primary
                        } else {
                            t.text.disabled
                        },
                        (width - value_galley.size().x - space::S3).max(0.0),
                    );
                    // The label is body and the value is caption, so the two share
                    // one baseline rather than each centring its own box: see
                    // [`super::row_baseline`]. The label is the principal text.
                    let baseline = super::row_baseline(
                        ui.painter(),
                        &t.fonts.body,
                        header.top(),
                        header.height(),
                    );
                    super::galley_on_baseline(ui.painter(), header.left(), baseline, label_galley);
                    // `.value { font-weight: medium }` -- the caption step, regular
                    // weight (medium rounds down), right-aligned.
                    let vw = value_galley.size().x;
                    super::galley_on_baseline(
                        ui.painter(),
                        header.right() - vw,
                        baseline,
                        value_galley,
                    );
                }
            }

            // The input row is the handle's height, as Azur's `.input` is.
            let (row, mut response) =
                ui.allocate_exact_size(vec2(width, handle_d), Sense::click_and_drag());

            let (lo, hi) = (*self.range.start(), *self.range.end());
            let span = (hi - lo).max(f32::EPSILON);
            let usable = (row.width() - handle_d).max(1.0);

            if enabled {
                if let Some(pos) = response.interact_pointer_pos() {
                    let x = (pos.x - row.left() - handle_d * 0.5) / usable;
                    let mut v = lo + x.clamp(0.0, 1.0) * span;
                    if let Some(step) = self.step {
                        if step > 0.0 {
                            v = lo + ((v - lo) / step).round() * step;
                        }
                    }
                    let v = v.clamp(lo.min(hi), hi.max(lo));
                    if (v - *self.value).abs() > f32::EPSILON {
                        *self.value = v;
                        response.mark_changed();
                    }
                }
            }

            if !ui.is_rect_visible(row) {
                return response;
            }

            let frac = ((*self.value - lo) / span).clamp(0.0, 1.0);
            let cx = row.left() + handle_d * 0.5 + usable * frac;
            let rail = Rect::from_min_max(
                pos2(row.left(), row.center().y - rail_h * 0.5),
                pos2(row.right(), row.center().y + rail_h * 0.5),
            );
            let corner = CornerRadius::same(radius::CIRCULAR);
            let p = ui.painter();

            let (track_bg, filled, handle) = if !enabled {
                (t.bg.control_disabled, t.accent.disabled, t.accent.disabled)
            } else {
                let h = if response.is_pointer_button_down_on() {
                    t.accent.active
                } else if response.hovered() {
                    t.accent.hover
                } else {
                    t.accent.default
                };
                (t.stroke.subtle, t.accent.default, h)
            };
            p.rect_filled(rail, corner, track_bg);
            if frac > 0.0 {
                let mut fill = rail;
                fill.max.x = cx;
                p.rect_filled(fill, corner, filled);
            }

            // `2px solid text-on-accent` around the handle, over `shadow-4`.
            let center = pos2(cx, row.center().y);
            let handle_rect = Rect::from_center_size(center, vec2(handle_d, handle_d));
            p.add(shadow::S4.as_shape(handle_rect, corner));
            let ring = if enabled {
                t.text.on_accent
            } else {
                t.bg.control_disabled
            };
            p.circle_filled(center, handle_d * 0.5, ring);
            p.circle_filled(center, handle_d * 0.5 - 2.0, handle);

            if response.has_focus() {
                focus_ring(p, handle_rect, corner, t.stroke.focus);
            }
            response
        })
        .inner
    }
}
