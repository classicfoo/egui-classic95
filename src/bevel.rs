use egui::{Painter, Rect, Stroke, pos2};

use crate::Palette;

/// Directional edge conventions for classic 3D frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bevel {
    /// A raised control: light at top-left, dark at bottom-right.
    Raised,
    /// A pushed-in button.
    Pressed,
    /// A recessed entry field or view well.
    Sunken,
    /// A window frame with a strong dark keyline.
    Window,
}

/// Paint a crisp, two-layer bevel around `rect`.
///
/// The primitive is drawn with egui line segments so it stays lightweight and
/// scales with the UI. Keep it at its native one-point thickness for the
/// sharpest edge.
pub fn paint_bevel(painter: &Painter, rect: Rect, bevel: Bevel, palette: &Palette) {
    let (outer_light, inner_light, inner_dark, outer_dark) = match bevel {
        Bevel::Raised => (
            palette.highlight,
            palette.light,
            palette.shadow,
            palette.dark_shadow,
        ),
        Bevel::Pressed => (
            palette.dark_shadow,
            palette.shadow,
            palette.light,
            palette.highlight,
        ),
        Bevel::Sunken => (
            palette.shadow,
            palette.dark_shadow,
            palette.highlight,
            palette.light,
        ),
        Bevel::Window => (
            palette.dark_shadow,
            palette.highlight,
            palette.shadow,
            palette.dark_shadow,
        ),
    };

    draw_edges(painter, rect, outer_light, outer_dark, 0.5);
    if rect.width() > 3.0 && rect.height() > 3.0 {
        draw_edges(painter, rect.shrink(1.0), inner_light, inner_dark, 0.5);
    }
}

fn draw_edges(
    painter: &Painter,
    rect: Rect,
    top_left: egui::Color32,
    bottom_right: egui::Color32,
    inset: f32,
) {
    let left = rect.left() + inset;
    let top = rect.top() + inset;
    let right = rect.right() - inset;
    let bottom = rect.bottom() - inset;

    painter.line_segment(
        [pos2(left, top), pos2(right, top)],
        Stroke::new(1.0, top_left),
    );
    painter.line_segment(
        [pos2(left, top), pos2(left, bottom)],
        Stroke::new(1.0, top_left),
    );
    painter.line_segment(
        [pos2(left, bottom), pos2(right, bottom)],
        Stroke::new(1.0, bottom_right),
    );
    painter.line_segment(
        [pos2(right, top), pos2(right, bottom)],
        Stroke::new(1.0, bottom_right),
    );
}
