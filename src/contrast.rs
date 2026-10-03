//! Measuring colour, instead of counting tokens.
//!
//! Every claim this design system makes about a colour being legible, too loud or
//! too close to its neighbour is a measurement, and this is the ruler. It is public
//! because the claims do not stop at the crate boundary: an application that adds a
//! role of its own — a chart series, a file-kind hue, a hover grey it wants further
//! off the surface — has made the same kind of claim and should hold itself to the
//! same floor in its own tests.
//!
//! # Counting tokens is not measuring contrast
//!
//! The rule this module exists for. "Four steps up the ramp" is not a quantity of
//! anything: the dark neutral ramp is perceptually finer than the light one, so the
//! same number of named steps is a different amount of contrast on each side.
//!
//! Measured, rather than argued: this system's panel seam was
//! `background-control-active`, which is **four** names above `background-layer` in
//! the dark theme and **two** in the light one. That looks like a glaring asymmetry
//! in the token names, and in [`lstar`] it is very nearly symmetric — ΔL\* **15.4**
//! dark against **13.9** light. Nothing was wrong with it; it was simply more
//! contrast than wanted, which is a different sentence and leads to a different fix.
//!
//! So: [`ratio`] for "can this be read", [`apart`] for "do these two read as
//! different surfaces", and neither of them for "how many tokens apart are they",
//! which is not a question about what anybody sees.
//!
//! # Two floors, and why the second one is not a standard
//!
//! [`TEXT`] and [`SHAPE`] are WCAG's, and they are about *ink*: 4.5:1 for a line of
//! text, 3:1 for text at 24px and up or for a shape identified by its boundary.
//!
//! [`SAME`] is not a standard. It is a measured position — the point below which two
//! neutral surfaces stop reading as two surfaces — and it exists because a
//! multi-panel window is mostly this problem: a seam against the panel it divides, a
//! header against the bar above it, a hover against the row beside it. WCAG has
//! nothing to say about any of those, because none of them is ink.
//!
//! # Composite before you measure
//!
//! The `*_subtle` status fills are translucent, and so is a drop target's wash and
//! anything [`Theme::heat`](crate::Theme::heat) returns. A ratio taken against a
//! translucent colour is a ratio against nothing: put it [`over`] the surface it
//! will actually land on first, per surface, because that is what changes the answer.
//!
//! And for anything whose real colour is decided by the *renderer* rather than by a
//! token — text drawn with antialiasing, a fill under a wash under a selection —
//! measure it off the framebuffer. This module measures the palette; a screenshot
//! measures the window. Both are worth doing, and only the second one catches a
//! colour that two correct decisions produced together.

use egui::Color32;

/// WCAG AA for a line of body text: 4.5:1.
pub const TEXT: f32 = 4.5;
/// WCAG AA for large text, and for a shape read by its boundary: 3:1.
pub const SHAPE: f32 = 3.0;
/// Below this many ΔL\*, two neutral surfaces are one surface at a glance.
///
/// Not a standard — see the [module documentation](self). Three is deliberately
/// close to the real margins in this token set (the tightest is the dark theme's
/// seam against a column header, at 3.9), so a role that drifts fails here rather
/// than in front of somebody.
pub const SAME: f32 = 3.0;

/// WCAG 2.1 relative luminance, `0.0`–`1.0`.
///
/// Alpha is ignored: composite with [`over`] first if the colour is translucent.
pub fn luminance(c: Color32) -> f32 {
    0.2126 * linear(c.r()) + 0.7152 * linear(c.g()) + 0.0722 * linear(c.b())
}

/// The WCAG contrast ratio between two opaque colours: 1.0 for identical, 21.0 for
/// black on white. Order does not matter.
pub fn ratio(a: Color32, b: Color32) -> f32 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// CIELAB `L*`, `0.0` (black) to `100.0` (white).
///
/// The scale to compare two *surfaces* on. A weighted byte sum orders two greys
/// correctly and measures them badly — a fixed step in bytes is a much larger
/// perceptual step near black than near white, which is exactly what makes a dark
/// and a light ladder look asymmetric on paper when they are not.
pub fn lstar(c: Color32) -> f32 {
    let y = luminance(c);
    if y <= 0.008_856 {
        y * 903.3
    } else {
        116.0 * y.cbrt() - 16.0
    }
}

/// How far apart two surfaces look, in ΔL\*. Compare against [`SAME`].
pub fn apart(a: Color32, b: Color32) -> f32 {
    (lstar(a) - lstar(b)).abs()
}

/// Composite a translucent colour over an opaque one, so the result can be measured.
pub fn over(fg: Color32, bg: Color32) -> Color32 {
    bg.blend(fg)
}

/// sRGB byte to linear light.
fn linear(byte: u8) -> f32 {
    let c = byte as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::palette;

    #[test]
    fn the_ends_of_the_scale_are_where_they_should_be() {
        assert!((ratio(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01);
        assert!((ratio(palette::GRAY_5, palette::GRAY_5) - 1.0).abs() < 0.001);
        assert!(lstar(Color32::BLACK) < 0.01);
        assert!((lstar(Color32::WHITE) - 100.0).abs() < 0.01);
    }

    /// The measurement the module documentation is about, kept as a test so the
    /// sentence stays true: the old seam was four token names up the dark ramp and
    /// two up the light one, and within 1.5 ΔL\* of itself across the two.
    #[test]
    fn the_dark_and_light_ramps_are_not_the_same_ramp() {
        let dark = apart(crate::Theme::dark().bg.control_active, palette::GRAY_2);
        let light = apart(crate::Theme::light().bg.control_active, crate::tokens::paper::SHEET);
        assert!(
            (dark - 15.4).abs() < 0.2 && (light - 13.9).abs() < 0.2,
            "the seam measurement moved: dark ΔL* {dark:.1}, light ΔL* {light:.1}"
        );
        // And the naive reading — count the names — would have called them 4 and 2.
        assert!(
            (dark - light).abs() < 1.6,
            "two names apart and four names apart measured {:.1} apart",
            dark - light
        );
    }

    #[test]
    fn a_translucent_fill_has_to_be_composited_first() {
        let t = crate::Theme::dark();
        // Straight against the tint, `danger` ink looks like a comfortable pass. Over
        // the surface the tint actually lands on it is a full ratio lower, and that is
        // the number that matters.
        let bare = ratio(t.status.danger, t.status.danger_subtle);
        let real = ratio(t.status.danger, over(t.status.danger_subtle, t.bg.layer));
        assert!(
            real < bare,
            "compositing made no difference, so the fill is not translucent"
        );
        assert!(real >= 4.0);
    }
}
