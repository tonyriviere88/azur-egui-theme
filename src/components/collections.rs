//! `ListView`, `TreeView`, `Table`, `Tabs`.
//!
//! egui has no collections, so all four are painted here. They share one set of
//! row rules, which is where Azur's list vocabulary lives:
//!
//! | state | fill | extra |
//! | --- | --- | --- |
//! | rest | — | |
//! | hover | `background-card-hover` | |
//! | selected | `accent-subtle` | a 2px `accent-default` bar on the left |
//! | selected + hover | `accent-subtle-hover` | |
//! | disabled | — | `text-disabled` |

use super::{theme_of, truncated};
use crate::icons::{chevron_down, chevron_right, focus_ring_inset, sort_asc, sort_desc, Icon};
use crate::theme::Theme;
use crate::tokens::{radius, row, space, typography};
use egui::{
    pos2, vec2, Align, Color32, CornerRadius, InnerResponse, Layout, Rect, Response, Sense, Stroke,
    StrokeKind, Ui, UiBuilder, Widget,
};
use std::borrow::Cow;
use std::collections::HashSet;

/// The fill for a row in a given state, or `None` for "leave the surface alone".
///
/// Azur's list vocabulary, as the table at the top of this file states it. A dense
/// application window wants [`crate::desktop::row_fill`] instead — on a near-black
/// panel `accent-subtle` is a navy barely off the surface, and that module says why at
/// length. The *hover* needs no such split: it reads `background-card-hover`, which
/// [`crate::desktop::apply`] moves, so a window that has adopted the preset gets its
/// own hover here for free.
fn row_fill(t: &Theme, selected: bool, hovered: bool) -> Option<Color32> {
    match (selected, hovered) {
        (true, true) => Some(t.accent.subtle_hover),
        (true, false) => Some(t.accent.subtle),
        (false, true) => Some(t.bg.card_hover),
        (false, false) => None,
    }
}

/// The container `ListView`, `TreeView` and `Table` share: `background-layer`, a
/// `stroke-subtle` border, `radius-medium`.
fn container_frame(t: &Theme) -> egui::Frame {
    egui::Frame::new()
        .fill(t.bg.layer)
        .stroke(Stroke::new(1.0, t.stroke.subtle))
        .corner_radius(CornerRadius::same(radius::MEDIUM))
        .inner_margin(egui::Margin::ZERO)
}

/// One entry of a [`ListView`].
pub struct ListRow<'a> {
    pub label: Cow<'a, str>,
    /// The second line, in caption / `text-secondary`. Doubles the row's content
    /// but not its height — Azur's comfortable row is 40px either way.
    pub secondary: Cow<'a, str>,
    pub icon: Option<Icon<'a>>,
    pub disabled: bool,
}

impl<'a> ListRow<'a> {
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            secondary: Cow::Borrowed(""),
            icon: None,
            disabled: false,
        }
    }

    pub fn secondary(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.secondary = text.into();
        self
    }

    pub fn icon(mut self, icon: Icon<'a>) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<'a> From<&'a str> for ListRow<'a> {
    fn from(label: &'a str) -> Self {
        Self::new(label)
    }
}

/// Azur's `ListView`: a bordered list of single-select rows.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::{ListRow, ListView};
/// let mut selected = Some(0);
/// ui.add(
///     ListView::new(&mut selected)
///         .rows([
///             ListRow::new("scan_01.e57").secondary("482 MB"),
///             ListRow::new("scan_02.e57").secondary("511 MB"),
///         ])
///         .max_height(200.0),
/// );
/// # }
/// ```
pub struct ListView<'a> {
    selected: &'a mut Option<usize>,
    rows: Vec<ListRow<'a>>,
    compact: bool,
    max_height: Option<f32>,
    width: Option<f32>,
}

impl<'a> ListView<'a> {
    pub fn new(selected: &'a mut Option<usize>) -> Self {
        Self {
            selected,
            rows: Vec::new(),
            compact: false,
            max_height: None,
            width: None,
        }
    }

    pub fn rows<I>(mut self, rows: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<ListRow<'a>>,
    {
        self.rows = rows.into_iter().map(Into::into).collect();
        self
    }

    /// 28px rows instead of 40px — Azur's `density="compact"`.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Scroll past this height rather than growing.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for ListView<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let height = if self.compact {
            row::COMPACT
        } else {
            row::COMFORTABLE
        };
        let width = self.width.unwrap_or_else(|| ui.available_width());
        // Taken before anything is allocated, so two `ListView`s in one parent get
        // different ids -- a `ScrollArea` keyed off the enclosing frame alone would
        // collide, and egui would scroll both together.
        let id = ui.next_auto_id();
        let mut changed = false;

        let outer = container_frame(&t).show(ui, |ui| {
            ui.set_width(width - 2.0);
            ui.spacing_mut().item_spacing.y = 0.0;
            let mut scroll = egui::ScrollArea::vertical()
                .id_salt(id)
                .auto_shrink([false, true]);
            if let Some(h) = self.max_height {
                scroll = scroll.max_height(h);
            }
            let out = scroll.show(ui, |ui| {
                for (i, item) in self.rows.iter().enumerate() {
                    let row_enabled = enabled && !item.disabled;
                    let (rect, response) = ui.allocate_exact_size(
                        vec2(ui.available_width(), height),
                        if row_enabled {
                            Sense::click()
                        } else {
                            Sense::hover()
                        },
                    );
                    let selected = *self.selected == Some(i);
                    if response.clicked() {
                        *self.selected = Some(i);
                        changed = true;
                    }
                    if !ui.is_rect_visible(rect) {
                        continue;
                    }

                    if let Some(fill) = row_fill(&t, selected, row_enabled && response.hovered()) {
                        ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);
                    }
                    // `border-left: 2px solid accent-default` when selected.
                    if selected {
                        ui.painter().rect_filled(
                            Rect::from_min_size(rect.min, vec2(2.0, rect.height())),
                            CornerRadius::ZERO,
                            t.accent.mark,
                        );
                    }

                    // `padding: 0 space-4`, `gap: space-3`.
                    let mut x = rect.left() + space::S4;
                    if let Some(icon) = item.icon {
                        icon(
                            ui.painter(),
                            super::icon_rect(rect, x, 16.0),
                            if row_enabled {
                                t.text.secondary
                            } else {
                                t.text.disabled
                            },
                        );
                        x += 16.0 + space::S3;
                    }
                    let text_w = (rect.right() - space::S4 - x).max(0.0);
                    let label = truncated(
                        ui,
                        &item.label,
                        t.fonts.body.clone(),
                        t.text_for(row_enabled),
                        text_w,
                    );

                    if item.secondary.is_empty() {
                        super::galley_left_center(
                            ui,
                            Rect::from_min_max(pos2(x, rect.top()), rect.max),
                            label,
                        );
                    } else {
                        let secondary = truncated(
                            ui,
                            &item.secondary,
                            t.fonts.caption.clone(),
                            if row_enabled {
                                t.text.secondary
                            } else {
                                t.text.disabled
                            },
                            text_w,
                        );
                        let total = label.size().y + secondary.size().y;
                        let top = rect.center().y - total * 0.5;
                        ui.painter()
                            .galley(pos2(x, top), label.clone(), Color32::PLACEHOLDER);
                        ui.painter().galley(
                            pos2(x, top + label.size().y),
                            secondary,
                            Color32::PLACEHOLDER,
                        );
                    }

                    if response.has_focus() {
                        focus_ring_inset(ui.painter(), rect, CornerRadius::ZERO, t.stroke.focus);
                    }
                }
            });
            // A list longer than its box says so at the edge it is cut off at, on
            // [`container_frame`]'s own fill — which is what the rows are sitting on.
            super::scroll_fades_of(ui.painter(), &out, t.bg.layer);
        });

        let mut response = outer.response;
        if changed {
            response.mark_changed();
        }
        response
    }
}

/// One node of a [`TreeView`].
pub struct TreeNode<'a> {
    /// Stable across frames — expansion and selection are keyed on it.
    pub id: u64,
    pub label: Cow<'a, str>,
    pub icon: Option<Icon<'a>>,
    pub disabled: bool,
    pub children: Vec<TreeNode<'a>>,
}

impl<'a> TreeNode<'a> {
    pub fn new(id: u64, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            disabled: false,
            children: Vec::new(),
        }
    }

    pub fn icon(mut self, icon: Icon<'a>) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn children<I: IntoIterator<Item = TreeNode<'a>>>(mut self, children: I) -> Self {
        self.children = children.into_iter().collect();
        self
    }
}

/// What a [`TreeView`] remembers between frames: which nodes are open, and which
/// one is selected.
///
/// Kept by the caller rather than in egui's memory, because a tree's expansion is
/// application state — it survives the widget being scrolled out of existence, and
/// something else usually wants to read it.
#[derive(Clone, Debug, Default)]
pub struct TreeNodeState {
    expanded: HashSet<u64>,
    pub selected: Option<u64>,
}

impl TreeNodeState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start with a set of nodes already open.
    pub fn with_expanded<I: IntoIterator<Item = u64>>(ids: I) -> Self {
        Self {
            expanded: ids.into_iter().collect(),
            selected: None,
        }
    }

    pub fn is_expanded(&self, id: u64) -> bool {
        self.expanded.contains(&id)
    }

    pub fn set_expanded(&mut self, id: u64, expanded: bool) {
        if expanded {
            self.expanded.insert(id);
        } else {
            self.expanded.remove(&id);
        }
    }

    pub fn toggle(&mut self, id: u64) {
        if !self.expanded.insert(id) {
            self.expanded.remove(&id);
        }
    }
}

/// Azur's `TreeView`: 30px rows, `space-6` of indent per level, a chevron on
/// anything with children.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::{TreeNode, TreeNodeState, TreeView};
/// let mut state = TreeNodeState::with_expanded([1]);
/// ui.add(TreeView::new(&mut state).nodes([
///     TreeNode::new(1, "Project").children([TreeNode::new(2, "Scan 1")]),
/// ]));
/// # }
/// ```
pub struct TreeView<'a> {
    state: &'a mut TreeNodeState,
    nodes: Vec<TreeNode<'a>>,
    max_height: Option<f32>,
    width: Option<f32>,
}

impl<'a> TreeView<'a> {
    pub fn new(state: &'a mut TreeNodeState) -> Self {
        Self {
            state,
            nodes: Vec::new(),
            max_height: None,
            width: None,
        }
    }

    pub fn nodes<I: IntoIterator<Item = TreeNode<'a>>>(mut self, nodes: I) -> Self {
        self.nodes = nodes.into_iter().collect();
        self
    }

    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

/// `.row { height: 30px }`.
const TREE_ROW: f32 = 30.0;

impl Widget for TreeView<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let id = ui.next_auto_id();
        let mut changed = false;

        let outer = container_frame(&t).show(ui, |ui| {
            ui.set_width(width - 2.0);
            ui.spacing_mut().item_spacing.y = 0.0;
            // `.root { padding: space-2 0 }`.
            ui.add_space(space::S2);
            let mut scroll = egui::ScrollArea::vertical()
                .id_salt(id)
                .auto_shrink([false, true]);
            if let Some(h) = self.max_height {
                scroll = scroll.max_height(h);
            }
            let out = scroll.show(ui, |ui| {
                let mut stack: Vec<(&TreeNode<'_>, usize)> =
                    self.nodes.iter().rev().map(|n| (n, 0usize)).collect();
                while let Some((node, depth)) = stack.pop() {
                    if tree_row(ui, &t, node, depth, self.state, enabled) {
                        changed = true;
                    }
                    if self.state.is_expanded(node.id) {
                        for child in node.children.iter().rev() {
                            stack.push((child, depth + 1));
                        }
                    }
                }
            });
            super::scroll_fades_of(ui.painter(), &out, t.bg.layer);
            ui.add_space(space::S2);
        });

        let mut response = outer.response;
        if changed {
            response.mark_changed();
        }
        response
    }
}

/// One tree row. Returns whether the selection changed.
fn tree_row(
    ui: &mut Ui,
    t: &Theme,
    node: &TreeNode<'_>,
    depth: usize,
    state: &mut TreeNodeState,
    enabled: bool,
) -> bool {
    let row_enabled = enabled && !node.disabled;
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), TREE_ROW),
        if row_enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let selected = state.selected == Some(node.id);
    let has_children = !node.children.is_empty();

    // `padding: 0 space-3`, and one `space-6` of indent per level.
    let indent = space::S6 * depth as f32;
    let chevron_rect = super::icon_rect(rect, rect.left() + space::S3 + indent, 18.0);

    let mut changed = false;
    if response.clicked() {
        if has_children
            && chevron_rect.contains(response.interact_pointer_pos().unwrap_or_default())
        {
            state.toggle(node.id);
        } else {
            state.selected = Some(node.id);
        }
        changed = true;
    }
    if response.double_clicked() && has_children {
        state.toggle(node.id);
    }

    if !ui.is_rect_visible(rect) {
        return changed;
    }

    let corner = CornerRadius::same(radius::SMALL);
    if let Some(fill) = row_fill(t, selected, row_enabled && response.hovered()) {
        ui.painter().rect_filled(rect, corner, fill);
    }

    let mut x = rect.left() + space::S3 + indent;
    if has_children {
        let color = if row_enabled {
            t.text.secondary
        } else {
            t.text.disabled
        };
        if state.is_expanded(node.id) {
            chevron_down(ui.painter(), chevron_rect, color);
        } else {
            chevron_right(ui.painter(), chevron_rect, color);
        }
    }
    x += 18.0 + space::S2;

    if let Some(icon) = node.icon {
        icon(
            ui.painter(),
            super::icon_rect(rect, x, 16.0),
            if row_enabled {
                t.text.secondary
            } else {
                t.text.disabled
            },
        );
        x += 16.0 + space::S2;
    }

    let galley = truncated(
        ui,
        &node.label,
        t.fonts.body.clone(),
        t.text_for(row_enabled),
        (rect.right() - space::S3 - x).max(0.0),
    );
    super::galley_left_center(
        ui,
        Rect::from_min_max(pos2(x, rect.top()), rect.max),
        galley,
    );

    if response.has_focus() {
        focus_ring_inset(ui.painter(), rect, corner, t.stroke.focus);
    }
    changed
}

/// One column of a [`Table`].
pub struct TableColumn<'a> {
    pub header: Cow<'a, str>,
    /// A fixed width, or `None` to share what is left evenly.
    pub width: Option<f32>,
    pub sortable: bool,
    /// Right-align the header and its cells — for numbers.
    pub numeric: bool,
}

impl<'a> TableColumn<'a> {
    pub fn new(header: impl Into<Cow<'a, str>>) -> Self {
        Self {
            header: header.into(),
            width: None,
            sortable: false,
            numeric: false,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    pub fn numeric(mut self, numeric: bool) -> Self {
        self.numeric = numeric;
        self
    }
}

impl<'a> From<&'a str> for TableColumn<'a> {
    fn from(header: &'a str) -> Self {
        Self::new(header)
    }
}

/// Which column a [`Table`] is sorted by, and which way.
pub type Sort = Option<(usize, bool)>;

/// Azur's `Table`: a sticky header strip over ruled rows.
///
/// The header does not scroll with the body — Azur gets that from
/// `position: sticky`, this gets it by putting the header outside the scroll area,
/// which is the same result and the only one egui can give.
///
/// Cells are drawn by a closure rather than declared as data, so a cell can hold
/// anything a `Ui` can:
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::{Table, TableColumn};
/// let mut sort = Some((0usize, true));
/// let mut selected = None;
/// Table::new()
///     .columns([
///         TableColumn::new("Name").sortable(true),
///         TableColumn::new("Points").numeric(true).width(90.0),
///     ])
///     .sort(&mut sort)
///     .selection(&mut selected)
///     .show(ui, 3, |ui, r, c| {
///         ui.label(format!("r{r}c{c}"));
///     });
/// # }
/// ```
pub struct Table<'a> {
    columns: Vec<TableColumn<'a>>,
    sort: Option<&'a mut Sort>,
    selected: Option<&'a mut Option<usize>>,
    row_height: f32,
    max_height: Option<f32>,
    striped: bool,
}

impl Default for Table<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Table<'a> {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            sort: None,
            selected: None,
            row_height: row::TABLE,
            max_height: None,
            striped: false,
        }
    }

    pub fn columns<I>(mut self, columns: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<TableColumn<'a>>,
    {
        self.columns = columns.into_iter().map(Into::into).collect();
        self
    }

    /// Make the sortable headers live. The `bool` is "ascending".
    pub fn sort(mut self, sort: &'a mut Sort) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Make rows selectable.
    pub fn selection(mut self, selected: &'a mut Option<usize>) -> Self {
        self.selected = Some(selected);
        self
    }

    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }

    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Tint alternate rows with `Theme::row_alt`.
    ///
    /// Off by default, and the one place this file departs from Azur: its `Table`
    /// separates rows with a `stroke-subtle` rule and no stripe. A wide desktop
    /// table often wants both.
    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    /// Resolve column widths against the space available.
    fn widths(&self, total: f32) -> Vec<f32> {
        let fixed: f32 = self.columns.iter().filter_map(|c| c.width).sum();
        let flexible = self.columns.iter().filter(|c| c.width.is_none()).count();
        let share = if flexible > 0 {
            ((total - fixed) / flexible as f32).max(48.0)
        } else {
            0.0
        };
        self.columns
            .iter()
            .map(|c| c.width.unwrap_or(share))
            .collect()
    }

    pub fn show(
        mut self,
        ui: &mut Ui,
        rows: usize,
        mut cell: impl FnMut(&mut Ui, usize, usize),
    ) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let id = ui.next_auto_id();
        let columns = std::mem::take(&mut self.columns);
        let widths = {
            let mut probe = Table::new();
            probe.columns = columns;
            let w = probe.widths(ui.available_width() - 2.0);
            self.columns = probe.columns;
            w
        };
        let mut changed = false;

        let outer = container_frame(&t).show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

            // ---- Header: `background-layer-alt`, ruled with `stroke-default` --
            let (head, _) = ui.allocate_exact_size(
                vec2(
                    ui.available_width(),
                    typography::LINE_BODY + space::S3 * 2.0,
                ),
                Sense::hover(),
            );
            ui.painter()
                .rect_filled(head, CornerRadius::ZERO, t.bg.layer_alt);
            let mut x = head.left();
            for (i, column) in self.columns.iter().enumerate() {
                let cell_rect =
                    Rect::from_min_size(pos2(x, head.top()), vec2(widths[i], head.height()));
                x += widths[i];
                let sortable = column.sortable && self.sort.is_some();
                let response = ui.interact(
                    cell_rect,
                    id.with(("azur-th", i)),
                    if sortable {
                        Sense::click()
                    } else {
                        Sense::hover()
                    },
                );
                if sortable && response.hovered() {
                    ui.painter()
                        .rect_filled(cell_rect, CornerRadius::ZERO, t.bg.control_hover);
                }
                let active_sort = self.sort.as_ref().and_then(|s| **s);
                let sorted_here = active_sort.filter(|(c, _)| *c == i);
                let color = if sortable && response.hovered() {
                    t.text.primary
                } else {
                    t.text.secondary
                };

                let arrow = if sorted_here.is_some() {
                    12.0 + space::S2
                } else {
                    0.0
                };
                let inner = cell_rect.shrink2(vec2(space::S4, 0.0));
                let galley = truncated(
                    ui,
                    &column.header,
                    t.fonts.body_strong.clone(),
                    color,
                    (inner.width() - arrow).max(0.0),
                );
                let text_x = if column.numeric {
                    inner.right() - galley.size().x - arrow
                } else {
                    inner.left()
                };
                ui.painter().galley(
                    pos2(text_x, inner.center().y - galley.size().y * 0.5),
                    galley.clone(),
                    Color32::PLACEHOLDER,
                );
                if let Some((_, ascending)) = sorted_here {
                    let r = super::icon_rect(cell_rect, text_x + galley.size().x + space::S2, 12.0);
                    if ascending {
                        sort_asc(ui.painter(), r, color);
                    } else {
                        sort_desc(ui.painter(), r, color);
                    }
                }
                if response.clicked() {
                    if let Some(sort) = self.sort.as_deref_mut() {
                        *sort = match *sort {
                            Some((c, asc)) if c == i => Some((i, !asc)),
                            _ => Some((i, true)),
                        };
                        changed = true;
                    }
                }
                if response.has_focus() {
                    focus_ring_inset(ui.painter(), cell_rect, CornerRadius::ZERO, t.stroke.focus);
                }
            }
            ui.painter().rect_filled(
                Rect::from_min_size(head.left_bottom() - vec2(0.0, 1.0), vec2(head.width(), 1.0)),
                CornerRadius::ZERO,
                t.stroke.default,
            );

            // ---- Body ------------------------------------------------------
            let mut scroll = egui::ScrollArea::vertical()
                .id_salt(id)
                .auto_shrink([false, true]);
            if let Some(h) = self.max_height {
                scroll = scroll.max_height(h);
            }
            let out = scroll.show_rows(ui, self.row_height, rows, |ui, range| {
                for r in range {
                    let selectable = self.selected.is_some() && enabled;
                    let (rect, response) = ui.allocate_exact_size(
                        vec2(ui.available_width(), self.row_height),
                        if selectable {
                            Sense::click()
                        } else {
                            Sense::hover()
                        },
                    );
                    let selected = self
                        .selected
                        .as_ref()
                        .map(|s| **s == Some(r))
                        .unwrap_or(false);
                    if response.clicked() {
                        if let Some(sel) = self.selected.as_deref_mut() {
                            *sel = Some(r);
                            changed = true;
                        }
                    }
                    if !ui.is_rect_visible(rect) {
                        continue;
                    }

                    if self.striped && r % 2 == 1 && !selected {
                        ui.painter()
                            .rect_filled(rect, CornerRadius::ZERO, t.row_alt);
                    }
                    if let Some(fill) = row_fill(&t, selected, response.hovered()) {
                        ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);
                    }

                    let mut x = rect.left();
                    for (c, column) in self.columns.iter().enumerate() {
                        // `.td { padding: space-3 space-4 }`.
                        let cell_rect = Rect::from_min_size(
                            pos2(x, rect.top()),
                            vec2(widths[c], rect.height()),
                        )
                        .shrink2(vec2(space::S4, space::S3));
                        x += widths[c];
                        let layout = if column.numeric {
                            Layout::right_to_left(Align::Center)
                        } else {
                            Layout::left_to_right(Align::Center)
                        };
                        let mut child =
                            ui.new_child(UiBuilder::new().max_rect(cell_rect).layout(layout));
                        child.set_clip_rect(cell_rect.intersect(ui.clip_rect()));
                        cell(&mut child, r, c);
                    }

                    // `.tr:not(:last-child) .td { border-bottom: 1px stroke-subtle }`.
                    if r + 1 < rows {
                        ui.painter().rect_filled(
                            Rect::from_min_size(
                                rect.left_bottom() - vec2(0.0, 1.0),
                                vec2(rect.width(), 1.0),
                            ),
                            CornerRadius::ZERO,
                            t.stroke.subtle,
                        );
                    }
                }
            });
            // The body only. The header stays where it is and is not scrolled past, so
            // fading the rows *under* it would say there is more above when the thing
            // above is a fixed header.
            super::scroll_fades_of(ui.painter(), &out, t.bg.layer);
        });

        let mut response = outer.response;
        if changed {
            response.mark_changed();
        }
        response
    }
}

/// Azur's `Tabs`: labels over a rule, with an accent indicator under the
/// selected one.
///
/// ```no_run
/// # fn demo(ui: &mut egui::Ui) {
/// use azur_egui_theme::components::Tabs;
/// let mut tab = 0;
/// ui.add(Tabs::new(&mut tab).tabs(["Geometry", "Appearance", "Advanced"]));
/// # }
/// ```
pub struct Tabs<'a> {
    selected: &'a mut usize,
    tabs: Vec<(Cow<'a, str>, Option<Icon<'a>>, bool)>,
}

impl<'a> Tabs<'a> {
    pub fn new(selected: &'a mut usize) -> Self {
        Self {
            selected,
            tabs: Vec::new(),
        }
    }

    pub fn tabs<I, S>(mut self, tabs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Cow<'a, str>>,
    {
        self.tabs = tabs.into_iter().map(|s| (s.into(), None, false)).collect();
        self
    }

    /// Add one tab, with an icon and/or disabled.
    pub fn tab(
        mut self,
        label: impl Into<Cow<'a, str>>,
        icon: Option<Icon<'a>>,
        disabled: bool,
    ) -> Self {
        self.tabs.push((label.into(), icon, disabled));
        self
    }
}

/// The height a [`Tabs`] strip takes, its rule included.
///
/// Here for the reason [`menu_item_height`](crate::components::menu_item_height) is:
/// a caller that puts something *beside* a tab strip -- a close button in the corner
/// of a panel whose tabs are its title bar -- has to know how tall the strip is
/// before the strip is drawn, and one that reserves less than this gets its own
/// content painted over by the strip's rule and indicator.
///
/// `.tab { padding: space-3 space-4 }` around a body line, plus
/// `.tablist { border-bottom: 1px }`.
pub fn tabs_height() -> f32 {
    typography::LINE_BODY + space::S3 * 2.0 + 1.0
}

/// How wide one tab is: its label, its padding, and its icon when it has one.
fn tab_width(painter: &egui::Painter, font: &egui::FontId, label: &str, icon: bool) -> f32 {
    let galley = painter.layout_no_wrap(label.to_owned(), font.clone(), Color32::PLACEHOLDER);
    // `.tab { padding: space-3 space-4 }`, and `gap: space-2` before the label.
    galley.size().x + if icon { 16.0 + space::S2 } else { 0.0 } + space::S4 * 2.0
}

/// How wide a strip of these labels wants to be, its gaps included.
///
/// The width counterpart of [`tabs_height`], and the same idea as
/// [`MenuItemMetrics::width`](crate::components::MenuItemMetrics::width): a caller
/// that puts anything *beside* a tab strip has to know where the tabs stop, and
/// [`Tabs`] itself claims the whole width it is given -- for its rule, which spans
/// the strip. Measured with the same function the component lays its tabs out with,
/// so the two cannot drift.
///
/// Pass `icons` as the number of leading tabs that carry an icon; every tab with one
/// is `16 + space-2` wider.
pub fn tabs_width<'a>(
    painter: &egui::Painter,
    t: &Theme,
    labels: impl IntoIterator<Item = &'a str>,
    icons: usize,
) -> f32 {
    let mut total = 0.0;
    let mut n = 0usize;
    for label in labels {
        total += tab_width(painter, &t.fonts.body, label, n < icons);
        n += 1;
    }
    // `.tablist { gap: space-2 }`, between them and not after the last.
    total + space::S2 * n.saturating_sub(1) as f32
}

/// The band a tab strip's *labels* occupy: everything above the rule.
///
/// What a caller aligns to. See [`tabs_height`].
pub fn tabs_label_band(strip: Rect) -> Rect {
    Rect::from_min_size(
        strip.min,
        vec2(strip.width(), (tabs_height() - 1.0).min(strip.height())),
    )
}

impl Widget for Tabs<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme_of(ui);
        let enabled = ui.is_enabled();
        let height = tabs_height() - 1.0;
        let width = ui.available_width();
        let (strip, mut response) =
            ui.allocate_exact_size(vec2(width, tabs_height()), Sense::hover());
        if !ui.is_rect_visible(strip) {
            return response;
        }

        // `.tablist { border-bottom: 1px stroke-subtle }`.
        ui.painter().rect_filled(
            Rect::from_min_size(pos2(strip.left(), strip.bottom() - 1.0), vec2(width, 1.0)),
            CornerRadius::ZERO,
            t.stroke.subtle,
        );

        // One baseline for the whole strip, measured once from the labels' own font.
        // The ink baseline, not the line box's: a tab can carry an icon, and an icon
        // centred beside a line box centred is the 1.5px the eye reads as the strip
        // being slightly out of true. It is also what anything a caller puts beside
        // the strip -- see `tabs_label_band` -- has to line up with.
        let baseline = super::ink_baseline(ui.painter(), &t.fonts.body, strip.top(), height);

        let mut x = strip.left();
        for (i, (label, icon, disabled)) in self.tabs.iter().enumerate() {
            let tab_enabled = enabled && !disabled;
            let w = tab_width(ui.painter(), &t.fonts.body, label, icon.is_some());
            let rect = Rect::from_min_size(pos2(x, strip.top()), vec2(w, height));
            x += w + space::S2; // `.tablist { gap: space-2 }`

            let tab_response = ui.interact(
                rect,
                response.id.with(("azur-tab", i)),
                if tab_enabled {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            );
            let selected = *self.selected == i;
            if tab_response.clicked() && !selected {
                *self.selected = i;
                response.mark_changed();
            }

            if tab_enabled && tab_response.hovered() && !selected {
                // Only the top corners round, as Azur's do.
                ui.painter().rect_filled(
                    rect,
                    CornerRadius {
                        nw: radius::SMALL,
                        ne: radius::SMALL,
                        sw: 0,
                        se: 0,
                    },
                    t.bg.control_hover,
                );
            }

            let color = if !tab_enabled {
                t.text.disabled
            } else if selected || tab_response.hovered() {
                t.text.primary
            } else {
                t.text.secondary
            };
            let mut cx = rect.left() + space::S4;
            if let Some(icon) = icon {
                icon(ui.painter(), super::icon_rect(rect, cx, 16.0), color);
                cx += 16.0 + space::S2;
            }
            let galley =
                ui.painter()
                    .layout_no_wrap(label.to_string(), t.fonts.body.clone(), color);
            super::galley_on_baseline(ui.painter(), cx, baseline, galley);

            if selected {
                // `.indicator { height: 2px; bottom: -1px }`.
                ui.painter().rect_filled(
                    Rect::from_min_size(
                        pos2(rect.left(), strip.bottom() - 2.0),
                        vec2(rect.width(), 2.0),
                    ),
                    CornerRadius::same(radius::SMALL),
                    t.accent.mark,
                );
            }
            if tab_response.has_focus() {
                focus_ring_inset(ui.painter(), rect, CornerRadius::ZERO, t.stroke.focus);
            }
        }
        response
    }
}

/// A titled group of rows, which is how Azur's gallery frames a `ListView`.
///
/// Not a component in its own right — a convenience so a panel of collections
/// keeps one heading style.
pub fn collection_label(ui: &mut Ui, text: &str) -> InnerResponse<()> {
    let t = theme_of(ui);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(text)
                .font(t.fonts.body.clone())
                .color(t.text.secondary),
        );
    })
}

/// Paint a `stroke-subtle` outline around `rect` at `radius-medium` — the
/// collection container, for a scroll area you have built yourself.
pub fn collection_outline(ui: &Ui, rect: Rect) {
    let t = theme_of(ui);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(radius::MEDIUM),
        Stroke::new(1.0, t.stroke.subtle),
        StrokeKind::Inside,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What a caller reserves for a tab strip has to be what the strip takes.
    ///
    /// The bug this pins: a panel whose title bar is its tabs reserved a dense row
    /// -- 24 points, the height of every other strip in that window -- and the
    /// component drew 37, so its rule and its selected-tab indicator landed on the
    /// table header underneath. Nothing in either file looked wrong.
    #[test]
    fn a_strip_takes_exactly_the_height_it_publishes() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        let mut selected = 0usize;
        let mut got = None;
        let _ = ctx.run_ui(Default::default(), |ui| {
            got = Some(
                ui.add(Tabs::new(&mut selected).tabs(["One", "Two", "Three"]))
                    .rect
                    .height(),
            );
        });
        assert_eq!(got, Some(tabs_height()));
    }

    /// And the band a caller aligns to is the strip without its rule -- the line the
    /// labels are on, which is what anything beside them is centred in.
    #[test]
    fn the_label_band_is_the_strip_above_its_rule() {
        let strip = Rect::from_min_size(egui::pos2(10.0, 20.0), vec2(400.0, tabs_height()));
        let band = tabs_label_band(strip);
        assert_eq!(band.min, strip.min);
        assert_eq!(band.width(), strip.width());
        assert_eq!(band.height(), tabs_height() - 1.0);
        // Never past the strip it came from, however the strip was sized.
        let squeezed = tabs_label_band(Rect::from_min_size(strip.min, vec2(400.0, 10.0)));
        assert!(squeezed.height() <= 10.0);
    }
}
