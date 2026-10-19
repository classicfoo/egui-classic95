use std::collections::HashSet;

use egui::{
    Align, CornerRadius, FontId, Id, Margin, Rect, Response, ScrollArea, Sense, Stroke, StrokeKind,
    TextStyle, Ui, pos2, vec2,
};

use crate::theme::palette_for;
use crate::{BevelFrame, ClassicScrollBar, Palette};

/// A node in a [`ClassicTreeView`].
#[derive(Clone, Debug)]
pub struct ClassicTreeNode {
    label: String,
    children: Vec<Self>,
    initially_expanded: bool,
}

impl ClassicTreeNode {
    /// Create a leaf or parent node with an application-provided label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
            initially_expanded: false,
        }
    }

    /// Set this node's children.
    pub fn children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        self.children = children.into_iter().collect();
        self
    }

    /// Add one child node.
    pub fn child(mut self, child: Self) -> Self {
        self.children.push(child);
        self
    }

    /// Return this node's label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Return this node's child nodes.
    pub fn child_nodes(&self) -> &[Self] {
        &self.children
    }

    /// Show this node expanded the first time its tree is displayed.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.initially_expanded = expanded;
        self
    }
}

/// An indented, selectable tree with classic plus/minus expanders and a reusable scrollbar when
/// its rows overflow.
#[derive(Debug)]
pub struct ClassicTreeView<'a> {
    nodes: Vec<ClassicTreeNode>,
    selected: &'a mut Option<Vec<usize>>,
    palette: Option<Palette>,
    id_salt: Id,
    max_height: Option<f32>,
    row_height: f32,
}

/// Interaction results from showing a [`ClassicTreeView`].
pub struct ClassicTreeViewResponse {
    /// Selected node path, represented by its child indexes from the root.
    pub selected: Option<Vec<usize>>,
    /// Whether the selected node changed.
    pub changed: bool,
    /// Path clicked during this frame, including the already-selected node.
    pub clicked: Option<Vec<usize>>,
    /// Whether a node was expanded or collapsed during this frame.
    pub expanded_changed: bool,
    /// Responses for currently visible rows in display order.
    pub rows: Vec<Response>,
    /// Response for the recessed tree view frame.
    pub response: Response,
}

impl<'a> ClassicTreeView<'a> {
    /// Create a tree view from root nodes and the selected node path.
    pub fn new<I>(nodes: I, selected: &'a mut Option<Vec<usize>>) -> Self
    where
        I: IntoIterator<Item = ClassicTreeNode>,
    {
        Self {
            nodes: nodes.into_iter().collect(),
            selected,
            palette: None,
            id_salt: Id::NULL.with("classic95-tree-view"),
            max_height: None,
            row_height: 18.0,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set a stable identifier when multiple tree views share the same UI.
    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_salt = Id::NULL.with(salt);
        self
    }

    /// Limit the visible tree height and scroll when expanded nodes exceed it.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height.max(0.0));
        self
    }

    /// Set the height of each tree row in egui points.
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height.max(1.0);
        self
    }

    /// Show the tree and update the selected path and expand/collapse state.
    pub fn show(self, ui: &mut Ui) -> ClassicTreeViewResponse {
        let Self {
            nodes,
            selected,
            palette,
            id_salt,
            max_height,
            row_height,
        } = self;
        let palette = palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let tree_id = ui.scope_id().with(id_salt);
        let focus_id = tree_id.with("focus");
        let mut initial_expanded = HashSet::new();
        collect_initially_expanded(&nodes, &tree_id, &mut Vec::new(), &mut initial_expanded);
        let mut expanded = ui
            .ctx()
            .data(|data| data.get_temp::<HashSet<Id>>(tree_id))
            .unwrap_or(initial_expanded);
        let visible = visible_nodes(&nodes, &tree_id, &expanded);
        let natural_height = visible.len().max(3) as f32 * row_height;
        let visible_height = max_height
            .map(|height| height.min(natural_height))
            .unwrap_or(natural_height);
        let available_width = ui.available_width().max(1.0);
        let has_vertical_overflow = visible.len() as f32 * row_height > visible_height;
        let scrollbar_width = if has_vertical_overflow {
            ClassicScrollBar::WIDTH
        } else {
            0.0
        };
        let content_width = (available_width - scrollbar_width).max(1.0);
        let scroll_position_id = ui.scope_id().with((id_salt, "classic-scroll-position"));
        let (mut scroll_position, mut pending_scroll_position) = ui.ctx().data(|data| {
            data.get_temp::<(f32, Option<f32>)>(scroll_position_id)
                .unwrap_or_default()
        });
        let scrollbar_id = ui.scope_id().with((id_salt, "classic-scrollbar"));
        let mut changed = false;
        if let Some(path) = selected.as_ref() {
            if !path_exists(&nodes, path) {
                *selected = None;
                changed = true;
            } else if !visible.iter().any(|node| node.path == *path) {
                *selected = (1..path.len()).rev().find_map(|prefix_len| {
                    visible
                        .iter()
                        .find(|node| node.path == path[..prefix_len])
                        .map(|node| node.path.clone())
                });
                changed = true;
            }
        }

        let mut clicked = None;
        let mut expanded_changed = false;
        let mut row_responses = Vec::with_capacity(visible.len());
        let mut focus_response = None;
        let font_id = ui
            .style()
            .text_styles
            .get(&TextStyle::Body)
            .cloned()
            .unwrap_or_else(|| FontId::proportional(13.0));
        let frame = BevelFrame::sunken()
            .palette(palette)
            .fill(palette.input)
            .inner_margin(Margin::same(1))
            .show(ui, |ui| {
                focus_response = Some(ui.interact(
                    ui.available_rect_before_wrap(),
                    focus_id,
                    Sense::focusable_noninteractive(),
                ));
                ui.set_min_width(available_width);
                ui.allocate_ui_with_layout(
                    vec2(available_width, visible_height),
                    egui::Layout::left_to_right(Align::Min),
                    |ui| {
                        ui.allocate_ui_with_layout(
                            vec2(content_width, visible_height),
                            egui::Layout::top_down(Align::Min),
                            |ui| {
                                let mut scroll_area = ScrollArea::vertical()
                                    .id_salt(id_salt.with("rows"))
                                    .max_height(visible_height)
                                    .auto_shrink([false, false])
                                    .scroll_bar_visibility(
                                        egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                                    );
                                if let Some(offset) = pending_scroll_position.take() {
                                    scroll_area = scroll_area.vertical_scroll_offset(offset);
                                }
                                let output = scroll_area.show(ui, |ui| {
                                    ui.set_min_width(content_width);
                                    for item in &visible {
                                        let (_, rect) =
                                            ui.allocate_space(vec2(content_width, row_height));
                                        let row_id = tree_id.with(("node", item.path.clone()));
                                        let row_response =
                                            ui.interact(rect, row_id, Sense::click());
                                        if row_response.clicked() {
                                            clicked = Some(item.path.clone());
                                            if selected.as_ref() != Some(&item.path) {
                                                *selected = Some(item.path.clone());
                                                changed = true;
                                            }
                                            ui.ctx().memory_mut(|memory| {
                                                memory.request_focus(focus_id);
                                            });
                                        }

                                        if row_response.double_clicked()
                                            && !item.node.children.is_empty()
                                        {
                                            toggle_expansion(&mut expanded, item.id);
                                            expanded_changed = true;
                                        }

                                        let depth = item.path.len().saturating_sub(1);
                                        let slot_left =
                                            rect.left() + 4.0 + depth as f32 * TREE_INDENT;
                                        let expander_center =
                                            pos2(slot_left + 4.5, rect.center().y);
                                        let label_x = slot_left + 12.0;
                                        paint_connectors(
                                            ui,
                                            rect,
                                            depth,
                                            item.ancestors_continue.as_slice(),
                                            item.is_first,
                                            !item.node.children.is_empty(),
                                            expander_center,
                                            label_x,
                                            palette,
                                        );

                                        if !item.node.children.is_empty() {
                                            let expanded_here = expanded.contains(&item.id);
                                            let expander_rect = Rect::from_center_size(
                                                expander_center,
                                                vec2(9.0, 9.0),
                                            );
                                            ui.painter().rect_filled(
                                                expander_rect,
                                                CornerRadius::ZERO,
                                                palette.input,
                                            );
                                            ui.painter().rect_stroke(
                                                expander_rect,
                                                CornerRadius::ZERO,
                                                Stroke::new(1.0, palette.text),
                                                StrokeKind::Inside,
                                            );
                                            ui.painter().line_segment(
                                                [
                                                    pos2(
                                                        expander_rect.left() + 2.0,
                                                        expander_rect.center().y,
                                                    ),
                                                    pos2(
                                                        expander_rect.right() - 2.0,
                                                        expander_rect.center().y,
                                                    ),
                                                ],
                                                Stroke::new(1.0, palette.text),
                                            );
                                            if !expanded_here {
                                                ui.painter().line_segment(
                                                    [
                                                        pos2(
                                                            expander_rect.center().x,
                                                            expander_rect.top() + 2.0,
                                                        ),
                                                        pos2(
                                                            expander_rect.center().x,
                                                            expander_rect.bottom() - 2.0,
                                                        ),
                                                    ],
                                                    Stroke::new(1.0, palette.text),
                                                );
                                            }
                                            let expander_response = ui
                                                .interact(
                                                    expander_rect,
                                                    row_id.with("expander"),
                                                    Sense::click(),
                                                )
                                                .on_hover_cursor(egui::CursorIcon::PointingHand);
                                            if expander_response.clicked() {
                                                clicked = Some(item.path.clone());
                                                if selected.as_ref() != Some(&item.path) {
                                                    *selected = Some(item.path.clone());
                                                    changed = true;
                                                }
                                                ui.ctx().memory_mut(|memory| {
                                                    memory.request_focus(focus_id);
                                                });
                                                let was_expanded = expanded.contains(&item.id);
                                                toggle_expansion(&mut expanded, item.id);
                                                if was_expanded
                                                    && selected.as_ref().is_some_and(|path| {
                                                        path.len() > item.path.len()
                                                            && path.starts_with(&item.path)
                                                    })
                                                {
                                                    *selected = Some(item.path.clone());
                                                    changed = true;
                                                }
                                                expanded_changed = true;
                                            }
                                        }

                                        let is_selected = selected.as_ref() == Some(&item.path);
                                        let galley = ui.painter().layout_no_wrap(
                                            item.node.label.clone(),
                                            font_id.clone(),
                                            if is_selected {
                                                palette.selection_text
                                            } else {
                                                palette.text
                                            },
                                        );
                                        if is_selected {
                                            let selection_rect = Rect::from_min_size(
                                                pos2(label_x - 1.0, rect.top()),
                                                vec2(galley.size().x + 3.0, rect.height()),
                                            );
                                            ui.painter().rect_filled(
                                                selection_rect,
                                                CornerRadius::ZERO,
                                                palette.selection,
                                            );
                                        }
                                        ui.painter().galley(
                                            pos2(label_x, rect.center().y - galley.size().y * 0.5),
                                            galley,
                                            if is_selected {
                                                palette.selection_text
                                            } else {
                                                palette.text
                                            },
                                        );
                                        row_response.widget_info(|| {
                                            egui::WidgetInfo::selected(
                                                egui::Role::TreeItem,
                                                true,
                                                is_selected,
                                                &item.node.label,
                                            )
                                        });
                                        row_responses.push(row_response);
                                    }
                                });
                                scroll_position = output.state.clamped_offset().y;
                            },
                        );
                        if has_vertical_overflow {
                            let old_position = scroll_position;
                            let _ = ClassicScrollBar::new(
                                &mut scroll_position,
                                (visible.len() as f32 * row_height - visible_height).max(0.0),
                                visible_height,
                                visible_height,
                            )
                            .palette(palette)
                            .scroll_step(row_height)
                            .id_salt(scrollbar_id)
                            .show(ui);
                            if scroll_position != old_position {
                                pending_scroll_position = Some(scroll_position);
                            }
                        }
                    },
                );
            });
        ui.ctx().data_mut(|data| {
            data.insert_temp(
                scroll_position_id,
                (scroll_position, pending_scroll_position),
            );
        });
        let focus = focus_response.expect("tree view focus response was not created");
        let selected_index = selected
            .as_ref()
            .and_then(|path| visible.iter().position(|node| node.path == *path));
        if focus.has_focus() && !visible.is_empty() {
            let action = ui.input(|input| {
                if input.key_pressed(egui::Key::ArrowDown) {
                    Some(TreeKey::Move(
                        selected_index.map_or(0, |index| (index + 1).min(visible.len() - 1)),
                    ))
                } else if input.key_pressed(egui::Key::ArrowUp) {
                    Some(TreeKey::Move(
                        selected_index.map_or(0, |index| index.saturating_sub(1)),
                    ))
                } else if input.key_pressed(egui::Key::Home) {
                    Some(TreeKey::Move(0))
                } else if input.key_pressed(egui::Key::End) {
                    Some(TreeKey::Move(visible.len() - 1))
                } else if input.key_pressed(egui::Key::ArrowRight) {
                    Some(TreeKey::Right)
                } else if input.key_pressed(egui::Key::ArrowLeft) {
                    Some(TreeKey::Left)
                } else {
                    None
                }
            });
            if let Some(action) = action {
                match action {
                    TreeKey::Move(target) => {
                        let target = target.min(visible.len() - 1);
                        let target_path = visible[target].path.clone();
                        if selected.as_ref() != Some(&target_path) {
                            *selected = Some(target_path);
                            changed = true;
                        }
                        if let Some(response) = row_responses.get(target) {
                            response.scroll_to_me(Some(Align::Center));
                        }
                    }
                    TreeKey::Right => {
                        if let Some(index) = selected_index {
                            let item = &visible[index];
                            if !item.node.children.is_empty() {
                                if expanded.insert(item.id) {
                                    expanded_changed = true;
                                } else if let Some(child) = visible.get(index + 1).filter(|child| {
                                    child.path.len() == item.path.len() + 1
                                        && child.path.starts_with(&item.path)
                                }) {
                                    if selected.as_ref() != Some(&child.path) {
                                        *selected = Some(child.path.clone());
                                        changed = true;
                                    }
                                    if let Some(response) = row_responses.get(index + 1) {
                                        response.scroll_to_me(Some(Align::Center));
                                    }
                                }
                            }
                        }
                    }
                    TreeKey::Left => {
                        if let Some(index) = selected_index {
                            let item = &visible[index];
                            if expanded.remove(&item.id) {
                                expanded_changed = true;
                            } else if !item.path.is_empty() {
                                let parent_path = item.path[..item.path.len() - 1].to_vec();
                                if selected.as_ref() != Some(&parent_path) {
                                    *selected = Some(parent_path);
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        ui.ctx()
            .data_mut(|data| data.insert_temp(tree_id, expanded));

        ClassicTreeViewResponse {
            selected: selected.clone(),
            changed,
            clicked,
            expanded_changed,
            rows: row_responses,
            response: frame.response,
        }
    }
}

#[derive(Clone, Copy)]
enum TreeKey {
    Move(usize),
    Right,
    Left,
}

struct VisibleNode<'a> {
    node: &'a ClassicTreeNode,
    id: Id,
    path: Vec<usize>,
    ancestors_continue: Vec<bool>,
    is_first: bool,
}

const TREE_INDENT: f32 = 16.0;

fn collect_initially_expanded(
    nodes: &[ClassicTreeNode],
    tree_id: &Id,
    path: &mut Vec<usize>,
    expanded: &mut HashSet<Id>,
) {
    for (index, node) in nodes.iter().enumerate() {
        path.push(index);
        let id = tree_id.with(("node", path.clone()));
        if node.initially_expanded {
            expanded.insert(id);
        }
        collect_initially_expanded(&node.children, tree_id, path, expanded);
        path.pop();
    }
}

fn visible_nodes<'a>(
    nodes: &'a [ClassicTreeNode],
    tree_id: &Id,
    expanded: &HashSet<Id>,
) -> Vec<VisibleNode<'a>> {
    fn append<'a>(
        nodes: &'a [ClassicTreeNode],
        tree_id: &Id,
        expanded: &HashSet<Id>,
        path: &mut Vec<usize>,
        result: &mut Vec<VisibleNode<'a>>,
    ) {
        for (index, node) in nodes.iter().enumerate() {
            path.push(index);
            let id = tree_id.with(("node", path.clone()));
            result.push(VisibleNode {
                node,
                id,
                path: path.clone(),
                ancestors_continue: Vec::new(),
                is_first: index == 0,
            });
            if !node.children.is_empty() && expanded.contains(&id) {
                append(&node.children, tree_id, expanded, path, result);
            }
            path.pop();
        }
    }

    let mut result = Vec::new();
    append(nodes, tree_id, expanded, &mut Vec::new(), &mut result);
    for index in 0..result.len() {
        let path = result[index].path.clone();
        result[index].ancestors_continue = (1..path.len())
            .map(|prefix_len| {
                result[index + 1..]
                    .iter()
                    .any(|future| future.path.starts_with(&path[..prefix_len]))
            })
            .collect();
    }
    result
}

fn toggle_expansion(expanded: &mut HashSet<Id>, id: Id) {
    if !expanded.remove(&id) {
        expanded.insert(id);
    }
}

fn path_exists(nodes: &[ClassicTreeNode], path: &[usize]) -> bool {
    if path.is_empty() {
        return false;
    }
    let mut siblings = nodes;
    for (depth, index) in path.iter().enumerate() {
        let Some(node) = siblings.get(*index) else {
            return false;
        };
        if depth + 1 < path.len() {
            siblings = &node.children;
        }
    }
    true
}

fn paint_connectors(
    ui: &Ui,
    row_rect: Rect,
    depth: usize,
    ancestors_continue: &[bool],
    is_first: bool,
    has_children: bool,
    current_center: egui::Pos2,
    label_x: f32,
    palette: Palette,
) {
    let painter = ui.painter();
    for (level, continues) in ancestors_continue.iter().enumerate() {
        let x = row_rect.left() + 4.0 + level as f32 * TREE_INDENT + 4.5;
        let top = if level + 1 == depth && is_first {
            row_rect.top() - row_rect.height() * 0.5
        } else {
            row_rect.top()
        };
        dotted_vertical(painter, x, top, row_rect.center().y, palette.text);
        if *continues {
            dotted_vertical(
                painter,
                x,
                row_rect.center().y,
                row_rect.bottom(),
                palette.text,
            );
        }
    }
    if depth == 0 {
        return;
    }
    let parent_x = row_rect.left() + 4.0 + (depth - 1) as f32 * TREE_INDENT + 4.5;
    let end_x = if has_children {
        current_center.x - 4.5
    } else {
        label_x - 2.0
    };
    dotted_horizontal(painter, parent_x, end_x, row_rect.center().y, palette.text);
}

fn dotted_vertical(painter: &egui::Painter, x: f32, top: f32, bottom: f32, color: egui::Color32) {
    let mut y = top.ceil();
    while y < bottom {
        painter.rect_filled(
            Rect::from_min_size(pos2(x.round(), y), vec2(1.25, 1.25)),
            CornerRadius::ZERO,
            color,
        );
        y += 2.0;
    }
}

fn dotted_horizontal(painter: &egui::Painter, left: f32, right: f32, y: f32, color: egui::Color32) {
    let y = y.round();
    let painter = painter.with_clip_rect(Rect::from_min_max(
        pos2(left, y - 1.0),
        pos2(right, y + 2.0),
    ));
    let mut x = left.ceil();
    while x < right {
        painter.rect_filled(
            Rect::from_min_size(pos2(x, y), vec2(1.25, 1.25)),
            CornerRadius::ZERO,
            color,
        );
        x += 2.0;
    }
}
