//! `Tooltip`, `Dialog`, `Drawer`.
//!
//! These are the overlays that are *panels* rather than popovers: they sit on
//! `background-layer` behind a scrim, not on `background-layer-alt` beside an
//! anchor, and they carry the two heaviest shadows in the set.

use super::theme_of;
use crate::icons::close;
use crate::theme::Theme;
use crate::tokens::{radius, shadow, space, typography};
use crate::Theme as AzurTheme;
use egui::{
    vec2, Align, Align2, CornerRadius, Frame, Id, Layout, Margin, Rect, Response, Sense, Stroke, Ui,
};
use std::borrow::Cow;

/// Azur's `Tooltip`, on any response.
///
/// egui's own tooltip would work, but it is framed like a popover —
/// `radius-large`, `menu_margin` — where Azur's tooltip is `radius-small` with
/// `space-2 space-3` of padding and caption type. This is that frame.
///
/// **A keyboard shortcut at the end is set in `text-secondary`**, which is the one
/// rule this adds to the frame — see [`shortcut_in`] for what counts as one, and why
/// it is recognised rather than passed in. `Refresh (F5)` therefore comes out with
/// `Refresh` as the sentence and `(F5)` as the aside, which is what it already means
/// on the page: two colours saying the thing the brackets were there to say.
///
/// It hangs off the response's own rect, which is right for a widget and wrong for one
/// hit-test standing in for many rows — [`tooltip_at_pointer`] is that case.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::tooltip;
/// let r = ui.button("Export");
/// tooltip(r, "Write the current selection to disk");
/// # }
/// ```
pub fn tooltip(response: Response, text: &str) -> Response {
    show_tooltip(response, text, false)
}

/// The same tooltip, put under the **pointer** instead of under the widget.
///
/// For a widget that is one hit-test over many things — a virtualised list where the row is
/// arithmetic on the pointer's `y` rather than a widget of its own — the anchor egui would use
/// is the whole block, so a tooltip about *this row* comes up below the last row on screen.
/// That is not a tooltip the user can connect to what they are pointing at. This anchors it
/// where they are looking.
///
/// Use [`tooltip`] for anything that is its own widget: a tooltip that hangs off the button's
/// own edge is steadier than one that slides around with the pointer, and the button's rect is
/// already the right rect.
///
/// It is held [`CURSOR_CLEARANCE`] off the pointer so the cursor is not sitting on top of the
/// first word.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::tooltip_at_pointer;
/// let r = ui.allocate_response(ui.available_size(), egui::Sense::hover());
/// tooltip_at_pointer(r, "The row under the pointer");
/// # }
/// ```
pub fn tooltip_at_pointer(response: Response, text: &str) -> Response {
    show_tooltip(response, text, true)
}

/// The same tooltip at the pointer, with a body the **caller** draws.
///
/// [`tooltip_at_pointer`] is for a sentence, which is what a tooltip nearly always is, and its measure
/// is deliberately narrow — `spacing.tooltip_width`, a paragraph's width, because a tooltip is read in
/// one glance and a long line is not. A few tooltips are not sentences. A row of a file listing has a
/// name, a size, a date and a state to say about the same thing, and that reads as a small table: keys
/// in one column and values in another, each on its own line. Laid out in a paragraph's measure, every
/// value long enough to matter wraps under its own key.
///
/// So this one has **no measure of its own** — the window is the only bound, and the frame hugs
/// whatever the body turns out to be. A caller that wants one sets it on the `Ui` it is given, which is
/// also the only place that knows what the body is.
///
/// Everything else is [`tooltip_at_pointer`]'s: the same frame, the same anchor, the same
/// [`CURSOR_CLEARANCE`] off the pointer, and the same delay before it appears. The frame is the one
/// thing that must not be re-derived out here — it is `radius-small`, `space-2 space-3` and a `S16`
/// shadow, and a hand-rolled popup beside it would be a second tooltip that nearly matches.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::tooltip_at_pointer_ui;
/// let r = ui.allocate_response(ui.available_size(), egui::Sense::hover());
/// tooltip_at_pointer_ui(r, |ui| {
///     ui.label("Name");
///     ui.label("readme.md");
/// });
/// # }
/// ```
pub fn tooltip_at_pointer_ui(response: Response, add: impl FnOnce(&mut Ui)) -> Response {
    let t = theme_of_ctx(&response.ctx);
    let mut tip = egui::Tooltip::for_enabled(&response)
        .at_pointer()
        .gap(CURSOR_CLEARANCE);
    tip.popup = tip
        .popup
        .frame(tooltip_frame(&t))
        // `Tooltip` takes the paragraph measure from the style; the window is what "unbounded"
        // means for something that has to stay inside it.
        .width(response.ctx.viewport_rect().width());
    tip.show(add);
    response
}

/// A body the caller draws, hung off the **widget** rather than the pointer.
///
/// [`tooltip_at_pointer_ui`]'s reasoning about the measure, with [`tooltip`]'s anchor: for a field or
/// a button — something that is its own widget and whose own rect is the right one to hang from —
/// that wants a small table rather than a sentence.
///
/// The case it exists for is a **syntax tooltip**: what you may type on the left, what it does on the
/// right, one to a line. That cannot be a string, because a value has to start at the same `x` on
/// every line and the caption face is not monospaced, so the columns have to be measured and drawn.
/// See `azur-file-explorer`'s filter box, which lays [`crate::filter::MARKERS`] out this way.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::tooltip_ui;
/// let r = ui.button("Filter");
/// tooltip_ui(r, |ui| {
///     ui.label("!word");
///     ui.label("leave it out");
/// });
/// # }
/// ```
pub fn tooltip_ui(response: Response, add: impl FnOnce(&mut Ui)) -> Response {
    let t = theme_of_ctx(&response.ctx);
    let mut tip = egui::Tooltip::for_enabled(&response);
    tip.popup = tip
        .popup
        .frame(tooltip_frame(&t))
        .width(response.ctx.viewport_rect().width());
    tip.show(add);
    response
}

/// Azur's tooltip frame: `radius-small`, `space-2 space-3`, and the shadow a tooltip floats on.
fn tooltip_frame(t: &Theme) -> Frame {
    Frame::new()
        .fill(t.bg.layer_alt)
        .stroke(Stroke::new(1.0, t.stroke.subtle))
        .corner_radius(CornerRadius::same(radius::SMALL))
        .shadow(shadow::S16)
        .inner_margin(Margin::symmetric(space::S3 as i8, space::S2 as i8))
}

/// How far [`tooltip_at_pointer`] is held off the pointer: `space-6`, which is about the height
/// of the arrow cursor's own ink.
///
/// The anchor is the hotspot — the arrow's tip — and the arrow hangs down and to the right of it,
/// so at egui's own tooltip gap of 4 the cursor would be over the tooltip's first line. This is
/// measured down the axis egui places the tooltip on, and the arrow has no ink *above* its
/// hotspot, so it clears the cursor whichever side of the pointer there is room on.
pub const CURSOR_CLEARANCE: f32 = space::S6;

/// Azur's tooltip frame and two-colour text, for both anchors.
fn show_tooltip(response: Response, text: &str, at_pointer: bool) -> Response {
    let t = theme_of_ctx(&response.ctx);
    let mut tip = egui::Tooltip::for_enabled(&response);
    if at_pointer {
        // `at_pointer` and not `at_pointer_fixed`: the fixed one only records a position when
        // the popup's open state lives in `Memory`, and a tooltip's is passed in — it would
        // come out anchored at the origin.
        tip = tip.at_pointer().gap(CURSOR_CLEARANCE);
    }
    tip.popup = tip.popup.frame(tooltip_frame(&t));
    // Split before the closure, so the two halves are `String`s it can own and the
    // borrow of the caller's `text` ends here.
    let (label, keys) = match shortcut_in(text) {
        Some((label, keys)) => (label.to_owned(), Some(keys.to_owned())),
        None => (text.to_owned(), None),
    };
    let font = t.fonts.caption.clone();
    let (ink, aside) = (t.text.primary, t.text.secondary);
    tip.show(|ui| {
        let Some(keys) = keys else {
            // One colour and no job to build, which is every tooltip that is only a
            // sentence.
            ui.label(egui::RichText::new(label).font(font).color(ink));
            return;
        };
        // **One galley for both halves**, not two labels in a row: they have to sit on
        // one baseline and break as one paragraph, and `(F5)` orphaned onto a line of
        // its own by a horizontal layout is exactly what two widgets would give.
        let mut job = egui::text::LayoutJob::default();
        job.append(
            &label,
            0.0,
            egui::TextFormat {
                font_id: font.clone(),
                color: ink,
                ..Default::default()
            },
        );
        job.append(
            &keys,
            0.0,
            egui::TextFormat {
                font_id: font,
                color: aside,
                ..Default::default()
            },
        );
        job.wrap.max_width = ui.available_width();
        ui.label(job);
    });
    response
}

/// Where a tooltip's sentence stops and its keyboard shortcut starts, brackets and the
/// space before them belonging to the shortcut.
///
/// `Some(("Refresh", " (F5)"))`, `Some(("Up", " (Alt+Up)"))`, and `None` for anything
/// that is only a sentence. The two halves concatenate back to the input, so this is a
/// split and not a parse.
///
/// # Why this is recognised rather than declared
///
/// A tooltip is one string at every call site in every application built on this — it
/// is what a widget's `tooltip("Up (Alt+Up)")` takes, it is often what its `Id` is
/// keyed on, and `Label (Keys)` is already the convention Windows itself writes them
/// in. An extra parameter would have to be threaded through every wrapper on the way
/// down and would be forgotten by the first caller who did not know the rule existed.
/// Recognising the shape means the rule applies to tooltips that were written before
/// it did.
///
/// # What counts
///
/// The guard is the interesting half, because the failure to avoid is a *prose*
/// parenthetical going grey — `Show hidden files (including dotfiles)` must not lose
/// its ending. So all of this has to hold:
///
/// - the text **ends** with `)`, and the matching `(` has a space before it and a
///   sentence before that;
/// - there is **no whitespace inside** the brackets, which is what `(including
///   dotfiles)` fails and no shortcut ever does;
/// - every `+`-separated part is non-empty and **starts with an uppercase letter or a
///   numeral**, which is how key names are written and how `(optional)` is caught;
/// - and at least one uppercase letter appears, so `(2)` is a count rather than a key.
///
/// A numeral rather than an ASCII digit, because a key is named by whatever is printed
/// on it: the leftmost key of an AZERTY number row is `²`, and `(Ctrl+²)` is the
/// shortcut it writes. That character is `No` rather than `Nd`, so an `is_ascii_digit`
/// guard reads the whole tooltip as prose and dims none of it — which is how
/// `Show the console (Ctrl+²)` came to be the one shortcut in a window wearing
/// `text-primary`. The uppercase rule is untouched and is what still holds the guard
/// shut: `(²)` on its own is not a key, and `(2)` is still a count.
///
/// What it will still take for a shortcut is a one-word capitalised aside — `(Beta)`.
/// That is the residue, it is very rare next to the thing being caught, and dimming it
/// is a small wrong: it reads as an aside either way.
pub fn shortcut_in(text: &str) -> Option<(&str, &str)> {
    let inner = text.strip_suffix(')')?;
    // The *last* `(`, so a sentence with brackets of its own earlier on keeps them.
    let open = inner.rfind('(')?;
    let keys = &inner[open + 1..];
    // A space before the bracket, and something before the space. Both matter: they
    // are what make this an aside at the end of a sentence rather than the whole of
    // one, and `(F5)` alone is not a tooltip.
    let label = inner[..open].strip_suffix(' ')?;
    if label.trim().is_empty() || !looks_like_keys(keys) {
        return None;
    }
    Some((label, &text[label.len()..]))
}

/// Whether what was between the brackets is a key or a chord.
fn looks_like_keys(keys: &str) -> bool {
    if keys.is_empty() || keys.contains(char::is_whitespace) {
        return false;
    }
    keys.split('+').all(|part| {
        part.chars()
            .next()
            // `is_numeric` and not `is_ascii_digit`: `²` is a key on this keyboard and a
            // numeral in Unicode's `No` rather than its `Nd`. See `shortcut_in`.
            .is_some_and(|c| c.is_ascii_uppercase() || c.is_numeric())
    }) && keys.chars().any(|c| c.is_ascii_uppercase())
}

fn theme_of_ctx(ctx: &egui::Context) -> std::sync::Arc<Theme> {
    super::theme(ctx)
}

/// A `Dialog`'s action row, held until the dialog is drawn.
type Footer<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

/// `Dialog`'s three widths: 360 / 480 / 640px.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DialogSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl DialogSize {
    fn max_width(self) -> f32 {
        match self {
            Self::Small => 360.0,
            Self::Medium => 480.0,
            Self::Large => 640.0,
        }
    }
}

/// The panel frame shared by `Dialog` and `Drawer`: `background-layer`, a
/// `stroke-subtle` edge, and no padding of its own — each section brings its own.
fn panel_frame(t: &AzurTheme, corner: CornerRadius, elevation: egui::Shadow) -> Frame {
    Frame::new()
        .fill(t.bg.layer)
        .stroke(Stroke::new(1.0, t.stroke.subtle))
        .corner_radius(corner)
        .shadow(elevation)
        .inner_margin(Margin::ZERO)
}

/// A `header` row: `padding: space-5 space-6`, with a `stroke-subtle` rule
/// against the body below it.
fn header(ui: &mut Ui, t: &AzurTheme, add: impl FnOnce(&mut Ui)) {
    Frame::new()
        .inner_margin(Margin::symmetric(space::S6 as i8, space::S5 as i8))
        .show(ui, add);
    rule(ui, t);
}

/// Azur's `Dialog`: a modal panel over a scrim.
///
/// ```no_run
/// # fn demo(ctx: &egui::Context) {
/// use azur_egui_theme::components::{Button, Dialog, Variant};
/// let mut open = true;
/// Dialog::new("Discard changes?", &mut open).show(ctx, |ui| {
///     ui.label("Everything since the last save will be lost.");
/// });
/// # }
/// ```
pub struct Dialog<'a> {
    title: Cow<'a, str>,
    open: &'a mut bool,
    size: DialogSize,
    /// Show the ✕, and close on Escape / a click on the scrim.
    dismissible: bool,
    footer: Option<Footer<'a>>,
}

impl<'a> Dialog<'a> {
    pub fn new(title: impl Into<Cow<'a, str>>, open: &'a mut bool) -> Self {
        Self {
            title: title.into(),
            open,
            size: DialogSize::default(),
            dismissible: true,
            footer: None,
        }
    }

    pub fn size(mut self, size: DialogSize) -> Self {
        self.size = size;
        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    /// The action row. Azur right-aligns it with `space-3` between buttons, which
    /// is the layout this closure runs in.
    pub fn footer(mut self, footer: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.footer = Some(Box::new(footer));
        self
    }

    /// Draw the dialog, if it is open. Returns whatever the body returned.
    pub fn show<R>(self, ctx: &egui::Context, body: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        if !*self.open {
            return None;
        }
        let t = theme_of_ctx(ctx);
        let corner = CornerRadius::same(radius::LARGE);
        let max_width = self.size.max_width();

        let response = egui::Modal::new(Id::new(("azur-dialog", self.title.as_ref())))
            .frame(panel_frame(&t, corner, shadow::S64))
            .backdrop_color(t.bg.overlay)
            .show(ctx, |ui| {
                ui.set_max_width(max_width);
                ui.spacing_mut().item_spacing.y = 0.0;
                let mut close_clicked = false;

                // Header: `title-small`, with the ✕ at 28px.
                header(ui, &t, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(self.title.as_ref())
                                .font(t.fonts.title_small.clone())
                                .color(t.text.primary),
                        );
                        if self.dismissible {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                close_clicked = ui
                                    .add(
                                        super::IconButton::new(&close)
                                            .size(super::Size::Small)
                                            .variant(super::Variant::Subtle),
                                    )
                                    .clicked();
                            });
                        }
                    });
                });

                // Body: `padding: space-6`, scrolling if it outgrows the viewport.
                let inner = Frame::new()
                    .inner_margin(Margin::same(space::S6 as i8))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = space::S3;
                        egui::ScrollArea::vertical()
                            .id_salt("azur-dialog-body")
                            .max_height(ctx.content_rect().height() * 0.6)
                            .show(ui, body)
                            .inner
                    })
                    .inner;

                if let Some(footer) = self.footer {
                    rule(ui, &t);
                    Frame::new()
                        .inner_margin(Margin::symmetric(space::S6 as i8, space::S5 as i8))
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.x = space::S3;
                            ui.with_layout(Layout::right_to_left(Align::Center), footer);
                        });
                }
                (inner, close_clicked)
            });

        let dismissed = self.dismissible && response.should_close();
        let (inner, close_clicked) = response.inner;
        if close_clicked || dismissed {
            *self.open = false;
        }
        Some(inner)
    }
}

/// Which edge a `Drawer` slides in from.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DrawerSide {
    Left,
    #[default]
    Right,
}

/// Azur's `Drawer`: a full-height panel against one edge, over a scrim.
///
/// Widths follow `Drawer.module.css`: 280 / 360 / 480px.
///
/// ```no_run
/// # fn demo(ctx: &egui::Context) {
/// use azur_egui_theme::components::Drawer;
/// let mut open = true;
/// Drawer::new("Layer properties", &mut open).show(ctx, |ui| {
///     ui.label("…");
/// });
/// # }
/// ```
pub struct Drawer<'a> {
    title: Cow<'a, str>,
    open: &'a mut bool,
    side: DrawerSide,
    size: DialogSize,
    dismissible: bool,
}

impl<'a> Drawer<'a> {
    pub fn new(title: impl Into<Cow<'a, str>>, open: &'a mut bool) -> Self {
        Self {
            title: title.into(),
            open,
            side: DrawerSide::default(),
            size: DialogSize::default(),
            dismissible: true,
        }
    }

    pub fn side(mut self, side: DrawerSide) -> Self {
        self.side = side;
        self
    }

    /// 280 / 360 / 480px for small / medium / large.
    pub fn size(mut self, size: DialogSize) -> Self {
        self.size = size;
        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    fn width(&self) -> f32 {
        match self.size {
            DialogSize::Small => 280.0,
            DialogSize::Medium => 360.0,
            DialogSize::Large => 480.0,
        }
    }

    pub fn show<R>(self, ctx: &egui::Context, body: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        if !*self.open {
            return None;
        }
        let t = theme_of_ctx(ctx);
        let width = self.width();
        let screen = ctx.content_rect();
        let align = match self.side {
            DrawerSide::Left => Align2::LEFT_TOP,
            DrawerSide::Right => Align2::RIGHT_TOP,
        };

        // A drawer is square against its edge and rounded on neither: it spans the
        // full height, so there are no outer corners to round.
        let response = egui::Modal::new(Id::new(("azur-drawer", self.title.as_ref())))
            .frame(panel_frame(&t, CornerRadius::ZERO, shadow::S28))
            .backdrop_color(t.bg.overlay)
            .area(
                egui::Area::new(Id::new(("azur-drawer-area", self.title.as_ref())))
                    .anchor(align, vec2(0.0, 0.0)),
            )
            .show(ctx, |ui| {
                ui.set_width(width);
                ui.set_min_height(screen.height());
                ui.spacing_mut().item_spacing.y = 0.0;
                let mut close_clicked = false;

                header(ui, &t, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(self.title.as_ref())
                                .font(t.fonts.title_small.clone())
                                .color(t.text.primary),
                        );
                        if self.dismissible {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                close_clicked = ui
                                    .add(
                                        super::IconButton::new(&close)
                                            .size(super::Size::Small)
                                            .variant(super::Variant::Subtle),
                                    )
                                    .clicked();
                            });
                        }
                    });
                });

                let inner = Frame::new()
                    .inner_margin(Margin::same(space::S6 as i8))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = space::S3;
                        egui::ScrollArea::vertical()
                            .id_salt("azur-drawer-body")
                            .show(ui, body)
                            .inner
                    })
                    .inner;
                (inner, close_clicked)
            });

        let dismissed = self.dismissible && response.should_close();
        let (inner, close_clicked) = response.inner;
        if close_clicked || dismissed {
            *self.open = false;
        }
        Some(inner)
    }
}

/// A section heading inside a dialog, drawer or card: `subtitle`, with
/// `space-2` under it.
///
/// Not a component of Azur's own — its gallery composes these out of `Text` —
/// but every panel here needs one, and spelling it out at each call site is how
/// type ramps drift.
pub fn section_heading(ui: &mut Ui, text: &str) {
    let t = theme_of(ui);
    let (rect, _) = ui.allocate_exact_size(
        vec2(ui.available_width(), typography::LINE_SUBTITLE),
        Sense::hover(),
    );
    let galley = super::truncated(
        ui,
        text,
        t.fonts.subtitle.clone(),
        t.text.primary,
        rect.width(),
    );
    super::galley_left_center(ui, rect, galley);
    ui.add_space(space::S2);
}

/// A hairline the width of the container, in `stroke-subtle`.
pub(crate) fn rule(ui: &mut Ui, t: &AzurTheme) -> Rect {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, t.stroke.subtle);
    rect
}

#[cfg(test)]
mod tests {
    use super::shortcut_in;

    /// Every shape a shortcut is written in, and the two halves it comes back as.
    #[test]
    fn a_tooltip_ending_in_a_chord_is_split_from_it() {
        let cases = [
            ("Refresh (F5)", "Refresh", " (F5)"),
            ("Up (Alt+Up)", "Up", " (Alt+Up)"),
            ("Close (Esc)", "Close", " (Esc)"),
            ("Next match (Enter)", "Next match", " (Enter)"),
            (
                "Previous match (Shift+Enter)",
                "Previous match",
                " (Shift+Enter)",
            ),
            (
                "Flatten this folder's whole tree (Ctrl+E)",
                "Flatten this folder's whole tree",
                " (Ctrl+E)",
            ),
            (
                "Put the tab back (Ctrl+Shift+T)",
                "Put the tab back",
                " (Ctrl+Shift+T)",
            ),
            // A digit is a key when something else in the chord is a name.
            ("Actual size (Ctrl+0)", "Actual size", " (Ctrl+0)"),
            // And so is a numeral that is not a digit: `²` is the leftmost key of an
            // AZERTY number row, and the whole reason the guard asks for a numeral
            // rather than an ASCII one.
            ("Show the console (Ctrl+²)", "Show the console", " (Ctrl+²)"),
            ("Large icons (Ctrl+1)", "Large icons", " (Ctrl+1)"),
            // Brackets earlier in the sentence stay in the sentence: the *last* pair
            // is the one that could be a shortcut.
            (
                "Copy (as text) to the clipboard (Ctrl+C)",
                "Copy (as text) to the clipboard",
                " (Ctrl+C)",
            ),
        ];
        for (whole, label, keys) in cases {
            assert_eq!(shortcut_in(whole), Some((label, keys)), "{whole:?}");
            // A split, not a parse: the halves put the input back together.
            assert_eq!(format!("{label}{keys}"), whole);
        }
    }

    /// And everything that must keep its ending, which is the half that matters: a
    /// sentence dimmed from its last bracket onwards has lost a word.
    #[test]
    fn a_parenthetical_that_is_prose_keeps_its_colour() {
        let cases = [
            // The whole reason for the guard: a real aside, in words.
            "Show hidden files (including dotfiles)",
            "Number the lines (off by default)",
            // Lowercase, so not a key name however short it is.
            "Rename (optional)",
            // A count, not a chord — no uppercase letter anywhere in it. Which is what
            // holds the guard shut now that a numeral counts as the start of a key name:
            // neither of these is a shortcut, and `²` on its own is not one either.
            "New folder (2)",
            "The second pane (²)",
            // No space before the bracket, so the two are one word.
            "Open readme(F5)",
            // Nothing before the bracket at all.
            "(F5)",
            // Brackets that do not close at the end, or do not open.
            "Refresh (F5",
            "Refresh F5)",
            // Empty, and whitespace inside — a chord is written without spaces.
            "Refresh ()",
            "Refresh (Alt + Up)",
            // No brackets whatsoever, which is most tooltips.
            "Write the current selection to disk",
            "",
        ];
        for whole in cases {
            assert_eq!(shortcut_in(whole), None, "{whole:?}");
        }
    }
}
