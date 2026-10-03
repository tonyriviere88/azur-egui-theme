# Application icons

Two unrelated things go by that name, so this is in two parts:

- **[Part A](#part-a--glyphs-inside-the-window) — glyphs inside the window**
  (§1–10). Vector art, painted at 12–20px, colored by whichever component hosts
  it. A `Print` icon in a `CommandBar`.
- **[Part B](#part-b--the-executables-icon) — the icon on the executable**
  (§12–18). A raster bitmap at fixed pixel sizes with fixed colors, shown by
  Explorer, the taskbar and Alt-Tab. The mark on the `.exe`.

They share a design language and nothing else: no rule from one transfers to the
other. Part A is enforced by the API — `icons::Pen` and `icons::stroke_width` are
the rules as code, and the examples here are compiled by `cargo test`. Part B is
a checklist, because Rust never sees those pixels.

---

# Part A — glyphs inside the window

`azur_egui_theme::icons` carries only the glyphs the *system* needs — chevrons,
checks, the ✕ on a dialog, window buttons, status marks. Everything that belongs
to a particular application — Print, Export, Measure, Layer, Rebuild — stays in
that application.

This is how to draw those so they look like they came from the same hand. An
app's Print icon is not shown in isolation: it sits in a `CommandBar` between
two built-in ones, or in a `MenuItem` above a submenu chevron. If it is a
different weight, or a different size on the same grid, or carries its own
color, that row stops looking designed.

The rules below are the ones the built-in glyphs follow, and Azur's own SVGs
before them (`azur-design-system/src/components/**/*.stories.tsx`, which is where
the design source keeps its icon art). `icons::Pen` is the geometry as code, so
following them is mostly a matter of using it.

---

## 1. The contract

```rust
pub type Icon<'a> = &'a dyn Fn(&egui::Painter, egui::Rect, egui::Color32);
```

A plain function of that shape, so `&my_icons::print` is already an `Icon`.

- **Paint into the rect you are handed.** Do not allocate space, read input,
  create `Id`s, or return anything. The component owns layout and interaction;
  the icon owns pixels.
- **Use the color you are handed** — every stroke, every fill. This one rule is
  what makes an application icon follow variant, hover, pressed, disabled,
  selected, and the light/dark switch without knowing any of them exist.
- **Fill the rect you are handed.** Never a fixed size. The caller decides
  whether this is a 14px menu tick or a 20px rail glyph.
- **Be cheap and pure.** A table calls this once per visible row per frame. No
  allocation you can avoid, no `Context` lookups, no `request_repaint`.

```rust
use azur_egui_theme::icons::{Icon, Pen};
use egui::{Color32, Painter, Rect};

/// A sheet of paper with a folded corner.
pub fn document(painter: &Painter, rect: Rect, color: Color32) {
    let pen = Pen::new(painter, rect, color);
    // The sheet: one outline, its top-right corner cut off.
    pen.closed_path(&[(4.0, 2.0), (9.5, 2.0), (12.5, 5.0), (12.5, 14.0), (4.0, 14.0)]);
    // The fold, back across the cut.
    pen.path(&[(9.5, 2.0), (9.5, 5.0), (12.5, 5.0)]);
}

let icon: Icon = &document;
```

## 2. Geometry: a 16-unit grid

Azur's art is authored in a `viewBox="0 0 16 16"`, with some 20 and a couple of
12. `Pen` is that grid: coordinates go in as the numbers written in the SVG, and
come out mapped onto whatever box the component handed you.

```rust
use azur_egui_theme::icons::Pen;
use egui::{Color32, Painter, Rect};

/// <svg viewBox="0 0 20 20">
///   <circle cx="10" cy="10" r="2.6"/>
///   <path d="M10 3v2M10 15v2M3 10h2M15 10h2M5 5l1.4 1.4M13.6 13.6L15 15
///            M15 5l-1.4 1.4M6.4 13.6L5 15"/>
/// </svg>
fn settings(painter: &Painter, rect: Rect, color: Color32) {
    let pen = Pen::new(painter, rect, color).units(20.0);
    pen.circle(10.0, 10.0, 2.6);
    for (x1, y1, x2, y2) in [
        (10.0, 3.0, 10.0, 5.0),
        (10.0, 15.0, 10.0, 17.0),
        (3.0, 10.0, 5.0, 10.0),
        (15.0, 10.0, 17.0, 10.0),
        (5.0, 5.0, 6.4, 6.4),
        (13.6, 13.6, 15.0, 15.0),
        (15.0, 5.0, 13.6, 6.4),
        (6.4, 13.6, 5.0, 15.0),
    ] {
        pen.line(x1, y1, x2, y2);
    }
}
```

Rules on that grid:

- **Leave a margin.** Flat-sided art stops 2 units short of the edge on a
  16 grid (3 on a 20) — Azur's plus is `M8 2v12M2 8h12`, not `M8 0v16`. The box a
  component hands you is flush with the text line beside it, and art that touches
  its edges reads a size too big.
- **Round shapes may overshoot to 1 unit**, which is what Azur's status circles
  do (`<circle cx="8" cy="8" r="7">`). A circle drawn to the same extent as a
  square looks smaller than it; the extra unit cancels that out.
- **Centre optically, not arithmetically.** A magnifier is a circle up and left
  of centre with the handle filling the corner — its bounding box is centred, its
  circle is not. Where you do this, say so in a comment.
- **Do not pixel-snap.** The one group that does is the window buttons, because
  the platform draws its caption glyphs as 1px hairlines at any title-bar height;
  `icons::hairline` is private on purpose. Everything else lives on a logical
  grid and lets egui's antialiasing place it.

## 3. Weight and terminals

One weight per glyph, from `icons::stroke_width(rect)`: an eighth of the box,
clamped to 1–2px.

| box | stroke |
| --- | --- |
| 10px (window buttons) | 1.0 — fixed, not from this rule |
| 12px | 1.5 |
| 14px | 1.75 |
| 16px and up | 2.0 |

Use it rather than a number of your own, and use it for *every* line in the
glyph. A 1px detail line next to a 2px body reads as a rendering bug at these
sizes, not as hierarchy.

`Pen` starts there. `Pen::stroke_units(1.4)` instead takes the weight from the
artwork's own `stroke-width` — faithful to the SVG, and about half a pixel
lighter than the built-ins at 16px and up. That is a decision for a whole icon
set, never for one glyph in a row of others.

**Terminals** are the one place the port cannot be literal. Azur strokes with
`stroke-linecap="round"` and `stroke-linejoin="round"`; `egui::Stroke` is
`{ width, color }` and has no cap or join setting. At 1–2px the difference is
sub-pixel, so accept it — do not fake caps with circles at the endpoints, which
costs shapes and reads lumpy.

## 4. Stroke, not fill

`fill="none" stroke="currentColor"` is the house style, and the design system is
emphatic about it: across its 49 pieces of icon art there are 72 stroked elements
and 4 filled ones — all four of them dots of radius 0.9. Draw outlines.

Fill is for the marks too small to outline:

- dots — a radio's centre, an ellipsis, the tittle on an `i` or `!` (radius ≤ 1
  unit)
- solid triangles — a sort arrow, a caret
- a counter badge's disc

Never inside one glyph: two tones, a gradient, a shadow, a second color, or any
color from `Theme`. An icon is one silhouette in one color.

## 5. Color

You do not choose it. The component computes a foreground for the state it is in
and hands it over:

| component | what it passes |
| --- | --- |
| `Button`, `IconButton` | the variant's foreground — `text.on_accent` on primary, `text.primary` on secondary/subtle/outline, `accent.default` when `pressed`, `text.disabled` when disabled |
| `MenuItem`, `ListRow`, `TreeNode` | the row foreground; `accent.default` when selected, `status.danger` on a `danger` item |
| `NavigationRail`, `CommandBar` | `text.secondary`, `accent.default` when selected |
| `Tabs` | `text.secondary`, `text.primary` when selected or hovered |
| `TitleBar` | `text.secondary` |
| `MessageBar`, `Toast` | `Intent::color` — the status hue |
| `TextField`, `SearchBox`, `NumberField` | `text.secondary` for an affix, `text.tertiary` when empty |

So: **paint with the `Color32` argument and nothing else.**

- No `Color32::WHITE`, no `BLACK`, no hex literal. Any of them is wrong in one of
  the two palettes.
- No `Theme` lookup inside an icon body. The color already accounts for state;
  a token cannot.
- Do not encode meaning in an icon's color. A delete icon is not red — the
  `danger` variant that hosts it is.
- If a glyph genuinely needs a second tier, derive it:
  `color.gamma_multiply(0.55)`. That fades toward whatever the icon is sitting
  on, in either palette.

When **you** are the one painting — an icon in a panel you laid out yourself,
with no component in between — pick the role the way a component would:

```rust
use azur_egui_theme::{components::theme, icons};

fn paint_status(ui: &egui::Ui, rect: egui::Rect, ok: bool) {
    let t = theme(ui.ctx());
    let color = if ok { t.status.success } else { t.status.danger };
    icons::success(ui.painter(), rect, color);
}
```

| context | role |
| --- | --- |
| decoration, an affix, a metadata glyph | `text.secondary` |
| something you can click, at rest | `text.primary` |
| selected, active, current | `accent.default` |
| on an accent or danger fill | `text.on_accent` |
| disabled | `text.disabled` |
| meaning: ok / attention / problem / note | `status.success` / `warning` / `danger` / `info` |

## 6. Size

The component picks the box. These are the ones in use:

| where | box |
| --- | --- |
| `Button` leading/trailing icon | 12 / 14 / 16 — the label's font size, per Azur's `.icon svg { width: 1em }` |
| `IconButton` glyph | 14 / 16 / 18 |
| `TextField` / `SearchBox` / `NumberField` affix, `Select` trigger | 14 / 16 / 20 (`tokens::control::ICON_*`) |
| `MenuItem` icon | 16 (tick 14, submenu chevron 12) |
| `ListRow`, `TreeNode`, `Tabs` | 16 (tree chevron 18) |
| `Table` sort mark | 12 |
| `MessageBar`, `Toast` | 16 |
| `CommandBar`, `NavigationRail` | 20 |
| `TitleBar` app icon | 16 |
| window buttons | 10 |

Which means: **design the glyph once, at 16, and check it at 12 and 20.** If it
needs a different body to survive 12px, it has too much detail — simplify the
one body rather than branching on `rect.width()`. Detail that dies at 12px was
never doing anything at 16.

For a standalone icon of your own, take the box from `tokens::control::ICON_SMALL`
/ `ICON_MEDIUM` / `ICON_LARGE` (14/16/20) rather than a literal, so it tracks the
control scale around it.

## 7. Transcribing an SVG

If you have the artwork, this is mechanical:

1. `viewBox="0 0 N N"` → `Pen::new(..).units(N)`. Non-square viewBoxes do not
   belong in an icon; re-author it square.
2. Delete `fill`, `stroke`, `stroke-width`, `stroke-linecap`, `stroke-linejoin`,
   `class`, `aria-*`. Color and weight come from the caller.
3. `M x y L x y …` → `pen.path(&[…])`. Trailing `Z` → `pen.closed_path`.
   `H`/`V` are the same point with one coordinate carried over.
4. `<line>` → `pen.line`, `<circle>` → `pen.circle` / `circle_filled`,
   `<rect>` → `pen.rect` / `rect_filled` (`rx` in grid units), `<polygon>` →
   `closed_path` or `polygon`.
5. Curves: `Pen` is polylines and primitives. A 1-unit corner radius — Azur's
   `a1 1 0 01-1 1` — is 1px at a 16px box; drop it. A curve that actually carries
   the shape wants a handful of sampled points, or
   `painter.add(egui::Shape::CubicBezier(..))` with `pen.at(..)` for the control
   points and `pen.stroke()` for the line.
6. Keep the original `d` attribute in a comment above the function. That comment
   is how the next person diffs your icon against the design source.

```rust
use azur_egui_theme::icons::Pen;
use egui::{Color32, Painter, Rect};

/// <svg viewBox="0 0 20 20">
///   <rect x="3" y="5" width="14" height="10" rx="1.5"/>
///   <path d="M4 6l6 5 6-5"/>
/// </svg>
pub fn mail(painter: &Painter, rect: Rect, color: Color32) {
    let pen = Pen::new(painter, rect, color).units(20.0);
    pen.rect(3.0, 5.0, 14.0, 10.0, 1.5);
    pen.path(&[(4.0, 6.0), (10.0, 11.0), (16.0, 6.0)]);
}
```

## 8. Where they live

One module, one function per glyph, no types:

```rust
// app/src/icons.rs
use azur_egui_theme::icons::Pen;
use egui::{Color32, Painter, Rect};

/// <svg viewBox="0 0 16 16"><path d="M4 8h8M8 4v8"/></svg>
pub fn add(painter: &Painter, rect: Rect, color: Color32) {
    let pen = Pen::new(painter, rect, color);
    pen.line(4.0, 8.0, 12.0, 8.0);
    pen.line(8.0, 4.0, 8.0, 12.0);
}
```

- Name after the thing, not the action, when both read the same: `printer`,
  `layers`, `ruler`. Match the design system's name where it has one.
- Pass as `&icons::add`. Do not wrap glyphs in a struct or an enum — the function
  *is* the icon. If you need to look one up, a `&[(&str, Icon<'static>)]` table
  is enough.
- Do not re-export or shadow the built-ins. An app's `icons::check` that differs
  from `azur_egui_theme::icons::check` will end up next to the real one inside a
  `Select`.

## 9. In-app icon checklist

Before an icon set ships, for every glyph:

- [ ] signature is exactly `(&Painter, Rect, Color32)`; nothing allocated, nothing
      sensed, nothing returned
- [ ] every stroke and fill uses the `Color32` argument, or
      `argument.gamma_multiply(..)`
- [ ] no literal color, no `Theme` lookup, no gradient, no second tone
- [ ] geometry is relative to the rect — no size, position or DPI constant
- [ ] one weight throughout, from `stroke_width` (or one `stroke_units` for the
      whole set)
- [ ] outline, not fill, unless the mark is a dot, a triangle or a disc
- [ ] art clears the edge by ~2 units of 16, round shapes by 1
- [ ] legible at 12px; still balanced at 20px
- [ ] checked in **both** palettes — `cargo run --example gallery -- light`
- [ ] the source `d` attribute, or a sentence on the intent, sits above the
      function

## 10. Where this deviates from the CSS

Stated rather than hidden, so a later diff against the design source does not
read these as bugs:

- **Weight.** `stroke_width` reaches 2px at a 16px box where the SVGs stroke
  1.4–1.6. The built-in glyphs are consequently a touch heavier than the web
  ones. `Pen::stroke_units` is the literal alternative.
- **Terminals.** No round caps or joins — `egui::Stroke` has no such field.
- **Curves.** Approximated by polylines, or dropped where they are a 1px corner
  radius.
- **`currentColor`.** CSS inherits it down the tree; here the component passes it
  explicitly. Same result, and the reason an icon must not look it up itself.

# Part B — the executable's icon

Everything above is about glyphs *inside* the window. The icon on the `.exe`
itself is a different medium with different rules: a raster bitmap, at fixed
pixel sizes, in colors that cannot follow the theme because it is one image for
both. None of Part A applies to it — not the grid, not the stroke weight, not
`currentColor`.

What it still has to do is look like the same product. The desktop is where a
user meets the application first, and an off-palette blue or a shadowed bevel
there makes the window that opens afterwards look like a different program.

## 12. Three surfaces, one mark

| surface | what it actually is | who supplies it |
| --- | --- | --- |
| Explorer, a pinned taskbar button, a shortcut, Alt-Tab before launch | a multi-size `.ico` in the executable's PE resources | the linker, via `build.rs` |
| the running window: its taskbar button and Alt-Tab entry | one RGBA bitmap handed over at startup | `ViewportBuilder::with_icon` |
| inside the window's own title bar, 16px | a painted `Icon`, one theme color | `TitleBar::icon` |

All three have to be the same mark. Two failure modes come from forgetting that:

- **Only the PE resource.** egui's default window icon is a white `e` on black
  ("for egui or eframe", says its own doc). Ship the `.ico` and forget
  `with_icon`, and the taskbar button switches from your mark to that `e` the
  moment the window opens.
- **Only `with_icon`.** Explorer and any pinned shortcut show the generic
  application icon, because they never run the process.

## 13. Sizes

Ship, and look at, all of these:

| px | where Windows uses it |
| --- | --- |
| 16 | Explorer details/list view, small taskbar buttons, the window's own corner |
| 20, 24 | the same, at 125% and 150% display scaling |
| 32 | Alt-Tab, taskbar at 100%, Explorer medium icons |
| 40, 48 | 125%/150% of 32, and Explorer large icons |
| 64, 128 | Explorer extra-large |
| 256 | jumbo view, thumbnails, the installer, store listings |

Two rules:

- **Do not ship 256 alone and let Windows downscale.** A 2px detail in a 256px
  icon is an eighth of a pixel at 16 — the mark arrives as gray mush. This is the
  single most common way a good icon looks broken on a desktop.
- **Redraw for the small sizes.** 16 and 32 are pixel work, not scaling: drop
  interior detail, snap edges to whole pixels, keep the silhouette. The 256 can
  carry detail the 16 must not attempt.

If the mark cannot be recognised at 16px in grayscale, it is too complex —
simplify the mark, not the export.

## 14. Shape

- **Fill the square.** ~6% margin at 256 (about 16px of it), scaling down; at 16
  leave a single pixel. An icon with a fat margin looks smaller than its
  neighbours in the taskbar.
- **Put the mark on a plate**, and round it at **25% of the side**. That is
  `radius::LARGE` (8px) on a 32px control — the same corner family as the app's
  own cards and buttons: 256→64, 128→32, 48→12, 32→8, 16→4.
- **No drop shadow, no bevel, no outer glow, no gradient steeper than a flat
  fill.** Azur's `shadow` tokens are for surfaces that float above other
  surfaces; an icon floats above a wallpaper nobody chose.
- **No wordmark.** A single initial is the most lettering that survives 16px.
- **One silhouette, centred optically.** Two objects at 16px are noise.

## 15. Color

The taskbar can be light, dark, or transparent over a photograph, and the icon
cannot switch. So this is the one place an app's art carries fixed colors — and
they still come from `tokens`.

The accent moves between palettes (`#3AA0FF` dark, `#2678C2` light), so neither
end of the ramp is safe on its own. Pick the rung that holds both:

| | plate | mark |
| --- | --- | --- |
| **Default** | `AZURE_60` `#2678C2` | `WHITE` |
| Dark chrome | `GRAY_1` `#0E0F10` — the dark `bg.layer` | `AZURE_70` `#3AA0FF` |
| Light chrome | `paper::SHEET` `#F7F8FA` — the light `bg.card` | `AZURE_60` |

The default pair is exactly a primary `Button` in the light palette —
`text.on_accent` on `accent.default` — and dark enough to hold white against a
white taskbar.

- **Only palette colors.** An icon blue that is nearly the app's blue is worse
  than an obviously different one.
- **Full opacity inside the plate.** Alpha belongs only in the antialiasing of
  its rounded corners — a semi-transparent icon body picks up the wallpaper.
- **Test three backgrounds**: a light taskbar, a dark taskbar, and a busy
  photograph. The plate is what makes the third one survivable.

## 16. Building it in

Neither the `.ico` nor the build script belongs to this crate — it is the
application's build. On Windows the icon lives in the executable's resources:

```toml
[build-dependencies]
winresource = "0.1"
```

```rust,ignore
// build.rs
fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        winresource::WindowsResource::new()
            .set_icon("assets/app.ico")
            .compile()
            .unwrap();
    }
}
```

Keep the PNGs as the source of truth in `assets/`, and assemble the `.ico` from
them — every size, in one file:

```sh
magick assets/app-16.png assets/app-20.png assets/app-24.png assets/app-32.png \
       assets/app-48.png assets/app-64.png assets/app-128.png assets/app-256.png \
       assets/app.ico
```

Then hand the same art to the window, or the taskbar button changes identity at
launch:

```rust,ignore
let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_icon(
            eframe::icon_data::from_png_bytes(include_bytes!("../assets/app-256.png")).unwrap(),
        )
        .with_decorations(false), // Azur draws its own `TitleBar`
    ..Default::default()
};
```

`eframe::icon_data` is native-only. `IconData::default()` asks for the OS default
instead, which is the right choice only if you have no mark at all.

## 17. And the title bar

A borderless window draws Azur's `TitleBar`, and `TitleBar::icon` takes a painted
`Icon` at 16px in `text.secondary` — so the mark needs a **third** form, under
Part A's rules: one color, outline, 16-unit grid.

```rust,ignore
TitleBar::new("Surveyor").icon(&app_icons::mark).show(ui)
```

A multicolor logo cannot survive that, and a plate makes no sense there — the
title bar is already a surface. Draw the mark's silhouette; if it has none that
reads at 16px, use the initial, and take that as a signal about the 16px `.ico`
too.

## 18. Executable-icon checklist

- [ ] one mark, three forms: `.ico` in the exe, `with_icon` at startup, painted
      `Icon` in the title bar
- [ ] 16, 20, 24, 32, 40, 48, 64, 128, 256 all present in the `.ico`
- [ ] 16 and 32 redrawn by hand, not downscaled
- [ ] recognisable at 16px, in grayscale
- [ ] plate rounded at 25% of the side; ~6% margin at 256, 1px at 16
- [ ] no shadow, bevel, glow, or wordmark
- [ ] colors from `tokens::palette` only; opaque inside the plate
- [ ] checked on a light taskbar, a dark taskbar, and a photograph
- [ ] `with_icon` set, so the running window is not egui's `e`
- [ ] the PNG sources are in the repo, and the `.ico` is reproducible from them
