use egui::{Frame, InnerResponse, Margin, Stroke, Ui};

use crate::Palette;
use crate::theme::palette_for;

/// A classic toolbar surface for application-defined controls.
///
/// Use it to group [`crate::ClassicToolbarButton`] widgets and separators while
/// keeping commands, icons, and application state in the calling application.
#[derive(Clone, Debug)]
pub struct ClassicToolbar {
    palette: Option<Palette>,
    inner_margin: Margin,
}

impl ClassicToolbar {
    /// Create a full-width toolbar surface with classic horizontal padding.
    pub fn new() -> Self {
        Self {
            palette: None,
            inner_margin: Margin::symmetric(5, 3),
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set the content padding around the toolbar controls.
    pub fn inner_margin(mut self, margin: Margin) -> Self {
        self.inner_margin = margin;
        self
    }

    /// Show the toolbar contents and return both their value and the panel response.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let width = ui.available_width().max(1.0);
        let content_width =
            (width - self.inner_margin.leftf() - self.inner_margin.rightf()).max(1.0);
        let inner = Frame::new()
            .fill(palette.face)
            .stroke(Stroke::NONE)
            .inner_margin(self.inner_margin)
            .show(ui, |ui| {
                ui.set_min_width(content_width);
                add_contents(ui)
            });

        let rect = inner.response.rect;
        let painter = ui.painter();
        let left = rect.left() + 2.0;
        let right = rect.right() - 2.0;
        let top = rect.bottom() - 2.0;
        painter.line_segment(
            [egui::pos2(left, top), egui::pos2(right, top)],
            Stroke::new(1.0, palette.shadow),
        );
        painter.line_segment(
            [egui::pos2(left, top + 1.0), egui::pos2(right, top + 1.0)],
            Stroke::new(1.0, palette.highlight),
        );

        InnerResponse {
            inner: inner.inner,
            response: inner.response,
        }
    }
}

impl Default for ClassicToolbar {
    fn default() -> Self {
        Self::new()
    }
}
