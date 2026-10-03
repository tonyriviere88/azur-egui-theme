//! A one-window tour of the theme: every component, every state, both sides of
//! the palette.
//!
//!     cargo run --example gallery
//!     cargo run --example gallery -- light
//!     cargo run --example gallery -- --section=6
//!
//! It doubles as the visual test. If a change to the roles makes a control
//! disappear into its background, it shows up here first — and as the compile
//! test, since between them these sections touch every public item.
//!
//! It also takes the README's screenshots, so they can be remade rather than
//! recaptured by hand:
//!
//!     cargo run --example gallery -- --section=1 --screenshot=docs/gallery-dark.png
//!     cargo run --example gallery -- light --section=1 --screenshot=docs/gallery-light.png

use azur::components::*;
use azur::icons;
use azur::tokens::{chart, radius, space};
use azur_egui_theme as azur;
use egui::{vec2, Color32, CornerRadius, Pos2, Sense, Stroke, StrokeKind, Ui};

fn main() -> eframe::Result {
    // `cargo run --example gallery -- light` opens on the other side of the
    // palette, which is how the screenshots in the README are taken.
    let light = std::env::args().any(|a| a == "light");
    let section = std::env::args()
        .find_map(|a| a.strip_prefix("--section=").and_then(|n| n.parse().ok()))
        .unwrap_or(0);
    // Write the window to a PNG and exit. Asking egui for the frame rather than
    // asking the platform for the screen keeps the image exactly the size of what
    // was drawn -- no window frame, no DPI arithmetic, nothing behind bleeding in
    // around a rounded corner.
    let screenshot = std::env::args().find_map(|a| {
        a.strip_prefix("--screenshot=")
            .map(std::path::PathBuf::from)
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 820.0])
            .with_min_inner_size([720.0, 520.0])
            // The gallery draws Azur's own `TitleBar`, so it asks the platform not
            // to draw a second one.
            .with_decorations(false),
        ..Default::default()
    };
    eframe::run_native(
        "Azur egui theme",
        options,
        Box::new(move |cc| {
            azur::fonts::install(&cc.egui_ctx);
            let mut gallery = Gallery::new();
            if light {
                gallery.theme = azur::Theme::light();
            }
            gallery.section = section;
            gallery.screenshot = screenshot;
            Ok(Box::new(gallery))
        }),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fit {
    Width,
    Page,
    Actual,
}

struct Gallery {
    theme: azur::Theme,
    /// Set once per theme switch: installing a style every frame would throw away
    /// egui's own caches.
    installed: Option<bool>,

    section: usize,
    rail_collapsed: bool,
    tab: usize,

    // inputs
    name: String,
    notes: String,
    search: String,
    tolerance: f64,
    bad_email: String,

    // selection
    snap: bool,
    hidden: bool,
    mixed: bool,
    fit: Fit,
    opacity: f32,
    density: f32,

    // menus
    units: Option<usize>,
    filter_text: String,
    filter_pick: Option<usize>,

    // overlays
    confirm_open: bool,
    drawer_open: bool,
    popover_open: bool,

    // collections
    list_selection: Option<usize>,
    tree: TreeNodeState,
    table_sort: Sort,
    table_selection: Option<usize>,

    // feedback
    progress: f32,
    toaster: Toaster,
    show_banner: bool,

    /// `--screenshot=<path>`: where to write the window, and how many frames have
    /// gone by. Fonts, layout and the scroll area all settle over the first few.
    screenshot: Option<std::path::PathBuf>,
    frame: u32,
}

impl Gallery {
    fn new() -> Self {
        Self {
            theme: azur::Theme::dark(),
            installed: None,
            section: 0,
            rail_collapsed: false,
            tab: 0,
            name: "Site survey".to_owned(),
            notes: String::new(),
            search: String::new(),
            tolerance: 2.5,
            bad_email: "not-an-address".to_owned(),
            snap: true,
            hidden: false,
            mixed: true,
            fit: Fit::Width,
            opacity: 0.62,
            density: 40.0,
            units: Some(0),
            filter_text: String::new(),
            filter_pick: None,
            confirm_open: false,
            drawer_open: false,
            popover_open: false,
            list_selection: Some(1),
            tree: TreeNodeState::with_expanded([1, 2]),
            table_sort: Some((0, true)),
            table_selection: Some(2),
            progress: 0.42,
            toaster: Toaster::new(),
            show_banner: true,
            screenshot: None,
            frame: 0,
        }
    }

    /// Ask egui for the frame once things have settled, then write it out and quit.
    fn take_screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = self.screenshot.clone() else {
            return;
        };
        self.frame += 1;
        ctx.request_repaint();

        if self.frame == 8 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            return;
        }
        if self.frame < 8 {
            return;
        }

        let shot = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = shot {
            let [w, h] = image.size;
            let pixels: Vec<u8> = image
                .pixels
                .iter()
                .flat_map(|p| p.to_srgba_unmultiplied())
                .collect();
            match image::RgbaImage::from_raw(w as u32, h as u32, pixels) {
                Some(buffer) => match buffer.save(&path) {
                    Ok(()) => println!("wrote {} ({w}x{h})", path.display()),
                    Err(e) => eprintln!("could not write {}: {e}", path.display()),
                },
                None => eprintln!("screenshot was not the size it claimed"),
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

const SECTIONS: [&str; 8] = [
    "Foundations",
    "Actions",
    "Inputs",
    "Selection",
    "Menus",
    "Overlays",
    "Collections",
    "Feedback",
];

impl eframe::App for Gallery {
    fn clear_color(&self, _v: &egui::Visuals) -> [f32; 4] {
        self.theme.bg.canvas.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = &ui.ctx().clone();
        if self.installed != Some(self.theme.dark) {
            azur::install(ctx, &self.theme, &azur::StyleOptions::default());
            self.installed = Some(self.theme.dark);
        }
        let t = self.theme.clone();

        // ---- Window chrome ------------------------------------------------
        // The window manager owns this, not the app: it also changes under Win+Up,
        // a snap, or a double-click on the caption, and a local flag would drift.
        let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));

        egui::Panel::top("azur-title")
            .exact_size(azur::tokens::row::TITLE_BAR)
            .show_separator_line(false)
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                match TitleBar::new("Azur — egui theme gallery")
                    .icon(&icons::dot)
                    .maximized(maximized)
                    .show(ui)
                {
                    Some(TitleBarAction::Close) => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    Some(TitleBarAction::Minimize) => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                    Some(TitleBarAction::Maximize) => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                    }
                    Some(TitleBarAction::Restore) => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
                    }
                    Some(TitleBarAction::Drag) => {
                        // Dragging a maximized window restores it first, as the
                        // platform does — otherwise the drag has nowhere to go.
                        if maximized {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
                        }
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    None => {}
                }
            });

        egui::Panel::top("azur-commands")
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(t.bg.canvas))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let clicked = CommandBar::new([
                        CommandItem::button(&icons::hamburger, "Collapse the rail")
                            .active(self.rail_collapsed),
                        CommandItem::Divider,
                        CommandItem::button(&icons::plus, "New"),
                        CommandItem::button(&icons::search, "Find"),
                        CommandItem::button(&icons::ellipsis, "More").enabled(false),
                    ])
                    .show(ui);
                    if clicked == Some(0) {
                        self.rail_collapsed = !self.rail_collapsed;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let label = if t.dark { "Light theme" } else { "Dark theme" };
                        if ui.add(Button::new(label).size(Size::Small)).clicked() {
                            self.theme = if t.dark {
                                azur::Theme::light()
                            } else {
                                azur::Theme::dark()
                            };
                        }
                        ui.add(
                            Avatar::new("Tony Riviere")
                                .size(Size::Small)
                                .status(AvatarStatus::Online),
                        );
                    });
                });
            });

        let rail = NavigationRail::new(&mut self.section).collapsed(self.rail_collapsed);
        egui::Panel::left("azur-rail")
            .exact_size(rail.width())
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(t.bg.layer))
            .show(ui, |ui| {
                ui.add(rail.items([
                    NavItem::new(SECTIONS[0]).icon(&icons::dot),
                    NavItem::new(SECTIONS[1]).icon(&icons::check),
                    NavItem::new(SECTIONS[2]).icon(&icons::search),
                    NavItem::new(SECTIONS[3]).icon(&icons::dash),
                    NavItem::new(SECTIONS[4]).icon(&icons::chevron_right),
                    NavItem::new(SECTIONS[5]).icon(&icons::info).badge("2"),
                    NavItem::new(SECTIONS[6]).icon(&icons::hamburger),
                    NavItem::new(SECTIONS[7]).icon(&icons::warning).badge("•"),
                ]));
            });

        // ---- Content -------------------------------------------------------
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(t.bg.canvas)
                    .inner_margin(egui::Margin::same(space::S6 as i8)),
            )
            .show(ui, |ui| {
                if let Some(i) = Breadcrumb::new(["Gallery", SECTIONS[self.section]]).show(ui) {
                    if i == 0 {
                        self.section = 0;
                    }
                }
                ui.add_space(space::S3);
                ui.add(Text::new(SECTIONS[self.section]).variant(TextVariant::TitleLarge));
                ui.add_space(space::S5);

                egui::ScrollArea::vertical().show(ui, |ui| match self.section {
                    0 => foundations(ui, &t),
                    1 => actions(ui, self),
                    2 => inputs(ui, self),
                    3 => selection(ui, self),
                    4 => menus(ui, self),
                    5 => overlays(ui, ctx, self),
                    6 => collections(ui, self),
                    _ => feedback(ui, self),
                });
            });

        // The window has no decorations, so it has no frame to drag for a resize.
        resize_handles(ctx);

        self.toaster.show(ctx);
        self.take_screenshot(ctx);
    }
}

/// Two columns of cards, which is the shape most of this gallery is made of.
///
/// One closure rather than two, so a section can touch the same piece of state on
/// both sides without handing out two mutable borrows of it.
fn columns(ui: &mut Ui, add: impl FnOnce(&mut Ui, &mut Ui)) {
    ui.columns(2, |cols| {
        let (a, b) = cols.split_at_mut(1);
        add(&mut a[0], &mut b[0]);
    });
}

/// A titled card.
fn card(ui: &mut Ui, title: &str, add: impl FnOnce(&mut Ui)) {
    ui.add(
        Text::new(title)
            .variant(TextVariant::BodyStrong)
            .color(TextColor::Secondary),
    );
    ui.add_space(space::S2);
    Card::new().variant(CardVariant::Outline).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = space::S3;
        add(ui);
    });
    ui.add_space(space::S5);
}

// ---------------------------------------------------------------------------
// Foundations
// ---------------------------------------------------------------------------

fn foundations(ui: &mut Ui, t: &azur::Theme) {
    columns(ui, |left, right| {
        {
            let ui = left;
            card(ui, "Surfaces", |ui| {
                for (name, color) in [
                    ("bg.canvas", t.bg.canvas),
                    ("bg.layer", t.bg.layer),
                    ("bg.layer_alt", t.bg.layer_alt),
                    ("bg.card", t.bg.card),
                    ("bg.control", t.bg.control),
                    ("bg.control_hover", t.bg.control_hover),
                    ("bg.control_active", t.bg.control_active),
                ] {
                    swatch_row(ui, t, name, color);
                }
            });

            card(ui, "Text", |ui| {
                for (name, variant, color) in [
                    ("display", TextVariant::Display, TextColor::Primary),
                    ("title-large", TextVariant::TitleLarge, TextColor::Primary),
                    ("title", TextVariant::Title, TextColor::Primary),
                    ("title-small", TextVariant::TitleSmall, TextColor::Primary),
                    ("subtitle", TextVariant::Subtitle, TextColor::Primary),
                    ("body", TextVariant::Body, TextColor::Primary),
                    ("body-strong", TextVariant::BodyStrong, TextColor::Primary),
                    (
                        "caption / secondary",
                        TextVariant::Caption,
                        TextColor::Secondary,
                    ),
                ] {
                    ui.add(Text::new(name).variant(variant).color(color));
                }
                ui.add(Text::new("tertiary — metadata").color(TextColor::Tertiary));
                ui.add(Text::new("disabled").color(TextColor::Disabled));
                ui.add(Text::new("accent").color(TextColor::Accent));
                ui.add(Text::new("danger").color(TextColor::Danger));
            });
        }
        {
            let ui = right;
            card(ui, "Accent and status", |ui| {
                for (name, color) in [
                    ("accent.default", t.accent.default),
                    ("accent.hover", t.accent.hover),
                    ("accent.active", t.accent.active),
                    ("accent.subtle", t.accent.subtle),
                    ("status.success", t.status.success),
                    ("status.warning", t.status.warning),
                    ("status.danger", t.status.danger),
                ] {
                    swatch_row(ui, t, name, color);
                }
            });

            card(ui, "Chart series on a card", |ui| {
                let (rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 130.0), Sense::hover());
                let p = ui.painter();
                p.rect_filled(rect, CornerRadius::same(radius::SMALL), t.bg.layer_alt);
                for i in 1..4 {
                    let y = rect.top() + rect.height() * i as f32 / 4.0;
                    p.hline(rect.x_range(), y, Stroke::new(1.0, t.stroke.subtle));
                }
                // One wave per series, phase-shifted so they stay legible together.
                for (s, color) in chart::SERIES.iter().enumerate() {
                    let pts: Vec<Pos2> = (0..=60)
                        .map(|x| {
                            let fx = x as f32 / 60.0;
                            let phase = s as f32 * 0.7;
                            let v = 0.5 + 0.42 * ((fx * 6.0 + phase).sin() * (1.0 - fx * 0.3));
                            Pos2::new(
                                rect.left() + fx * rect.width(),
                                rect.bottom() - v * rect.height(),
                            )
                        })
                        .collect();
                    p.add(egui::Shape::line(pts, Stroke::new(1.5, *color)));
                }
                p.rect_stroke(
                    rect,
                    CornerRadius::same(radius::SMALL),
                    Stroke::new(1.0, t.stroke.subtle),
                    StrokeKind::Inside,
                );
            });

            card(ui, "Heat and thresholds", |ui| {
                for pct in [12.0f32, 48.0, 72.0, 94.0] {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 22.0), Sense::hover());
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius::same(radius::SMALL),
                        // The hue a caller passes is whatever that column's own
                        // graph is drawn in; the gallery stands in with the chart
                        // accent.
                        t.heat(pct / 100.0, chart::AZURE),
                    );
                    ui.painter().text(
                        Pos2::new(rect.left() + space::S3, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        format!("{pct:.0}% load"),
                        t.fonts.body.clone(),
                        t.threshold_text(pct, 60.0, 90.0),
                    );
                }
            });
        }
    });
}

/// A chip of the color with its role beside it — beside rather than on top, so
/// the label stays legible whether the role is near-black or a bright accent.
fn swatch_row(ui: &mut Ui, t: &azur::Theme, name: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 22.0), Sense::hover());
    let corner = CornerRadius::same(radius::SMALL);
    let chip = egui::Rect::from_min_size(rect.min, vec2(44.0, rect.height()));
    ui.painter().rect_filled(chip, corner, color);
    ui.painter().rect_stroke(
        chip,
        corner,
        Stroke::new(1.0, t.stroke.subtle),
        StrokeKind::Inside,
    );
    ui.painter().text(
        Pos2::new(chip.right() + space::S3, rect.center().y),
        egui::Align2::LEFT_CENTER,
        name,
        t.fonts.caption.clone(),
        t.text.secondary,
    );
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

fn actions(ui: &mut Ui, g: &mut Gallery) {
    columns(ui, |left, right| {
        {
            let ui = left;
            card(ui, "Button variants", |ui| {
                for (label, variant) in [
                    ("Primary", Variant::Primary),
                    ("Secondary", Variant::Secondary),
                    ("Subtle", Variant::Subtle),
                    ("Outline", Variant::Outline),
                    ("Danger", Variant::Danger),
                ] {
                    ui.horizontal(|ui| {
                        let _ = ui.add(Button::new(label).variant(variant).min_width(110.0));
                        let _ = ui.add(
                            Button::new("Icon")
                                .variant(variant)
                                .icon(&icons::check)
                                .min_width(96.0),
                        );
                        ui.add_enabled(
                            false,
                            Button::new("Disabled").variant(variant).min_width(96.0),
                        );
                    });
                }
            });

            card(ui, "Sizes, icons and loading", |ui| {
                ui.horizontal(|ui| {
                    let _ = ui.add(Button::new("Small").size(Size::Small));
                    let _ = ui.add(Button::new("Medium"));
                    let _ = ui.add(Button::new("Large").size(Size::Large));
                });
                ui.horizontal(|ui| {
                    let _ = ui.add(Button::new("Saving").loading(true));
                    let _ = ui.add(
                        Button::new("Open")
                            .variant(Variant::Secondary)
                            .trailing_icon(&icons::chevron_down),
                    );
                });
                let _ = ui.add(
                    Button::new("Full width")
                        .variant(Variant::Primary)
                        .full_width(true),
                );
            });

            card(ui, "IconButton", |ui| {
                ui.horizontal(|ui| {
                    for (variant, size) in [
                        (Variant::Primary, Size::Small),
                        (Variant::Secondary, Size::Medium),
                        (Variant::Subtle, Size::Medium),
                        (Variant::Outline, Size::Medium),
                        (Variant::Danger, Size::Large),
                    ] {
                        let _ = ui.add(
                            IconButton::new(&icons::close)
                                .variant(variant)
                                .size(size)
                                .tooltip("Close"),
                        );
                    }
                });
                ui.horizontal(|ui| {
                    ui.add(Text::new("Latched:").color(TextColor::Secondary));
                    let _ = ui.add(IconButton::new(&icons::hamburger).pressed(true));
                    ui.add_enabled(false, IconButton::new(&icons::search));
                });
            });
        }
        {
            let ui = right;
            card(ui, "Cards", |ui| {
                for (name, variant) in [
                    ("Elevated", CardVariant::Elevated),
                    ("Outline", CardVariant::Outline),
                    ("Filled", CardVariant::Filled),
                ] {
                    Card::new()
                        .variant(variant)
                        .padding(Padding::Small)
                        .interactive(true)
                        .show(ui, |ui| {
                            ui.add(Text::new(name).variant(TextVariant::BodyStrong));
                            ui.add(
                                Text::new("Interactive — hover and press it.")
                                    .color(TextColor::Secondary),
                            );
                        });
                }
            });

            card(ui, "Divider", |ui| {
                ui.add(Text::new("Above"));
                ui.add(Divider::horizontal());
                ui.add(Text::new("Between"));
                ui.add(Divider::horizontal().inset(true));
                ui.horizontal(|ui| {
                    ui.add(Text::new("Left"));
                    ui.add(Divider::vertical().length(20.0));
                    ui.add(Text::new("Right"));
                });
            });

            card(ui, "Avatars", |ui| {
                ui.horizontal(|ui| {
                    ui.add(Avatar::new("Ada Lovelace").size(Size::Small));
                    ui.add(Avatar::new("Grace Hopper").status(AvatarStatus::Away));
                    ui.add(
                        Avatar::new("Alan Turing")
                            .size(Size::Large)
                            .shape(AvatarShape::Square)
                            .status(AvatarStatus::Busy),
                    );
                    ui.add(Avatar::new("").status(AvatarStatus::Offline));
                });
            });

            card(ui, "Tabs", |ui| {
                ui.add(Tabs::new(&mut g.tab).tabs(["Geometry", "Appearance", "Advanced"]));
                ui.add_space(space::S3);
                ui.add(
                    Text::new(["Geometry panel", "Appearance panel", "Advanced panel"][g.tab])
                        .color(TextColor::Secondary),
                );
            });
        }
    });
}

// ---------------------------------------------------------------------------
// Inputs
// ---------------------------------------------------------------------------

fn inputs(ui: &mut Ui, g: &mut Gallery) {
    columns(ui, |left, right| {
        {
            let ui = left;
            card(ui, "TextField", |ui| {
                ui.add(
                    TextField::new(&mut g.name)
                        .label("Project name")
                        .required(true)
                        .placeholder("Untitled")
                        .message("Shown in the title bar")
                        .clearable(true),
                );
                ui.add(
                    TextField::new(&mut g.bad_email)
                        .label("Contact")
                        .error(true)
                        .message("That is not an email address"),
                );
                ui.add_enabled(
                    false,
                    TextField::new(&mut g.name)
                        .label("Disabled")
                        .message("Read-only"),
                );
            });

            card(ui, "Sizes", |ui| {
                for size in [Size::Small, Size::Medium, Size::Large] {
                    ui.add(TextField::new(&mut g.search).size(size).placeholder("Size"));
                }
            });

            card(ui, "SearchBox", |ui| {
                ui.add(SearchBox::new(&mut g.search).placeholder("Search scans"));
                ui.add(SearchBox::new(&mut g.search).size(Size::Small));
            });
        }
        {
            let ui = right;
            card(ui, "TextArea", |ui| {
                ui.add(
                    TextArea::new(&mut g.notes)
                        .label("Notes")
                        .placeholder("Anything worth remembering about this survey")
                        .rows(4),
                );
            });

            card(ui, "NumberField", |ui| {
                ui.add(
                    NumberField::new(&mut g.tolerance)
                        .label("Tolerance")
                        .range(0.0..=100.0)
                        .step(0.5)
                        .decimals(2)
                        .message("Millimetres"),
                );
                ui.add_enabled(false, NumberField::new(&mut g.tolerance).label("Disabled"));
            });
        }
    });
}

// ---------------------------------------------------------------------------
// Selection
// ---------------------------------------------------------------------------

fn selection(ui: &mut Ui, g: &mut Gallery) {
    columns(ui, |left, right| {
        {
            let ui = left;
            card(ui, "Checkbox", |ui| {
                ui.add(Checkbox::new(&mut g.snap, "Snap to grid"));
                ui.add(Checkbox::new(&mut g.hidden, "Show hidden layers"));
                ui.add(Checkbox::new(&mut g.mixed, "Partially applied").indeterminate(true));
                ui.add_enabled(false, Checkbox::new(&mut g.snap, "Disabled, checked"));
                ui.add_enabled(false, Checkbox::new(&mut g.hidden, "Disabled"));
                ui.horizontal(|ui| {
                    for size in [Size::Small, Size::Medium, Size::Large] {
                        ui.add(Checkbox::new(&mut g.snap, "").size(size));
                    }
                });
            });

            card(ui, "Radio", |ui| {
                RadioGroup::vertical().show(ui, |ui| {
                    ui.add(Radio::new(&mut g.fit, Fit::Width, "Fit width"));
                    ui.add(Radio::new(&mut g.fit, Fit::Page, "Fit page"));
                    ui.add_enabled(false, Radio::new(&mut g.fit, Fit::Actual, "Actual size"));
                });
                ui.add(Divider::horizontal());
                RadioGroup::horizontal().show(ui, |ui| {
                    ui.add(Radio::new(&mut g.fit, Fit::Width, "Width").size(Size::Small));
                    ui.add(Radio::new(&mut g.fit, Fit::Page, "Page").size(Size::Small));
                });
            });
        }
        {
            let ui = right;
            card(ui, "Switch", |ui| {
                ui.add(Switch::new(&mut g.snap, "Snap to grid"));
                ui.add(
                    Switch::new(&mut g.hidden, "Label first").label_position(SwitchLabel::Start),
                );
                ui.add_enabled(false, Switch::new(&mut g.snap, "Disabled, on"));
                ui.horizontal(|ui| {
                    for size in [Size::Small, Size::Medium, Size::Large] {
                        ui.add(Switch::new(&mut g.hidden, "").size(size));
                    }
                });
            });

            card(ui, "Slider", |ui| {
                ui.add(
                    Slider::new(&mut g.opacity, 0.0..=1.0)
                        .label("Opacity")
                        .decimals(2),
                );
                ui.add(
                    Slider::new(&mut g.density, 0.0..=100.0)
                        .label("Point density")
                        .suffix("%")
                        .step(5.0)
                        .size(Size::Large),
                );
                ui.add(Slider::new(&mut g.opacity, 0.0..=1.0).size(Size::Small));
                ui.add_enabled(
                    false,
                    Slider::new(&mut g.opacity, 0.0..=1.0)
                        .label("Disabled")
                        .decimals(2),
                );
            });
        }
    });
}

// ---------------------------------------------------------------------------
// Menus
// ---------------------------------------------------------------------------

fn menus(ui: &mut Ui, g: &mut Gallery) {
    columns(ui, |left, right| {
        {
            let ui = left;
            card(ui, "Select", |ui| {
                ui.add(
                    Select::new(&mut g.units)
                        .options(["Millimetres", "Centimetres", "Inches"])
                        .placeholder("Pick units"),
                );
                ui.add(
                    Select::new(&mut g.units)
                        .options([
                            SelectOption::new("Available"),
                            SelectOption::new("Unavailable").disabled(true),
                        ])
                        .size(Size::Small),
                );
                ui.add(
                    Select::new(&mut g.units)
                        .options(["Millimetres", "Centimetres", "Inches"])
                        .variant(Variant::Subtle),
                );
                ui.add_enabled(false, Select::new(&mut g.units).options(["Disabled"]));
            });

            card(ui, "ComboBox", |ui| {
                ui.add(
                    ComboBox::new(&mut g.filter_text, &mut g.filter_pick)
                        .options(["Bilinear", "Bicubic", "Nearest", "Lanczos"])
                        .placeholder("Type to filter"),
                );
                ui.add(
                    Text::new(match g.filter_pick {
                        Some(i) => format!("Picked #{i}"),
                        None => "Nothing picked".to_owned(),
                    })
                    .color(TextColor::Tertiary),
                );
            });
        }
        {
            let ui = right;
            card(ui, "Menu and context menu", |ui| {
                let trigger = ui.add(
                    Button::new("Edit")
                        .variant(Variant::Secondary)
                        .trailing_icon(&icons::chevron_down),
                );
                Menu::new(&trigger).show(ui.ctx(), |ui| {
                    let _ = ui.add(MenuItem::new("Undo").shortcut("Ctrl+Z"));
                    let _ = ui.add(MenuItem::new("Redo").shortcut("Ctrl+Y"));
                    menu_divider(ui);
                    let _ = ui.add(MenuItem::new("Snap to grid").selected(true));
                    let _ = ui.add(MenuItem::new("Transform").submenu(true));
                    ui.add_enabled(false, MenuItem::new("Paste"));
                    menu_divider(ui);
                    let _ = ui.add(MenuItem::new("Delete").icon(&icons::close).danger(true));
                });

                let target = Card::new()
                    .variant(CardVariant::Filled)
                    .padding(Padding::Small)
                    .interactive(true)
                    .show(ui, |ui| {
                        ui.add(Text::new("Right-click me").color(TextColor::Secondary));
                    })
                    .response;
                ContextMenu::new(&target).show(ui.ctx(), |ui| {
                    let _ = ui.add(MenuItem::new("Inspect"));
                    let _ = ui.add(MenuItem::new("Duplicate").shortcut("Ctrl+D"));
                });
            });

            card(ui, "Popover", |ui| {
                let anchor = ui.add(Button::new("Details").icon(&icons::info));
                if anchor.clicked() {
                    g.popover_open = !g.popover_open;
                }
                Popover::new(&anchor)
                    .open(&mut g.popover_open)
                    .width(260.0)
                    .show(ui.ctx(), |ui| {
                        ui.add(Text::new("Scan 4").variant(TextVariant::BodyStrong));
                        ui.add(
                            Text::new("18.2 M points, captured 2026-07-14.")
                                .color(TextColor::Secondary),
                        );
                        ui.add(ProgressBar::new(0.8));
                    });
            });

            card(ui, "Tooltips", |ui| {
                let r = ui.add(Button::new("Hover me").variant(Variant::Subtle));
                tooltip(
                    r,
                    "Azur's tooltip: caption type, radius-small, 280px wide at most.",
                );
            });
        }
    });
}

// ---------------------------------------------------------------------------
// Overlays
// ---------------------------------------------------------------------------

fn overlays(ui: &mut Ui, ctx: &egui::Context, g: &mut Gallery) {
    card(ui, "Dialog and drawer", |ui| {
        ui.horizontal(|ui| {
            if ui
                .add(Button::new("Discard changes…").variant(Variant::Danger))
                .clicked()
            {
                g.confirm_open = true;
            }
            if ui.add(Button::new("Open drawer")).clicked() {
                g.drawer_open = true;
            }
        });
    });

    card(ui, "What they are built on", |ui| {
        ui.add(Text::new(
            "A dialog is background-layer behind a background-overlay scrim, with \
             shadow-64. A drawer is the same panel against an edge, with shadow-28.",
        ));
    });

    let mut confirmed = false;
    Dialog::new("Discard changes?", &mut g.confirm_open)
        .size(DialogSize::Small)
        .footer(|ui| {
            // Right-to-left: the primary action is added first, so it lands last.
            if ui
                .add(Button::new("Discard").variant(Variant::Danger))
                .clicked()
            {
                confirmed = true;
            }
            let _ = ui.add(Button::new("Keep editing").variant(Variant::Subtle));
        })
        .show(ctx, |ui| {
            ui.add(Text::new("Everything since the last save will be lost."));
            ui.add(MessageBar::new("This cannot be undone.").intent(Intent::Warning));
        });
    if confirmed {
        g.confirm_open = false;
        g.toaster
            .push(Toast::new("Changes discarded").intent(Intent::Success));
    }

    Drawer::new("Layer properties", &mut g.drawer_open)
        .side(DrawerSide::Right)
        .show(ctx, |ui| {
            section_heading(ui, "Appearance");
            ui.add(
                Slider::new(&mut g.opacity, 0.0..=1.0)
                    .label("Opacity")
                    .decimals(2),
            );
            ui.add(Checkbox::new(&mut g.hidden, "Hidden"));
            ui.add_space(space::S5);
            section_heading(ui, "Metadata");
            ui.add(TextField::new(&mut g.name).label("Name"));
        });
}

// ---------------------------------------------------------------------------
// Collections
// ---------------------------------------------------------------------------

fn collections(ui: &mut Ui, g: &mut Gallery) {
    columns(ui, |left, right| {
        {
            let ui = left;
            card(ui, "ListView", |ui| {
                ui.add(
                    ListView::new(&mut g.list_selection)
                        .rows([
                            ListRow::new("scan_01.e57")
                                .secondary("482 MB")
                                .icon(&icons::dot),
                            ListRow::new("scan_02.e57")
                                .secondary("511 MB")
                                .icon(&icons::dot),
                            ListRow::new("scan_03.e57")
                                .secondary("Not imported")
                                .disabled(true),
                            ListRow::new("scan_04.e57")
                                .secondary("377 MB")
                                .icon(&icons::dot),
                        ])
                        .max_height(180.0),
                );
                ui.add(
                    ListView::new(&mut g.list_selection)
                        .rows(["Compact", "Rows", "Are", "28px"])
                        .compact(true)
                        .max_height(120.0),
                );
            });

            card(ui, "TreeView", |ui| {
                ui.add(
                    TreeView::new(&mut g.tree)
                        .nodes(
                            [TreeNode::new(1, "Site survey").icon(&icons::dot).children([
                                TreeNode::new(2, "Scans").children([
                                    TreeNode::new(3, "scan_01"),
                                    TreeNode::new(4, "scan_02"),
                                ]),
                                TreeNode::new(5, "Meshes").children([TreeNode::new(6, "shell")]),
                                TreeNode::new(7, "Archived").disabled(true),
                            ])],
                        )
                        .max_height(220.0),
                );
            });
        }
        {
            let ui = right;
            card(ui, "Table", |ui| {
                let rows = [
                    ("scan_01", 18.2, "Registered"),
                    ("scan_02", 21.7, "Registered"),
                    ("scan_03", 9.4, "Failed"),
                    ("scan_04", 15.1, "Pending"),
                    ("scan_05", 12.8, "Registered"),
                ];
                Table::new()
                    .columns([
                        TableColumn::new("Scan").sortable(true),
                        TableColumn::new("M points")
                            .numeric(true)
                            .width(96.0)
                            .sortable(true),
                        TableColumn::new("State").width(120.0),
                    ])
                    .sort(&mut g.table_sort)
                    .selection(&mut g.table_selection)
                    .max_height(240.0)
                    .show(ui, rows.len(), |ui, r, c| {
                        let (name, points, state) = rows[r];
                        match c {
                            0 => {
                                ui.add(Text::new(name).truncate(true));
                            }
                            1 => {
                                ui.add(Text::new(format!("{points:.1}")));
                            }
                            _ => {
                                ui.add(Badge::new(state).small(true).variant(match state {
                                    "Registered" => BadgeVariant::Success,
                                    "Failed" => BadgeVariant::Danger,
                                    _ => BadgeVariant::Neutral,
                                }));
                            }
                        }
                    });
                ui.add(
                    Text::new(match g.table_sort {
                        Some((c, true)) => format!("Sorted by column {c}, ascending"),
                        Some((c, false)) => format!("Sorted by column {c}, descending"),
                        None => "Unsorted".to_owned(),
                    })
                    .color(TextColor::Tertiary),
                );
            });
        }
    });
}

// ---------------------------------------------------------------------------
// Feedback
// ---------------------------------------------------------------------------

fn feedback(ui: &mut Ui, g: &mut Gallery) {
    columns(ui, |left, right| {
        {
            let ui = left;
            card(ui, "ProgressBar", |ui| {
                ui.add(ProgressBar::new(g.progress));
                ui.add(
                    ProgressBar::new(0.7)
                        .color(ProgressColor::Success)
                        .small(true),
                );
                ui.add(ProgressBar::new(0.35).color(ProgressColor::Warning));
                ui.add(
                    ProgressBar::new(0.9)
                        .color(ProgressColor::Danger)
                        .small(true),
                );
                ui.add(ProgressBar::indeterminate());
                ui.add(
                    Slider::new(&mut g.progress, 0.0..=1.0)
                        .label("Value")
                        .decimals(2),
                );
            });

            card(ui, "Spinner", |ui| {
                ui.horizontal(|ui| {
                    for size in [Size::Small, Size::Medium, Size::Large] {
                        ui.add(Spinner::new().size(size));
                    }
                });
            });

            card(ui, "Badge", |ui| {
                ui.horizontal(|ui| {
                    for (label, variant) in [
                        ("Neutral", BadgeVariant::Neutral),
                        ("Accent", BadgeVariant::Accent),
                        ("Success", BadgeVariant::Success),
                        ("Warning", BadgeVariant::Warning),
                        ("Danger", BadgeVariant::Danger),
                    ] {
                        ui.add(Badge::new(label).variant(variant));
                    }
                });
                ui.horizontal(|ui| {
                    for variant in [
                        BadgeVariant::Neutral,
                        BadgeVariant::Accent,
                        BadgeVariant::Success,
                        BadgeVariant::Warning,
                        BadgeVariant::Danger,
                    ] {
                        ui.add(Badge::dot().variant(variant));
                    }
                    ui.add(Badge::new("12").small(true).variant(BadgeVariant::Accent));
                });
            });
        }
        {
            let ui = right;
            card(ui, "MessageBar", |ui| {
                for (intent, title, body) in [
                    (
                        Intent::Info,
                        "Import queued",
                        "Four scans will be imported in the background.",
                    ),
                    (
                        Intent::Success,
                        "Registration complete",
                        "Mean error 1.8 mm across 12 scans.",
                    ),
                    (
                        Intent::Warning,
                        "Registration incomplete",
                        "Two scans could not be aligned.",
                    ),
                    (Intent::Danger, "Import failed", "scan_03.e57 is truncated."),
                ] {
                    ui.add(MessageBar::new(body).title(title).intent(intent));
                }
                if g.show_banner {
                    let dismissed = ui
                        .add(
                            MessageBar::new("Dismissible — click the close button.")
                                .intent(Intent::Info)
                                .dismissible(true),
                        )
                        .clicked();
                    if dismissed {
                        g.show_banner = false;
                    }
                }
            });

            card(ui, "Toast", |ui| {
                ui.horizontal(|ui| {
                    for (label, intent) in [
                        ("Info", Intent::Info),
                        ("Success", Intent::Success),
                        ("Warning", Intent::Warning),
                        ("Danger", Intent::Danger),
                    ] {
                        if ui.add(Button::new(label).size(Size::Small)).clicked() {
                            g.toaster.push(
                                Toast::new("Six scans written to D:\\exports.")
                                    .title(format!("{label} toast"))
                                    .intent(intent),
                            );
                        }
                    }
                });
                if ui
                    .add(Button::new("Sticky toast").variant(Variant::Subtle))
                    .clicked()
                {
                    g.toaster.push(
                        Toast::new("Stays until you close it.")
                            .title("No timeout")
                            .duration(None),
                    );
                }
            });

            card(ui, "egui's own widgets, themed", |ui| {
                ui.add(Text::new(
                    "Everything above is Azur's. These are egui's, wearing the token set:",
                ));
                ui.horizontal(|ui| {
                    let _ = ui.button("egui button");
                    ui.checkbox(&mut g.snap, "egui checkbox");
                });
                ui.add(egui::Slider::new(&mut g.opacity, 0.0..=1.0).text("egui slider"));
                ui.add(egui::TextEdit::singleline(&mut g.search).desired_width(f32::INFINITY));
                ui.hyperlink_to("An egui link", "https://example.com");
                egui::ComboBox::from_id_salt("egui-combo")
                    .selected_text("egui combo")
                    .show_ui(ui, |ui| {
                        // Without this, a hovered entry gains a border and so gains
                        // 2px of width, which re-lays out the popup under the cursor.
                        azur::list_items(ui, &azur::components::theme(ui.ctx()));
                        for name in ["First", "Second", "Third"] {
                            let _ = ui.selectable_label(false, name);
                        }
                    });
            });
        }
    });
}
