use egui::{Color32, CornerRadius, Frame, Margin, Response, Shadow, Stroke, Tooltip, vec2};

use crate::Palette;
use crate::theme::palette_for;

const DEFAULT_BACKGROUND: Color32 = Color32::from_rgb(255, 255, 225);

/// A Windows 95-style tooltip attached to any egui widget response.
///
/// The tooltip uses egui's standard hover delay and dismissal behavior, with
/// the small pale-yellow window shown in the 1995 Windows interface guidelines.
#[derive(Clone, Debug)]
pub struct ClassicTooltip {
    text: String,
    palette: Option<Palette>,
    background: Option<Color32>,
    max_width: f32,
}

impl ClassicTooltip {
    /// Create a tooltip with a short description of its control.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            palette: None,
            background: None,
            max_width: 320.0,
        }
    }

    /// Use a local palette for the tooltip text and outline.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set the tooltip background color. The default is pale yellow.
    pub fn background(mut self, color: Color32) -> Self {
        self.background = Some(color);
        self
    }

    /// Set the maximum text width in egui points.
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = width.max(1.0);
        self
    }

    /// Attach the tooltip to a widget response and return that response.
    ///
    /// Disabled widgets show their tooltip when hovered, too. The tooltip is
    /// positioned by egui near the pointer and adjusted to stay on-screen.
    pub fn show_for(self, response: Response) -> Response {
        let palette = self.palette.unwrap_or_else(|| palette_for(&response.ctx));
        describe_for_accessibility(&response, &self.text);

        let frame = Frame::new()
            .fill(self.background.unwrap_or(DEFAULT_BACKGROUND))
            .stroke(Stroke::new(1.0, palette.text))
            .corner_radius(CornerRadius::ZERO)
            .inner_margin(Margin::symmetric(3, 2))
            .shadow(Shadow::NONE);
        let mut tooltip = if response.enabled() {
            Tooltip::for_enabled(&response)
        } else {
            Tooltip::for_disabled(&response)
        }
        .at_pointer()
        .gap(4.0)
        .width(self.max_width);
        if let Some(pointer) = response.ctx.pointer_hover_pos() {
            tooltip.popup = tooltip
                .popup
                .at_position(pointer + vec2(4.0, 16.0))
                .gap(4.0);
        }
        tooltip.popup = tooltip.popup.frame(frame);

        let text = self.text;
        let max_width = self.max_width;
        tooltip.show(move |ui| {
            ui.set_max_width(max_width);
            ui.label(text);
        });

        response
    }
}

fn describe_for_accessibility(response: &Response, text: &str) {
    if response.ctx.has_accesskit_node(response.id) {
        response.ctx.accesskit_node_builder(response.id, |node| {
            if node.description().is_none() {
                node.set_description(text);
            }
            if node.role() != egui::Role::Label
                && node.labelled_by().is_empty()
                && node.label().is_none_or(str::is_empty)
            {
                node.set_label(text);
            }
        });
    }
}
