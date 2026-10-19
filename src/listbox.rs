use egui::{Id, Margin, Response, ScrollArea, Ui, vec2};

use crate::theme::palette_for;
use crate::{BevelFrame, Palette};

/// A recessed list box with full-width selectable rows.
#[derive(Debug)]
pub struct ClassicListBox<'a> {
    items: Vec<String>,
    selected: &'a mut usize,
    palette: Option<Palette>,
    id_salt: Id,
    max_height: Option<f32>,
}

/// Interaction results from showing a [`ClassicListBox`].
pub struct ClassicListBoxResponse {
    /// Selected item index after handling input.
    pub selected: usize,
    /// Whether the selected index changed.
    pub changed: bool,
    /// Index of the clicked item, including clicks on the already selected item.
    pub clicked: Option<usize>,
    /// Responses for each row in list order.
    pub items: Vec<Response>,
    /// Response for the list box frame.
    pub response: Response,
}

impl<'a> ClassicListBox<'a> {
    /// Create a list box from item labels and a selected item index.
    pub fn new<I, S>(items: I, selected: &'a mut usize) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            selected,
            palette: None,
            id_salt: Id::NULL.with("classic95-list-box"),
            max_height: None,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set a stable identifier when multiple list boxes share the same UI.
    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_salt = Id::NULL.with(salt);
        self
    }

    /// Limit the list box height and scroll when the items exceed it.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height.max(0.0));
        self
    }

    /// Show the list box and update the selected index when a row is clicked.
    pub fn show(self, ui: &mut Ui) -> ClassicListBoxResponse {
        let ClassicListBox {
            items,
            selected,
            palette,
            id_salt,
            max_height,
        } = self;
        let palette = palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let mut changed = false;
        if items.is_empty() {
            if *selected != 0 {
                *selected = 0;
                changed = true;
            }
        } else if *selected >= items.len() {
            *selected = items.len() - 1;
            changed = true;
        }

        let mut clicked = None;
        let mut item_responses = Vec::with_capacity(items.len());
        let frame = BevelFrame::sunken()
            .palette(palette)
            .fill(palette.input)
            .inner_margin(Margin::same(2))
            .show(ui, |ui| {
                let available_width = ui.available_width();
                ui.set_min_width(available_width);

                let mut scroll = ScrollArea::vertical()
                    .id_salt(id_salt)
                    .auto_shrink([true, true]);
                if let Some(height) = max_height {
                    scroll = scroll.max_height(height);
                }

                scroll.show(ui, |ui| {
                    for (index, label) in items.iter().enumerate() {
                        let row_width = ui.available_width();
                        let row = ui.add(
                            egui::Button::selectable(
                                *selected == index,
                                (label.as_str(), egui::Atom::grow()),
                            )
                            .min_size(vec2(row_width, 0.0)),
                        );
                        if row.clicked() {
                            clicked = Some(index);
                            if *selected != index {
                                *selected = index;
                                changed = true;
                            }
                        }
                        item_responses.push(row);
                    }
                });
            });

        ClassicListBoxResponse {
            selected: *selected,
            changed,
            clicked,
            items: item_responses,
            response: frame.response,
        }
    }
}
