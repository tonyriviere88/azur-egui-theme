//! Typography setup.
//!
//! Azur is specified in a UI sans and a monospace, and egui needs both plus a
//! heavier weight registered as a family before [`crate::Fonts`] can name it.
//! System fonts are used rather than embedded ones: nothing is added to the
//! binary, and the result looks native next to the platform's own windows.

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

/// Install this platform's UI and monospace faces, falling back to egui's
/// bundled fonts for anything missing.
///
/// Returns whether a proportional system face was found — worth logging if you
/// care that the app looks native, safe to ignore otherwise.
pub fn install(ctx: &egui::Context) -> bool {
    install_from(ctx, PROPORTIONAL, SEMIBOLD_FILES, MONOSPACE)
}

/// The same, from font files you choose. Each list is tried in order and the
/// first file that reads wins; a list that yields nothing leaves egui's own
/// font for that family in place.
///
/// The [`SEMIBOLD`] family is always registered, falling back to the
/// proportional stack, so [`crate::Fonts`] can never name a family that
/// does not exist — egui panics on that, and it would be a strange way for a
/// theme to take down an app.
pub fn install_from(
    ctx: &egui::Context,
    proportional: &[&str],
    semibold: &[&str],
    monospace: &[&str],
) -> bool {
    let mut fonts = egui::FontDefinitions::default();

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

    // Inserted at the front rather than replacing the stack, so egui's bundled
    // fonts stay as the fallback for glyphs the system face lacks.
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
