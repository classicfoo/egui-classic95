#![allow(clippy::too_many_lines)]

use eframe::egui;
use eframe::emath::GuiRounding;
use egui::scroll_area::ScrollBarVisibility;
use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Id, Layout, Margin, PointerButton, Rect,
    ResizeDirection, Response, RichText, Sense, Stroke, StrokeKind, Ui, UiBuilder, ViewportCommand,
    pos2, vec2,
};
use egui_classic95::{
    Bevel, BevelFrame, Classic95, ClassicButton, Palette, horizontal_separator_with_palette,
    paint_bevel,
};

const CLASSIC_TAB_HEIGHT: f32 = 19.0;
const CLASSIC_TAB_TEXT_PADDING: f32 = 14.0;
const CLASSIC_TAB_RAISE: f32 = 2.0;
const CLASSIC_TAB_PANEL_OVERLAP: f32 = 2.0;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 820.0])
            .with_min_inner_size([960.0, 680.0])
            .with_decorations(false)
            .with_resizable(true)
            .with_title("Widget Factory"),
        ..Default::default()
    };

    eframe::run_native(
        "Widget Factory",
        options,
        Box::new(|cc| {
            let app = FactoryApp::default();
            app.theme.install(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    Overview,
    Buttons,
    TextFields,
    Selection,
    Ranges,
    Containers,
    Palette,
}

impl Page {
    const ALL: [Self; 7] = [
        Self::Overview,
        Self::Buttons,
        Self::TextFields,
        Self::Selection,
        Self::Ranges,
        Self::Containers,
        Self::Palette,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Buttons => "Buttons",
            Self::TextFields => "Text fields",
            Self::Selection => "Selection",
            Self::Ranges => "Ranges & feedback",
            Self::Containers => "Containers",
            Self::Palette => "Palette",
        }
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|page| *page == self).unwrap_or(0)
    }
}

struct FactoryApp {
    theme: Classic95,
    installed_palette: Option<Palette>,
    page: Page,
    name: String,
    password: String,
    notes: String,
    accepted: bool,
    enabled_option: bool,
    classic_spacing: bool,
    radio_choice: usize,
    combo_choice: String,
    slider: f32,
    progress: f32,
    active_tab: usize,
    last_action: String,
    widget_probe: String,
    catalog_scroll: f32,
    scrollbar_drag_start: Option<f32>,
    show_toolbar: bool,
    show_status_bar: bool,
    show_inspector: bool,
}

impl Default for FactoryApp {
    fn default() -> Self {
        Self {
            theme: Classic95::default(),
            installed_palette: None,
            page: Page::Overview,
            name: "Jane User".to_owned(),
            password: "win95".to_owned(),
            notes: "Type here to try the recessed multiline editor.".to_owned(),
            accepted: true,
            enabled_option: false,
            classic_spacing: true,
            radio_choice: 0,
            combo_choice: "Standard".to_owned(),
            slider: 68.0,
            progress: 42.0,
            active_tab: 0,
            last_action: "Ready. Click a control to inspect it.".to_owned(),
            widget_probe: "No control selected".to_owned(),
            catalog_scroll: 0.0,
            scrollbar_drag_start: None,
            show_toolbar: true,
            show_status_bar: true,
            show_inspector: true,
        }
    }
}

impl eframe::App for FactoryApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::from(self.theme.palette.face).to_array()
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if self.installed_palette != Some(self.theme.palette) {
            self.theme.install(ui.ctx());
            self.installed_palette = Some(self.theme.palette);
        }

        let full_rect = ui.max_rect();
        ui.painter()
            .rect_filled(full_rect, CornerRadius::ZERO, self.theme.palette.face);

        paint_bevel(ui.painter(), full_rect, Bevel::Raised, &self.theme.palette);

        // Leave an extra client-area point inside the two-layer window bevel,
        // so the status-bar frame and full-width rules cannot visually merge
        // with the outer edge.
        let content_rect = full_rect.shrink(3.0);
        ui.scope_builder(UiBuilder::new().max_rect(content_rect), |ui| {
            // Keep every child painter inside the inner window edge. Child UIs
            // can grow their layout rect when content is wider than expected,
            // but that must not let separators or the status bar paint over the
            // raised window bevel.
            ui.shrink_clip_rect(content_rect);
            ui.set_min_size(content_rect.size());
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            self.draw_window(ui, content_rect.width());
        });
        self.resize_handles(ui, full_rect);
    }
}

impl FactoryApp {
    fn draw_window(&mut self, ui: &mut Ui, layout_width: f32) {
        let p = self.theme.palette;
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            self.title_bar(ui, p);
            self.menu_bar(ui);
            if self.show_toolbar {
                self.toolbar(ui, p);
            }

            let status_height = if self.show_status_bar { 30.0 } else { 0.0 };
            let main_height = (ui.available_height() - status_height).max(250.0);
            let nav_width = 176.0;
            let inspector_width = 220.0;
            let inspector_gap = if self.show_inspector { 12.0 } else { 6.0 };
            let content_width = (layout_width
                - nav_width
                - if self.show_inspector {
                    inspector_width
                } else {
                    0.0
                }
                - inspector_gap)
                .max(320.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                ui.allocate_ui_with_layout(
                    vec2(nav_width, main_height),
                    Layout::top_down(Align::Min),
                    |ui| self.draw_navigation(ui, p),
                );
                ui.allocate_ui_with_layout(
                    vec2(content_width, main_height),
                    Layout::top_down(Align::Min),
                    |ui| self.draw_catalog(ui, p),
                );
                if self.show_inspector {
                    ui.allocate_ui_with_layout(
                        vec2(inspector_width, main_height),
                        Layout::top_down(Align::Min),
                        |ui| self.draw_inspector(ui, p),
                    );
                }
            });

            if self.show_status_bar {
                ui.add_space(5.0);
                self.status_bar(ui, p);
            }
        });
    }

    fn title_bar(&mut self, ui: &mut Ui, p: Palette) {
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(vec2(width, 19.0), Sense::hover());
        // The top face row plus the 18-pixel blue strip matches the reference.
        let caption_rect = Rect::from_min_max(
            rect.min + vec2(1.0, 1.0),
            rect.right_bottom() - vec2(1.0, 0.0),
        );
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::ZERO, p.face);
        painter.rect_filled(caption_rect, CornerRadius::ZERO, p.active_title);
        let icon_rect = Rect::from_min_size(
            pos2(rect.left() + 5.0, caption_rect.center().y - 8.0),
            vec2(16.0, 16.0),
        );
        draw_app_icon(painter, icon_rect);
        painter.text(
            pos2(rect.left() + 27.0, caption_rect.center().y),
            Align2::LEFT_CENTER,
            "Widget Factory",
            FontId::proportional(13.0),
            p.title_text,
        );

        let controls_width = 3.0 * 16.0 + 2.0;
        let controls_left = caption_rect.right() - controls_width - 2.0;
        let drag_rect = Rect::from_min_max(
            pos2(icon_rect.right() + 2.0, caption_rect.top()),
            pos2(controls_left - 3.0, caption_rect.bottom()),
        );
        let drag = ui.interact(
            drag_rect,
            Id::unique("factory-title-drag"),
            Sense::click_and_drag(),
        );
        if drag.drag_started_by(PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }

        let icon_response =
            ui.interact(icon_rect, Id::unique("factory-system-menu"), Sense::click());
        egui::Popup::menu(&icon_response)
            .style(|style: &mut egui::Style| {
                style.visuals.popup_shadow = egui::Shadow::NONE;
            })
            .show(|ui| {
                let maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
                if ui
                    .add_enabled(maximized, egui::Button::new("Restore"))
                    .clicked()
                {
                    ui.ctx()
                        .send_viewport_cmd(ViewportCommand::Maximized(false));
                }
                if ui.button("Minimize").clicked() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
                }
                if ui
                    .add_enabled(!maximized, egui::Button::new("Maximize"))
                    .clicked()
                {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(true));
                }
                ui.separator();
                if ui.button("Close").clicked() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                }
                paint_popup_bevel(ui, p);
            });

        let button_size = vec2(16.0, 14.0);
        let button_top = caption_rect.top() + 2.0;
        let min_rect = Rect::from_min_size(pos2(controls_left, button_top), button_size)
            .round_to_pixels(ui.pixels_per_point());
        let max_rect = min_rect.translate(vec2(16.0, 0.0));
        let close_rect = min_rect.translate(vec2(34.0, 0.0));
        if caption_button(ui, min_rect, "factory-minimize", CaptionGlyph::Minimize, p).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
        }

        let maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
        let max_glyph = if maximized {
            CaptionGlyph::Restore
        } else {
            CaptionGlyph::Maximize
        };
        if caption_button(ui, max_rect, "factory-maximize", max_glyph, p).clicked() {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(!maximized));
        }
        if caption_button(ui, close_rect, "factory-close", CaptionGlyph::Close, p).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
    }

    fn menu_bar(&mut self, ui: &mut Ui) {
        let p = self.theme.palette;
        let menu_style = move |style: &mut egui::Style| {
            egui::menu::menu_style(style);
            style.visuals.popup_shadow = egui::Shadow::NONE;
            for visuals in [
                &mut style.visuals.widgets.hovered,
                &mut style.visuals.widgets.active,
                &mut style.visuals.widgets.open,
            ] {
                visuals.bg_fill = p.selection;
                visuals.weak_bg_fill = p.selection;
                visuals.fg_stroke.color = p.selection_text;
            }
        };
        egui::MenuBar::new()
            .style(menu_style)
            .config(egui::menu::MenuConfig::new().style(menu_style))
            .ui(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                ui.add_space(4.0);
                ui.menu_button("File", |ui| {
                    ui.set_width(190.0);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    if ui.button("Reset samples").clicked() {
                        self.reset_samples();
                    }
                    classic_menu_separator(ui, p);
                    ui.menu_button("Open page", |ui| {
                        ui.set_width(190.0);
                        ui.spacing_mut().item_spacing.y = 0.0;
                        for page in Page::ALL {
                            if classic_menu_choice(ui, self.page == page, page.label(), p).clicked()
                            {
                                self.page = page;
                                self.catalog_scroll = 0.0;
                                self.last_action = format!("Opened {}", page.label());
                            }
                        }
                        paint_popup_bevel(ui, p);
                    });
                    ui.menu_button("Options", |ui| {
                        ui.set_width(190.0);
                        ui.spacing_mut().item_spacing.y = 0.0;
                        classic_menu_toggle(ui, &mut self.show_toolbar, "Show toolbar", p);
                        classic_menu_toggle(ui, &mut self.show_status_bar, "Show status bar", p);
                        classic_menu_toggle(ui, &mut self.show_inspector, "Show details pane", p);
                        paint_popup_bevel(ui, p);
                    });
                    classic_menu_separator(ui, p);
                    if ui.button("Exit").clicked() {
                        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                    }
                    paint_popup_bevel(ui, p);
                });
                ui.menu_button("Edit", |ui| {
                    ui.set_width(190.0);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    if ui.button("Clear notes").clicked() {
                        self.notes.clear();
                        self.last_action = "Notes cleared".to_owned();
                    }
                    if ui.button("Copy name").clicked() {
                        ui.ctx().copy_text(self.name.clone());
                        self.last_action = "Name copied to the clipboard".to_owned();
                    }
                    paint_popup_bevel(ui, p);
                });
                ui.menu_button("View", |ui| {
                    ui.set_width(190.0);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    for page in Page::ALL {
                        if classic_menu_choice(ui, self.page == page, page.label(), p).clicked() {
                            self.page = page;
                            self.catalog_scroll = 0.0;
                            self.last_action = format!("Opened {}", page.label());
                        }
                    }
                    classic_menu_separator(ui, p);
                    classic_menu_toggle(ui, &mut self.show_toolbar, "Toolbar", p);
                    classic_menu_toggle(ui, &mut self.show_status_bar, "Status bar", p);
                    classic_menu_toggle(ui, &mut self.show_inspector, "Details pane", p);
                    paint_popup_bevel(ui, p);
                });
                ui.menu_button("Help", |ui| {
                    ui.set_width(190.0);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label("Widget Factory");
                    ui.label("A control gallery for egui-classic95.");
                    paint_popup_bevel(ui, p);
                });
            });
        horizontal_separator_with_palette(ui, &self.theme.palette);
    }

    fn toolbar(&mut self, ui: &mut Ui, p: Palette) {
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            if toolbar_icon_button(ui, p, "Catalog", ToolbarGlyph::Catalog).clicked() {
                self.page = Page::Overview;
                self.catalog_scroll = 0.0;
                self.last_action = "Catalog opened".to_owned();
            }
            if toolbar_icon_button(ui, p, "Reset samples", ToolbarGlyph::Reset).clicked() {
                self.reset_samples();
            }
            vertical_separator_with_palette(ui, &p);
            ui.add_space(5.0);
            ui.label(RichText::new("Control Gallery").small());
        });
        horizontal_separator_with_palette(ui, &p);
    }

    fn draw_navigation(&mut self, ui: &mut Ui, p: Palette) {
        ui.set_min_width(ui.available_width());
        ui.label(RichText::new("Controls").strong());
        ui.add_space(3.0);
        BevelFrame::sunken()
            .palette(p)
            .fill(p.input)
            .inner_margin(Margin::same(1))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                for page in Page::ALL {
                    let response = ui.selectable_value(&mut self.page, page, page.label());
                    if response.clicked() {
                        self.last_action = format!("Opened {}", page.label());
                        self.catalog_scroll = 0.0;
                    }
                }
            });
        ui.add_space(8.0);
        ui.label(RichText::new("Classic edges").strong());
        edge_legend(ui, p, "Button", LegendStyle::Raised);
        edge_legend(ui, p, "Field", LegendStyle::Sunken);
        edge_legend(ui, p, "Selected", LegendStyle::Selected);
    }

    fn draw_catalog(&mut self, ui: &mut Ui, p: Palette) {
        let region = ui.available_size();
        let bar_width = 16.0;
        let viewport_width = (region.x - bar_width).max(1.0);
        ui.horizontal(|ui| {
            let scroll = ui.allocate_ui_with_layout(
                vec2(viewport_width, region.y),
                Layout::top_down(Align::Min),
                |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("factory-catalog-scroll")
                        .scroll_offset(vec2(0.0, self.catalog_scroll))
                        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            self.draw_page_contents(ui, p);
                        })
                },
            );
            let output = scroll.inner;
            let max_scroll = (output.content_size.y - output.inner_rect.height()).max(0.0);
            self.catalog_scroll = output.state.clamped_offset().y.clamp(0.0, max_scroll);
            self.draw_catalog_scrollbar(ui, p, region.y, max_scroll, output.inner_rect.height());
        });
    }

    fn draw_catalog_scrollbar(
        &mut self,
        ui: &mut Ui,
        p: Palette,
        height: f32,
        max_scroll: f32,
        viewport_height: f32,
    ) {
        let (bar_rect, _) = ui.allocate_exact_size(vec2(16.0, height), Sense::hover());
        if max_scroll <= 0.0 || height < 36.0 {
            return;
        }

        let arrow_size = 16.0_f32.min(height * 0.5);
        let up_rect = Rect::from_min_size(bar_rect.min, vec2(16.0, arrow_size));
        let down_rect = Rect::from_min_size(
            pos2(bar_rect.left(), bar_rect.bottom() - arrow_size),
            vec2(16.0, arrow_size),
        );
        if scroll_arrow_button(ui, up_rect, "catalog-scroll-up", p, false).clicked() {
            self.catalog_scroll = (self.catalog_scroll - 32.0).max(0.0);
        }
        if scroll_arrow_button(ui, down_rect, "catalog-scroll-down", p, true).clicked() {
            self.catalog_scroll = (self.catalog_scroll + 32.0).min(max_scroll);
        }

        let track_rect = Rect::from_min_max(
            pos2(bar_rect.left(), up_rect.bottom()),
            pos2(bar_rect.right(), down_rect.top()),
        );
        let track_response = ui.interact(
            track_rect,
            Id::unique("catalog-scroll-track"),
            Sense::click(),
        );
        ui.painter()
            .rect_filled(track_rect, CornerRadius::ZERO, p.light);
        for y in (track_rect.top() as i32..track_rect.bottom() as i32).step_by(2) {
            for x in (track_rect.left() as i32..track_rect.right() as i32).step_by(2) {
                if ((x / 2) + (y / 2)) & 1 == 0 {
                    ui.painter().rect_filled(
                        Rect::from_min_size(pos2(x as f32, y as f32), vec2(2.0, 2.0)),
                        CornerRadius::ZERO,
                        p.highlight,
                    );
                }
            }
        }

        let track_height = track_rect.height().max(1.0);
        let content_height = viewport_height + max_scroll;
        let thumb_height = ((track_height * viewport_height / content_height.max(1.0)).max(18.0))
            .min(track_height);
        let thumb_travel = (track_height - thumb_height).max(0.0);
        let thumb_top =
            track_rect.top() + thumb_travel * (self.catalog_scroll / max_scroll).clamp(0.0, 1.0);
        let thumb_rect = Rect::from_min_size(
            pos2(bar_rect.left(), thumb_top),
            vec2(track_rect.width(), thumb_height),
        );
        let thumb = ui.interact(
            thumb_rect,
            Id::unique("catalog-scroll-thumb"),
            Sense::drag(),
        );
        if track_response.clicked() && !thumb.hovered() {
            let pointer_y = track_response
                .interact_pointer_pos()
                .map_or(thumb_rect.center().y, |pos| pos.y);
            let direction = if pointer_y < thumb_rect.top() {
                -1.0
            } else {
                1.0
            };
            self.catalog_scroll =
                (self.catalog_scroll + direction * viewport_height * 0.85).clamp(0.0, max_scroll);
        }
        if thumb.drag_started_by(PointerButton::Primary) {
            self.scrollbar_drag_start = Some(self.catalog_scroll);
        }
        if thumb.dragged_by(PointerButton::Primary) {
            let start = self.scrollbar_drag_start.unwrap_or(self.catalog_scroll);
            let drag_distance = thumb.total_drag_delta().map_or(0.0, |delta| delta.y);
            self.catalog_scroll = (start
                + drag_distance * max_scroll / thumb_travel.max(1.0))
            .clamp(0.0, max_scroll);
        }
        if thumb.drag_stopped_by(PointerButton::Primary) {
            self.scrollbar_drag_start = None;
        }
        ui.painter()
            .rect_filled(thumb_rect, CornerRadius::ZERO, p.face);
        paint_bevel(
            ui.painter(),
            thumb_rect,
            if thumb.is_pointer_button_down_on() {
                Bevel::Pressed
            } else {
                Bevel::Raised
            },
            &p,
        );
    }

    fn resize_handles(&self, ui: &mut Ui, rect: Rect) {
        let edge = 5.0;
        let corner = 9.0;
        let handles = [
            (
                "resize-n",
                Rect::from_min_max(
                    pos2(rect.left() + corner, rect.top() - edge),
                    pos2(rect.right() - corner, rect.top() + edge),
                ),
                ResizeDirection::North,
                egui::CursorIcon::ResizeVertical,
            ),
            (
                "resize-s",
                Rect::from_min_max(
                    pos2(rect.left() + corner, rect.bottom() - edge),
                    pos2(rect.right() - corner, rect.bottom() + edge),
                ),
                ResizeDirection::South,
                egui::CursorIcon::ResizeVertical,
            ),
            (
                "resize-e",
                Rect::from_min_max(
                    pos2(rect.right() - edge, rect.top() + corner),
                    pos2(rect.right() + edge, rect.bottom() - corner),
                ),
                ResizeDirection::East,
                egui::CursorIcon::ResizeHorizontal,
            ),
            (
                "resize-w",
                Rect::from_min_max(
                    pos2(rect.left() - edge, rect.top() + corner),
                    pos2(rect.left() + edge, rect.bottom() - corner),
                ),
                ResizeDirection::West,
                egui::CursorIcon::ResizeHorizontal,
            ),
            (
                "resize-ne",
                Rect::from_min_max(
                    pos2(rect.right() - corner, rect.top() - corner),
                    pos2(rect.right() + edge, rect.top() + edge),
                ),
                ResizeDirection::NorthEast,
                egui::CursorIcon::ResizeNeSw,
            ),
            (
                "resize-nw",
                Rect::from_min_max(
                    pos2(rect.left() - edge, rect.top() - corner),
                    pos2(rect.left() + corner, rect.top() + edge),
                ),
                ResizeDirection::NorthWest,
                egui::CursorIcon::ResizeNwSe,
            ),
            (
                "resize-se",
                Rect::from_min_max(
                    pos2(rect.right() - corner, rect.bottom() - corner),
                    pos2(rect.right() + edge, rect.bottom() + edge),
                ),
                ResizeDirection::SouthEast,
                egui::CursorIcon::ResizeNwSe,
            ),
            (
                "resize-sw",
                Rect::from_min_max(
                    pos2(rect.left() - edge, rect.bottom() - corner),
                    pos2(rect.left() + corner, rect.bottom() + edge),
                ),
                ResizeDirection::SouthWest,
                egui::CursorIcon::ResizeNeSw,
            ),
        ];
        for (id, handle_rect, direction, cursor) in handles {
            let response = ui
                .interact(handle_rect, Id::unique(id), Sense::drag())
                .on_hover_cursor(cursor);
            if response.drag_started_by(PointerButton::Primary) {
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::BeginResize(direction));
            }
        }
    }

    fn draw_page_contents(&mut self, ui: &mut Ui, p: Palette) {
        match self.page {
            Page::Overview => {
                section(ui, p, "Control gallery", |ui| {
                    ui.label("Try the controls below or choose a category from the catalog.");
                });
                self.show_buttons(ui, p);
                self.show_text_fields(ui, p);
                self.show_selection(ui, p);
                self.show_ranges(ui, p);
                self.show_containers(ui, p);
                self.show_palette(ui, p);
            }
            Page::Buttons => self.show_buttons(ui, p),
            Page::TextFields => self.show_text_fields(ui, p),
            Page::Selection => self.show_selection(ui, p),
            Page::Ranges => self.show_ranges(ui, p),
            Page::Containers => self.show_containers(ui, p),
            Page::Palette => self.show_palette(ui, p),
        }
    }

    fn show_buttons(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "Command buttons", |ui| {
            ui.label("The button edge turns inward while pressed.");
            ui.add_space(7.0);
            ui.horizontal(|ui| {
                let normal = ClassicButton::new("Apply")
                    .palette(p)
                    .min_size(vec2(82.0, 26.0))
                    .show(ui);
                if normal.clicked() {
                    self.last_action = "Apply button clicked".to_owned();
                }
                self.observe("Apply", &normal);

                let default = ClassicButton::new("OK")
                    .palette(p)
                    .min_size(vec2(82.0, 26.0))
                    .default_button(true)
                    .show(ui);
                if default.clicked() {
                    self.last_action = "Default OK button clicked".to_owned();
                }
                self.observe("Default OK", &default);

                let disabled = ClassicButton::new("Cancel")
                    .palette(p)
                    .min_size(vec2(82.0, 26.0))
                    .enabled(false)
                    .show(ui);
                self.observe("Disabled Cancel", &disabled);
            });
            ui.add_space(7.0);
            ui.horizontal(|ui| {
                if ClassicButton::new("Copy name")
                    .palette(p)
                    .show(ui)
                    .clicked()
                {
                    ui.ctx().copy_text(self.name.clone());
                    self.last_action = "Name copied to the clipboard".to_owned();
                }
                if ClassicButton::new("Run sample")
                    .palette(p)
                    .show(ui)
                    .clicked()
                {
                    self.progress = 100.0;
                    self.last_action = "Sample run complete".to_owned();
                }
            });
        });
    }

    fn show_text_fields(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "Text fields", |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                let response = classic_text_edit(
                    ui,
                    vec2(ui.available_width(), 24.0),
                    egui::TextEdit::singleline(&mut self.name).hint_text("Enter a name"),
                    p,
                );
                if response.changed() {
                    self.last_action = "Name field edited".to_owned();
                }
                self.observe("Name field", &response);
            });
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Password:");
                let response = classic_text_edit(
                    ui,
                    vec2(ui.available_width(), 24.0),
                    egui::TextEdit::singleline(&mut self.password)
                        .password(true)
                        .hint_text("Password"),
                    p,
                );
                if response.changed() {
                    self.last_action = "Password field edited".to_owned();
                }
                self.observe("Password field", &response);
            });
            ui.add_space(5.0);
            ui.label("Notes:");
            let response = classic_text_edit(
                ui,
                vec2(ui.available_width(), 72.0),
                egui::TextEdit::multiline(&mut self.notes)
                    .desired_rows(3)
                    .hint_text("Write a note..."),
                p,
            );
            if response.changed() {
                self.last_action = "Notes field edited".to_owned();
            }
            self.observe("Multiline editor", &response);
        });
    }

    fn show_selection(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "Selection controls", |ui| {
            let response = classic_checkbox(ui, &mut self.accepted, "Remember this choice", p);
            if response.changed() {
                self.last_action = format!("Remember choice: {}", self.accepted);
            }
            self.observe("Remember checkbox", &response);

            ui.horizontal(|ui| {
                for (index, label) in ["Typical", "Compact", "Custom"].into_iter().enumerate() {
                    let response = ui.radio_value(&mut self.radio_choice, index, label);
                    if response.changed() {
                        self.last_action = format!("Selected {label}");
                    }
                    self.observe(label, &response);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Display mode:");
                let mut selection_changed = false;
                let combo = BevelFrame::sunken()
                    .palette(p)
                    .fill(p.input)
                    .inner_margin(Margin::same(1))
                    .show(ui, |ui| {
                        ui.visuals_mut().button_frame = false;
                        egui::ComboBox::from_id_salt("factory-display-mode")
                            .selected_text(self.combo_choice.clone())
                            .width(156.0)
                            .icon(move |ui, rect, _visuals, is_open| {
                                draw_combo_arrow(ui, rect, p, is_open);
                            })
                            .popup_style(egui::style::StyleModifier::new(|style| {
                                style.visuals.popup_shadow = egui::Shadow::NONE;
                            }))
                            .show_ui(ui, |ui| {
                                for option in ["Standard", "Large icons", "Details", "List"] {
                                    let choice = ui.selectable_value(
                                        &mut self.combo_choice,
                                        option.to_owned(),
                                        option,
                                    );
                                    selection_changed |= choice.changed();
                                }
                                paint_popup_bevel(ui, p);
                            })
                    })
                    .inner;
                let response = combo.response;
                if selection_changed {
                    self.last_action = format!("Display mode: {}", self.combo_choice);
                }
                self.observe("Display mode combo", &response);
            });
        });
    }

    fn show_ranges(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "Ranges and feedback", |ui| {
            let response = ui
                .horizontal(|ui| {
                    ui.label("Volume:");
                    let value_width = 36.0;
                    let slider_width = (ui.available_width() - value_width - 4.0).max(100.0);
                    ui.spacing_mut().slider_width = slider_width;
                    let response = ui.add(
                        egui::Slider::new(&mut self.slider, 0.0..=100.0)
                            .show_value(false)
                            .trailing_fill(false)
                            .handle_shape(egui::style::HandleShape::Rect { aspect_ratio: 0.55 }),
                    );
                    draw_classic_slider(
                        ui,
                        p,
                        response.rect,
                        self.slider,
                        response.is_pointer_button_down_on(),
                    );
                    ui.add_space(4.0);
                    ui.label(format!("{:.0}%", self.slider));
                    response
                })
                .inner;
            if response.changed() {
                self.last_action = format!("Slider moved to {:.0}%", self.slider);
            }
            self.observe("Volume slider", &response);

            ui.add_space(3.0);
            ui.horizontal(|ui| {
                let value_width = 36.0;
                let bar_width = (ui.available_width() - value_width - 7.0).max(96.0);
                segmented_progress_bar(ui, p, bar_width, 18.0, self.progress);
                ui.add_space(5.0);
                ui.label(format!("{:.0}%", self.progress));
            });
            ui.add_space(4.0);
            if ClassicButton::new("Advance progress")
                .palette(p)
                .show(ui)
                .clicked()
            {
                self.progress = (self.progress + 10.0) % 110.0;
                self.last_action = format!("Progress advanced to {:.0}%", self.progress);
            }
        });
    }

    fn show_containers(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "Containers", |ui| {
            ui.horizontal(|ui| {
                BevelFrame::raised()
                    .palette(p)
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        ui.set_min_width(124.0);
                        ui.label(RichText::new("Raised panel").strong());
                        ui.label("A command surface");
                    });
                BevelFrame::sunken()
                    .palette(p)
                    .fill(p.input)
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        ui.set_min_width(124.0);
                        ui.label(RichText::new("Sunken well").strong());
                        ui.label("A content surface");
                    });
            });

            ui.add_space(7.0);
            let titles = ["General", "Details", "About"];
            let active_tab = self.active_tab;
            let tab_font = FontId::proportional(13.0);
            let old_spacing = ui.spacing().item_spacing;
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            let mut tab_rects = Vec::with_capacity(titles.len());
            let mut tab_responses = Vec::with_capacity(titles.len());
            ui.horizontal(|ui| {
                for (index, title) in titles.into_iter().enumerate() {
                    let text_width = ui
                        .painter()
                        .layout_no_wrap(title.to_owned(), tab_font.clone(), p.text)
                        .size()
                        .x
                        .ceil();
                    let width = text_width + CLASSIC_TAB_TEXT_PADDING;
                    let (_, rect) = ui.allocate_space(vec2(width, CLASSIC_TAB_HEIGHT));
                    let hit_rect = if index == active_tab {
                        rect
                    } else {
                        Rect::from_min_max(
                            pos2(rect.left(), rect.top() + CLASSIC_TAB_RAISE),
                            rect.right_bottom(),
                        )
                    };
                    let response = ui.interact(
                        hit_rect,
                        ui.make_persistent_id(("factory_tab", index)),
                        Sense::click(),
                    );
                    if response.clicked() {
                        self.active_tab = index;
                        self.last_action = format!("Opened {title} tab");
                    }
                    tab_rects.push(rect);
                    tab_responses.push(response);
                }
            });
            let panel = BevelFrame::raised()
                .palette(p)
                .fill(p.face)
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    match active_tab {
                        0 => {
                            ui.label("General settings live inside the selected page.");
                            let response = classic_checkbox(
                                ui,
                                &mut self.enabled_option,
                                "Enable this option",
                                p,
                            );
                            if response.changed() {
                                self.last_action =
                                    format!("Option enabled: {}", self.enabled_option);
                            }
                        }
                        1 => {
                            ui.label("Details include the current sample values.");
                            ui.label(format!("Name: {}", self.name));
                        }
                        _ => {
                            ui.label("egui-classic95");
                            ui.label("A small reusable Win95-inspired theme engine.");
                        }
                    }
                });
            ui.spacing_mut().item_spacing = old_spacing;
            for response in &tab_responses {
                self.observe("Tab", response);
            }
            paint_classic_tabs(ui, p, &titles, &tab_rects, panel.response.rect, active_tab);

            ui.add_space(5.0);
            section(ui, p, "Advanced options", |ui| {
                let response =
                    classic_checkbox(ui, &mut self.classic_spacing, "Use classic spacing", p);
                if response.changed() {
                    self.last_action = format!("Classic spacing: {}", self.classic_spacing);
                }
                ui.label("Secondary controls sit inside a labelled group box.");
            });
        });
    }

    fn show_palette(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "System palette", |ui| {
            ui.label("Change the color wells to see the theme update across the factory.");
            ui.add_space(6.0);
            palette_editor(ui, "Window face", &mut self.theme.palette.face);
            palette_editor(ui, "Active title", &mut self.theme.palette.active_title);
            palette_editor(ui, "Selection", &mut self.theme.palette.selection);
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                swatch(
                    ui,
                    "Highlight",
                    self.theme.palette.highlight,
                    &self.theme.palette,
                );
                swatch(ui, "Shadow", self.theme.palette.shadow, &self.theme.palette);
                swatch(
                    ui,
                    "Dark edge",
                    self.theme.palette.dark_shadow,
                    &self.theme.palette,
                );
            });
        });
    }

    fn draw_inspector(&mut self, ui: &mut Ui, p: Palette) {
        ui.set_min_width(ui.available_width());
        ui.label(RichText::new("Details").strong());
        ui.add_space(2.0);
        horizontal_separator_with_palette(ui, &p);
        ui.add_space(5.0);
        ui.label(RichText::new("Last action").strong());
        ui.label(&self.last_action);
        ui.add_space(7.0);
        ui.label(RichText::new("Last control").strong());
        ui.label(&self.widget_probe);
        ui.add_space(7.0);
        ui.label(RichText::new("Current page").strong());
        ui.label(self.page.label());
        ui.add_space(7.0);
        horizontal_separator_with_palette(ui, &p);
        ui.add_space(5.0);
        ui.label(RichText::new("Sample values").strong());
        ui.label(format!("Name: {}", self.name));
        ui.label(format!("Display: {}", self.combo_choice));
        ui.label(format!("Volume: {:.0}%", self.slider));
    }

    fn status_bar(&self, ui: &mut Ui, p: Palette) {
        // Reserve a small client-area gutter so the status bevel reads as its
        // own control instead of touching the window's raised outer frame.
        let (bounds, _) = ui.allocate_exact_size(vec2(ui.available_width(), 25.0), Sense::hover());
        let rect = bounds.shrink(2.0);
        ui.painter().rect_filled(rect, CornerRadius::ZERO, p.face);
        paint_bevel(ui.painter(), rect, Bevel::Sunken, &p);
        ui.painter().text(
            rect.left_center() + vec2(6.0, 0.0),
            Align2::LEFT_CENTER,
            self.page.label(),
            FontId::proportional(11.0),
            p.text,
        );
        ui.painter().line_segment(
            [
                rect.right_top() + vec2(-103.0, 3.0),
                rect.right_bottom() + vec2(-103.0, -3.0),
            ],
            Stroke::new(1.0, p.shadow),
        );
        ui.painter().text(
            rect.right_center() + vec2(-23.0, 0.0),
            Align2::RIGHT_CENTER,
            format!("Page {} of {}", self.page.index() + 1, Page::ALL.len()),
            FontId::proportional(11.0),
            p.text,
        );
        draw_size_grip(ui.painter(), rect, p);
    }

    fn observe(&mut self, label: &str, response: &Response) {
        if !response.hovered() && !response.has_focus() && !response.is_pointer_button_down_on() {
            return;
        }
        let state = if response.is_pointer_button_down_on() {
            "Pressed"
        } else if response.has_focus() {
            "Focused"
        } else if response.hovered() {
            "Hovered"
        } else {
            "Ready"
        };
        self.widget_probe = format!("{label} — {state}");
    }

    fn reset_samples(&mut self) {
        self.name = "Jane User".to_owned();
        self.password = "win95".to_owned();
        self.notes = "Type here to try the recessed multiline editor.".to_owned();
        self.accepted = true;
        self.enabled_option = false;
        self.classic_spacing = true;
        self.radio_choice = 0;
        self.combo_choice = "Standard".to_owned();
        self.slider = 68.0;
        self.progress = 42.0;
        self.active_tab = 0;
        self.theme.palette = Palette::default();
        self.catalog_scroll = 0.0;
        self.scrollbar_drag_start = None;
        self.last_action = "Sample values reset".to_owned();
    }
}

fn section(ui: &mut Ui, palette: Palette, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    let available_width = ui.available_width();
    let inner = egui::Frame::new()
        .fill(palette.face)
        .stroke(Stroke::NONE)
        .inner_margin(Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.set_min_width((available_width - 16.0).max(1.0));
            let heading = ui.label(RichText::new(title).strong());
            ui.add_space(5.0);
            add_contents(ui);
            heading.rect
        });
    paint_group_box(ui, inner.response.rect, inner.inner, palette);
    ui.add_space(9.0);
}

fn paint_group_box(ui: &Ui, rect: Rect, heading: Rect, palette: Palette) {
    let painter = ui.painter();
    let top = heading.center().y;
    let left_gap = (heading.left() - 4.0).max(rect.left() + 8.0);
    let right_gap = (heading.right() + 4.0).min(rect.right() - 8.0);

    // The heading interrupts the top edge just as it does on a Win95 dialog.
    for (start, end) in [(rect.left(), left_gap), (right_gap, rect.right())] {
        painter.line_segment(
            [pos2(start, top), pos2(end, top)],
            Stroke::new(1.0, palette.shadow),
        );
        painter.line_segment(
            [pos2(start, top + 1.0), pos2(end, top + 1.0)],
            Stroke::new(1.0, palette.highlight),
        );
    }
    painter.line_segment(
        [pos2(rect.left(), top), pos2(rect.left(), rect.bottom())],
        Stroke::new(1.0, palette.shadow),
    );
    painter.line_segment(
        [
            pos2(rect.left() + 1.0, top + 1.0),
            pos2(rect.left() + 1.0, rect.bottom() - 1.0),
        ],
        Stroke::new(1.0, palette.highlight),
    );
    painter.line_segment(
        [pos2(rect.right(), top), pos2(rect.right(), rect.bottom())],
        Stroke::new(1.0, palette.highlight),
    );
    painter.line_segment(
        [
            pos2(rect.right() - 1.0, top + 1.0),
            pos2(rect.right() - 1.0, rect.bottom() - 1.0),
        ],
        Stroke::new(1.0, palette.shadow),
    );
    painter.line_segment(
        [
            pos2(rect.left(), rect.bottom()),
            pos2(rect.right(), rect.bottom()),
        ],
        Stroke::new(1.0, palette.highlight),
    );
    painter.line_segment(
        [
            pos2(rect.left() + 1.0, rect.bottom() - 1.0),
            pos2(rect.right() - 1.0, rect.bottom() - 1.0),
        ],
        Stroke::new(1.0, palette.shadow),
    );
}

fn paint_classic_tabs(
    ui: &Ui,
    palette: Palette,
    titles: &[&str],
    tabs: &[Rect],
    panel: Rect,
    selected: usize,
) {
    let mut tab_palette = palette;
    tab_palette.light = Color32::from_rgb(0xDF, 0xDF, 0xDF);
    tab_palette.dark_shadow = Color32::BLACK;

    // The original page border supplies the lower edge beneath the inactive tabs.
    // A selected tab covers its part of the border and joins the page surface.
    let panel = panel.round_to_pixels(ui.pixels_per_point());
    paint_bevel(ui.painter(), panel, Bevel::Raised, &tab_palette);

    // Paint inactive tabs first so the selected tab sits above its neighbors.
    for (index, tab) in tabs.iter().copied().enumerate() {
        if index != selected {
            paint_classic_tab(ui, palette, &tab_palette, tab, panel, titles[index], false);
        }
    }
    if let Some(tab) = tabs.get(selected).copied() {
        paint_classic_tab(
            ui,
            palette,
            &tab_palette,
            tab,
            panel,
            titles[selected],
            true,
        );
    }
}

fn paint_classic_tab(
    ui: &Ui,
    palette: Palette,
    edge_palette: &Palette,
    slot: Rect,
    panel: Rect,
    title: &str,
    selected: bool,
) {
    let requested_top = slot.top() + if selected { 0.0 } else { CLASSIC_TAB_RAISE };
    let bottom = if selected {
        panel.top() + CLASSIC_TAB_PANEL_OVERLAP
    } else {
        panel.top()
    };
    let tab = Rect::from_min_max(pos2(slot.left(), requested_top), pos2(slot.right(), bottom))
        .round_to_pixels(ui.pixels_per_point());
    let painter = ui.painter();
    painter.rect_filled(tab, CornerRadius::ZERO, palette.face);

    let top = tab.top();
    let left = tab.left();
    let right = tab.right();
    let side_end = if selected {
        panel.top() + 1.0
    } else {
        panel.top()
    };

    // Draw edge runs as exact one-pixel rectangles so subpixel placement cannot blur them.
    paint_classic_tab_pixels(
        painter,
        left + 2.0,
        top,
        right - left - 4.0,
        1.0,
        edge_palette.highlight,
    );
    paint_classic_tab_pixels(
        painter,
        left + 1.0,
        top + 1.0,
        right - left - 3.0,
        1.0,
        edge_palette.highlight,
    );
    paint_classic_tab_pixels(
        painter,
        left + 2.0,
        top + 1.0,
        right - left - 4.0,
        1.0,
        edge_palette.light,
    );
    paint_classic_tab_pixels(
        painter,
        right - 2.0,
        top + 1.0,
        2.0,
        1.0,
        edge_palette.dark_shadow,
    );
    paint_classic_tab_pixels(
        painter,
        right - 1.0,
        top + 1.0,
        1.0,
        2.0,
        palette.face,
    );
    let side_height = side_end - top - 2.0;
    paint_classic_tab_pixels(
        painter,
        left,
        top + 2.0,
        1.0,
        side_height,
        edge_palette.highlight,
    );
    paint_classic_tab_pixels(
        painter,
        left + 1.0,
        top + 2.0,
        1.0,
        side_height,
        edge_palette.light,
    );
    paint_classic_tab_pixels(
        painter,
        right - 2.0,
        top + 2.0,
        1.0,
        side_height,
        edge_palette.shadow,
    );
    paint_classic_tab_pixels(
        painter,
        right - 1.0,
        top + 2.0,
        1.0,
        side_height,
        edge_palette.dark_shadow,
    );

    let text_center = pos2(slot.center().x, (top + panel.top()) * 0.5);
    painter.text(
        text_center,
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(13.0),
        palette.text,
    );
}

fn paint_classic_tab_pixels(
    painter: &egui::Painter,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: Color32,
) {
    painter.rect_filled(
        Rect::from_min_size(pos2(x, y), vec2(width, height)),
        CornerRadius::ZERO,
        color,
    );
}

fn classic_checkbox(ui: &mut Ui, checked: &mut bool, label: &str, palette: Palette) -> Response {
    let response = ui.checkbox(checked, label);
    let size = 13.0;
    let box_rect = Rect::from_center_size(
        pos2(response.rect.left() + size * 0.5, response.rect.center().y),
        vec2(size, size),
    );
    ui.painter()
        .rect_filled(box_rect, CornerRadius::ZERO, palette.input);
    paint_bevel(ui.painter(), box_rect, Bevel::Sunken, &palette);
    if *checked {
        let mark = box_rect.shrink(3.0);
        ui.painter().add(egui::Shape::line(
            vec![
                pos2(mark.left(), mark.center().y),
                pos2(mark.center().x - 0.5, mark.bottom()),
                pos2(mark.right(), mark.top()),
            ],
            Stroke::new(1.4, palette.text),
        ));
    }
    response
}

fn classic_menu_toggle(ui: &mut Ui, value: &mut bool, label: &str, palette: Palette) -> Response {
    let response = menu_row(ui, label, *value, palette);
    if response.clicked() {
        *value = !*value;
        let mut response = response;
        response.mark_changed();
        return response;
    }
    response
}

fn classic_menu_choice(ui: &mut Ui, selected: bool, label: &str, palette: Palette) -> Response {
    menu_row(ui, label, selected, palette)
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

fn menu_row(ui: &mut Ui, label: &str, checked: bool, palette: Palette) -> Response {
    let width = ui.available_width().max(190.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, 22.0), Sense::click());
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
    if checked {
        let x = rect.left() + 11.0;
        let y = rect.center().y;
        ui.painter().add(egui::Shape::line(
            vec![
                pos2(x - 3.0, y),
                pos2(x - 0.5, y + 3.0),
                pos2(x + 4.0, y - 3.0),
            ],
            Stroke::new(1.5, text_color),
        ));
    }
    ui.painter().text(
        pos2(rect.left() + 25.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        text_color,
    );
    response
}

fn classic_menu_separator(ui: &mut Ui, palette: Palette) {
    let width = ui.available_width().max(190.0);
    let (rect, _) = ui.allocate_exact_size(vec2(width, 7.0), Sense::hover());
    let x1 = rect.left() + 23.0;
    let x2 = rect.right() - 3.0;
    ui.painter().line_segment(
        [pos2(x1, rect.center().y), pos2(x2, rect.center().y)],
        Stroke::new(1.0, palette.shadow),
    );
    ui.painter().line_segment(
        [
            pos2(x1, rect.center().y + 1.0),
            pos2(x2, rect.center().y + 1.0),
        ],
        Stroke::new(1.0, palette.highlight),
    );
}

fn vertical_separator_with_palette(ui: &mut Ui, palette: &Palette) {
    let (rect, _) = ui.allocate_exact_size(vec2(4.0, 22.0), Sense::hover());
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
}

#[derive(Clone, Copy)]
enum ToolbarGlyph {
    Catalog,
    Reset,
}

fn toolbar_icon_button(
    ui: &mut Ui,
    palette: Palette,
    label: &str,
    glyph: ToolbarGlyph,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(26.0, 24.0), Sense::click());
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, palette.face);
    paint_bevel(
        ui.painter(),
        rect,
        if response.is_pointer_button_down_on() {
            Bevel::Pressed
        } else {
            Bevel::Raised
        },
        &palette,
    );
    let icon_rect = Rect::from_center_size(rect.center(), vec2(16.0, 16.0));
    match glyph {
        ToolbarGlyph::Catalog => draw_catalog_icon(ui.painter(), icon_rect, palette),
        ToolbarGlyph::Reset => draw_reset_icon(ui.painter(), icon_rect, palette),
    }
    response.on_hover_text(label).on_disabled_hover_text(label)
}

fn draw_catalog_icon(painter: &egui::Painter, rect: Rect, palette: Palette) {
    let black = palette.text;
    let page = Rect::from_min_max(
        rect.left_top() + vec2(2.0, 1.0),
        rect.right_bottom() - vec2(1.0, 1.0),
    );
    painter.rect_filled(page, CornerRadius::ZERO, palette.input);
    painter.rect_stroke(
        page,
        CornerRadius::ZERO,
        Stroke::new(1.0, black),
        StrokeKind::Inside,
    );
    painter.rect_filled(
        Rect::from_min_size(page.left_top(), vec2(page.width(), 3.0)),
        CornerRadius::ZERO,
        palette.active_title,
    );
    for y in [page.top() + 6.0, page.top() + 9.0, page.top() + 12.0] {
        painter.line_segment(
            [pos2(page.left() + 3.0, y), pos2(page.right() - 3.0, y)],
            Stroke::new(1.0, palette.shadow),
        );
    }
}

fn draw_reset_icon(painter: &egui::Painter, rect: Rect, palette: Palette) {
    let center = rect.center();
    let points = [
        pos2(center.x + 2.0, center.y - 5.0),
        pos2(center.x - 2.0, center.y - 5.0),
        pos2(center.x - 5.0, center.y - 2.0),
        pos2(center.x - 5.0, center.y + 2.0),
        pos2(center.x - 2.0, center.y + 5.0),
        pos2(center.x + 2.0, center.y + 5.0),
        pos2(center.x + 5.0, center.y + 2.0),
    ];
    painter.add(egui::Shape::line(
        points.to_vec(),
        Stroke::new(1.0, palette.text),
    ));
    painter.add(egui::Shape::convex_polygon(
        vec![
            pos2(center.x + 6.0, center.y - 6.0),
            pos2(center.x + 6.0, center.y - 1.0),
            pos2(center.x + 1.0, center.y - 3.5),
        ],
        palette.text,
        Stroke::NONE,
    ));
}

fn draw_combo_arrow(ui: &Ui, rect: Rect, palette: Palette, pressed: bool) {
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, palette.face);
    paint_bevel(
        ui.painter(),
        rect.shrink(1.0),
        if pressed {
            Bevel::Pressed
        } else {
            Bevel::Raised
        },
        &palette,
    );
    let center = rect.center()
        + if pressed {
            vec2(1.0, 1.0)
        } else {
            vec2(0.0, 0.0)
        };
    ui.painter().add(egui::Shape::convex_polygon(
        vec![
            pos2(center.x - 4.0, center.y - 2.0),
            pos2(center.x + 4.0, center.y - 2.0),
            pos2(center.x, center.y + 2.0),
        ],
        palette.text,
        Stroke::NONE,
    ));
}

#[derive(Clone, Copy)]
enum LegendStyle {
    Raised,
    Sunken,
    Selected,
}

fn edge_legend(ui: &mut Ui, palette: Palette, label: &str, style: LegendStyle) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(30.0, 15.0), Sense::hover());
        match style {
            LegendStyle::Raised => {
                ui.painter()
                    .rect_filled(rect, CornerRadius::ZERO, palette.face);
                paint_bevel(ui.painter(), rect, Bevel::Raised, &palette);
            }
            LegendStyle::Sunken => {
                ui.painter()
                    .rect_filled(rect, CornerRadius::ZERO, palette.input);
                paint_bevel(ui.painter(), rect, Bevel::Sunken, &palette);
            }
            LegendStyle::Selected => {
                ui.painter()
                    .rect_filled(rect, CornerRadius::ZERO, palette.selection);
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "Ab",
                    FontId::proportional(9.0),
                    palette.selection_text,
                );
            }
        }
        ui.label(label);
    });
}

fn classic_text_edit(
    ui: &mut Ui,
    size: egui::Vec2,
    edit: egui::TextEdit<'_>,
    palette: Palette,
) -> Response {
    BevelFrame::sunken()
        .palette(palette)
        .fill(palette.input)
        .inner_margin(Margin::same(2))
        .show(ui, |ui| {
            ui.add_sized(
                [(size.x - 4.0).max(1.0), (size.y - 4.0).max(1.0)],
                edit.frame(egui::Frame::NONE).margin(Margin::same(2)),
            )
        })
        .inner
}

fn segmented_progress_bar(ui: &mut Ui, palette: Palette, width: f32, height: f32, value: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, palette.face);
    paint_bevel(ui.painter(), rect, Bevel::Sunken, &palette);
    let inner = rect.shrink(3.0);
    let segment_width = 8.0;
    let gap = 2.0;
    let count = ((inner.width() + gap) / (segment_width + gap)).floor() as usize;
    let filled = ((value.clamp(0.0, 100.0) / 100.0) * count as f32).round() as usize;
    for index in 0..filled.min(count) {
        let x = inner.left() + index as f32 * (segment_width + gap);
        ui.painter().rect_filled(
            Rect::from_min_size(
                pos2(x, inner.top()),
                vec2(segment_width.min(inner.right() - x), inner.height()),
            ),
            CornerRadius::ZERO,
            palette.selection,
        );
    }
}

fn draw_classic_slider(ui: &Ui, palette: Palette, rect: Rect, value: f32, pressed: bool) {
    let half_handle = (rect.height() / 2.5) * 0.55;
    let left = rect.left() + half_handle;
    let right = rect.right() - half_handle;
    let center_x = egui::lerp(left..=right, (value / 100.0).clamp(0.0, 1.0));
    let center_y = rect.center().y;
    let rail = Rect::from_min_max(pos2(left, center_y - 2.5), pos2(right, center_y + 2.5));
    ui.painter()
        .rect_filled(rail, CornerRadius::ZERO, palette.face);
    paint_bevel(ui.painter(), rail, Bevel::Sunken, &palette);

    let thumb = Rect::from_center_size(pos2(center_x, center_y), vec2(12.0, rect.height() - 2.0));
    ui.painter()
        .rect_filled(thumb, CornerRadius::ZERO, palette.face);
    paint_bevel(
        ui.painter(),
        thumb,
        if pressed {
            Bevel::Pressed
        } else {
            Bevel::Raised
        },
        &palette,
    );
}

fn scroll_arrow_button(
    ui: &mut Ui,
    rect: Rect,
    id: &'static str,
    palette: Palette,
    down: bool,
) -> Response {
    let response = ui.interact(rect, Id::unique(id), Sense::click());
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, palette.face);
    paint_bevel(
        ui.painter(),
        rect,
        if response.is_pointer_button_down_on() {
            Bevel::Pressed
        } else {
            Bevel::Raised
        },
        &palette,
    );
    let center = rect.center()
        + if response.is_pointer_button_down_on() {
            vec2(1.0, 1.0)
        } else {
            vec2(0.0, 0.0)
        };
    let points = if down {
        vec![
            pos2(center.x - 4.0, center.y - 2.0),
            pos2(center.x + 4.0, center.y - 2.0),
            pos2(center.x, center.y + 2.0),
        ]
    } else {
        vec![
            pos2(center.x - 4.0, center.y + 2.0),
            pos2(center.x + 4.0, center.y + 2.0),
            pos2(center.x, center.y - 2.0),
        ]
    };
    ui.painter().add(egui::Shape::convex_polygon(
        points,
        palette.text,
        Stroke::NONE,
    ));
    response
}

fn palette_editor(ui: &mut Ui, label: &str, color: &mut Color32) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.color_edit_button_srgba(color);
        });
    });
}

fn swatch(ui: &mut Ui, label: &str, color: Color32, palette: &Palette) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(28.0, 16.0), Sense::hover());
        ui.painter().rect_filled(rect, CornerRadius::ZERO, color);
        paint_bevel(ui.painter(), rect, Bevel::Raised, palette);
        ui.label(label);
    });
}

#[derive(Clone, Copy)]
enum CaptionGlyph {
    Minimize,
    Maximize,
    Restore,
    Close,
}

fn caption_button(
    ui: &mut Ui,
    rect: Rect,
    id: &'static str,
    glyph: CaptionGlyph,
    palette: Palette,
) -> Response {
    let response = ui.interact(rect, Id::unique(id), Sense::click());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::ZERO, palette.face);
    if response.is_pointer_button_down_on() {
        paint_bevel(painter, rect, Bevel::Pressed, &palette);
    } else {
        paint_caption_button_bevel(painter, rect, &palette);
    }

    match glyph {
        CaptionGlyph::Minimize => {
            paint_caption_pixels(painter, rect, 4, 9, &["######", "######"], palette.text);
        }
        CaptionGlyph::Maximize => {
            paint_caption_pixels(
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
            );
        }
        CaptionGlyph::Restore => {
            paint_caption_pixels(
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
            );
        }
        CaptionGlyph::Close => {
            paint_caption_pixels(
                painter,
                rect,
                4,
                3,
                &[
                    "##....##",
                    ".##..##.",
                    "..####..",
                    "...##...",
                    "..####..",
                    ".##..##.",
                    "##....##",
                ],
                palette.text,
            );
        }
    }
    response
}

fn paint_caption_button_bevel(painter: &egui::Painter, rect: Rect, palette: &Palette) {
    paint_caption_rect(painter, rect, 14, 1, 1, 12, palette.shadow);
    paint_caption_rect(painter, rect, 1, 12, 14, 1, palette.shadow);
    paint_caption_rect(painter, rect, 0, 0, 15, 1, palette.highlight);
    paint_caption_rect(painter, rect, 0, 0, 1, 13, palette.highlight);
    paint_caption_rect(painter, rect, 15, 0, 1, 13, palette.text);
    paint_caption_rect(painter, rect, 0, 13, 16, 1, palette.text);
}

fn paint_caption_pixels(
    painter: &egui::Painter,
    rect: Rect,
    offset_x: usize,
    offset_y: usize,
    rows: &[&str],
    color: Color32,
) {
    // Draw the bitmap cells in one mesh so egui's antialiased thin-rectangle
    // path cannot widen the one-pixel strokes.
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
    painter: &egui::Painter,
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

fn draw_app_icon(painter: &egui::Painter, rect: Rect) {
    painter.rect_filled(
        rect,
        CornerRadius::ZERO,
        Color32::from_rgb(0x00, 0x00, 0x00),
    );
    let window = rect.shrink(1.0);
    painter.rect_filled(
        window,
        CornerRadius::ZERO,
        Color32::from_rgb(0xC0, 0xC0, 0xC0),
    );
    painter.rect_filled(
        Rect::from_min_max(window.left_top(), pos2(window.right(), window.top() + 3.0)),
        CornerRadius::ZERO,
        Color32::from_rgb(0x00, 0x00, 0x80),
    );
    painter.rect_filled(
        Rect::from_min_size(
            pos2(window.left() + 2.0, window.top() + 5.0),
            vec2(4.0, 4.0),
        ),
        CornerRadius::ZERO,
        Color32::from_rgb(0x00, 0x80, 0x80),
    );
    painter.rect_filled(
        Rect::from_min_size(
            pos2(window.left() + 8.0, window.top() + 5.0),
            vec2(4.0, 4.0),
        ),
        CornerRadius::ZERO,
        Color32::from_rgb(0x00, 0x00, 0x80),
    );
    painter.rect_filled(
        Rect::from_min_size(
            pos2(window.left() + 2.0, window.top() + 10.0),
            vec2(10.0, 2.0),
        ),
        CornerRadius::ZERO,
        Color32::from_rgb(0x80, 0x80, 0x80),
    );
}

fn draw_size_grip(painter: &egui::Painter, rect: Rect, palette: Palette) {
    let start = rect.right_bottom() - vec2(13.0, 3.0);
    for (x_offset, y_offset) in [
        (8.0, 0.0),
        (5.0, 3.0),
        (2.0, 6.0),
        (11.0, 3.0),
        (8.0, 6.0),
        (5.0, 9.0),
    ] {
        let point = start + vec2(x_offset, y_offset);
        painter.line_segment(
            [point, point + vec2(2.0, 0.0)],
            Stroke::new(1.0, palette.shadow),
        );
        painter.line_segment(
            [point + vec2(0.0, 1.0), point + vec2(2.0, 1.0)],
            Stroke::new(1.0, palette.highlight),
        );
    }
}
