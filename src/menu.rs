use egui::{
    Color32, CornerRadius, FontId, InnerResponse, Margin, Painter, Rect, Response, Sense, Stroke,
    Style, Ui, Vec2, Widget, pos2, vec2,
};

use crate::theme::palette_for;
use crate::{Bevel, ClassicTooltip, Palette, paint_bevel};

/// A classic menu bar that applies Win95 menu and popup colors to egui menus.
#[derive(Clone, Copy, Debug, Default)]
pub struct ClassicMenuBar {
    palette: Option<Palette>,
}

impl ClassicMenuBar {
    /// Create a menu bar using the palette installed in the egui context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Show application-defined menu buttons with classic interaction styling.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let bar_style = move |style: &mut Style| classic_menu_style(style, palette);
        let popup_style = move |style: &mut Style| classic_menu_style(style, palette);
        egui::MenuBar::new()
            .style(bar_style)
            .config(egui::menu::MenuConfig::new().style(popup_style))
            .ui(ui, add_contents)
    }
}

/// Apply the library's menu-bar and popup style to an egui style.
pub fn classic_menu_style(style: &mut Style, palette: Palette) {
    egui::menu::menu_style(style);
    set_popup_surface(style, palette);
    set_menu_selection(style, palette);
}

/// Apply the library's popup style to an egui style.
pub fn classic_popup_style(style: &mut Style, palette: &Palette) {
    egui::menu::menu_style(style);
    set_popup_surface(style, *palette);
    set_menu_selection(style, *palette);
}

fn set_popup_surface(style: &mut Style, palette: Palette) {
    style.visuals.popup_shadow = egui::Shadow::NONE;
    style.visuals.window_fill = palette.face;
    style.visuals.window_stroke = Stroke::new(1.0, palette.highlight);
    style.visuals.window_corner_radius = CornerRadius::ZERO;
    style.visuals.menu_corner_radius = CornerRadius::ZERO;
    style.spacing.menu_margin = Margin::same(2);
}

fn set_menu_selection(style: &mut Style, palette: Palette) {
    for visuals in [
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        visuals.bg_fill = palette.selection;
        visuals.weak_bg_fill = palette.selection;
        visuals.fg_stroke.color = palette.selection_text;
    }
}

/// Draw the bevel around the contents of an egui menu popup.
#[derive(Clone, Copy, Debug)]
pub struct ClassicMenuPopup {
    palette: Option<Palette>,
    min_width: f32,
    max_width: f32,
}

impl ClassicMenuPopup {
    /// Create a popup contents wrapper with the classic minimum menu width.
    pub fn new() -> Self {
        Self {
            palette: None,
            min_width: 190.0,
            max_width: 190.0,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set the minimum width of the popup contents.
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width.max(0.0);
        self.max_width = self.max_width.max(self.min_width);
        self
    }

    /// Set the maximum width of the popup contents.
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = width.max(self.min_width);
        self
    }

    /// Show popup contents and paint the raised outline around them.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        ui.set_min_width(self.min_width);
        ui.set_max_width(self.max_width);
        ui.spacing_mut().item_spacing.y = 0.0;
        let result = add_contents(ui);
        paint_popup_bevel(ui, palette);
        result
    }
}

impl Default for ClassicMenuPopup {
    fn default() -> Self {
        Self::new()
    }
}

/// A fixed-height classic menu row with an optional check or radio mark.
pub struct ClassicMenuItem<'a> {
    label: String,
    mark: MenuMark<'a>,
    palette: Option<Palette>,
    min_width: f32,
}

enum MenuMark<'a> {
    None,
    Check(&'a mut bool),
    Radio(bool),
}

impl<'a> ClassicMenuItem<'a> {
    /// Create a menu row that toggles a checked value when clicked.
    pub fn check(label: impl Into<String>, checked: &'a mut bool) -> Self {
        Self {
            label: label.into(),
            mark: MenuMark::Check(checked),
            palette: None,
            min_width: 190.0,
        }
    }

    /// Create a menu row that shows a radio mark when `selected` is true.
    ///
    /// The application handles the click and updates its own selected value.
    pub fn radio(label: impl Into<String>, selected: bool) -> Self {
        Self {
            label: label.into(),
            mark: MenuMark::Radio(selected),
            palette: None,
            min_width: 190.0,
        }
    }

    /// Create a menu row without a mark.
    pub fn button(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            mark: MenuMark::None,
            palette: None,
            min_width: 190.0,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set the minimum row width.
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width.max(0.0);
        self
    }

    /// Show the row and return its egui response.
    pub fn show(self, ui: &mut Ui) -> Response {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let width = ui.available_width().max(self.min_width);
        let (rect, mut response) = ui.allocate_exact_size(vec2(width, 22.0), Sense::click());
        let highlighted = response.hovered() || response.is_pointer_button_down_on();
        let text_color = if highlighted {
            palette.selection_text
        } else {
            palette.text
        };
        if highlighted {
            ui.painter()
                .rect_filled(rect, CornerRadius::ZERO, palette.selection);
        }

        match self.mark {
            MenuMark::None => {}
            MenuMark::Check(checked) => {
                if *checked {
                    paint_pixel_bitmap(
                        ui.painter(),
                        pos2(rect.left() + 8.0, rect.center().y - 3.0),
                        &MENU_CHECK_PIXELS,
                        text_color,
                    );
                }
                if response.clicked() {
                    *checked = !*checked;
                    response.mark_changed();
                }
            }
            MenuMark::Radio(selected) => {
                if selected {
                    paint_pixel_bitmap(
                        ui.painter(),
                        pos2(rect.left() + 9.0, rect.center().y - 2.0),
                        &MENU_RADIO_PIXELS,
                        text_color,
                    );
                }
            }
        }
        ui.painter().text(
            pos2(rect.left() + 25.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            self.label,
            FontId::proportional(13.0),
            text_color,
        );
        response
    }
}

impl Widget for ClassicMenuItem<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// A two-tone etched separator for a classic popup menu.
#[derive(Clone, Copy, Debug, Default)]
pub struct ClassicMenuSeparator {
    palette: Option<Palette>,
}

impl ClassicMenuSeparator {
    /// Create a separator using the palette installed in the egui context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Paint the separator.
    pub fn show(self, ui: &mut Ui) -> Response {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let width = ui.available_width().max(190.0);
        let (rect, response) = ui.allocate_exact_size(vec2(width, 7.0), Sense::hover());
        let x1 = rect.left() + 2.0;
        let x2 = rect.right() - 2.0;
        let separator_y = rect.center().y + 1.0;
        ui.painter().line_segment(
            [pos2(x1, separator_y), pos2(x2, separator_y)],
            Stroke::new(1.0, palette.shadow),
        );
        ui.painter().line_segment(
            [pos2(x1, separator_y + 1.0), pos2(x2, separator_y + 1.0)],
            Stroke::new(1.0, palette.highlight),
        );
        response
    }
}

impl Widget for ClassicMenuSeparator {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// A raised toolbar button that lets the application draw its own icon.
#[derive(Clone, Debug)]
pub struct ClassicToolbarButton {
    label: String,
    palette: Option<Palette>,
    size: Vec2,
    selected: bool,
    enabled: bool,
}

impl ClassicToolbarButton {
    /// Create a button with application-provided tooltip text.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            palette: None,
            size: vec2(24.0, 22.0),
            selected: false,
            enabled: true,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set the button size in egui points.
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    /// Display the pressed option-set appearance for a toolbar mode button.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Enable or disable interaction with the toolbar button.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Show the button and let the application paint its own icon.
    pub fn show(self, ui: &mut Ui, draw_icon: impl FnOnce(&Painter, Rect)) -> Response {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let (rect, response) = ui
            .add_enabled_ui(self.enabled, |ui| {
                ui.allocate_exact_size(self.size, Sense::click())
            })
            .inner;
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, palette.face);
        if response.is_pointer_button_down_on() || self.selected {
            paint_bevel(ui.painter(), rect, Bevel::Pressed, &palette);
        } else {
            paint_bevel(ui.painter(), rect, Bevel::Raised, &palette);
        }
        let icon_size = if self.size.x >= 32.0 && self.size.y >= 30.0 {
            24.0
        } else {
            16.0
        };
        let icon_rect = Rect::from_center_size(rect.center(), vec2(icon_size, icon_size));
        draw_icon(ui.painter(), icon_rect);
        let enabled = response.enabled();
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, &self.label));
        ClassicTooltip::new(self.label)
            .palette(palette)
            .show_for(response)
    }
}

/// Draw a narrow vertical etched separator, commonly used between toolbar groups.
pub fn vertical_separator(ui: &mut Ui) -> Response {
    let palette = palette_for(ui.ctx());
    vertical_separator_with_palette(ui, &palette)
}

/// Draw a narrow vertical etched separator with a custom palette.
pub fn vertical_separator_with_palette(ui: &mut Ui, palette: &Palette) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(4.0, 22.0), Sense::hover());
    let x = rect.center().x;
    ui.painter().line_segment(
        [
            pos2(x - 1.0, rect.top() + 2.0),
            pos2(x - 1.0, rect.bottom() - 2.0),
        ],
        Stroke::new(1.0, palette.shadow),
    );
    ui.painter().line_segment(
        [
            pos2(x + 1.0, rect.top() + 2.0),
            pos2(x + 1.0, rect.bottom() - 2.0),
        ],
        Stroke::new(1.0, palette.highlight),
    );
    response
}

fn paint_popup_bevel(ui: &Ui, palette: Palette) {
    let margin = ui.spacing().menu_margin;
    let stroke = ui.visuals().window_stroke.width;
    let outer_margin = vec2(
        margin.leftf().max(margin.rightf()) + stroke,
        margin.topf().max(margin.bottomf()) + stroke,
    );
    let rect = ui.min_rect().expand2(outer_margin);
    let painter = ui.ctx().layer_painter(ui.layer_id());
    paint_bevel(&painter, rect, Bevel::Raised, &palette);
}

const MENU_CHECK_PIXELS: [&str; 7] = [
    "......#", ".....##", "#...###", "##.###.", "#####..", ".###...", "..#....",
];
const MENU_RADIO_PIXELS: [&str; 5] = [".###.", "#####", "#####", "#####", ".###."];

fn paint_pixel_bitmap(painter: &Painter, origin: egui::Pos2, rows: &[&str], color: Color32) {
    let mut mesh = egui::epaint::Mesh::default();
    for (row, pixels) in rows.iter().enumerate() {
        for (column, pixel) in pixels.bytes().enumerate() {
            if pixel == b'#' {
                let rect =
                    Rect::from_min_size(origin + vec2(column as f32, row as f32), vec2(1.0, 1.0));
                mesh.add_colored_rect(rect, color);
            }
        }
    }
    if !mesh.is_empty() {
        painter.add(egui::Shape::Mesh(std::sync::Arc::new(mesh)));
    }
}
