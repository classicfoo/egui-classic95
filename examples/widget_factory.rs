#![allow(clippy::too_many_lines)]

use eframe::egui;
use egui::scroll_area::ScrollBarVisibility;
use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Id, Layout, Margin, PointerButton, Rect,
    ResizeDirection, Response, RichText, Sense, Stroke, StrokeKind, Ui, ViewportCommand, pos2,
    vec2,
};
use egui_classic95::{
    Bevel, BevelFrame, Classic95, ClassicButton, ClassicCheckbox, ClassicComboBox, ClassicGroupBox,
    ClassicListBox, ClassicListView, ClassicListViewColumn, ClassicMenuBar, ClassicMenuItem,
    ClassicMenuPopup, ClassicMenuSeparator, ClassicProgressBar, ClassicRadioButton,
    ClassicScrollBar, ClassicSlider, ClassicStatusBar, ClassicSystemMenu, ClassicTabs,
    ClassicTextEdit, ClassicTitleBar, ClassicToolbar, ClassicToolbarButton, ClassicTooltip,
    ClassicTreeNode, ClassicTreeView, ClassicWindowFrame, Palette,
    horizontal_separator_with_palette, paint_bevel, vertical_separator_with_palette,
};

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
    Lists,
    Tooltips,
    Palette,
}

impl Page {
    const ALL: [Self; 9] = [
        Self::Overview,
        Self::Buttons,
        Self::TextFields,
        Self::Selection,
        Self::Ranges,
        Self::Containers,
        Self::Lists,
        Self::Tooltips,
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
            Self::Lists => "Lists & toolbar",
            Self::Tooltips => "Tooltips",
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
    combo_enabled: bool,
    combo_choice: String,
    slider: f32,
    progress: f32,
    active_tab: usize,
    button_selected: bool,
    toolbar_mode: bool,
    listview_selected: Option<usize>,
    treeview_selected: Option<Vec<usize>>,
    last_action: String,
    widget_probe: String,
    catalog_scroll: f32,
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
            combo_enabled: true,
            combo_choice: "Standard".to_owned(),
            slider: 68.0,
            progress: 42.0,
            active_tab: 0,
            button_selected: true,
            toolbar_mode: true,
            listview_selected: Some(2),
            treeview_selected: Some(vec![0, 0, 1, 0]),
            last_action: "Ready. Click a control to inspect it.".to_owned(),
            widget_probe: "No control selected".to_owned(),
            catalog_scroll: 0.0,
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
        ClassicWindowFrame::new()
            .palette(self.theme.palette)
            .fill(self.theme.palette.face)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                self.draw_window(ui, ui.available_width());
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

            let status_height = if self.show_status_bar { 25.0 } else { 0.0 };
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
                self.status_bar(ui, p);
            }
        });
    }

    fn title_bar(&mut self, ui: &mut Ui, p: Palette) {
        let maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
        let title_bar = ClassicTitleBar::new("Widget Factory")
            .palette(p)
            .maximized(maximized)
            .id(Id::unique("widget-factory-title-bar"))
            .show(ui, draw_app_icon);

        ClassicSystemMenu::new(&title_bar.system_menu)
            .palette(p)
            .show(|ui| {
                if ui
                    .add_enabled(maximized, ClassicMenuItem::button("Restore").palette(p))
                    .clicked()
                {
                    ui.ctx()
                        .send_viewport_cmd(ViewportCommand::Maximized(false));
                }
                if ui
                    .add(ClassicMenuItem::button("Minimize").palette(p))
                    .clicked()
                {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
                }
                if ui
                    .add_enabled(!maximized, ClassicMenuItem::button("Maximize").palette(p))
                    .clicked()
                {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(true));
                }
                ui.add(ClassicMenuSeparator::new().palette(p));
                if ui
                    .add(ClassicMenuItem::button("Close").palette(p))
                    .clicked()
                {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                }
            });

        if title_bar.minimize.as_ref().is_some_and(Response::clicked) {
            ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
        }
        if title_bar.maximize.as_ref().is_some_and(Response::clicked) {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(!maximized));
        }
        if title_bar.close.as_ref().is_some_and(Response::clicked) {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
    }

    fn menu_bar(&mut self, ui: &mut Ui) {
        let p = self.theme.palette;
        ClassicMenuBar::new().palette(p).show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            ui.spacing_mut().button_padding.x = 4.0;
            ui.add_space(4.0);
            ui.menu_button("File", |ui| {
                ClassicMenuPopup::new().palette(p).show(ui, |ui| {
                    if ui
                        .add(ClassicMenuItem::button("Reset samples").palette(p))
                        .clicked()
                    {
                        self.reset_samples();
                    }
                    ui.add(ClassicMenuSeparator::new().palette(p));
                    ui.menu_button("Open page", |ui| {
                        ClassicMenuPopup::new().palette(p).show(ui, |ui| {
                            for page in Page::ALL {
                                if ui
                                    .add(
                                        ClassicMenuItem::radio(page.label(), self.page == page)
                                            .palette(p),
                                    )
                                    .clicked()
                                {
                                    self.page = page;
                                    self.catalog_scroll = 0.0;
                                    self.last_action = format!("Opened {}", page.label());
                                }
                            }
                        });
                    });
                    ui.menu_button("Options", |ui| {
                        ClassicMenuPopup::new().palette(p).show(ui, |ui| {
                            ui.add(
                                ClassicMenuItem::check("Show toolbar", &mut self.show_toolbar)
                                    .palette(p),
                            );
                            ui.add(
                                ClassicMenuItem::check(
                                    "Show status bar",
                                    &mut self.show_status_bar,
                                )
                                .palette(p),
                            );
                            ui.add(
                                ClassicMenuItem::check(
                                    "Show details pane",
                                    &mut self.show_inspector,
                                )
                                .palette(p),
                            );
                        });
                    });
                    ui.add(ClassicMenuSeparator::new().palette(p));
                    if ui.add(ClassicMenuItem::button("Exit").palette(p)).clicked() {
                        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                    }
                });
            });
            ui.menu_button("Edit", |ui| {
                ClassicMenuPopup::new().palette(p).show(ui, |ui| {
                    if ui
                        .add(ClassicMenuItem::button("Clear notes").palette(p))
                        .clicked()
                    {
                        self.notes.clear();
                        self.last_action = "Notes cleared".to_owned();
                    }
                    if ui
                        .add(ClassicMenuItem::button("Copy name").palette(p))
                        .clicked()
                    {
                        ui.ctx().copy_text(self.name.clone());
                        self.last_action = "Name copied to the clipboard".to_owned();
                    }
                });
            });
            ui.menu_button("View", |ui| {
                ClassicMenuPopup::new().palette(p).show(ui, |ui| {
                    for page in Page::ALL {
                        if ui
                            .add(ClassicMenuItem::radio(page.label(), self.page == page).palette(p))
                            .clicked()
                        {
                            self.page = page;
                            self.catalog_scroll = 0.0;
                            self.last_action = format!("Opened {}", page.label());
                        }
                    }
                    ui.add(ClassicMenuSeparator::new().palette(p));
                    ui.add(ClassicMenuItem::check("Toolbar", &mut self.show_toolbar).palette(p));
                    ui.add(
                        ClassicMenuItem::check("Status bar", &mut self.show_status_bar).palette(p),
                    );
                    ui.add(
                        ClassicMenuItem::check("Details pane", &mut self.show_inspector).palette(p),
                    );
                });
            });
            ui.menu_button("Help", |ui| {
                ClassicMenuPopup::new().palette(p).show(ui, |ui| {
                    ui.add(ClassicMenuItem::button("Widget Factory").palette(p));
                    ui.label("A control gallery for egui-classic95.");
                });
            });
        });
        horizontal_separator_with_palette(ui, &self.theme.palette);
    }

    fn toolbar(&mut self, ui: &mut Ui, p: Palette) {
        ClassicToolbar::new().palette(p).show(ui, |ui| {
            ui.horizontal(|ui| {
                if ClassicToolbarButton::new("Catalog")
                    .palette(p)
                    .show(ui, |painter, rect| draw_catalog_icon(painter, rect, p))
                    .clicked()
                {
                    self.page = Page::Overview;
                    self.catalog_scroll = 0.0;
                    self.last_action = "Catalog opened".to_owned();
                }
                if ClassicToolbarButton::new("Reset samples")
                    .palette(p)
                    .show(ui, |painter, rect| draw_reset_icon(painter, rect, p))
                    .clicked()
                {
                    self.reset_samples();
                }
                vertical_separator_with_palette(ui, &p);
                ui.label(RichText::new("Control Gallery").small());
            });
        });
    }

    fn draw_navigation(&mut self, ui: &mut Ui, p: Palette) {
        ui.set_min_width(ui.available_width());
        ui.label(RichText::new("Controls").strong());
        ui.add_space(3.0);
        let mut selected_page = self.page.index();
        let list = ClassicListBox::new(
            Page::ALL.iter().map(|page| page.label()),
            &mut selected_page,
        )
        .palette(p)
        .id_salt("factory-page-list")
        .show(ui);
        if let Some(index) = list.clicked {
            self.page = Page::ALL[index];
            self.last_action = format!("Opened {}", self.page.label());
            self.catalog_scroll = 0.0;
        }
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
            ui.spacing_mut().item_spacing.x = 0.0;
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
            ClassicScrollBar::new(
                &mut self.catalog_scroll,
                max_scroll,
                output.inner_rect.height(),
                region.y,
            )
            .palette(p)
            .id_salt("widget-factory-catalog-scrollbar")
            .show(ui);
        });
    }

    fn resize_handles(&self, ui: &mut Ui, rect: Rect) {
        let edge = 5.0;
        let corner = 9.0;
        let southeast_corner = if self.show_status_bar { 17.0 } else { corner };
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
                    pos2(rect.right() - southeast_corner, rect.bottom() + edge),
                ),
                ResizeDirection::South,
                egui::CursorIcon::ResizeVertical,
            ),
            (
                "resize-e",
                Rect::from_min_max(
                    pos2(rect.right() - edge, rect.top() + corner),
                    pos2(rect.right() + edge, rect.bottom() - southeast_corner),
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
                    pos2(
                        rect.right() - southeast_corner,
                        rect.bottom() - southeast_corner,
                    ),
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
                self.show_lists(ui, p);
                self.show_tooltips(ui, p);
                self.show_palette(ui, p);
            }
            Page::Buttons => self.show_buttons(ui, p),
            Page::TextFields => self.show_text_fields(ui, p),
            Page::Selection => self.show_selection(ui, p),
            Page::Ranges => self.show_ranges(ui, p),
            Page::Containers => self.show_containers(ui, p),
            Page::Lists => self.show_lists(ui, p),
            Page::Tooltips => self.show_tooltips(ui, p),
            Page::Palette => self.show_palette(ui, p),
        }
    }

    fn show_buttons(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "Command buttons", |ui| {
            ui.label("Selected buttons stay recessed with a checker pattern.");
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

                let toggle = ClassicButton::new("Pin")
                    .palette(p)
                    .min_size(vec2(82.0, 26.0))
                    .selected(self.button_selected)
                    .show(ui);
                if toggle.clicked() {
                    self.button_selected = !self.button_selected;
                    self.last_action = if self.button_selected {
                        "Pin button selected".to_owned()
                    } else {
                        "Pin button cleared".to_owned()
                    };
                }
                self.observe("Selected Pin", &toggle);
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
                let response = ClassicTextEdit::new(
                    egui::TextEdit::singleline(&mut self.name).hint_text("Enter a name"),
                )
                .size(vec2(ui.available_width(), 24.0))
                .palette(p)
                .show(ui);
                if response.changed() {
                    self.last_action = "Name field edited".to_owned();
                }
                self.observe("Name field", &response);
            });
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Password:");
                let response = ClassicTextEdit::new(
                    egui::TextEdit::singleline(&mut self.password)
                        .password(true)
                        .hint_text("Password"),
                )
                .size(vec2(ui.available_width(), 24.0))
                .palette(p)
                .show(ui);
                if response.changed() {
                    self.last_action = "Password field edited".to_owned();
                }
                self.observe("Password field", &response);
            });
            ui.add_space(5.0);
            ui.label("Notes:");
            let response = ClassicTextEdit::new(
                egui::TextEdit::multiline(&mut self.notes)
                    .desired_rows(3)
                    .hint_text("Write a note..."),
            )
            .size(vec2(ui.available_width(), 72.0))
            .palette(p)
            .show(ui);
            if response.changed() {
                self.last_action = "Notes field edited".to_owned();
            }
            self.observe("Multiline editor", &response);
        });
    }

    fn show_selection(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "Selection controls", |ui| {
            let response = ClassicCheckbox::new(&mut self.accepted, "Remember this choice")
                .palette(p)
                .show(ui);
            if response.changed() {
                self.last_action = format!("Remember choice: {}", self.accepted);
            }
            self.observe("Remember checkbox", &response);

            ui.horizontal(|ui| {
                for (index, label) in ["Typical", "Compact", "Custom"].into_iter().enumerate() {
                    let response = ClassicRadioButton::new(&mut self.radio_choice, index, label)
                        .palette(p)
                        .show(ui);
                    if response.changed() {
                        self.last_action = format!("Selected {label}");
                    }
                    self.observe(label, &response);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Display mode:");
                let mut selection_changed = false;
                let response = ClassicComboBox::new(self.combo_choice.clone())
                    .palette(p)
                    .enabled(self.combo_enabled)
                    .id_salt("factory-display-mode")
                    .width(156.0)
                    .show(ui, |ui| {
                        let options = ["Standard", "Large icons", "Details", "List"];
                        for option in options {
                            let choice = ui.selectable_value(
                                &mut self.combo_choice,
                                option.to_owned(),
                                option,
                            );
                            selection_changed |= choice.changed();
                        }
                    })
                    .response;
                if selection_changed {
                    self.last_action = format!("Display mode: {}", self.combo_choice);
                }
                self.observe("Display mode combo", &response);

                let enabled_response = ClassicCheckbox::new(&mut self.combo_enabled, "Enabled")
                    .palette(p)
                    .show(ui);
                if enabled_response.changed() {
                    self.last_action = format!(
                        "Display mode combo {}",
                        if self.combo_enabled {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    );
                }
                self.observe("Display mode enabled", &enabled_response);
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
                    let response = ClassicSlider::new(&mut self.slider, 0.0..=100.0)
                        .palette(p)
                        .show_value(false)
                        .trailing_fill(false)
                        .handle_aspect_ratio(0.55)
                        .show(ui);
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
                ui.add(
                    ClassicProgressBar::new(self.progress)
                        .palette(p)
                        .size(vec2(bar_width, 18.0)),
                );
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
            let mut active_tab = self.active_tab;
            let tabs = ClassicTabs::new(titles, &mut active_tab)
                .palette(p)
                .id_salt("widget-factory-containers")
                .show(ui, |ui, active_tab| match active_tab {
                    0 => {
                        ui.label("General settings live inside the selected page.");
                        let response =
                            ClassicCheckbox::new(&mut self.enabled_option, "Enable this option")
                                .palette(p)
                                .show(ui);
                        if response.changed() {
                            self.last_action = format!("Option enabled: {}", self.enabled_option);
                        }
                    }
                    1 => {
                        ui.label("Details include the current sample values.");
                        ui.label(format!("Name: {}", self.name));
                        ()
                    }
                    _ => {
                        ui.label("egui-classic95");
                        ui.label("A small reusable Win95-inspired theme engine.");
                        ()
                    }
                });
            self.active_tab = tabs.selected;
            if tabs.changed {
                self.last_action = format!("Opened {} tab", titles[tabs.selected]);
            }
            for response in &tabs.tabs {
                self.observe("Tab", response);
            }

            ui.add_space(5.0);
            section(ui, p, "Advanced options", |ui| {
                let response =
                    ClassicCheckbox::new(&mut self.classic_spacing, "Use classic spacing")
                        .palette(p)
                        .show(ui);
                if response.changed() {
                    self.last_action = format!("Classic spacing: {}", self.classic_spacing);
                }
                ui.label("Secondary controls sit inside a labelled group box.");
            });
        });
    }

    fn show_lists(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "Toolbar", |ui| {
            ui.label("Raised buttons represent commands; a selected mode stays pressed.");
            ui.add_space(5.0);
            ClassicToolbar::new().palette(p).show(ui, |ui| {
                ui.horizontal(|ui| {
                    let open = ClassicToolbarButton::new("Open selected item")
                        .palette(p)
                        .show(ui, |painter, rect| draw_catalog_icon(painter, rect, p));
                    if open.clicked() {
                        self.last_action = "Open selected item".to_owned();
                    }
                    self.observe("Open toolbar button", &open);

                    let mode = ClassicToolbarButton::new("Selection mode")
                        .palette(p)
                        .selected(self.toolbar_mode)
                        .show(ui, |painter, rect| draw_list_icon(painter, rect, p));
                    if mode.clicked() {
                        self.toolbar_mode = !self.toolbar_mode;
                        self.last_action = if self.toolbar_mode {
                            "Selection mode enabled".to_owned()
                        } else {
                            "Selection mode disabled".to_owned()
                        };
                    }
                    self.observe("Selection mode toolbar button", &mode);

                    vertical_separator_with_palette(ui, &p);
                    let reset = ClassicToolbarButton::new("Reset samples")
                        .palette(p)
                        .show(ui, |painter, rect| draw_reset_icon(painter, rect, p));
                    if reset.clicked() {
                        self.reset_samples();
                    }
                    self.observe("Reset toolbar button", &reset);
                    ui.label("View commands");
                });
            });
        });

        section(ui, p, "Report list view", |ui| {
            ui.label("Choose a row with the mouse or use the arrow, Home, and End keys.");
            ui.add_space(5.0);
            let entries = [
                ["Command.com", "MS-DOS Application", "93 KB", "08/24/95"],
                ["Config.sys", "System Configuration", "1 KB", "08/24/95"],
                ["Desktop.ini", "Configuration Settings", "1 KB", "09/14/95"],
                ["Explorer.exe", "Application", "184 KB", "08/24/95"],
                ["Himem.sys", "System File", "33 KB", "08/24/95"],
                ["Readme.txt", "Text Document", "8 KB", "10/02/95"],
                ["Win.com", "Application", "13 KB", "08/24/95"],
                ["Win.ini", "Configuration Settings", "2 KB", "08/24/95"],
                ["WinHelp.exe", "Application", "43 KB", "08/24/95"],
                ["Winver.exe", "Application", "24 KB", "08/24/95"],
            ];
            let view = ClassicListView::new(
                [
                    ClassicListViewColumn::new("Name").width(164.0),
                    ClassicListViewColumn::new("Type").width(184.0),
                    ClassicListViewColumn::new("Size").width(72.0),
                    ClassicListViewColumn::new("Modified"),
                ],
                entries,
                &mut self.listview_selected,
            )
            .palette(p)
            .id_salt("factory-report-list")
            .max_height(160.0)
            .show(ui);
            if let Some(index) = view.clicked {
                self.last_action = format!("Selected {}", entries[index][0]);
            } else if view.changed {
                if let Some(index) = view.selected {
                    self.last_action = format!("Selected {}", entries[index][0]);
                }
            }
            if let Some(index) = view.selected {
                if let Some(response) = view.rows.get(index) {
                    self.observe("Report row", response);
                }
                ui.add_space(3.0);
                ui.label(format!("Selected item: {}", entries[index][0]));
            }
        });

        section(ui, p, "Tree view", |ui| {
            ui.label(
                "Use plus/minus boxes to expand branches; arrow keys move through the outline.",
            );
            ui.add_space(5.0);
            let nodes = sample_tree_nodes();
            let tree = ClassicTreeView::new(nodes.clone(), &mut self.treeview_selected)
                .palette(p)
                .id_salt("factory-folder-tree")
                .max_height(198.0)
                .show(ui);
            if let Some(path) = tree.clicked.as_deref() {
                if let Some(label) = tree_node_label(&nodes, path) {
                    self.last_action = format!("Selected folder {label}");
                }
            } else if tree.changed {
                if let Some(path) = tree.selected.as_deref() {
                    if let Some(label) = tree_node_label(&nodes, path) {
                        self.last_action = format!("Selected folder {label}");
                    }
                }
            }
            if let Some(response) = tree.rows.iter().find(|response| response.has_focus()) {
                self.observe("Tree row", response);
            }
        });
    }

    fn show_tooltips(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "Tooltips", |ui| {
            ui.label("Pause over a control to see a short description.");
            ui.add_space(5.0);
            let response = ClassicButton::new("Hover for a tooltip")
                .palette(p)
                .show(ui);
            let response = ClassicTooltip::new("A brief description of this command.")
                .palette(p)
                .show_for(response);
            self.observe("Tooltip button", &response);

            ui.add_space(5.0);
            let response = ClassicButton::new("Unavailable command")
                .palette(p)
                .enabled(false)
                .show(ui);
            let response = ClassicTooltip::new("This command is unavailable in the current view.")
                .palette(p)
                .show_for(response);
            self.observe("Disabled tooltip", &response);

            ui.label("Tooltips follow the pointer and stay within the window bounds.");
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
        ClassicStatusBar::new().palette(p).show(ui, |ui, bounds| {
            ui.painter().text(
                bounds.left_center() + vec2(6.0, 0.0),
                Align2::LEFT_CENTER,
                self.page.label(),
                FontId::proportional(11.0),
                p.text,
            );
            ui.painter().line_segment(
                [
                    bounds.right_top() + vec2(-103.0, 3.0),
                    bounds.right_bottom() + vec2(-103.0, -3.0),
                ],
                Stroke::new(1.0, p.shadow),
            );
            ui.painter().text(
                bounds.right_center() + vec2(-23.0, 0.0),
                Align2::RIGHT_CENTER,
                format!("Page {} of {}", self.page.index() + 1, Page::ALL.len()),
                FontId::proportional(11.0),
                p.text,
            );
        });
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
        self.combo_enabled = true;
        self.combo_choice = "Standard".to_owned();
        self.slider = 68.0;
        self.progress = 42.0;
        self.active_tab = 0;
        self.toolbar_mode = true;
        self.listview_selected = Some(2);
        self.treeview_selected = Some(vec![0, 0, 1, 0]);
        self.theme.palette = Palette::default();
        self.catalog_scroll = 0.0;
        self.last_action = "Sample values reset".to_owned();
    }
}

fn section(ui: &mut Ui, palette: Palette, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    ClassicGroupBox::new(title)
        .palette(palette)
        .show(ui, add_contents);
    ui.add_space(3.0);
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

fn draw_list_icon(painter: &egui::Painter, rect: Rect, palette: Palette) {
    let page = Rect::from_min_max(
        rect.left_top() + vec2(1.0, 1.0),
        rect.right_bottom() - vec2(1.0, 1.0),
    );
    painter.rect_filled(page, CornerRadius::ZERO, palette.input);
    painter.rect_stroke(
        page,
        CornerRadius::ZERO,
        Stroke::new(1.0, palette.text),
        StrokeKind::Inside,
    );
    painter.rect_filled(
        Rect::from_min_size(
            page.left_top() + vec2(1.0, 3.0),
            vec2(page.width() - 2.0, 3.0),
        ),
        CornerRadius::ZERO,
        palette.selection,
    );
    for y in [page.top() + 2.0, page.top() + 8.0, page.top() + 11.0] {
        painter.line_segment(
            [pos2(page.left() + 2.0, y), pos2(page.right() - 2.0, y)],
            Stroke::new(1.0, palette.shadow),
        );
    }
}

fn sample_tree_nodes() -> Vec<ClassicTreeNode> {
    vec![
        ClassicTreeNode::new("Desktop").expanded(true).children([
            ClassicTreeNode::new("My Computer")
                .expanded(true)
                .children([
                    ClassicTreeNode::new("3½ Floppy (A:)"),
                    ClassicTreeNode::new("Local Disk (C:)")
                        .expanded(true)
                        .children([
                            ClassicTreeNode::new("Windows").expanded(true).children([
                                ClassicTreeNode::new("System"),
                                ClassicTreeNode::new("Fonts"),
                                ClassicTreeNode::new("Help"),
                            ]),
                            ClassicTreeNode::new("Program Files"),
                        ]),
                    ClassicTreeNode::new("Data (D:)"),
                ]),
            ClassicTreeNode::new("Network Neighborhood"),
            ClassicTreeNode::new("Recycle Bin"),
        ]),
    ]
}

fn tree_node_label<'a>(nodes: &'a [ClassicTreeNode], path: &[usize]) -> Option<&'a str> {
    let mut siblings = nodes;
    let mut selected = None;
    for index in path {
        let node = siblings.get(*index)?;
        selected = Some(node.label());
        siblings = node.child_nodes();
    }
    selected
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
