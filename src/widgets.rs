//! Small per-`Ui` restylings for the places egui's own widgets fight the token
//! set. Both are scoped: call them inside an [`egui::Ui::scope`] (or a popup
//! closure) and nothing leaks into what the caller draws next.
//!
//! These are for egui's widgets. Azur's own are in [`crate::components`].

use crate::theme::Theme;
use crate::tokens::radius;
use egui::{Color32, CornerRadius, Stroke};

/// Make egui's selectable rows read as Azur menu entries.
///
/// Two things are wrong with the default in a list. `Button::selectable` — which
/// backs [`egui::Ui::selectable_label`], and so every entry in a combo popup —
/// draws no frame while an entry is neither selected nor hovered, then a fully
/// stroked one the moment it is. `Frame::total_margin` counts `stroke.width`, so
/// an appearing border does not just decorate: it makes the entry 2px wider and
/// taller.
///
/// In a combo popup that is a feedback loop. The popup takes its minimum width
/// from its content, so the wider entry widens the popup, which widens the
/// content, which widens the popup; the scrollbar flickers in and out as the
/// height changes and the whole thing creeps outwards under the cursor. Clearing
/// the stroke in *every* state fixes both halves at once: no border to appear,
/// and identical geometry hovered or not.
///
/// The second is color. A selected entry takes `selection.bg_fill`, which the
/// installed style points at `accent-default` because three other widgets need it
/// there — but Azur's selected menu entry, list row and tree node are all
/// `accent-subtle`. This puts that back for the rows in scope.
pub fn list_items(ui: &mut egui::Ui, t: &Theme) {
    let corner = CornerRadius::same(radius::SMALL);
    ui.visuals_mut().selection.bg_fill = t.accent.subtle;
    // Width only. `Stroke::NONE` is transparent, and `selection.stroke.color` is also the colour
    // selected *text* is drawn in — so clearing the whole stroke here would paint the selection
    // in a searchable combo's own field in nothing at all. Zero width draws no border, which is
    // all this wanted.
    let ink = ui.visuals().selection.stroke.color;
    ui.visuals_mut().selection.stroke = Stroke::new(0.0, ink);

    let w = &mut ui.visuals_mut().widgets;
    // Only `weak_bg_fill`: that is the one a button reads, and `bg_fill` is
    // documented as never being allowed to go transparent.
    for (state, fill) in [
        // Transparent at rest, so the entries read as a list and not as a column
        // of buttons.
        (&mut w.inactive, Color32::TRANSPARENT),
        (&mut w.hovered, t.bg.control_hover),
        (&mut w.active, t.bg.control_active),
        (&mut w.open, t.bg.control_hover),
    ] {
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::NONE;
        state.corner_radius = corner;
    }
}

/// Point egui's scrollbar at the theme's scrollbar roles for one `Ui`.
///
/// [`crate::install`] already does this globally, so reach for this only where a
/// scroll area sits on a surface the global colors do not suit — a thumb wants to
/// read against *that* surface, not against the canvas.
pub fn scrollbar(ui: &mut egui::Ui, t: &Theme) {
    ui.spacing_mut().scroll.foreground_color = false;
    let w = &mut ui.visuals_mut().widgets;
    w.inactive.bg_fill = t.scrollbar.thumb;
    w.hovered.bg_fill = t.scrollbar.thumb_hover;
    w.active.bg_fill = t.scrollbar.thumb_hover;
}
