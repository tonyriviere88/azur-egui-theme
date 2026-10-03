//! The [Azur design system](https://github.com/) as an [egui](https://docs.rs/egui) theme.
//!
//! Azur is specified as CSS custom properties and React components. This is that
//! specification for egui: the same tokens, the same semantic roles, and the
//! components egui has no equivalent for — checked in against
//! `azur-design-system/src/styles/tokens.css` and the component CSS beside it,
//! token by token.
//!
//! Four layers, each built from the one before:
//!
//! 1. [`tokens`] — the raw scales, transcribed whole from `tokens.css`. Values.
//! 2. [`Theme`] — the semantic roles: `bg.card`, `stroke.subtle`,
//!    `text.secondary`. Decisions, and the layer an app reads or overrides.
//! 3. [`mod@style`] — [`egui::Visuals`], [`egui::style::Spacing`] and the text styles,
//!    so egui's own widgets wear the token set.
//! 4. [`components`] — Azur's components, for everything egui does not have or
//!    draws differently: `Button`, `Checkbox`, `ListView`, `Table`, `Dialog`,
//!    `Toast`, `NavigationRail`, and the rest.
//!
//! # Getting started
//!
//! ```no_run
//! # fn demo(ctx: &egui::Context) {
//! // Fonts, dark palette, default density -- and the theme you paint with.
//! let theme = azur_egui_theme::install_dark(ctx);
//! # }
//! ```
//!
//! Then use the components, which read the theme out of the context:
//!
//! ```no_run
//! # fn demo(ui: &mut egui::Ui) {
//! use azur_egui_theme::components::{Button, Card, Variant};
//! Card::new().show(ui, |ui| {
//!     if ui.add(Button::new("Register").variant(Variant::Primary)).clicked() {
//!         // ...
//!     }
//! });
//! # }
//! ```
//!
//! Spell the install out as soon as you have an opinion about density, or want to
//! move a role:
//!
//! ```no_run
//! use azur_egui_theme::{fonts, install, tokens::control, StyleOptions, Theme};
//! # fn demo(ctx: &egui::Context) {
//! fonts::install(ctx);
//!
//! let mut theme = Theme::dark();
//! // Azur's canvas is GRAY_0. This app would rather not be quite that black.
//! theme.bg.canvas = azur_egui_theme::tokens::palette::GRAY_2;
//! theme.bg.layer = azur_egui_theme::tokens::palette::GRAY_3;
//!
//! install(ctx, &theme, &StyleOptions {
//!     control_height: control::SMALL, // dense, table-heavy window
//!     selectable_labels: false,       // rows are painted by hand
//!     ..Default::default()
//! });
//! # }
//! ```
//!
//! Keep the `Theme` next to your app state for anything you paint yourself:
//! `painter.rect_filled(rect, radius, theme.bg.card)`.
//!
//! # A dense application window
//!
//! Azur is specified for the browser, and a file manager or a process table is not a
//! page: it is panes divided by seams, hundreds of rows the pointer sweeps across, and
//! a window that stays open all day. [`desktop`] is the decisions that shape of window
//! makes — one hover grey for everything, a selected row on the accent's surface ramp,
//! nothing structural rounded, 24px rows that touch — gathered so they are made once
//! rather than per application. Opt in with one call:
//!
//! ```no_run
//! # fn demo(ctx: &egui::Context) {
//! let theme = azur_egui_theme::desktop::install(ctx, true);
//! # }
//! ```
//!
//! # A filter field is its behaviour too
//!
//! The look of a filter field is specified here, so what typing in one *does* is
//! specified here as well: [`filter::Query`] is one line of typing turned into a question
//! you can ask of a candidate. Every word narrows, in any order; `!` excludes, `^` and
//! `$` anchor to an end. A filter that means one thing in one window and something else
//! in the next is the same kind of inconsistency as two greys.
//!
//! ```
//! # fn demo(rows: &[String]) {
//! let q = azur_egui_theme::filter::Query::parse("wor he");
//! let kept = rows.iter().filter(|row| q.matches(row));
//! # }
//! ```
//!
//! # Measuring, rather than counting tokens
//!
//! Every claim here about a colour being legible or too close to its neighbour is a
//! measurement, and [`contrast`] is the ruler — [`ratio`](contrast::ratio) for ink,
//! [`apart`](contrast::apart) for two surfaces. "Four steps up the ramp" is not a
//! quantity of anything: the dark neutral ramp is perceptually finer than the light
//! one, so the same number of named steps is a different amount of contrast on each
//! side. It is public because an application that adds a role of its own has made the
//! same kind of claim and can hold itself to the same floor.
//!
//! # Icons
//!
//! [`icons`] carries the glyphs the components need, painted rather than loaded
//! from a font. An application's own icons end up in the same rows as these — a
//! `Print` glyph in a `CommandBar` between two built-in ones — so they have to be
//! built on the same grid, at the same weight, taking their color from the
//! caller. [`icons::guidelines`] is the brief for that — and for the icon on the
//! executable, which is a raster mark under different rules — while
//! [`icons::Pen`] is the glyph geometry as code.
//!
//! # Presentation
//!
//! A one-pixel stroke is only one pixel if the frame reaches the screen the way it
//! was drawn, and on Windows that depends on the graphics backend: the compositor
//! resamples windows presented through OpenGL or D3D12 and leaves Vulkan alone.
//! [`render`] carries that decision and the measurement behind it, so an
//! application gets it by writing
//!
//! ```no_run
//! # fn demo() -> eframe::NativeOptions {
//! eframe::NativeOptions {
//!     renderer: eframe::Renderer::Wgpu,
//!     wgpu_options: azur_egui_theme::render::wgpu_options(),
//!     ..Default::default()
//! }
//! # }
//! ```
//!
//! # Shipping the binary
//!
//! [`deploy`] is `cargo deploy`: build the release binary and copy it to where it is
//! actually run from, replacing a copy that is *running* if that is what is in the
//! way. It is here for the same reason [`render`] is: it is a decision about how an
//! Azur application is built rather than about how it looks, and every application
//! that consumes this crate needs it. An application opts in with a two-line
//! `.cargo/config.toml` and an `examples/deploy.rs` that is one call; both are in
//! [`deploy`]'s own documentation.
//!
//! # What does not survive the port
//!
//! Three groups of tokens have no egui equivalent, and are transcribed for
//! reference rather than applied: line heights (an [`egui::FontId`] has no
//! leading — the components use the values in [`tokens::typography`] for the rows
//! they lay out themselves), easing curves (egui animates on one fixed curve), and
//! z-index ([`tokens::layer`] maps each stop onto an [`egui::Order`]).
//!
//! Azur's CSS transitions are not reproduced per property either. egui cross-fades
//! widget state on [`egui::Style::animation_time`], which [`install`] sets to
//! [`tokens::motion::FAST`]; the components that genuinely animate — `Spinner`, an
//! indeterminate `ProgressBar`, `Switch`, `Toast` — drive themselves off the frame
//! clock.
//!
//! # Versioning
//!
//! A theme is only meaningful for the exact egui it styles. This crate tracks
//! **egui 0.35**, and the application must resolve to that same version — two egui
//! versions in one graph are two different `Context` types, and the theme would
//! silently style neither.

pub mod components;
pub mod contrast;
/// `cargo deploy`, shared by every application that consumes this crate. Nothing to do
/// with drawing; see [`deploy`] for why it is here rather than in each of them.
pub mod deploy;
pub mod desktop;
pub mod filter;
pub mod fonts;
pub mod icons;
pub mod palettes;
/// Behind the `wgpu` feature, on by default; see [`render`] for why a design system
/// has an opinion about the graphics backend at all.
#[cfg(feature = "wgpu")]
pub mod render;
pub mod style;
pub mod theme;
pub mod tokens;
pub mod widgets;

pub use style::{
    install, install_dark, install_light, line_height, spacing, style, text_style, visuals,
    StyleOptions,
};
pub use theme::{Accents, Backgrounds, Fonts, Scrollbar, Status, Strokes, TextColors, Theme};
pub use widgets::{list_items, scrollbar};
