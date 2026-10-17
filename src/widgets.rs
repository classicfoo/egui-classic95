use egui::{
    Color32, CornerRadius, FontId, Frame, InnerResponse, Margin, Response, Sense, Stroke,
    StrokeKind, TextStyle, Ui, Vec2, pos2, vec2,
};

use crate::theme::palette_for;
use crate::{Bevel, Palette, paint_bevel};

/// A rectangular push button with a raised or pressed Win95 edge.
#[derive(Clone, Debug)]
pub struct ClassicButton {
    label: String,
    palette: Option<Palette>,
    min_size: Vec2,
    enabled: bool,
    default: bool,
}

impl ClassicButton {
    /// Create a button with a text label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            palette: None,
            min_size: Vec2::ZERO,
            enabled: true,
            default: false,
        }
    }

    /// Use a custom palette for this button.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set a minimum size in egui points.
    pub fn min_size(mut self, size: Vec2) -> Self {
        self.min_size = size;
        self
    }

    /// Enable or disable interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Draw the heavier outline used by a dialog's default command button.
    pub fn default_button(mut self, is_default: bool) -> Self {
        self.default = is_default;
        self
    }

    /// Show the button and return its egui response.
    pub fn show(self, ui: &mut Ui) -> Response {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let font_id = ui
            .style()
            .text_styles
            .get(&TextStyle::Button)
            .cloned()
            .unwrap_or_else(|| FontId::proportional(13.0));
        let text_color = if self.enabled {
            palette.text
        } else {
            palette.disabled_text
        };
        let galley = ui
            .painter()
            .layout_no_wrap(self.label.clone(), font_id, text_color);
        let content_size = galley.size() + ui.spacing().button_padding * 2.0;
        let desired_size = content_size
            .max(ui.spacing().interact_size)
            .max(self.min_size);
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(desired_size, sense);
        let painter = ui.painter();

        painter.rect_filled(rect, CornerRadius::ZERO, palette.face);
        if self.default {
            painter.rect_stroke(
                rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, palette.dark_shadow),
                StrokeKind::Inside,
            );
            let bevel = if response.is_pointer_button_down_on() {
                Bevel::Pressed
            } else {
                Bevel::Raised
            };
            paint_bevel(painter, rect.shrink(2.0), bevel, &palette);
        } else {
            let bevel = if response.is_pointer_button_down_on() {
                Bevel::Pressed
            } else {
                Bevel::Raised
            };
            paint_bevel(painter, rect, bevel, &palette);
        }

        if response.has_focus() && self.enabled {
            painter.rect_stroke(
                rect.shrink(4.0),
                CornerRadius::ZERO,
                Stroke::new(1.0, palette.text),
                StrokeKind::Inside,
            );
        }

        let offset = if response.is_pointer_button_down_on() {
            vec2(1.0, 1.0)
        } else {
            Vec2::ZERO
        };
        let text_pos = rect.center() - galley.size() * 0.5 + offset;
        painter.galley(text_pos, galley, text_color);
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, self.enabled, &self.label)
        });
        response
    }
}

impl egui::Widget for ClassicButton {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// A filled egui frame surrounded by a classic two-layer bevel.
#[derive(Clone, Debug)]
pub struct BevelFrame {
    bevel: Bevel,
    palette: Option<Palette>,
    fill: Option<Color32>,
    inner_margin: Margin,
}

impl BevelFrame {
    /// Create a raised panel.
    pub fn raised() -> Self {
        Self::new(Bevel::Raised)
    }

    /// Create a recessed well or field.
    pub fn sunken() -> Self {
        Self::new(Bevel::Sunken)
    }

    /// Create a window-style frame with a strong dark keyline.
    pub fn window() -> Self {
        Self::new(Bevel::Window)
    }

    /// Create a frame using a specific edge convention.
    pub fn new(bevel: Bevel) -> Self {
        Self {
            bevel,
            fill: None,
            palette: None,
            inner_margin: Margin::same(6),
        }
    }

    /// Use a custom palette for the frame and its fill.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set the interior fill color.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Set the padding inside the frame.
    pub fn inner_margin(mut self, margin: Margin) -> Self {
        self.inner_margin = margin;
        self
    }

    /// Show the frame contents and return their response.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let inner = Frame::new()
            .fill(self.fill.unwrap_or(palette.face))
            .stroke(Stroke::NONE)
            .inner_margin(self.inner_margin)
            .show(ui, add_contents);
        paint_bevel(ui.painter(), inner.response.rect, self.bevel, &palette);
        inner
    }
}

/// A two-tone etched horizontal rule.
pub fn horizontal_separator(ui: &mut Ui) -> Response {
    let palette = palette_for(ui.ctx());
    horizontal_separator_with_palette(ui, &palette)
}

/// Draw a two-tone etched rule using a custom palette.
pub fn horizontal_separator_with_palette(ui: &mut Ui, palette: &Palette) -> Response {
    // Child UIs can report a minimum width larger than their visible column.
    // Keep separators inside both the active layout and its clipping region.
    let available = ui.available_rect_before_wrap();
    let right = available.right().min(ui.clip_rect().right() - 1.0);
    let width = (right - available.left()).max(1.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, 2.0), Sense::hover());
    let painter = ui.painter();
    painter.line_segment(
        [
            pos2(rect.left(), rect.top() + 0.5),
            pos2(rect.right() - 0.5, rect.top() + 0.5),
        ],
        Stroke::new(1.0, palette.highlight),
    );
    painter.line_segment(
        [
            pos2(rect.left(), rect.bottom() - 0.5),
            pos2(rect.right() - 0.5, rect.bottom() - 0.5),
        ],
        Stroke::new(1.0, palette.shadow),
    );
    response
}
