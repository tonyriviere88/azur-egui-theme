//! Typography setup.
//!
//! Azur is specified in a UI sans and a monospace, and egui needs both plus a
//! heavier weight registered as a family before [`crate::Fonts`] can name it.
//! System fonts are used rather than embedded ones: nothing is added to the
//! binary, and the result looks native next to the platform's own windows.
//!
//! # There is no bundled fallback behind them
//!
//! [`install_from`] starts from [`egui::FontDefinitions::empty`] rather than from
//! `default()`, so what an Azur window draws with is exactly the faces found below
//! and nothing else. epaint's own four — NotoEmoji, Ubuntu-Light, emoji-icon-font
//! and Hack — used to sit behind them, and they were 1,414,020 bytes of the
//! largest data objects in a consumer's executable to cover a font read that does
//! not fail: Segoe UI and Consolas are always on a Windows machine, and this is a
//! Windows design system.
//!
//! Being explicit about it is the load-bearing part. `default()` is `empty()`
//! *only while epaint's `default_fonts` feature is off*, and that feature is not
//! this crate's to control: any crate anywhere in a consumer's graph that enables
//! it turns the four back on for everybody, and the cost would reappear silently.
//! `empty()` cannot be unified back on.
//!
//! What it costs is emoji, which Segoe UI does not carry — a filename with one in
//! it draws a missing-glyph box. Adding `seguiemj.ttf` to [`PROPORTIONAL`] as a
//! fallback would take it back at no cost to the binary, in monochrome, since
//! epaint does not draw colour fonts.

use std::sync::Arc;

/// The family name registered for the heavier weight, and the one
/// [`crate::Fonts::body_strong`] and its heavier siblings ask for.
pub const SEMIBOLD: &str = "semibold";

/// Candidate files for the proportional UI face, best first.
#[cfg(windows)]
pub const PROPORTIONAL: &[&str] = &[r"C:\Windows\Fonts\segoeui.ttf"];
/// Candidate files for the heavier weight, best first.
///
/// Segoe UI's semibold face is `seguisb.ttf` — the *short* spelling, where the rest
/// of the family is `segoeui`, `segoeuib`, `segoeuil`. Getting it wrong is silent
/// and expensive: the loader falls through to the next candidate, and every role
/// Azur marks semibold renders at Bold (700) instead of 600.
#[cfg(windows)]
pub const SEMIBOLD_FILES: &[&str] = &[
    r"C:\Windows\Fonts\seguisb.ttf", // Segoe UI Semibold, 600 — the one Azur means
    r"C:\Windows\Fonts\segoeuib.ttf", // Segoe UI Bold, 700 — a last resort
];
/// Candidate files for the monospace face, best first.
#[cfg(windows)]
pub const MONOSPACE: &[&str] = &[r"C:\Windows\Fonts\consola.ttf"];

#[cfg(target_os = "macos")]
pub const PROPORTIONAL: &[&str] = &[
    "/System/Library/Fonts/SFNS.ttf",
    "/System/Library/Fonts/Helvetica.ttc",
];
#[cfg(target_os = "macos")]
pub const SEMIBOLD_FILES: &[&str] = &["/System/Library/Fonts/SFNSRounded.ttf"];
#[cfg(target_os = "macos")]
pub const MONOSPACE: &[&str] = &["/System/Library/Fonts/Menlo.ttc"];

#[cfg(not(any(windows, target_os = "macos")))]
pub const PROPORTIONAL: &[&str] = &[
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
];
#[cfg(not(any(windows, target_os = "macos")))]
pub const SEMIBOLD_FILES: &[&str] = &[
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
];
#[cfg(not(any(windows, target_os = "macos")))]
pub const MONOSPACE: &[&str] = &[
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
    "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
];

/// Install this platform's UI and monospace faces.
///
/// Returns whether a proportional system face was found — worth logging if you
/// care that the app looks native, safe to ignore otherwise. There is nothing
/// behind these faces if the read fails; see the module header.
pub fn install(ctx: &egui::Context) -> bool {
    install_from(ctx, PROPORTIONAL, SEMIBOLD_FILES, MONOSPACE)
}

/// The same, from font files you choose. Each list is tried in order and the
/// first file that reads wins; a list that yields nothing leaves that family
/// bound to no font at all, which draws nothing rather than falling back.
///
/// Every family egui and [`crate::Fonts`] can name is *registered* either way,
/// which is the part that has to hold: epaint panics on a family that is absent
/// from the map — it is fine with one bound to an empty list — and it would be a
/// strange way for a theme to take down an app. [`egui::FontDefinitions::empty`]
/// registers `Proportional` and `Monospace`, and [`SEMIBOLD`] is inserted below,
/// falling back to the proportional stack.
pub fn install_from(
    ctx: &egui::Context,
    proportional: &[&str],
    semibold: &[&str],
    monospace: &[&str],
) -> bool {
    // `empty`, not `default`: the four fonts epaint would otherwise put behind these
    // are 1.4 MB of somebody's executable for a font read that does not fail. See the
    // module header, including why this cannot be left to the feature flag.
    let mut fonts = egui::FontDefinitions::empty();

    let mut load = |key: &str, candidates: &[&str]| -> bool {
        for path in candidates {
            if let Ok(bytes) = std::fs::read(path) {
                fonts
                    .font_data
                    .insert(key.to_owned(), Arc::new(egui::FontData::from_owned(bytes)));
                return true;
            }
        }
        false
    };

    let has_prop = load("azur-prop", proportional);
    let has_semi = load("azur-semi", semibold);
    let has_mono = load("azur-mono", monospace);

    // Inserted at the front rather than replacing the stack, so a consumer that has
    // added a face of its own to either family keeps it as the fallback for glyphs
    // the system face lacks. Nothing of epaint's is in there to keep.
    if has_prop {
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "azur-prop".to_owned());
    }
    if has_mono {
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "azur-mono".to_owned());
    }

    // A real family for the heavier weight, so a heading can be heavier without
    // faking it by scaling the regular weight up.
    let proportional_stack = fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let semibold_stack = if has_semi {
        let mut stack = vec!["azur-semi".to_owned()];
        stack.extend(proportional_stack.iter().cloned());
        stack
    } else {
        proportional_stack
    };
    fonts
        .families
        .insert(egui::FontFamily::Name(SEMIBOLD.into()), semibold_stack);

    ctx.set_fonts(fonts);
    has_prop
}
