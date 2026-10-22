use egui::emath::GuiRounding;
use egui::{
    Align2, Color32, CornerRadius, FontId, Id, InnerResponse, Painter, Rect, Response, Sense, Ui,
    UiBuilder, Widget, pos2, vec2,
};

use crate::menu::{ClassicMenuItem, ClassicMenuPopup, ClassicMenuSeparator, classic_popup_style};
use crate::theme::palette_for;
use crate::{Bevel, Palette, paint_bevel};

/// A raised window frame with an inset, clipped client area.
///
/// The frame fills the space available to the widget. Its contents are clipped
/// to the inset so child painting cannot cover the outer bevel.
#[derive(Clone, Debug)]
pub struct ClassicWindowFrame {
    palette: Option<Palette>,
    fill: Option<Color32>,
    content_inset: f32,
}

impl ClassicWindowFrame {
    /// Create a frame that fills the available UI region.
    pub fn new() -> Self {
        Self {
            palette: None,
            fill: None,
            content_inset: 3.0,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set the color behind the window contents.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Set the gap between the raised frame and the clipped client area.
    pub fn content_inset(mut self, inset: f32) -> Self {
        self.content_inset = inset.max(0.0);
        self
    }

    /// Paint the frame and show contents inside its clipped inset.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let outer_rect = ui.available_rect_before_wrap();
        let (allocated_rect, response) = ui.allocate_exact_size(outer_rect.size(), Sense::hover());
        let outer_rect = allocated_rect;
        ui.painter().rect_filled(
            outer_rect,
            CornerRadius::ZERO,
            self.fill.unwrap_or(palette.face),
        );
        paint_bevel(ui.painter(), outer_rect, Bevel::Raised, &palette);

        let content_rect = outer_rect.shrink(self.content_inset);
        let content = ui.scope_builder(UiBuilder::new().max_rect(content_rect), |ui| {
            ui.shrink_clip_rect(content_rect);
            ui.set_min_size(content_rect.size());
            add_contents(ui)
        });

        InnerResponse {
            inner: content.inner,
            response,
        }
    }
}

impl Default for ClassicWindowFrame {
    fn default() -> Self {
        Self::new()
    }
}

/// The pixel glyph drawn in a classic window caption button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptionButtonKind {
    /// Minimize the application window.
    Minimize,
    /// Maximize the application window.
    Maximize,
    /// Restore a maximized application window.
    Restore,
    /// Close the application window.
    Close,
}

/// A 16 by 14 pixel window caption button.
#[derive(Clone, Debug)]
pub struct CaptionButton {
    kind: CaptionButtonKind,
    palette: Option<Palette>,
    id: Option<Id>,
}

impl CaptionButton {
    /// Create a caption button with the requested glyph.
    pub fn new(kind: CaptionButtonKind) -> Self {
        Self {
            kind,
            palette: None,
            id: None,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set a stable identifier when placing the button at a custom position.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Draw the button at a caller-provided rectangle.
    pub fn show_at(self, ui: &mut Ui, rect: Rect) -> Response {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let id = self.id.unwrap_or_else(|| ui.next_auto_id());
        draw_caption_button(ui, rect, id, self.kind, palette)
    }
}

impl Widget for CaptionButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let (rect, response) = ui.allocate_exact_size(vec2(16.0, 14.0), Sense::click());
        paint_caption_button(ui, rect, &response, self.kind, palette);
        response
    }
}

/// A draggable active or inactive title bar with optional standard window controls.
#[derive(Clone, Debug)]
pub struct ClassicTitleBar {
    title: String,
    palette: Option<Palette>,
    active: bool,
    maximized: bool,
    window_controls: bool,
    height: f32,
    id: Option<Id>,
}

/// Responses and layout rectangles produced by [`ClassicTitleBar::show`].
pub struct ClassicTitleBarResponse {
    /// Full title-bar rectangle.
    pub rect: Rect,
    /// Rectangle reserved for the application-provided icon.
    pub icon_rect: Rect,
    /// Clicking this response opens an application-provided system menu.
    pub system_menu: Response,
    /// Dragging this response asks egui to start dragging the native window.
    /// Double-clicking it can be used to toggle maximization.
    pub drag: Response,
    /// Minimize button, when standard window controls are enabled.
    pub minimize: Option<Response>,
    /// Maximize or restore button, when standard window controls are enabled.
    pub maximize: Option<Response>,
    /// Close button, when standard window controls are enabled.
    pub close: Option<Response>,
}

impl ClassicTitleBar {
    /// Create a title bar with application-provided title text.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            palette: None,
            active: true,
            maximized: false,
            window_controls: true,
            height: 19.0,
            id: None,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Choose whether to paint the active or inactive caption color.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Select the restore glyph when the application window is maximized.
    pub fn maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    /// Show or hide the standard minimize, maximize, and close buttons.
    pub fn window_controls(mut self, show: bool) -> Self {
        self.window_controls = show;
        self
    }

    /// Set the title bar height in egui points.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(1.0);
        self
    }

    /// Set a stable identifier when the title bar is not the only one in its UI.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Draw the title bar and let the application paint its own icon.
    pub fn show(
        self,
        ui: &mut Ui,
        draw_icon: impl FnOnce(&Painter, Rect),
    ) -> ClassicTitleBarResponse {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(vec2(width, self.height), Sense::hover());
        let caption_rect = Rect::from_min_max(
            rect.min + vec2(1.0, 1.0),
            rect.right_bottom() - vec2(1.0, 0.0),
        );
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::ZERO, palette.face);
        painter.rect_filled(
            caption_rect,
            CornerRadius::ZERO,
            if self.active {
                palette.active_title
            } else {
                palette.inactive_title
            },
        );

        let icon_rect = Rect::from_min_size(
            pos2(rect.left() + 5.0, caption_rect.center().y - 8.0),
            vec2(16.0, 16.0),
        );
        draw_icon(painter, icon_rect);
        painter.text(
            pos2(rect.left() + 27.0, caption_rect.center().y),
            Align2::LEFT_CENTER,
            self.title,
            FontId::proportional(13.0),
            palette.title_text,
        );

        let id = self
            .id
            .unwrap_or_else(|| ui.make_persistent_id("classic95-title-bar"));
        let controls_width = if self.window_controls { 50.0 } else { 0.0 };
        let controls_left = caption_rect.right() - controls_width - 2.0;
        let drag_rect = Rect::from_min_max(
            pos2(icon_rect.right() + 2.0, caption_rect.top()),
            pos2(controls_left - 3.0, caption_rect.bottom()),
        );
        let drag = ui.interact(drag_rect, id.with("drag"), Sense::click_and_drag());
        if drag.drag_started_by(egui::PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        let system_menu = ui.interact(icon_rect, id.with("system-menu"), Sense::click());
        let (minimize, maximize, close) = if self.window_controls {
            let button_size = vec2(16.0, 14.0);
            let button_top = caption_rect.top() + 2.0;
            let min_rect = Rect::from_min_size(pos2(controls_left, button_top), button_size)
                .round_to_pixels(ui.pixels_per_point());
            let max_rect = min_rect.translate(vec2(16.0, 0.0));
            let close_rect = min_rect.translate(vec2(34.0, 0.0));
            let minimize = draw_caption_button(
                ui,
                min_rect,
                id.with("minimize"),
                CaptionButtonKind::Minimize,
                palette,
            );
            let maximize = draw_caption_button(
                ui,
                max_rect,
                id.with("maximize"),
                if self.maximized {
                    CaptionButtonKind::Restore
                } else {
                    CaptionButtonKind::Maximize
                },
                palette,
            );
            let close = draw_caption_button(
                ui,
                close_rect,
                id.with("close"),
                CaptionButtonKind::Close,
                palette,
            );
            (Some(minimize), Some(maximize), Some(close))
        } else {
            (None, None, None)
        };

        ClassicTitleBarResponse {
            rect,
            icon_rect,
            system_menu,
            drag,
            minimize,
            maximize,
            close,
        }
    }
}

/// A classic styled menu opened from a title-bar icon or another response.
pub struct ClassicSystemMenu<'a> {
    response: &'a Response,
    palette: Option<Palette>,
}

impl<'a> ClassicSystemMenu<'a> {
    /// Create a popup menu anchored to a clickable response.
    pub fn new(response: &'a Response) -> Self {
        Self {
            response,
            palette: None,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Show the menu with application-defined actions and return its egui response.
    pub fn show<R>(self, add_contents: impl FnOnce(&mut Ui) -> R) -> Option<InnerResponse<R>> {
        let palette = self
            .palette
            .unwrap_or_else(|| palette_for(&self.response.ctx));
        egui::Popup::menu(self.response)
            .style(move |style: &mut egui::Style| classic_popup_style(style, &palette))
            .show(|ui| {
                ClassicMenuPopup::new()
                    .palette(palette)
                    .show(ui, add_contents)
            })
    }
}

/// A complete classic application window, including its frame and title bar.
///
/// The component handles dragging, minimize, maximize/restore, close, and the
/// standard system-menu actions. Double-clicking the draggable title-bar area
/// toggles maximization. The application provides its icon and client contents.
/// Native applications should disable platform decorations in their startup
/// options (for example, `ViewportBuilder::with_decorations(false)`).
#[derive(Clone, Debug)]
pub struct ClassicWindow {
    title: String,
    palette: Option<Palette>,
    fill: Option<Color32>,
    content_inset: f32,
    id: Option<Id>,
}

impl ClassicWindow {
    /// Create a window with the supplied title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            palette: None,
            fill: None,
            content_inset: 3.0,
            id: None,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set the color behind the window contents.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Set the gap between the raised frame and the clipped client area.
    pub fn content_inset(mut self, inset: f32) -> Self {
        self.content_inset = inset.max(0.0);
        self
    }

    /// Set a stable identifier when the UI contains multiple windows with the same title.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Draw the window and its standard system menu, then show application contents.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        draw_icon: impl FnOnce(&Painter, Rect),
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.show_impl(ui, draw_icon, false, |_| {}, add_contents)
    }

    /// Draw the window and its system menu with application-specific menu items.
    ///
    /// The application items appear after Restore, Minimize, and Maximize, and
    /// before the standard Close item.
    pub fn show_with_system_menu<R>(
        self,
        ui: &mut Ui,
        draw_icon: impl FnOnce(&Painter, Rect),
        add_system_menu_items: impl FnOnce(&mut Ui),
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.show_impl(ui, draw_icon, true, add_system_menu_items, add_contents)
    }

    fn show_impl<R>(
        self,
        ui: &mut Ui,
        draw_icon: impl FnOnce(&Painter, Rect),
        has_custom_system_menu: bool,
        add_system_menu_items: impl FnOnce(&mut Ui),
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
        let active = ui.input(|input| input.viewport().focused.unwrap_or(true));
        let id = self
            .id
            .unwrap_or_else(|| ui.make_persistent_id(("classic95-window", self.title.as_str())));

        ClassicWindowFrame::new()
            .palette(palette)
            .fill(self.fill.unwrap_or(palette.face))
            .content_inset(self.content_inset)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    let content_spacing = ui.spacing().item_spacing;
                    ui.spacing_mut().item_spacing.y = 0.0;

                    let title_bar = ClassicTitleBar::new(self.title)
                        .palette(palette)
                        .active(active)
                        .maximized(maximized)
                        .id(id.with("title-bar"))
                        .show(ui, draw_icon);

                    if title_bar.drag.double_clicked() {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                    }

                    ClassicSystemMenu::new(&title_bar.system_menu)
                        .palette(palette)
                        .show(|ui| {
                            if ui
                                .add_enabled(
                                    maximized,
                                    ClassicMenuItem::button("Restore").palette(palette),
                                )
                                .clicked()
                            {
                                ui.ctx()
                                    .send_viewport_cmd(egui::ViewportCommand::Maximized(false));
                            }
                            if ui
                                .add(ClassicMenuItem::button("Minimize").palette(palette))
                                .clicked()
                            {
                                ui.ctx()
                                    .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }
                            if ui
                                .add_enabled(
                                    !maximized,
                                    ClassicMenuItem::button("Maximize").palette(palette),
                                )
                                .clicked()
                            {
                                ui.ctx()
                                    .send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                            }

                            ui.add(ClassicMenuSeparator::new().palette(palette));
                            if has_custom_system_menu {
                                add_system_menu_items(ui);
                                ui.add(ClassicMenuSeparator::new().palette(palette));
                            }
                            if ui
                                .add(ClassicMenuItem::button("Close").palette(palette))
                                .clicked()
                            {
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });

                    if title_bar.minimize.as_ref().is_some_and(Response::clicked) {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                    if title_bar.maximize.as_ref().is_some_and(Response::clicked) {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                    }
                    if title_bar.close.as_ref().is_some_and(Response::clicked) {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    ui.scope(|ui| {
                        ui.spacing_mut().item_spacing = content_spacing;
                        add_contents(ui)
                    })
                    .inner
                })
                .inner
            })
    }
}

impl Default for ClassicWindow {
    fn default() -> Self {
        Self::new("")
    }
}

fn draw_caption_button(
    ui: &mut Ui,
    rect: Rect,
    id: Id,
    kind: CaptionButtonKind,
    palette: Palette,
) -> Response {
    let response = ui.interact(rect, id, Sense::click());
    paint_caption_button(ui, rect, &response, kind, palette);
    response
}

fn paint_caption_button(
    ui: &mut Ui,
    rect: Rect,
    response: &Response,
    kind: CaptionButtonKind,
    palette: Palette,
) {
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::ZERO, palette.face);
    if response.is_pointer_button_down_on() {
        paint_bevel(painter, rect, Bevel::Pressed, &palette);
    } else {
        paint_caption_button_bevel(painter, rect, &palette);
    }

    match kind {
        CaptionButtonKind::Minimize => {
            paint_caption_pixels(painter, rect, 4, 9, &["######", "######"], palette.text);
        }
        CaptionButtonKind::Maximize => paint_caption_pixels(
            painter,
            rect,
            3,
            2,
            &[
                "#########",
                "#########",
                "#.......#",
                "#.......#",
                "#.......#",
                "#.......#",
                "#.......#",
                "#.......#",
                "#########",
            ],
            palette.text,
        ),
        CaptionButtonKind::Restore => paint_caption_pixels(
            painter,
            rect,
            3,
            2,
            &[
                "  ########",
                "  ########",
                "  #......#",
                "#########.",
                "#########.",
                "#......##.",
                "#......##.",
                "#......##.",
                "#......##.",
                "#########.",
            ],
            palette.text,
        ),
        CaptionButtonKind::Close => paint_caption_pixels(
            painter,
            rect,
            4,
            3,
            &[
                "##....##", ".##..##.", "..####..", "...##...", "..####..", ".##..##.", "##....##",
            ],
            palette.text,
        ),
    }
}

fn paint_caption_button_bevel(painter: &Painter, rect: Rect, palette: &Palette) {
    paint_caption_rect(painter, rect, 14, 1, 1, 12, palette.shadow);
    paint_caption_rect(painter, rect, 1, 12, 14, 1, palette.shadow);
    paint_caption_rect(painter, rect, 0, 0, 15, 1, palette.highlight);
    paint_caption_rect(painter, rect, 0, 0, 1, 13, palette.highlight);
    paint_caption_rect(painter, rect, 15, 0, 1, 13, palette.text);
    paint_caption_rect(painter, rect, 0, 13, 16, 1, palette.text);
}

fn paint_caption_pixels(
    painter: &Painter,
    rect: Rect,
    offset_x: usize,
    offset_y: usize,
    rows: &[&str],
    color: Color32,
) {
    let mut mesh = egui::Mesh::default();
    for (y, row) in rows.iter().enumerate() {
        let pixels = row.as_bytes();
        let mut x = 0;
        while x < pixels.len() {
            if pixels[x] != b'#' {
                x += 1;
                continue;
            }

            let start = x;
            while x < pixels.len() && pixels[x] == b'#' {
                x += 1;
            }
            mesh.add_colored_rect(
                Rect::from_min_size(
                    rect.min + vec2((offset_x + start) as f32, (offset_y + y) as f32),
                    vec2((x - start) as f32, 1.0),
                ),
                color,
            );
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

fn paint_caption_rect(
    painter: &Painter,
    button_rect: Rect,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: Color32,
) {
    let rect = Rect::from_min_size(
        button_rect.min + vec2(x as f32, y as f32),
        vec2(width as f32, height as f32),
    );
    painter.rect_filled(rect, CornerRadius::ZERO, color);
}
