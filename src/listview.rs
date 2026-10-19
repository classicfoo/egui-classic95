use egui::{
    Align, Align2, CornerRadius, FontId, Id, Layout, Margin, Rect, Response, ScrollArea, Sense,
    TextStyle, Ui, pos2, vec2,
};

use crate::theme::palette_for;
use crate::{Bevel, BevelFrame, ClassicScrollBar, Palette, paint_bevel};

/// A report-view column in a [`ClassicListView`].
#[derive(Clone, Debug)]
pub struct ClassicListViewColumn {
    title: String,
    width: Option<f32>,
}

impl ClassicListViewColumn {
    /// Create a column whose width shares the remaining view width equally.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: None,
        }
    }

    /// Give this column a preferred width in egui points.
    ///
    /// Fixed widths are reduced proportionally if they exceed the view width.
    /// Any remaining width is shared among columns without a fixed width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(0.0));
        self
    }
}

/// A recessed, single-selection report view with classic column headers and a reusable scrollbar
/// when its rows overflow.
#[derive(Debug)]
pub struct ClassicListView<'a> {
    columns: Vec<ClassicListViewColumn>,
    rows: Vec<Vec<String>>,
    selected: &'a mut Option<usize>,
    palette: Option<Palette>,
    id_salt: Id,
    max_height: Option<f32>,
    row_height: f32,
}

/// Interaction results from showing a [`ClassicListView`].
pub struct ClassicListViewResponse {
    /// Selected source row after handling input.
    pub selected: Option<usize>,
    /// Whether the selection changed.
    pub changed: bool,
    /// Source row clicked during this frame, including the already-selected row.
    pub clicked: Option<usize>,
    /// Responses for the header cells in column order.
    pub headers: Vec<Response>,
    /// Responses for the visible report rows in source order.
    pub rows: Vec<Response>,
    /// Response for the recessed list view frame.
    pub response: Response,
}

impl<'a> ClassicListView<'a> {
    /// Create a report view from column titles, row values, and a selected row.
    ///
    /// Row values can be any iterable of values convertible to `String`.
    pub fn new<C, R, Row, Cell>(columns: C, rows: R, selected: &'a mut Option<usize>) -> Self
    where
        C: IntoIterator<Item = ClassicListViewColumn>,
        R: IntoIterator<Item = Row>,
        Row: IntoIterator<Item = Cell>,
        Cell: Into<String>,
    {
        Self {
            columns: columns.into_iter().collect(),
            rows: rows
                .into_iter()
                .map(|row| row.into_iter().map(Into::into).collect())
                .collect(),
            selected,
            palette: None,
            id_salt: Id::NULL.with("classic95-list-view"),
            max_height: None,
            row_height: 20.0,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set a stable identifier when multiple list views share the same UI.
    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_salt = Id::NULL.with(salt);
        self
    }

    /// Limit the visible rows and scroll when the data exceeds the height.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height.max(0.0));
        self
    }

    /// Set the height of each report row in egui points.
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height.max(1.0);
        self
    }

    /// Show the list view and update the selected source row when clicked.
    pub fn show(self, ui: &mut Ui) -> ClassicListViewResponse {
        let Self {
            columns,
            rows,
            selected,
            palette,
            id_salt,
            max_height,
            row_height,
        } = self;
        let palette = palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let focus_id = ui.scope_id().with((id_salt, "focus"));
        let mut changed = false;
        if rows.is_empty() {
            if selected.take().is_some() {
                changed = true;
            }
        } else if selected.is_some_and(|index| index >= rows.len()) {
            *selected = Some(rows.len() - 1);
            changed = true;
        }

        let header_height = 22.0;
        let natural_rows_height = rows.len().max(3) as f32 * row_height;
        let visible_rows_height = max_height
            .map(|height| height.min(natural_rows_height))
            .unwrap_or(natural_rows_height);
        let content_height = header_height + visible_rows_height;
        let available_width = ui.available_width().max(1.0);
        let has_vertical_overflow = rows.len() as f32 * row_height > visible_rows_height;
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
        let column_widths = column_widths(&columns, content_width);
        let font_id = ui
            .style()
            .text_styles
            .get(&TextStyle::Body)
            .cloned()
            .unwrap_or_else(|| FontId::proportional(13.0));
        let mut clicked = None;
        let mut header_responses = Vec::with_capacity(columns.len());
        let mut row_responses = Vec::with_capacity(rows.len());
        let mut focus_response = None;
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
                    vec2(available_width, content_height),
                    Layout::left_to_right(Align::Min),
                    |ui| {
                        ui.allocate_ui_with_layout(
                            vec2(content_width, content_height),
                            Layout::top_down(Align::Min),
                            |ui| {
                                let (header_rect, _) = ui.allocate_exact_size(
                                    vec2(content_width, header_height),
                                    Sense::hover(),
                                );
                                let mut x = header_rect.left();
                                for (column_index, (column, width)) in
                                    columns.iter().zip(&column_widths).enumerate()
                                {
                                    let rect = Rect::from_min_size(
                                        pos2(x, header_rect.top()),
                                        vec2(*width, header_height),
                                    );
                                    ui.painter().rect_filled(
                                        rect,
                                        CornerRadius::ZERO,
                                        palette.face,
                                    );
                                    paint_bevel(ui.painter(), rect, Bevel::Raised, &palette);
                                    let text_rect = rect.shrink2(vec2(5.0, 1.0));
                                    ui.painter().with_clip_rect(text_rect).text(
                                        text_rect.left_center(),
                                        Align2::LEFT_CENTER,
                                        &column.title,
                                        font_id.clone(),
                                        palette.text,
                                    );
                                    let response = ui.interact(
                                        rect,
                                        ui.make_persistent_id((id_salt, "header", column_index)),
                                        Sense::hover(),
                                    );
                                    header_responses.push(response);
                                    x += width;
                                }

                                let mut scroll_area = ScrollArea::vertical()
                                    .id_salt(id_salt.with("rows"))
                                    .max_height(visible_rows_height)
                                    .auto_shrink([false, false])
                                    .scroll_bar_visibility(
                                        egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                                    );
                                if let Some(offset) = pending_scroll_position.take() {
                                    scroll_area = scroll_area.vertical_scroll_offset(offset);
                                }
                                let output = scroll_area.show(ui, |ui| {
                                    ui.set_min_width(content_width);
                                    for (row_index, values) in rows.iter().enumerate() {
                                        let width = ui.available_width().max(1.0);
                                        let (_, rect) = ui.allocate_space(vec2(width, row_height));
                                        let row_id =
                                            ui.make_persistent_id((id_salt, "row", row_index));
                                        let response = ui.interact(rect, row_id, Sense::click());
                                        if response.clicked() {
                                            clicked = Some(row_index);
                                            if *selected != Some(row_index) {
                                                *selected = Some(row_index);
                                                changed = true;
                                            }
                                            ui.ctx().memory_mut(|memory| {
                                                memory.request_focus(focus_id);
                                            });
                                        }

                                        if let Some(value) = values.first() {
                                            response.widget_info(|| {
                                                egui::WidgetInfo::selected(
                                                    egui::Role::ListBoxOption,
                                                    true,
                                                    *selected == Some(row_index),
                                                    value,
                                                )
                                            });
                                        }

                                        let is_selected = *selected == Some(row_index);
                                        let color = if is_selected {
                                            palette.selection_text
                                        } else {
                                            palette.text
                                        };
                                        if is_selected {
                                            ui.painter().rect_filled(
                                                rect,
                                                CornerRadius::ZERO,
                                                palette.selection,
                                            );
                                        }

                                        let mut x = rect.left();
                                        for (column_index, column_width) in
                                            column_widths.iter().enumerate()
                                        {
                                            let cell_rect = Rect::from_min_size(
                                                pos2(x, rect.top()),
                                                vec2(*column_width, rect.height()),
                                            );
                                            if let Some(value) = values.get(column_index) {
                                                ui.painter()
                                                    .with_clip_rect(
                                                        cell_rect.shrink2(vec2(5.0, 0.0)),
                                                    )
                                                    .text(
                                                        cell_rect.left_center() + vec2(5.0, 0.0),
                                                        Align2::LEFT_CENTER,
                                                        value,
                                                        font_id.clone(),
                                                        color,
                                                    );
                                            }
                                            x += column_width;
                                        }
                                        row_responses.push(response);
                                    }
                                });
                                scroll_position = output.state.clamped_offset().y;
                            },
                        );
                        if has_vertical_overflow {
                            let old_position = scroll_position;
                            let _ = ClassicScrollBar::new(
                                &mut scroll_position,
                                (rows.len() as f32 * row_height - visible_rows_height).max(0.0),
                                visible_rows_height,
                                content_height,
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

        let focus = focus_response.expect("list view focus response was not created");
        if focus.has_focus() && !rows.is_empty() {
            let target = ui.input(|input| {
                if input.key_pressed(egui::Key::ArrowDown) {
                    Some((*selected).map_or(0, |index| (index + 1).min(rows.len() - 1)))
                } else if input.key_pressed(egui::Key::ArrowUp) {
                    Some((*selected).map_or(0, |index| index.saturating_sub(1)))
                } else if input.key_pressed(egui::Key::Home) {
                    Some(0)
                } else if input.key_pressed(egui::Key::End) {
                    Some(rows.len() - 1)
                } else {
                    None
                }
            });
            if let Some(target) = target {
                if *selected != Some(target) {
                    *selected = Some(target);
                    changed = true;
                }
                if let Some(response) = row_responses.get(target) {
                    response.scroll_to_me(Some(Align::Center));
                }
            }
        }

        ClassicListViewResponse {
            selected: *selected,
            changed,
            clicked,
            headers: header_responses,
            rows: row_responses,
            response: frame.response,
        }
    }
}

fn column_widths(columns: &[ClassicListViewColumn], available_width: f32) -> Vec<f32> {
    if columns.is_empty() {
        return Vec::new();
    }

    let fixed_sum: f32 = columns.iter().filter_map(|column| column.width).sum();
    let flexible_count = columns
        .iter()
        .filter(|column| column.width.is_none())
        .count();
    let mut widths: Vec<f32> = columns
        .iter()
        .map(|column| column.width.unwrap_or(0.0))
        .collect();

    if fixed_sum > available_width {
        let scale = available_width / fixed_sum;
        for (column, width) in columns.iter().zip(&mut widths) {
            if column.width.is_some() {
                *width *= scale;
            }
        }
    } else if flexible_count > 0 {
        let flexible_width = (available_width - fixed_sum) / flexible_count as f32;
        for (column, width) in columns.iter().zip(&mut widths) {
            if column.width.is_none() {
                *width = flexible_width;
            }
        }
    } else if let Some(last) = widths.last_mut() {
        *last += available_width - fixed_sum;
    }

    widths
}
