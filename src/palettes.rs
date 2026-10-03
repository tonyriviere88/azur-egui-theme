//! Named palettes for an application window: [`desktop`]'s theme in nine materials.
//!
//! ```no_run
//! use azur_egui_theme::palettes::Palette;
//! # fn demo(ctx: &egui::Context) {
//! let palette = Palette::parse("cobalt").unwrap_or_default();
//! let (theme, surfaces) = palette.build();
//! # }
//! ```
//!
//! A palette is **a variant of one of Azur's two sides rather than a side of its own**, and that
//! is the load-bearing part of this design: each one keeps its side's ink and status hues and
//! varies the surfaces and the accent, at the luminance they already had, which is what keeps the
//! contrast floors the ones already measured. A palette that wanted its own ink would be a second
//! design system.
//!
//! What a palette varies comes in two kinds, and they are different jobs:
//!
//! - **Roles** — [`Theme`]'s own fields. The [stone family](mod@family) varies only these: each of
//!   its six is the dark palette in another material.
//! - **Regions** — [`Surfaces`], one field per part of the window: title bar, path bar, panel,
//!   headers, sidebar, status bar, seam. Azur's roles answer "how far from the surface is this",
//!   and a palette that wants the sidebar told from the status bar is asking "which region is
//!   this" instead. [`celestine`](mod@celestine) and [`light`](mod@light) name theirs.
//!
//! Anything about *state* — one hover, one press, one selected row — stays Azur's.
//!
//! Every palette was cut for, and measured in, the file manager next door; the figures quoted
//! below that name its status line or its code palette are that application's roles, which it
//! holds to the same floors in its own tests.

use crate::desktop;
use crate::theme::Theme;
use egui::Color32;

/// A palette a window can be set to, by name.
///
/// Adding one is: a variant, a row in [`Palette::ALL`], its [`label`](Palette::label) and
/// [`key`](Palette::key), and an arm in [`Palette::build`]. Nothing else, and
/// `every_palette_is_reachable_and_round_trips` fails if any of those is forgotten.
///
/// Nine of them at the moment, six of which are one rule with a different mineral in it — see
/// [`family`](mod@family) — and the machinery is deliberately more than that needs: see
/// [`ALIASES`](Palette::ALIASES), which only exists because a palette has already been renamed
/// once.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Palette {
    /// Azur's own dark theme, graphite and azure, with [`desktop`]'s hover.
    #[default]
    Dark,
    /// Black. [`Dark`](Self::Dark)'s ladder sunk 3.5 ΔL\*, dead neutral, on a canvas of pure
    /// black. See [`family::ONYX`](mod@family).
    Onyx,
    /// Blue, and the palette the family was cut for: [`Dark`](Self::Dark)'s ladder rung for rung
    /// in stone and cobalt blue instead of graphite and azure. See [`family`](mod@family), which
    /// is where the hue and the two chromas are argued.
    Cobalt,
    /// Green. See [`family::MALACHITE`](mod@family).
    Malachite,
    /// Purple. See [`family::AMETHYST`](mod@family).
    Amethyst,
    /// Yellow. See [`family::CITRINE`](mod@family).
    Citrine,
    /// Metal: the family's ladder lifted as far as the measurements allow, with the rest of its
    /// brightness in the pigment. See [`family::SILVER`](mod@family).
    Silver,
    /// Pale sky blue, after the mineral: [`Theme::light`]'s ink, accent and status hues on a set
    /// of blue chrome surfaces of its own, around [`Light`](Self::Light)'s listing. See
    /// [`celestine`](mod@celestine).
    ///
    /// It was called `light-blue` and then `light`, and the first still parses — see
    /// [`ALIASES`](Palette::ALIASES).
    Celestine,
    /// [`Light`](Self::Celestine) on grey surfaces: a near-white panel in grey chrome, and every
    /// other colour — the ink, the accent, the status hues — [`Celestine`](Self::Celestine)'s.
    /// See [`light`](mod@light).
    ///
    /// It was called `white` while `light` was the blue one, which still parses — see
    /// [`ALIASES`](Palette::ALIASES).
    Light,
}

impl Palette {
    /// Every palette, in the order the menu lists them: Azur's own dark theme first because it is
    /// the default, then [the family](mod@family) up its own ladder — black, the four hues, metal
    /// — and the two light ones last, the grey before the blue. [`menu`] groups them by side
    /// under a heading each, which is why the dark ones come first as a block.
    pub const ALL: [Self; 9] = [
        Self::Dark,
        Self::Onyx,
        Self::Cobalt,
        Self::Malachite,
        Self::Amethyst,
        Self::Citrine,
        Self::Silver,
        Self::Light,
        Self::Celestine,
    ];

    /// Names [`parse`](Palette::parse) still answers to, and what they mean now.
    ///
    /// **A settings file outlives the name in it.** `theme=light-blue` was a real value that real
    /// settings files were written with, and when that palette was renamed the word did not stop
    /// existing on disk. Dropping it would not have failed loudly either — an unreadable name
    /// falls back to the default — so somebody's window would have quietly opened dark one
    /// morning.
    ///
    /// Each alias points where its palette went: `light-blue` was always the blue one, which is
    /// [`Celestine`](Palette::Celestine) now, and `white` was always the grey one, which is
    /// [`Light`](Palette::Light). `light` itself changed hands — it is the grey palette's key now —
    /// so a file written with it opens on the grey one, which is what the rename meant.
    ///
    /// The rule for adding to this list: an alias is for a name an application *used to write
    /// itself*, not for spellings somebody might guess. It is a compatibility record, and it
    /// should be readable as one.
    pub const ALIASES: [(&'static str, Self); 2] =
        [("light-blue", Self::Celestine), ("white", Self::Light)];

    /// What a menu calls it.
    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark theme",
            Self::Onyx => "Onyx theme",
            Self::Cobalt => "Cobalt theme",
            Self::Malachite => "Malachite theme",
            Self::Amethyst => "Amethyst theme",
            Self::Citrine => "Citrine theme",
            Self::Silver => "Silver theme",
            Self::Celestine => "Celestine theme",
            Self::Light => "Light theme",
        }
    }

    /// What a settings file calls it.
    ///
    /// `dark` and `light` are the two words settings files have always used, so a file written
    /// before there were more still reads correctly — which is the reason the key is spelled out
    /// here rather than derived from the variant name.
    pub fn key(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Onyx => "onyx",
            Self::Cobalt => "cobalt",
            Self::Malachite => "malachite",
            Self::Amethyst => "amethyst",
            Self::Citrine => "citrine",
            Self::Silver => "silver",
            Self::Celestine => "celestine",
            Self::Light => "light",
        }
    }

    /// The palette `text` names, or `None`.
    ///
    /// Case-insensitive, and `_` reads as `-` so that `light_blue` works as well as
    /// `light-blue` — a setting typed by hand should not turn on a hyphen. Retired names are
    /// answered too; see [`ALIASES`](Palette::ALIASES).
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim().replace('_', "-");
        let named = |key: &str| key.eq_ignore_ascii_case(&text);
        Self::ALL
            .into_iter()
            .find(|p| named(p.key()))
            .or_else(|| {
                Self::ALIASES
                    .into_iter()
                    .find(|(key, _)| named(key))
                    .map(|(_, p)| p)
            })
    }

    /// Whether this palette is a dark one, for the handful of decisions that really are about
    /// which side of the palette we are on rather than about which palette it is.
    pub fn is_dark(self) -> bool {
        !matches!(self, Self::Celestine | Self::Light)
    }

    /// The palette, built: the roles, with [`desktop::apply`]'s hover already in them, and the
    /// regions.
    ///
    /// **The desktop preset first, so the palette has the last word.** A palette that names its
    /// own hover — Onyx, Silver, Celestine — would otherwise have it silently put back.
    pub fn build(self) -> (Theme, Surfaces) {
        let mut t = if self.is_dark() {
            Theme::dark()
        } else {
            Theme::light()
        };
        desktop::apply(&mut t);
        let surfaces = match self {
            Self::Dark => Surfaces::from_theme(&t),
            Self::Onyx => family::ONYX.apply(&mut t),
            Self::Cobalt => family::COBALT.apply(&mut t),
            Self::Malachite => family::MALACHITE.apply(&mut t),
            Self::Amethyst => family::AMETHYST.apply(&mut t),
            Self::Citrine => family::CITRINE.apply(&mut t),
            Self::Silver => family::SILVER.apply(&mut t),
            Self::Celestine => celestine::apply(&mut t),
            Self::Light => light::apply(&mut t),
        };
        (t, surfaces)
    }
}

/// The palettes as the entries of a menu, grouped by side: `Dark themes` and `Light themes`, a
/// heading each, in [`Palette::ALL`]'s order. Answers with the one clicked, if any.
///
/// **Driven by [`Palette::ALL`]**, so a new palette appears here by existing, under the heading
/// [`is_dark`](Palette::is_dark) puts it. The current one is ticked.
///
/// Each entry carries the same `dot` the [`submenu`] row does — there is no appearance icon in
/// the set, and a row that opens a choice reading in the same visual family as the choice beats
/// an unrelated shape standing in for one. A click closes the whole menu stack, not just the level
/// it landed in, as a nested menu has to.
pub fn menu(ui: &mut egui::Ui, current: Palette) -> Option<Palette> {
    use crate::components::{menu_divider, menu_header, MenuItem};
    let mut chosen = None;
    for (heading, dark) in [("Dark themes", true), ("Light themes", false)] {
        if !dark {
            menu_divider(ui);
        }
        menu_header(ui, heading);
        for palette in Palette::ALL.into_iter().filter(|p| p.is_dark() == dark) {
            let item = MenuItem::new(palette.label())
                .selected(palette == current)
                .icon(&crate::icons::dot);
            if ui.add(item).clicked() {
                chosen = Some(palette);
                ui.close();
            }
        }
    }
    chosen
}

/// A `Theme` entry that opens [`menu`] beside it, for an application menu.
///
/// Behind one entry rather than side by side in the open, because the palettes are a *choice
/// between* things and not a row of switches, which a submenu says by shape: one row naming the
/// question, and the answers a level in.
pub fn submenu(ui: &mut egui::Ui, current: Palette) -> Option<Palette> {
    use crate::components::MenuItem;
    let entry = MenuItem::new("Theme").icon(&crate::icons::dot);
    crate::components::submenu(ui, entry, |ui| menu(ui, current)).and_then(|shown| shown.inner)
}

/// The surfaces a window's *regions* are painted with — the part a palette varies beyond roles.
///
/// # Why these are not Azur roles
///
/// Because a palette wants to tell them apart and Azur's roles cannot. In Azur's light theme the
/// sidebar, the column headers and the status bar are all `background-layer-alt` and the title bar
/// is `background-layer`; [`Palette::Celestine`] gives those four surfaces three *different*
/// colours, and makes the panel and its headers the same one. No remapping of `layer` / `layer_alt`
/// expresses that, because the question it answers is "which region is this", and Azur's question
/// is "how far from the surface is this".
///
/// So this is one field per region, and [`Surfaces::from_theme`] is the mapping every region had
/// before regions were named — which is what [`Palette::Dark`] and the whole
/// [family](mod@family) still use, to the byte.
///
/// **A region, not a widget.** What goes here is a surface some part of the window *is*; a fill a
/// control takes because it is hovered or pressed or selected stays Azur's, because that ladder
/// is about state and states are the same everywhere. [`desktop::hover_fill`] is still the one
/// hover.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Surfaces {
    /// The strip the tabs sit in, and the bands of tabs below it when panes are stacked.
    pub titlebar: Color32,
    /// **One surface, three things**: the focused pane's active tab, the path bar under it, and
    /// the filter field at the right-hand end of that bar.
    ///
    /// They are welded on purpose — the tab is the visible end of the bar, which is why a
    /// hairline is not drawn between them — so they are one value and cannot drift apart.
    pub bar: Color32,
    /// A pane's listing, and the surface a preview or a console panel sits on.
    pub panel: Color32,
    /// The column headers along the top of a listing.
    pub header: Color32,
    /// The panel down the left: drives, bookmarks, places.
    pub sidebar: Color32,
    /// The line along the bottom of a pane.
    pub status: Color32,
    /// **The line between two panels**, and nothing else — not the surface the path bar is
    /// painted with, which is [`bar`](Surfaces::bar).
    ///
    /// Those two were one value (`stroke-subtle`) until a palette wanted a pale bar and a visible
    /// seam, which one colour cannot be. In [`Palette::Dark`] they are still the same colour; they
    /// are simply no longer the same *decision*. See [`desktop::seam`].
    pub separator: Color32,
}

impl Surfaces {
    /// Every region on the Azur role it already had, which is what [`Palette::Dark`] wears.
    ///
    /// Read it as the record of what a window painted before regions were named, because that
    /// is exactly what it is — `a_palette_with_no_surfaces_of_its_own_paints_what_it_always_did`
    /// holds it.
    pub fn from_theme(t: &Theme) -> Self {
        Self {
            titlebar: t.bg.layer,
            bar: t.stroke.subtle,
            panel: t.bg.layer,
            header: t.bg.layer_alt,
            sidebar: t.bg.layer_alt,
            status: t.bg.layer_alt,
            separator: t.stroke.subtle,
        }
    }

    /// Every region, named, for the measurements over them.
    ///
    /// Exhaustive by construction — it destructures `self`, so adding a field to [`Surfaces`]
    /// stops compiling here until it is listed. A test that iterates six of seven regions is a
    /// test that passes because of the one it forgot.
    pub fn each(&self) -> [(&'static str, Color32); 7] {
        let Self {
            titlebar,
            bar,
            panel,
            header,
            sidebar,
            status,
            separator,
        } = *self;
        [
            ("title bar", titlebar),
            ("path bar", bar),
            ("panel", panel),
            ("column headers", header),
            ("sidebar", sidebar),
            ("status bar", status),
            ("seam", separator),
        ]
    }
}

/// The **stone family**: six palettes cut by one rule, and the Azur roles they are carried into.
///
/// # One rotation, and nothing else
///
/// None of these is a new dark theme. Each is [`Palette::Dark`]'s window in a different
/// **material**: every rung below replaces a graphite or an azure one **at that rung's own
/// relative luminance**, so a palette is a hue and a chroma and nothing more.
///
/// That is the load-bearing property, because [`contrast`](crate::contrast) measures in the two
/// quantities luminance decides — [`ratio`](crate::contrast::ratio) is a function of it and
/// [`lstar`](crate::contrast::lstar) is a function of it — so **every contrast figure and every
/// ΔL\* already measured carries over.** The seam is 3.93 ΔL\* off a column header in all six as
/// it is in the dark palette, the checkerboard 15.4, the selection bar 3.1:1 on the row it marks,
/// `status.danger` 4.67:1 on the status line. Nothing was retuned, and nothing needed to be.
/// `every_rung_is_the_dark_palettes_own_lightness` is the guard: it holds every rung to 0.25
/// ΔL\* of the ladder position it stands in for, which is half a byte step down here and no
/// more — the worst in the six is 0.21.
///
/// It also means none of them **names a region of its own**: the seven [`Surfaces`] come off Azur
/// roles through [`Surfaces::from_theme`] exactly as [`Palette::Dark`]'s do, and what moved is what
/// those roles *are*. Compare [`celestine`](mod@celestine), which had to name regions because it
/// wanted the sidebar and the status bar told apart — a question about structure, where this one is
/// about colour.
///
/// # The mineral picks the hue *and* the chroma
///
/// Each palette is measured off the stone it is named for, and both of its numbers fall out of
/// that one measurement:
///
/// | palette | the stone | `L*` | `C*` | Lab hue | share of sRGB's chroma at that lightness |
/// | --- | --- | --- | --- | --- | --- |
/// | [`COBALT`](family::COBALT) | cobalt blue `#0047AB` | 32.80 | 62.64 | 291.1 | **100.0%** |
/// | [`MALACHITE`](family::MALACHITE) | malachite `#0BDA51` | 76.56 | 89.42 | 143.1 | 99.3% |
/// | [`CITRINE`](family::CITRINE) | citrine `#E4D00A` | 82.72 | 82.69 | 96.8 | 98.9% |
/// | [`AMETHYST`](family::AMETHYST) | amethyst `#9966CC` | 52.55 | 60.72 | 311.6 | **61.3%** |
/// | [`ONYX`](family::ONYX) | onyx `#353839` | 23.27 | 1.50 | — | 0% |
/// | [`SILVER`](family::SILVER) | silver `#C0C0C0` | 77.70 | 0.01 | — | 0% |
///
/// **The last column is the interesting one.** Three of these stones sit *on* the sRGB gamut
/// boundary at their own hue and lightness — cobalt blue exactly on it — because they are
/// pigments, and a pigment is as saturated as its medium allows. Amethyst is a pale violet quartz
/// and takes 61% of what is there, which is why the amethyst palette reads as a gem rather than as
/// neon. Nobody chose that; it was measured.
///
/// So: **the pigment ramp takes the share its mineral takes, and the stone ramp takes half of
/// that.** Half because a rock is not a pigment — the panels have to read as *dark* first and
/// coloured second, or the listing competes with what it is listing.
///
/// **A share of a ceiling and not a fixed chroma**, because the ceiling is not flat: it collapses
/// toward white and opens up toward black. [`celestine`](mod@celestine)'s table is the same
/// measurement from the other end — there the bluest colour that exists at `L*` 98 is `C*` 3.4 —
/// and down here, at cobalt's hue, it runs the other way:
///
/// | rung | `L*` | the most chroma sRGB has there | cobalt takes |
/// | --- | --- | --- | --- |
/// | [`Cut::layer`](family::Cut::layer) — the panel | 7.55 | 30.2 | 15.1 |
/// | [`Cut::control_hover`](family::Cut::control_hover) — the hover | 34.77 | 65.0 | 33.1 |
/// | [`Cut::accent_active`](family::Cut::accent_active) — a selected row | 32.26 | 61.8 | 61.6 |
/// | [`Cut::accent_mark`](family::Cut::accent_mark) — the accent as ink | 64.37 | 59.1 | 58.4 |
///
/// Read the last two rows together: the accent's fill is *more* chromatic than the ink above it
/// even though it is half as bright, which is why cobalt reads as a deep colour rather than a
/// bright one. It is also why [`Cut::accent_mark`](family::Cut::accent_mark) stays at azure's own
/// ink rung instead of coming down to where the ceiling is higher. That was tried — `L*` 60 buys
/// `C*` 66 — and it costs the
/// one figure the ink cannot spend: the 2px selection bar is `accent.mark` **on** `accent.active`,
/// and at `L*` 60 that falls to 2.69:1, under the 3:1 a shape wants. At the rung it kept it is
/// 3.13:1, which is the 3.14:1 the dark palette measures.
///
/// # The two with no hue
///
/// [`ONYX`](family::ONYX) and [`SILVER`](family::SILVER) cannot be told apart by a rotation,
/// having nothing to rotate. They are told apart by **where their ladder sits**, and each then
/// has to do by hand the one job a hue
/// was doing for the others:
///
/// - **Onyx sinks 3.5 ΔL\***, and its canvas clamps at `#000000` — the window's own colour is
///   black, 4.0 ΔL\* under the panel that sits on it, which still reads as a surface. Its pigment
///   stays on Azur's own lightnesses: a plain mid-grey selection. Matte.
/// - **Silver lifts 1.7 ΔL\***, and its pigment goes the other way — a bright steel selection and
///   near-white marks, `accent.mark` at `L*` 84.9 against Azur's 64.4.
///
/// **1.7 is the whole of the room there is, and it is a measurement rather than a preference.**
/// Two values stop a dark palette lifting its surfaces: `status.danger` drops under 4.5:1 on a
/// status bar lighter than `L*` 11.52, and the file manager's code-comment green drops under
/// 4.5:1 on a panel lighter than `L*` 10.13. The dark palette's own rungs are 9.72 and 7.55, so the
/// room is **1.80 and 2.58 ΔL\*** — and 1.7 is inside both. A palette that wanted a genuinely pale
/// window would need status and code inks of its own, which is exactly what the light side has;
/// Silver's brightness is in its marks instead, where nothing else is measured against it.
///
/// # The hover, which is the other thing a hue was doing
///
/// `azur::desktop::row_fill` fills a selected row with `accent-active` and a hovered one with
/// `background-control-hover`, and in the dark palette those two are **2.5 ΔL\* apart**: a
/// selected row is told from a hovered one almost entirely by *hue*. The four chromatic palettes
/// inherit that and it works. The two hueless ones would have a hovered row and a selected row in
/// the same grey, so both bring the hover **down** — Onyx to `L*` 26.2 and Silver to 28.0, which
/// leaves 6.1 and 11.9 clear of their selections. The floor under that is
/// `the_hueless_palettes_tell_a_selected_row_from_a_hovered_one`, and it is asserted only for
/// those two: for the others the same
/// measurement would fail, correctly.
///
/// It costs a little tidiness. Sixteen rungs on one grey ramp crowd, so Onyx's hover lands within
/// 1.3 ΔL\* of its field outline and of a pressed badge. Neither of those is a row, which is the
/// only place the question gets asked.
///
/// # The one thing these palettes cannot reach
///
/// `azur::desktop::press_fill` is a function of `t.dark` rather than a token, so the fill a
/// control takes **while the button is held** is Azur's `GRAY_9` in every dark palette. Its
/// lightness is right — every [`Cut::stroke_strong`](family::Cut::stroke_strong) here is the same
/// `L*` — but its hue is graphite, so in the four chromatic palettes a pressed toolbar button
/// flashes grey where its
/// hover was coloured. The hover is reachable because `azur::desktop::apply` writes it into
/// `background-control-hover` and `hover_fill` reads the token back; the press has no token behind
/// it. Fixing it belongs in the design system, and until then it is a transient state whose ladder
/// position is correct — and in [`ONYX`](family::ONYX) and [`SILVER`](family::SILVER) it is
/// simply right.
pub mod family {
    use egui::Color32;

    /// `0xrrggbb`, so a sixteen-rung table fits on sixteen lines and can be read down the column
    /// as a ramp. Every value in this module is one of these and nothing is derived at runtime.
    const fn hex(v: u32) -> Color32 {
        Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
    }

    /// One palette of the family, as the sixteen rungs it is cut from.
    ///
    /// Eight of stone and eight of pigment, named after the Azur role each one *leads* — the fan
    /// out to the roles that share it is [`Cut::apply`], which is the only place the mapping
    /// exists. Read the two halves as ramps: each field is one step further from the surface than
    /// the one above it, in the order [`Palette::Dark`](super::Palette::Dark) climbs its own.
    ///
    /// There is no `hover` field on the pigment side and no `press` anywhere: a press is
    /// `azur::desktop`'s and the accent's hover is [`Cut::accent_hover`].
    #[derive(Clone, Copy)]
    pub struct Cut {
        /// The window itself, and so the fill behind a code block and the console's log.
        pub canvas: Color32,
        /// A pane's listing, the title bar, a card, a control at rest.
        pub layer: Color32,
        /// One step off it: the popover surface, the column headers, the sidebar, the status bar,
        /// the tab band, the zebra stripe.
        pub layer_alt: Color32,
        /// The seam, the path bar and the tab welded to it, and a field's fill.
        pub control: Color32,
        /// The loudest neutral: a panel edge, the find bar's outline, the checkerboard's dark
        /// square, and the inert fill a row takes while it is dragged.
        pub control_active: Color32,
        /// A control's own outline, and the scrollbar.
        pub stroke_control: Color32,
        /// The band under the pointer, on anything you hover in order to *go* somewhere.
        pub control_hover: Color32,
        /// The outline of a control under the pointer, and the cursor ring off a selection.
        pub stroke_strong: Color32,

        /// An accent-tinted fill quiet enough to put text on: every other hit in the find bar, an
        /// accent badge, and a selection in a pane that has not got the keyboard.
        pub accent_subtle: Color32,
        pub accent_subtle_hover: Color32,
        pub accent_subtle_active: Color32,
        /// **A selected row, at rest.** The whole reason the pigment ramp is walked from this end:
        /// a selected row is read *through*, so its fill goes down the ramp rather than up it.
        pub accent_active: Color32,
        /// A selected row under the pointer — and a primary button, the current hit in the find
        /// bar, and a `Kind::Code` glyph.
        pub accent_default: Color32,
        /// A primary button under the pointer.
        pub accent_hover: Color32,
        /// The accent as **ink**: the 2px bar down a selected row, the cursor ring on one, the
        /// focus ring, the selected count on the status line, and the branch beside it.
        pub accent_mark: Color32,
        /// A link in a rendered document, which Azur puts a rung above the ink because it is read
        /// on the canvas rather than through it.
        pub link: Color32,
    }

    impl Cut {
        /// Carry the sixteen rungs into every Azur role that wears one, and answer with the seven
        /// regions that fall out.
        ///
        /// Exhaustive over the neutral *surfaces* and the accent, and deliberately not over the
        /// ink: `text.primary` down to `text.disabled` stay Azur's, for the reason the
        /// [module](super) gives — a palette that wanted its own ink would be a second design
        /// system, and those four are a near-white and three greys whose job is to recede.
        /// [`Cut::link`] is the exception because a link is accent-coloured by role.
        ///
        /// The status hues are untouched for that reason and one more: they are *categorical*,
        /// and a category that changes colour with the palette is a category nobody can learn.
        pub fn apply(&self, az: &mut super::Theme) -> super::Surfaces {
            az.bg.canvas = self.canvas;

            az.bg.layer = self.layer;
            // A disabled control sinks back into the surface, as it does in Azur's dark theme.
            az.bg.control_disabled = self.layer;

            az.bg.layer_alt = self.layer_alt;
            az.bg.card = self.layer_alt;
            az.row_alt = self.layer_alt;

            az.bg.control = self.control;
            az.stroke.subtle = self.control;

            az.bg.control_active = self.control_active;
            az.stroke.default = self.control_active;
            // Here rather than on the pigment ramp, because the point of a disabled accent is
            // that it stops reading as live.
            az.accent.disabled = self.control_active;

            az.stroke.control = self.stroke_control;
            az.scrollbar.thumb = self.stroke_control;

            // **The hover, in this palette's material.** `azur::desktop::apply` has already put
            // both hover tokens on its own rung — see `Palette::build`, which runs it first for
            // exactly this reason — and that rung is `GRAY_8`, graphite. Both tokens, so a hovered
            // row, button, tab, menu entry and card stay one colour.
            az.bg.control_hover = self.control_hover;
            az.bg.card_hover = self.control_hover;

            az.stroke.strong = self.stroke_strong;
            az.scrollbar.thumb_hover = self.stroke_strong;

            // The scrim behind a modal: the canvas at 72%, which is what Azur's own is — its
            // `rgba(5,6,7,.72)` is `GRAY_0` less a byte or two.
            let c = self.canvas;
            az.bg.overlay = Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 184);

            az.accent.subtle = self.accent_subtle;
            az.status.info_subtle = self.accent_subtle;
            az.accent.subtle_hover = self.accent_subtle_hover;
            az.accent.subtle_active = self.accent_subtle_active;

            az.accent.active = self.accent_active;
            az.accent.default = self.accent_default;
            az.accent.hover = self.accent_hover;

            az.accent.mark = self.accent_mark;
            az.stroke.focus = self.accent_mark;
            az.status.info = self.accent_mark;

            az.text.link = self.link;

            super::Surfaces::from_theme(az)
        }
    }

    /// Black. The family's ladder sunk 3.5 ΔL\*, dead neutral, on a canvas of pure black.
    pub const ONYX: Cut = Cut {
        canvas: hex(0x000000),               // L*  0.00  C*  0.0
        layer: hex(0x0e0e0e),                // L*  3.97
        layer_alt: hex(0x141414),            // L*  6.32
        control: hex(0x1c1c1c),              // L* 10.27
        control_active: hex(0x2f2f2f),       // L* 19.40
        stroke_control: hex(0x3b3b3b),       // L* 24.87
        control_hover: hex(0x3e3e3e),        // L* 26.21 — its own; see "the hover"
        stroke_strong: hex(0x595959),        // L* 37.82
        accent_subtle: hex(0x191919),        // L*  8.76
        accent_subtle_hover: hex(0x292929),  // L* 16.59
        accent_subtle_active: hex(0x3a3a3a), // L* 24.42
        accent_active: hex(0x4c4c4c),        // L* 32.32
        accent_default: hex(0x5d5d5d),       // L* 39.49
        accent_hover: hex(0x757575),         // L* 49.24
        accent_mark: hex(0x9c9c9c),          // L* 64.36
        link: hex(0xc4c4c4),                 // L* 79.16
    };

    /// Blue. Lab hue 291.1 at the whole of sRGB's chroma, which is cobalt blue itself:
    /// [`accent_active`](Cut::accent_active) is `#0046A8` against the pigment's `#0047AB`.
    pub const COBALT: Cut = Cut {
        canvas: hex(0x07051c),               // L*  2.15  C* 13.3
        layer: hex(0x131529),                // L*  7.54  C* 15.1
        layer_alt: hex(0x171930),            // L*  9.73  C* 17.0
        control: hex(0x1c213d),              // L* 13.66  C* 20.0
        control_active: hex(0x2e345b),       // L* 22.97  C* 26.0
        stroke_control: hex(0x38406c),       // L* 28.35  C* 28.4
        control_hover: hex(0x464e83),        // L* 34.78  C* 33.1
        stroke_strong: hex(0x555d9b),        // L* 41.42  C* 37.5
        accent_subtle: hex(0x001642),        // L*  8.71  C* 32.6
        accent_subtle_hover: hex(0x012561),  // L* 16.47  C* 41.6
        accent_subtle_active: hex(0x013586), // L* 24.56  C* 53.1
        accent_active: hex(0x0046a8),        // L* 32.27  C* 61.6 — the pigment
        accent_default: hex(0x0056cd),       // L* 39.69  C* 72.1
        accent_hover: hex(0x006cfc),         // L* 49.20  C* 83.9
        accent_mark: hex(0x8095fe),          // L* 64.35  C* 58.4
        link: hex(0xbcbfff),                 // L* 79.14  C* 34.2
    };

    /// Green. Lab hue 143.1 at 99.3% of the ceiling — malachite is a copper pigment and sits
    /// almost exactly on the gamut boundary, as cobalt does.
    pub const MALACHITE: Cut = Cut {
        canvas: hex(0x050905),               // L*  2.16  C*  2.0
        layer: hex(0x0f190f),                // L*  7.51  C*  8.0
        layer_alt: hex(0x111e11),            // L*  9.71  C* 11.4
        control: hex(0x152716),              // L* 13.64  C* 15.0
        control_active: hex(0x233d24),       // L* 23.07  C* 20.4
        stroke_control: hex(0x2c4a2d),       // L* 28.44  C* 22.9
        control_hover: hex(0x375a39),        // L* 34.84  C* 25.4
        stroke_strong: hex(0x436b44),        // L* 41.42  C* 28.8
        accent_subtle: hex(0x031e01),        // L*  8.57  C* 20.2
        accent_subtle_hover: hex(0x003109),  // L* 16.58  C* 32.4
        accent_subtle_active: hex(0x004513), // L* 24.65  C* 39.8
        accent_active: hex(0x00591b),        // L* 32.31  C* 47.3
        accent_default: hex(0x016d23),       // L* 39.69  C* 54.4
        accent_hover: hex(0x03882e),         // L* 49.28  C* 63.6
        accent_mark: hex(0x08b443),          // L* 64.24  C* 76.9
        link: hex(0x0ce255),                 // L* 79.11  C* 91.6
    };

    /// Purple. Lab hue 311.6 at **61.3%** of the ceiling, which is the share the gemstone itself
    /// takes — amethyst is a pale violet quartz, not a pigment, and that one number is the whole
    /// difference between this palette and a neon one.
    pub const AMETHYST: Cut = Cut {
        canvas: hex(0x0b0610),               // L*  2.16  C*  4.7
        layer: hex(0x1d1226),                // L*  7.53  C* 15.8
        layer_alt: hex(0x22162b),            // L*  9.71  C* 16.4
        control: hex(0x2b1d37),              // L* 13.64  C* 19.5
        control_active: hex(0x412f54),       // L* 22.99  C* 25.7
        stroke_control: hex(0x503966),       // L* 28.36  C* 30.3
        control_hover: hex(0x62467a),        // L* 34.77  C* 34.0
        stroke_strong: hex(0x725591),        // L* 41.40  C* 37.8
        accent_subtle: hex(0x240c3c),        // L*  8.69  C* 35.1
        accent_subtle_hover: hex(0x371957),  // L* 16.48  C* 42.8
        accent_subtle_active: hex(0x4c257a), // L* 24.53  C* 55.3
        accent_active: hex(0x643199),        // L* 32.27  C* 64.9
        accent_default: hex(0x7a3dbb),       // L* 39.69  C* 75.7
        accent_hover: hex(0x905cc8),         // L* 49.20  C* 64.5
        accent_mark: hex(0xb58bd8),          // L* 64.36  C* 45.2
        link: hex(0xd3bbe8),                 // L* 79.17  C* 25.6
    };

    /// Yellow. Lab hue 96.8 at 98.9% of the ceiling. Gold at the ink rungs and olive at the
    /// surface ones, which is not a compromise but what a yellow *is* down here: there is no
    /// bright yellow at `L*` 32, and citrine's own darker tones are exactly this brown-gold.
    pub const CITRINE: Cut = Cut {
        canvas: hex(0x090803),               // L*  2.15  C*  2.0
        layer: hex(0x18170d),                // L*  7.55  C*  6.0
        layer_alt: hex(0x1d1b12),            // L*  9.71  C*  6.4
        control: hex(0x262315),              // L* 13.64  C* 10.0
        control_active: hex(0x3c3720),       // L* 22.99  C* 15.3
        stroke_control: hex(0x4a4325),       // L* 28.39  C* 19.3
        control_hover: hex(0x595231),        // L* 34.76  C* 20.5
        stroke_strong: hex(0x6a623b),        // L* 41.40  C* 23.5
        accent_subtle: hex(0x1e1900),        // L*  8.75  C* 13.2
        accent_subtle_hover: hex(0x2f2902),  // L* 16.48  C* 23.2
        accent_subtle_active: hex(0x413b02), // L* 24.50  C* 32.8
        accent_active: hex(0x544d02),        // L* 32.25  C* 40.1
        accent_default: hex(0x675f01),       // L* 39.70  C* 46.9
        accent_hover: hex(0x827604),         // L* 49.14  C* 54.3
        accent_mark: hex(0xac9e06),          // L* 64.35  C* 67.3
        link: hex(0xd9c60a),                 // L* 79.13  C* 79.6
    };

    /// Metal. The family's ladder lifted 1.7 ΔL\* — all the room there is — with the brightness
    /// that could not go into the surfaces put into the pigment instead: a steel selection and a
    /// near-white mark at `L*` 84.9.
    pub const SILVER: Cut = Cut {
        canvas: hex(0x0e0e0e),               // L*  3.97  C*  0.0
        layer: hex(0x1a1a1a),                // L*  9.26
        layer_alt: hex(0x1e1e1e),            // L* 11.26
        control: hex(0x262626),              // L* 15.16
        control_active: hex(0x3b3b3b),       // L* 24.87
        stroke_control: hex(0x474747),       // L* 30.16
        control_hover: hex(0x424242),        // L* 27.97 — its own; see "the hover"
        stroke_strong: hex(0x666666),        // L* 43.19
        accent_subtle: hex(0x303030),        // L* 19.87 ┐
        accent_subtle_hover: hex(0x3e3e3e),  // L* 26.21 │ the metal: the pigment ramp lifted, and
        accent_subtle_active: hex(0x4b4b4b), // L* 31.89 │ capped at `L*` 45.6, where the module
        accent_active: hex(0x5e5e5e),        // L* 39.90 │ glyph on a selected row would fall under
        accent_default: hex(0x6a6a6a),       // L* 44.82 ┘ the 3:1 a shape wants
        accent_hover: hex(0x727272),         // L* 48.04
        accent_mark: hex(0xd4d4d4),          // L* 84.91 — white-hot, against Azur's 64.4
        link: hex(0xc4c4c4),                 // L* 79.16
    };
}

pub mod celestine {
    //! The five colours [`Palette::Celestine`](super::Palette::Celestine) is specified as, and the
    //! Azur roles they are carried back into.
    //!
    //! One module per palette that has surfaces of its own, named after it.
    //! [`Palette::Dark`](super::Palette::Dark) has none — it takes every region straight from an
    //! Azur role, which is [`Surfaces::from_theme`](super::Surfaces::from_theme).
    //!
    //! Every value in this module is one of the five. Nothing is interpolated and nothing is
    //! derived, which is the property that makes the palette checkable by reading it against the
    //! request it came from.
    //!
    //! # Why a bluer near-white is a darker one
    //!
    //! The five regions below [`TITLEBAR`] were asked to be *more blue* without it, and the reason
    //! they also moved in lightness is a property of sRGB rather than a liberty taken here: **the
    //! chroma available to a colour collapses as it approaches white.** Measured at Lab hue 262, in
    //! `C*`:
    //!
    //! | lightness | the most chroma sRGB has there |
    //! | --- | --- |
    //! | `L*` 98 | 3.4 |
    //! | `L*` 97 | 5.1 |
    //! | `L*` 95 | 7.9 |
    //! | `L*` 93 | 11.0 |
    //! | `L*` 90.7 — [`TITLEBAR`]'s | 14.6 |
    //!
    //! The previous values were already spending 61–76% of that ceiling, so there was no room to
    //! make them bluer where they stood: the old bar was `L*` 97.69, where the bluest colour that
    //! exists at all is `C*` 4.02, and it was at 3.06.
    //!
    //! So the four content surfaces came **down 0.46 `L*` together** — the ladder keeps its shape
    //! exactly, every rung in the same order the same distance apart — and each then takes **95% of
    //! the chroma its new lightness allows**, which is as blue as sRGB goes there. That buys 1.3×
    //! to 1.8× the chroma for about half a point of lightness, and the panel ends at `C*` 8.04
    //! against [`TITLEBAR`]'s 9.91, so the window now reads as one blue family rather than as blue
    //! chrome around pale grey panels.
    //!
    //! [`SEPARATOR`] moved for a second reason as well as chroma: the preview's checkerboard is
    //! measured between the panel and `background-control-active`, and wants 8 ΔL\* between them.
    //! The darker panel no longer left that, so the seam came down to `L*` 85.43 — where it can
    //! hold `C*` 16.49, the most saturated thing in the palette, which suits the one surface whose
    //! whole job is to be a boundary.
    //!
    //! **[`TITLEBAR`] is untouched**, and so is everything reading `background-layer-alt` from it:
    //! the context menus, the popovers, the tab band and the code-block fill. Those were judged
    //! right as they were.
    //!
    //! # The listing is [`Light`](super::Palette::Light)'s
    //!
    //! Since then [`PANEL`] and [`STATUS`] — the file list, its column headers and the status bar —
    //! were asked to be [`light`](super::light)'s: `#fcfcfc` and `#f5f5f5`, near-white greys where
    //! the blue ones above were. They are the same constants rather than copies, so the two
    //! palettes cannot drift apart. The blue is now the chrome's — [`TITLEBAR`], [`BAR`] and
    //! [`SEPARATOR`] — around a listing that reads as paper, and the table above is the record of
    //! where the old panel stood.
    //!
    //! It leaves [`BAR`], `L*` 97.32, a step *under* the panel at 98.96 rather than the lightest
    //! thing in the window; the tab welded to it still reads as one piece with it, which is the
    //! only thing the bar's lightness was for. The checkerboard has 13.5 ΔL\* now, against the 8 it
    //! wants.

    use egui::Color32;

    pub const TITLEBAR: Color32 = Color32::from_rgb(0xd8, 0xe6, 0xf7);
    pub const BAR: Color32 = Color32::from_rgb(0xf2, 0xf8, 0xff);
    /// [`light::PANEL`](super::light::PANEL), `#fcfcfc` — see "the listing is Light's".
    pub const PANEL: Color32 = super::light::PANEL;
    /// [`light::STATUS`](super::light::STATUS), `#f5f5f5`.
    pub const STATUS: Color32 = super::light::STATUS;
    pub const SEPARATOR: Color32 = Color32::from_rgb(0xc0, 0xd8, 0xf4);

    /// The regions, as given. `header` is [`PANEL`] because that is what was asked for — the
    /// headers and the listing under them are one surface here, where Azur's own light theme has
    /// them a rung apart. It stays a field of its own rather than becoming an alias: the next
    /// palette may well separate them again.
    pub(super) fn surfaces() -> super::Surfaces {
        super::Surfaces {
            titlebar: TITLEBAR,
            bar: BAR,
            panel: PANEL,
            header: PANEL,
            // **[`TITLEBAR`], because the side panel is chrome.** It began as a rung of its own,
            // one step *lighter* than the panel, and read as too bright — which is the reading
            // Azur's own light theme agrees with: there the sidebar sits with the popovers and the
            // status bar on `background-layer-alt`, a step down from the listing, not up from it.
            //
            // It is the title bar's surface rather than a new value between the two because there
            // was no room for one: the panel and the title bar were 3.98 ΔL* apart, so anything
            // convincingly darker than the panel was within 2 ΔL* of the title bar anyway. Making
            // it the same surface says what the near-miss only implied — the strip along the top
            // and the panel down the left are one piece of chrome — and both of its edges are
            // drawn by a seam, so nothing depends on the two being told apart.
            sidebar: TITLEBAR,
            status: STATUS,
            separator: SEPARATOR,
        }
    }

    /// Carry the same six colours back into the Azur roles, for everything the window paints
    /// that is *not* one of the named regions, and answer with the regions.
    ///
    /// The regions above cover the window's structure; these five cover the rest of it — a
    /// context menu, a popover, a code block, a zebra stripe, a field, a dragged row. Without
    /// this the leftovers would keep Azur's own grey-blue ladder, and the one that shows is the
    /// tab band: it exists to stand in for the title bar for the row below it, so a band 3.9 ΔL*
    /// off the title bar is a band that has visibly failed at its only job.
    ///
    /// **Two rungs, not five.** [`PANEL`] is everything content sits on and [`TITLEBAR`] is
    /// everything set one step off it, which is the same two-rung shape Azur's light theme has —
    /// `SHEET` and `SHADE` — read off this palette's own colours instead of `paper`'s.
    pub fn apply(az: &mut super::Theme) -> super::Surfaces {
        // Content sits on the panel: a card, a field, a button at rest.
        az.bg.layer = PANEL;
        az.bg.card = PANEL;
        az.bg.control = PANEL;

        // One step off it. `layer_alt` is the popover surface — egui's `window_fill`, so every
        // menu and tooltip in the window — and it is also the tab band, an unfocused pane's
        // active tab, a hovered tab and the zebra stripe.
        az.bg.layer_alt = TITLEBAR;
        az.bg.control_disabled = TITLEBAR;
        az.row_alt = TITLEBAR;

        // The window's own colour, showing through the panels that sit on it: the fill behind a
        // code block and behind the console's log, which is what makes monospace read as
        // recessed. One step down from the panel, as in the light theme, and the same step the
        // popovers take.
        az.bg.canvas = TITLEBAR;

        // Lines, and the inert fill. `stroke.subtle` is every border the design system draws at
        // popover elevation plus the find bar's edge, and `control_active` is what a row or a tab
        // takes while it is dragged — both want the palette's darkest rung, which is the seam's.
        az.stroke.subtle = SEPARATOR;
        az.bg.control_active = SEPARATOR;

        // **The hover, in this palette's hue.** `azur::desktop::apply` has already put both hover
        // tokens on its own rung — see `Palette::build`, which runs it first for exactly this
        // reason — and that rung is the *light* theme's: Lab hue 272, which against hue-262
        // surfaces reads as a lavender-grey band. So the band keeps its lightness to the hundredth
        // and takes this palette's hue instead.
        //
        // **Its lightness is not negotiable**, which is why only the hue and the chroma move:
        // `L*` 74.50 is a measured position — `azur::desktop::place_hover` has the argument, and
        // the short version is that the ink on a row does not change when the pointer arrives, so
        // a hover cannot cross to the other side of it. Holding `L*` exactly means every figure
        // already measured on this band still holds: a name on it is 7.32:1 against the 7.33:1 it
        // was, and the metadata columns 3.60:1.
        //
        // `C*` 18.2 and not the 37.9 that "as blue as sRGB allows" would give here: the ceiling
        // explodes away from white, and 95% of it at this lightness is a saturated sky blue rather
        // than a tinted neutral. A hover is a *surface*, so it is held to the palette's own range
        // — just past [`SEPARATOR`]'s 16.5, the most chromatic thing here.
        //
        // Both tokens, so a hovered row, button, tab, menu entry and card stay one colour.
        az.bg.control_hover = Color32::from_rgb(0x9f, 0xba, 0xd8);
        az.bg.card_hover = az.bg.control_hover;

        surfaces()
    }
}

pub mod light {
    //! [`Palette::Light`](super::Palette::Light): [`celestine`](super::celestine) with **grey
    //! surfaces** — a near-white panel in grey chrome — and every other colour exactly as
    //! Celestine has it.
    //!
    //! # The surfaces
    //!
    //! The [`celestine`](super::celestine) palette's shape — the headers welded to the panel, the
    //! sidebar on the title bar's chrome, the path bar the lightest thing in the window — with the
    //! panel pushed up to almost white, so a pane reads as paper and everything around it as grey.
    //!
    //! | region | value | `L*` |
    //! | --- | --- | --- |
    //! | [`BAR`] — the path bar and the tab welded to it | `#ffffff` | 100.00 |
    //! | [`PANEL`] — a pane and its headers | `#fcfcfc` | 98.96 |
    //! | [`STATUS`] | `#f5f5f5` | 96.54 |
    //! | [`TITLEBAR`] — the title bar and the sidebar | `#ececec` | 93.40 |
    //! | [`SEPARATOR`] | `#dddddd` | 88.12 |
    //!
    //! The file manager's checkerboard wants 8 ΔL\* between the panel and
    //! `background-control-active`, and has 10.8.
    //!
    //! # Only the backgrounds
    //!
    //! Every `background-*` role is one of these five or a grey of its own, and nothing else
    //! moves: the ink, the strokes, the scrollbar, the accent — a selected row, its bar, the focus
    //! ring, a link — and the status hues are [`Theme::light`](super::Theme::light)'s, unchanged.
    //! The surfaces got lighter and the ink did not, so every contrast figure measured on the
    //! light palette holds here and rises.
    //!
    //! **One thing this cannot reach**, for the reason [`family`](super::family) gives:
    //! [`desktop::press_fill`](super::desktop::press_fill) is a function of `t.dark` rather than a
    //! token, so a button *while it is held* flashes `paper::SLATE_5`, a blue-grey at the right
    //! lightness.

    use egui::Color32;

    const fn grey(v: u8) -> Color32 {
        Color32::from_rgb(v, v, v)
    }

    pub const BAR: Color32 = grey(0xff);
    pub const PANEL: Color32 = grey(0xfc);
    pub const STATUS: Color32 = grey(0xf5);
    pub const TITLEBAR: Color32 = grey(0xec);
    pub const SEPARATOR: Color32 = grey(0xdd);

    /// The five surfaces carried into the roles [`celestine::apply`](super::celestine::apply)
    /// carries its own into, and answer with the regions.
    pub fn apply(az: &mut super::Theme) -> super::Surfaces {
        az.bg.layer = PANEL;
        az.bg.card = PANEL;
        az.bg.control = PANEL;

        az.bg.layer_alt = TITLEBAR;
        az.bg.control_disabled = TITLEBAR;
        az.row_alt = TITLEBAR;
        az.bg.canvas = TITLEBAR;

        az.stroke.subtle = SEPARATOR;
        az.bg.control_active = SEPARATOR;

        // The hover at `L*` 74.42 — the grey of `SLATE_6`, where `desktop::apply` put it, whose
        // 74.50 Celestine holds to the hundredth for the reason `celestine::apply` gives: the
        // ink does not change when the pointer arrives, so the band cannot move towards it.
        az.bg.control_hover = grey(0xb7);
        az.bg.card_hover = az.bg.control_hover;

        // The scrim: the light theme's `rgba(16,20,29,.4)` with its blue taken out.
        az.bg.overlay = Color32::from_rgba_unmultiplied(16, 16, 16, 102);

        super::Surfaces {
            titlebar: TITLEBAR,
            bar: BAR,
            panel: PANEL,
            header: PANEL,
            sidebar: TITLEBAR,
            status: STATUS,
            separator: SEPARATOR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contrast::{apart, lstar, ratio, SAME, SHAPE, TEXT};

    fn all() -> impl Iterator<Item = (Palette, Theme, Surfaces)> {
        Palette::ALL.into_iter().map(|p| {
            let (t, s) = p.build();
            (p, t, s)
        })
    }

    /// Every palette can be named, saved and read back — which is the whole of what
    /// [`Palette::ALL`] promises, since it is what a menu and a settings file both read.
    #[test]
    fn every_palette_is_reachable_and_round_trips() {
        for p in Palette::ALL {
            assert_eq!(Palette::parse(p.key()), Some(p), "{:?} does not parse back", p);
            // The two spellings a setting typed by hand can take, and the case it can arrive in.
            assert_eq!(Palette::parse(&p.key().to_uppercase()), Some(p), "{:?}", p);
            assert_eq!(Palette::parse(&p.key().replace('-', "_")), Some(p), "{:?}", p);
            assert_eq!(Palette::parse(&format!("  {}  ", p.key())), Some(p), "{:?}", p);
            // And it is built on the side it says it is.
            assert_eq!(p.build().0.dark, p.is_dark(), "{:?} builds on the other side", p);
        }
        // Distinct names, both kinds. Two palettes sharing a key would make a settings file
        // ambiguous, and two sharing a label would make the menu unreadable.
        for (i, a) in Palette::ALL.iter().enumerate() {
            for b in &Palette::ALL[i + 1..] {
                assert_ne!(a.key(), b.key(), "{a:?} and {b:?} share a key");
                assert_ne!(a.label(), b.label(), "{a:?} and {b:?} share a label");
            }
        }
        assert_eq!(Palette::parse("chartreuse"), None);
        assert_eq!(Palette::parse(""), None);
        // **Every retired name still answers**, which is the half of this that has already gone
        // wrong once: `light-blue` was written into real settings files, and a name that stops
        // parsing does not fail loudly — it falls back to the default and opens the window dark.
        for (retired, means) in Palette::ALIASES {
            assert_eq!(Palette::parse(retired), Some(means), "the alias {retired} stopped working");
            assert_eq!(Palette::parse(&retired.to_uppercase()), Some(means), "{retired}");
            assert_eq!(Palette::parse(&retired.replace('-', "_")), Some(means), "{retired}");
            // And an alias is a *retired* name: one that is still a palette's own key is a
            // duplicate entry rather than a compatibility record.
            assert!(
                !Palette::ALL.iter().any(|p| p.key() == retired),
                "{retired} is both a live key and an alias"
            );
        }
        assert_eq!(Palette::default(), Palette::Dark);
        // At least one palette per side, said as a sweep rather than a list, so a palette added
        // later is covered without this line being edited.
        assert!(Palette::ALL.iter().any(|p| p.is_dark()), "no palette is a dark one");
        assert!(Palette::ALL.iter().any(|p| !p.is_dark()), "no palette is a light one");
    }

    /// A palette that names no region of its own paints what a window always painted.
    ///
    /// It is also the map. Reading it against [`Surfaces::from_theme`] is how you find out what a
    /// region *was*, which is the first question to ask when a palette looks wrong.
    #[test]
    fn a_palette_with_no_surfaces_of_its_own_paints_what_it_always_did() {
        let (t, s) = Palette::Dark.build();
        assert_eq!(s.titlebar, t.bg.layer, "the title bar");
        assert_eq!(s.panel, t.bg.layer, "a pane");
        assert_eq!(s.header, t.bg.layer_alt, "the column headers");
        assert_eq!(s.sidebar, t.bg.layer_alt, "the sidebar");
        assert_eq!(s.status, t.bg.layer_alt, "the status bar");
        assert_eq!(s.bar, t.stroke.subtle, "the path bar");
        assert_eq!(s.separator, t.stroke.subtle, "the seam");
        // And the two that were one value: still one value here, which is the point — the split is
        // a split of the *decision*, not of this palette's colours.
        assert_eq!(s.bar, s.separator);

        // Whereas a palette with surfaces of its own is the case they had to come apart for.
        for p in [Palette::Celestine, Palette::Light] {
            let (t, s) = p.build();
            assert_ne!(s.bar, s.separator, "{p:?} wants a pale bar and a seam you can see");
            assert_ne!(s, Surfaces::from_theme(&t), "{p:?} is painting Azur's roles");
        }

        // And every palette in the family is back on the map, because each varies the roles
        // rather than the regions.
        for (p, t, s) in all() {
            if p.is_dark() {
                assert_eq!(s, Surfaces::from_theme(&t), "{p:?} has named a region of its own");
            }
        }
    }

    /// [`Palette::Light`] is what it says: a panel that is almost white, every background a grey,
    /// and every colour that is not a background [`Palette::Celestine`]'s.
    #[test]
    fn the_light_palette_is_celestine_on_grey() {
        let grey = |c: Color32| {
            let [r, g, b, _] = c.to_srgba_unmultiplied();
            r == g && g == b
        };
        let (t, s) = Palette::Light.build();
        let (l, _) = Palette::Celestine.build();
        assert_eq!(s.bar, Color32::WHITE, "its top rung is not white");
        let panel = lstar(s.panel);
        assert!(panel > 98.5, "the panel is L* {panel:.2}, which is not almost white");

        let b = t.bg;
        let mut backgrounds = s.each().to_vec();
        backgrounds.extend([
            ("canvas", b.canvas),
            ("layer", b.layer),
            ("layer-alt", b.layer_alt),
            ("card", b.card),
            ("card hover", b.card_hover),
            ("overlay", b.overlay),
            ("control", b.control),
            ("control hover", b.control_hover),
            ("control active", b.control_active),
            ("control disabled", b.control_disabled),
            ("row alt", t.row_alt),
        ]);
        for (what, c) in backgrounds {
            assert!(grey(c), "the {what} is {c:?}, which is not a grey");
        }
        // The hover kept the light palette's lightness, which is the one thing about it that is
        // measured rather than chosen.
        let moved = apart(b.control_hover, l.bg.control_hover);
        assert!(moved < 0.5, "the hover moved {moved:.2} ΔL* off Celestine's");

        // And nothing that is not a background moved at all.
        let (a, x) = (t.accent, l.accent);
        let (i, y) = (t.text, l.text);
        let (k, z) = (t.stroke, l.stroke);
        let (st, sl) = (t.status, l.status);
        for (what, ours, theirs) in [
            ("text.primary", i.primary, y.primary),
            ("text.secondary", i.secondary, y.secondary),
            ("text.tertiary", i.tertiary, y.tertiary),
            ("text.disabled", i.disabled, y.disabled),
            ("text.link", i.link, y.link),
            ("accent.subtle", a.subtle, x.subtle),
            ("accent.active", a.active, x.active),
            ("accent.mark", a.mark, x.mark),
            ("stroke.default", k.default, z.default),
            ("stroke.strong", k.strong, z.strong),
            ("stroke.focus", k.focus, z.focus),
            ("status.success", st.success, sl.success),
            ("status.warning", st.warning, sl.warning),
            ("status.danger", st.danger, sl.danger),
            ("scrollbar", t.scrollbar.thumb, l.scrollbar.thumb),
        ] {
            assert_eq!(ours, theirs, "the {what} is not Celestine's");
        }
    }

    /// Every palette in [`family`](mod@family) is [`Palette::Dark`] in another material, stated as
    /// a measurement.
    ///
    /// Every rung holds the *lightness* of the rung it stands in for, and
    /// [`ratio`](crate::contrast::ratio) and [`lstar`](crate::contrast::lstar) are both functions
    /// of nothing else — so every figure measured on the dark palette holds on these six.
    ///
    /// Two palettes declare a shift, and it is the shift that tells them apart: [`Palette::Onyx`]
    /// sinks 3.5 ΔL\* and [`Palette::Silver`] lifts 1.7. Their ladders keep their *shape* to the
    /// same tolerance. [`Palette::Silver`] also declares its pigment its own.
    ///
    /// It is asserted the other way as well — that every rung really did move — because a role
    /// forgotten in `Cut::apply` is a graphite rung in a coloured window, and that is a defect no
    /// ratio would see. And each rung is checked to be in the right corner of the colour wheel,
    /// which catches a table row pasted into the wrong palette.
    #[test]
    fn every_rung_is_the_dark_palettes_own_lightness() {
        /// **Half a byte step, down where this ladder lives.** A chromatic rung has three channels
        /// to spend and gets within 0.05; a *grey* has one, and near black one byte is a third of
        /// a ΔL\* wide — so Onyx and Silver drift up to 0.21.
        const ROUNDING: f32 = 0.25;

        /// One row per palette: how far its stone is shifted off the dark ladder, whether it
        /// names its own hover, whether its pigment is its own, and which channel is the largest
        /// in every rung it has (`=` for the two that have no hue).
        const CUTS: [(Palette, f32, bool, bool, char); 6] = [
            (Palette::Onyx, -3.5, true, false, '='),
            (Palette::Cobalt, 0.0, false, false, 'b'),
            (Palette::Malachite, 0.0, false, false, 'g'),
            (Palette::Amethyst, 0.0, false, false, 'b'),
            (Palette::Citrine, 0.0, false, false, 'r'),
            (Palette::Silver, 1.7, true, true, '='),
        ];

        /// The stone, in a fixed order so two palettes zip. `Cut::apply` is the other half of
        /// this list and the two are each other's proof.
        fn stone(t: &Theme, s: &Surfaces) -> Vec<(&'static str, Color32)> {
            let mut out = s.each().to_vec();
            out.extend([
                ("bg.canvas", t.bg.canvas),
                ("bg.layer", t.bg.layer),
                ("bg.layer_alt", t.bg.layer_alt),
                ("bg.card", t.bg.card),
                ("bg.control", t.bg.control),
                ("bg.control_active", t.bg.control_active),
                ("bg.control_disabled", t.bg.control_disabled),
                ("row_alt", t.row_alt),
                ("stroke.default", t.stroke.default),
                ("stroke.subtle", t.stroke.subtle),
                ("stroke.strong", t.stroke.strong),
                ("stroke.control", t.stroke.control),
                ("scrollbar.thumb", t.scrollbar.thumb),
                ("scrollbar.thumb_hover", t.scrollbar.thumb_hover),
                ("accent.disabled", t.accent.disabled),
            ]);
            out
        }

        /// The pigment, the same way.
        fn pigment(t: &Theme) -> Vec<(&'static str, Color32)> {
            vec![
                ("accent.subtle", t.accent.subtle),
                ("accent.subtle_hover", t.accent.subtle_hover),
                ("accent.subtle_active", t.accent.subtle_active),
                ("accent.active", t.accent.active),
                ("accent.default", t.accent.default),
                ("accent.hover", t.accent.hover),
                ("accent.mark", t.accent.mark),
                ("stroke.focus", t.stroke.focus),
                ("status.info", t.status.info),
                ("status.info_subtle", t.status.info_subtle),
                ("text.link", t.text.link),
            ]
        }

        /// The band under the pointer, which the two hueless palettes move and the other four do
        /// not. Both tokens, because a hovered row and a hovered button are one colour.
        fn hover(t: &Theme) -> Vec<(&'static str, Color32)> {
            vec![
                ("bg.control_hover", t.bg.control_hover),
                ("bg.card_hover", t.bg.card_hover),
            ]
        }

        let (dark, dark_s) = Palette::Dark.build();
        for (palette, shift, own_hover, own_pigment, biggest) in CUTS {
            let (t, s) = palette.build();
            let name = palette.key();

            // The stone, shifted by the constant this palette declares. `max(0.0)` is the one
            // clamp in the family: Onyx's canvas would be below black, so it is black.
            let theirs = stone(&dark, &dark_s);
            for ((what, mine), (_, theirs)) in stone(&t, &s).into_iter().zip(theirs) {
                let want = (lstar(theirs) + shift).max(0.0);
                let got = lstar(mine);
                assert!(
                    (got - want).abs() <= ROUNDING,
                    "{name}: {what} is L* {got:.2} against the L* {want:.2} its ladder position \
                     asks for"
                );
            }
            let same_lightness = |mine: Vec<(&str, Color32)>, theirs: Vec<(&str, Color32)>| {
                for ((what, mine), (_, theirs)) in mine.into_iter().zip(theirs) {
                    let (got, want) = (lstar(mine), lstar(theirs));
                    assert!(
                        (got - want).abs() <= ROUNDING,
                        "{name}: {what} is L* {got:.2} against the dark palette's L* {want:.2}"
                    );
                }
            };
            if !own_pigment {
                same_lightness(pigment(&t), pigment(&dark));
            }
            if !own_hover {
                same_lightness(hover(&t), hover(&dark));
            }

            // Every rung moved, and moved into the right corner of the wheel.
            let mine_all = stone(&t, &s).into_iter().chain(pigment(&t)).chain(hover(&t));
            let dark_all = stone(&dark, &dark_s)
                .into_iter()
                .chain(pigment(&dark))
                .chain(hover(&dark));
            for ((what, mine), (_, theirs)) in mine_all.zip(dark_all) {
                assert_ne!(
                    mine, theirs,
                    "{name}: {what} is still the dark palette's own colour, so `Cut::apply` does \
                     not set it — a graphite rung in a coloured window"
                );
                let (r, g, b) = (mine.r(), mine.g(), mine.b());
                let ok = match biggest {
                    '=' => r == g && g == b,
                    'r' => r >= g && r > b,
                    'g' => g > r && g > b,
                    _ => b > r && b > g,
                };
                assert!(
                    ok,
                    "{name}: {what} is {mine:?}, whose largest channel is not {biggest:?} — a row \
                     of the table has landed in the wrong palette"
                );
            }
        }

        // No two cuts are the same cut, said on the three rungs a reader would recognise: the
        // panel, the selected row and the mark.
        for (i, (a, ..)) in CUTS.iter().enumerate() {
            for (b, ..) in &CUTS[i + 1..] {
                let ((x, xs), (y, ys)) = (a.build(), b.build());
                for (what, p, q) in [
                    ("panel", xs.panel, ys.panel),
                    ("selected row", x.accent.active, y.accent.active),
                    ("mark", x.accent.mark, y.accent.mark),
                ] {
                    assert_ne!(p, q, "{:?} and {:?} share a {what}", a, b);
                }
            }
        }

        // **The claim [`Palette::Cobalt`] is named for**: at the rung a selected row is filled
        // with, the gamut boundary at hue 291 *is* the pigment — cobalt blue, `#0047AB`, to
        // within three bytes on one channel.
        let pigment = Color32::from_rgb(0x00, 0x47, 0xab);
        let got = Palette::Cobalt.build().0.accent.active;
        for (channel, mine, theirs) in [
            ("red", got.r(), pigment.r()),
            ("green", got.g(), pigment.g()),
            ("blue", got.b(), pigment.b()),
        ] {
            assert!(
                mine.abs_diff(theirs) <= 3,
                "a selected row is {got:?}, whose {channel} is {mine} against cobalt blue's \
                 {theirs} — this is not cobalt"
            );
        }
    }

    /// Every region carries the side's text, in every palette.
    ///
    /// The floor a new palette is actually at risk of missing: a palette arrives as a list of
    /// surfaces somebody picked by eye, and the thing nobody checks by eye is whether the ink —
    /// which the palette does *not* restate — still reads on all of them. `text.secondary` is
    /// held to [`SHAPE`] rather than [`TEXT`]: it is metadata, read at a glance beside a name.
    #[test]
    fn every_region_carries_the_text() {
        for (p, t, s) in all() {
            for (what, fill) in s.each() {
                let got = ratio(t.text.primary, fill);
                assert!(got >= TEXT, "{p:?}: a name on the {what} is {got:.2}:1");
                let got = ratio(t.text.secondary, fill);
                assert!(got >= SHAPE, "{p:?}: metadata on the {what} is {got:.2}:1");
            }
        }
    }

    /// Celestine's file list, its headers and its status bar are the light palette's, so a
    /// window switched between the two keeps its listing and changes only its chrome.
    #[test]
    fn celestine_lists_on_the_light_palettes_surfaces() {
        let ((_, l), (_, w)) = (Palette::Celestine.build(), Palette::Light.build());
        for (what, ours, theirs) in [
            ("file list", l.panel, w.panel),
            ("column headers", l.header, w.header),
            ("status bar", l.status, w.status),
        ] {
            assert_eq!(ours, theirs, "celestine's {what} is not the light palette's");
        }
        // And only those: the chrome is still Celestine's own blue.
        assert_ne!(l.titlebar, w.titlebar);
        assert_ne!(l.separator, w.separator);
    }

    /// The close button is red in every palette, and its glyph reads on it — the one fill here
    /// that is deliberately not a role. See [`desktop::CLOSE_FILL`].
    #[test]
    fn the_close_button_is_red_whatever_the_palette() {
        let (fill, ink) = (desktop::CLOSE_FILL, desktop::CLOSE_INK);
        assert!(fill.r() > 2 * fill.g() && fill.r() > 2 * fill.b(), "{fill:?} is not a red");
        let got = ratio(ink, fill);
        assert!(got >= TEXT, "the close glyph is {got:.2}:1 on its red");
    }

    /// The seam reads against every region it divides, in ΔL\* — the ruler for two surfaces
    /// meeting. A seam has no job except to be a boundary, so one that has drifted onto a
    /// neighbour has lost the window's structure and nothing else would notice.
    #[test]
    fn the_separator_reads_against_everything_it_divides() {
        for (p, _, s) in all() {
            for (what, fill) in s.each() {
                if fill != s.separator {
                    let got = apart(s.separator, fill);
                    assert!(got >= SAME, "{p:?}: the seam is only {got:.1} ΔL* from the {what}");
                }
            }
        }
    }
}
