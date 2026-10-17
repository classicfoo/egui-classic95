#![allow(clippy::too_many_lines)]

use eframe::egui;
use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Id, Layout, Margin, PointerButton, Rect, Response,
    RichText, Sense, Stroke, StrokeKind, Ui, UiBuilder, ViewportCommand, pos2, vec2,
};
use egui_classic95::{
    Bevel, BevelFrame, Classic95, ClassicButton, Palette, horizontal_separator_with_palette,
    paint_bevel,
};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 820.0])
            .with_min_inner_size([960.0, 680.0])
            .with_decorations(false)
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
}

struct FactoryApp {
    theme: Classic95,
    installed_palette: Option<Palette>,
    page: Page,
    name: String,
    password: String,
    notes: String,
    accepted: bool,
    radio_choice: usize,
    combo_choice: String,
    slider: f32,
    progress: f32,
    active_tab: usize,
    last_action: String,
    widget_probe: String,
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
            radio_choice: 0,
            combo_choice: "Standard".to_owned(),
            slider: 68.0,
            progress: 42.0,
            active_tab: 0,
            last_action: "Ready. Click a control to inspect it.".to_owned(),
            widget_probe: "No widget sampled yet".to_owned(),
        }
    }
}

impl eframe::App for FactoryApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::from(self.theme.palette.desktop).to_array()
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if self.installed_palette != Some(self.theme.palette) {
            self.theme.install(ui.ctx());
            self.installed_palette = Some(self.theme.palette);
        }

        let full_rect = ui.max_rect();
        let desktop = self.theme.palette.desktop;
        ui.painter()
            .rect_filled(full_rect, CornerRadius::ZERO, desktop);

        let window_rect = full_rect.shrink(10.0);
        ui.painter()
            .rect_filled(window_rect, CornerRadius::ZERO, self.theme.palette.face);
        paint_bevel(
            ui.painter(),
            window_rect,
            Bevel::Window,
            &self.theme.palette,
        );

        let content_rect = window_rect.shrink(3.0);
        ui.scope_builder(UiBuilder::new().max_rect(content_rect), |ui| {
            ui.set_min_size(content_rect.size());
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            self.draw_window(ui);
        });
    }
}

impl FactoryApp {
    fn draw_window(&mut self, ui: &mut Ui) {
        let p = self.theme.palette;
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            self.title_bar(ui, p);
            self.menu_bar(ui);
            self.toolbar(ui, p);
            ui.add_space(5.0);

            let main_height = (ui.available_height() - 27.0).max(250.0);
            let total_width = ui.available_width();
            let nav_width = 176.0;
            let inspector_width = 220.0;
            let content_width = (total_width - nav_width - inspector_width - 12.0).max(320.0);

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
                ui.allocate_ui_with_layout(
                    vec2(inspector_width, main_height),
                    Layout::top_down(Align::Min),
                    |ui| self.draw_inspector(ui, p),
                );
            });

            ui.add_space(5.0);
            self.status_bar(ui, p);
        });
    }

    fn title_bar(&mut self, ui: &mut Ui, p: Palette) {
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(vec2(width, 27.0), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::ZERO, p.active_title);
        draw_app_icon(painter, rect.left_top() + vec2(7.0, 6.0));
        painter.text(
            pos2(rect.left() + 31.0, rect.center().y),
            Align2::LEFT_CENTER,
            "Widget Factory - egui-classic95",
            FontId::proportional(13.0),
            p.title_text,
        );

        let controls_width = 3.0 * 18.0 + 2.0 * 3.0;
        let controls_left = rect.right() - controls_width - 5.0;
        let drag_rect = Rect::from_min_max(rect.min, pos2(controls_left - 3.0, rect.bottom()));
        let drag = ui.interact(drag_rect, Id::new("factory-title-drag"), Sense::click_and_drag());
        if drag.drag_started_by(PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }

        let y = rect.center().y - 7.0;
        let min_rect = Rect::from_min_size(pos2(controls_left, y), vec2(18.0, 14.0));
        let max_rect = min_rect.translate(vec2(21.0, 0.0));
        let close_rect = min_rect.translate(vec2(42.0, 0.0));
        if caption_button(ui, min_rect, "factory-minimize", CaptionGlyph::Minimize, p).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
        }

        let maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
        if caption_button(ui, max_rect, "factory-maximize", CaptionGlyph::Maximize, p).clicked() {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(!maximized));
        }
        if caption_button(ui, close_rect, "factory-close", CaptionGlyph::Close, p).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
    }

    fn menu_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(3.0, 0.0);
            ui.add_space(4.0);
            ui.menu_button("File", |ui| {
                if ui.button("Reset samples").clicked() {
                    self.reset_samples();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui.button("Clear notes").clicked() {
                    self.notes.clear();
                    self.last_action = "Notes cleared".to_owned();
                }
                if ui.button("Select all samples").clicked() {
                    self.last_action = "Sample controls are ready for input".to_owned();
                }
            });
            ui.menu_button("View", |ui| {
                for page in Page::ALL {
                    ui.radio_value(&mut self.page, page, page.label());
                }
            });
            ui.menu_button("Help", |ui| {
                ui.label("Widget Factory");
                ui.label("A control gallery for egui-classic95.");
            });
        });
        horizontal_separator_with_palette(ui, &self.theme.palette);
    }

    fn toolbar(&mut self, ui: &mut Ui, p: Palette) {
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            if ClassicButton::new("Catalog")
                .palette(p)
                .min_size(vec2(76.0, 24.0))
                .show(ui)
                .clicked()
            {
                self.page = Page::Overview;
                self.last_action = "Catalog opened".to_owned();
            }
            if ClassicButton::new("Reset")
                .palette(p)
                .min_size(vec2(68.0, 24.0))
                .show(ui)
                .clicked()
            {
                self.reset_samples();
            }
            horizontal_separator_with_palette(ui, &p);
            ui.add_space(5.0);
            ui.label(RichText::new("Style:").small());
            egui::ComboBox::from_id_salt("factory-style-picker")
                .selected_text("Windows 95")
                .width(120.0)
                .show_ui(ui, |ui| {
                    ui.label("Windows 95");
                    ui.label("System palette");
                });
        });
        horizontal_separator_with_palette(ui, &p);
    }

    fn draw_navigation(&mut self, ui: &mut Ui, p: Palette) {
        BevelFrame::sunken()
            .palette(p)
            .fill(p.input)
            .inner_margin(Margin::same(7))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(RichText::new("CONTROL CATALOG").strong().small());
                ui.add_space(5.0);
                horizontal_separator_with_palette(ui, &p);
                ui.add_space(5.0);
                for page in Page::ALL {
                    let response = ui.selectable_value(&mut self.page, page, page.label());
                    if response.clicked() {
                        self.last_action = format!("Opened {}", page.label());
                    }
                }
                ui.add_space(12.0);
                ui.label(RichText::new("STATE KEY").strong().small());
                ui.add_space(4.0);
                ui.label("Raised  - available");
                ui.label("Sunken  - editable");
                ui.label("Navy    - selected");
                ui.add_space(8.0);
                ui.small("Try each sample, then read its current state in the inspector.");
            });
    }

    fn draw_catalog(&mut self, ui: &mut Ui, p: Palette) {
        egui::ScrollArea::vertical()
            .id_salt("factory-catalog-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                match self.page {
                    Page::Overview => {
                        section(ui, p, "Widget Factory", "LIVE SHOWCASE", |ui| {
                            ui.label("Every sample is a working egui control. Edit values, change selections, and press buttons to exercise the theme in place.");
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                feature_chip(ui, p, "7 catalog pages");
                                feature_chip(ui, p, "Editable controls");
                                feature_chip(ui, p, "Live palette");
                            });
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
            });
    }

    fn show_buttons(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "Command buttons", "PUSH / DEFAULT / DISABLED", |ui| {
            ui.label("A raised edge becomes a sunken edge while the pointer is down.");
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
                if ClassicButton::new("Copy selection")
                    .palette(p)
                    .show(ui)
                    .clicked()
                {
                    self.last_action = "Copy selection clicked".to_owned();
                }
                if ClassicButton::new("Run sample")
                    .palette(p)
                    .show(ui)
                    .clicked()
                {
                    self.last_action = "Run sample clicked".to_owned();
                }
            });
        });
    }

    fn show_text_fields(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "Text fields", "SINGLE LINE / PASSWORD / MULTILINE", |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                let response = ui.add_sized(
                    [ui.available_width(), 24.0],
                    egui::TextEdit::singleline(&mut self.name)
                        .hint_text("Enter a name"),
                );
                if response.changed() {
                    self.last_action = "Name field edited".to_owned();
                }
                self.observe("Name field", &response);
            });
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Password:");
                let response = ui.add_sized(
                    [ui.available_width(), 24.0],
                    egui::TextEdit::singleline(&mut self.password)
                        .password(true)
                        .hint_text("Password"),
                );
                if response.changed() {
                    self.last_action = "Password field edited".to_owned();
                }
                self.observe("Password field", &response);
            });
            ui.add_space(5.0);
            ui.label("Notes:");
            let response = ui.add_sized(
                [ui.available_width(), 72.0],
                egui::TextEdit::multiline(&mut self.notes)
                    .desired_rows(3)
                    .hint_text("Write a note..."),
            );
            if response.changed() {
                self.last_action = "Notes field edited".to_owned();
            }
            self.observe("Multiline editor", &response);
        });
    }

    fn show_selection(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "Selection controls", "CHECKBOX / RADIO / COMBO BOX", |ui| {
            let response = ui.checkbox(&mut self.accepted, "Remember this choice");
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
                let response = egui::ComboBox::from_id_salt("factory-display-mode")
                    .selected_text(self.combo_choice.clone())
                    .width(160.0)
                    .show_ui(ui, |ui| {
                        for option in ["Standard", "Large icons", "Details", "List"] {
                            let choice =
                                ui.selectable_value(&mut self.combo_choice, option.to_owned(), option);
                            selection_changed |= choice.changed();
                        }
                    })
                    .response;
                if selection_changed {
                    self.last_action = format!("Display mode: {}", self.combo_choice);
                }
                self.observe("Display mode combo", &response);
            });
        });
    }

    fn show_ranges(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "Ranges and feedback", "SLIDER / PROGRESS", |ui| {
            let response = ui.add(
                egui::Slider::new(&mut self.slider, 0.0..=100.0)
                    .text("Volume")
                    .suffix("%"),
            );
            if response.changed() {
                self.last_action = format!("Slider moved to {:.0}%", self.slider);
            }
            self.observe("Volume slider", &response);

            ui.add_space(3.0);
            ui.add(
                egui::ProgressBar::new((self.progress / 100.0).clamp(0.0, 1.0))
                    .desired_width(ui.available_width())
                    .show_percentage(),
            );
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
        section(ui, p, "Containers", "RAISED / SUNKEN / TABS", |ui| {
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
            ui.horizontal(|ui| {
                for (index, title) in ["General", "Details", "About"].into_iter().enumerate() {
                    let response = ui.selectable_value(&mut self.active_tab, index, title);
                    if response.clicked() {
                        self.last_action = format!("Opened {title} tab");
                    }
                    self.observe("Tab", &response);
                }
            });
            let active_tab = self.active_tab;
            BevelFrame::sunken()
                .palette(p)
                .fill(p.input)
                .inner_margin(Margin::same(8))
                .show(ui, |ui| match active_tab {
                    0 => {
                        ui.label("General settings live inside the selected page.");
                        ui.checkbox(&mut self.accepted, "Enable this option");
                    }
                    1 => {
                        ui.label("Details include the current sample values.");
                        ui.label(format!("Name: {}", self.name));
                    }
                    _ => {
                        ui.label("egui-classic95");
                        ui.label("A small reusable Win95-inspired theme engine.");
                    }
                });

            ui.add_space(5.0);
            egui::CollapsingHeader::new("Advanced options")
                .default_open(false)
                .show(ui, |ui| {
                    ui.checkbox(&mut self.accepted, "Use classic spacing");
                    ui.label("Collapsible groups keep secondary controls close by.");
                });
        });
    }

    fn show_palette(&mut self, ui: &mut Ui, p: Palette) {
        section(ui, p, "System palette", "EDIT COLORS LIVE", |ui| {
            ui.label("Change the color wells to see the theme update across the factory.");
            ui.add_space(6.0);
            palette_editor(ui, "Desktop", &mut self.theme.palette.desktop);
            palette_editor(ui, "Window face", &mut self.theme.palette.face);
            palette_editor(ui, "Active title", &mut self.theme.palette.active_title);
            palette_editor(ui, "Selection", &mut self.theme.palette.selection);
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                swatch(ui, "Highlight", self.theme.palette.highlight, &self.theme.palette);
                swatch(ui, "Shadow", self.theme.palette.shadow, &self.theme.palette);
                swatch(ui, "Dark edge", self.theme.palette.dark_shadow, &self.theme.palette);
            });
        });
    }

    fn draw_inspector(&mut self, ui: &mut Ui, p: Palette) {
        BevelFrame::sunken()
            .palette(p)
            .fill(p.input)
            .inner_margin(Margin::same(8))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(RichText::new("INTERACTION PROBE").strong().small());
                ui.add_space(4.0);
                ui.label(RichText::new(&self.last_action).strong());
                ui.add_space(5.0);
                horizontal_separator_with_palette(ui, &p);
                ui.add_space(5.0);
                ui.label(RichText::new("Widget state").strong());
                ui.label(&self.widget_probe);
                ui.add_space(7.0);
                ui.label(RichText::new("Current page").strong());
                ui.label(self.page.label());
                ui.add_space(7.0);
                ui.label(RichText::new("Palette samples").strong());
                swatch(ui, "Face", p.face, &p);
                swatch(ui, "Active title", p.active_title, &p);
                swatch(ui, "Selection", p.selection, &p);
                swatch(ui, "Input", p.input, &p);
                ui.add_space(7.0);
                horizontal_separator_with_palette(ui, &p);
                ui.add_space(6.0);
                ui.label(RichText::new("Keyboard and pointer").strong());
                ui.small("Tab to focus. Click to press. Drag sliders and scroll the catalog.");
                ui.add_space(8.0);
                ui.label(RichText::new("LIVE VALUES").strong().small());
                ui.label(format!("Name: {}", self.name));
                ui.label(format!("Choice: {}", self.combo_choice));
                ui.label(format!("Volume: {:.0}%", self.slider));
            });
    }

    fn status_bar(&self, ui: &mut Ui, p: Palette) {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 21.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, p.face);
        paint_bevel(ui.painter(), rect, Bevel::Sunken, &p);
        ui.painter().text(
            rect.left_center() + vec2(6.0, 0.0),
            Align2::LEFT_CENTER,
            format!("{}  |  egui-classic95", self.page.label()),
            FontId::proportional(11.0),
            p.text,
        );
        ui.painter().line_segment(
            [rect.right_top() + vec2(-128.0, 3.0), rect.right_bottom() + vec2(-128.0, -3.0)],
            Stroke::new(1.0, p.shadow),
        );
        ui.painter().text(
            rect.right_center() + vec2(-7.0, 0.0),
            Align2::RIGHT_CENTER,
            "7 pages",
            FontId::proportional(11.0),
            p.text,
        );
    }

    fn observe(&mut self, label: &str, response: &Response) {
        let state = if response.is_pointer_button_down_on() {
            "pressed"
        } else if response.has_focus() {
            "focused"
        } else if response.hovered() {
            "hovered"
        } else {
            "idle"
        };
        self.widget_probe = format!("{label}: {state}");
    }

    fn reset_samples(&mut self) {
        self.name = "Jane User".to_owned();
        self.password = "win95".to_owned();
        self.notes = "Type here to try the recessed multiline editor.".to_owned();
        self.accepted = true;
        self.radio_choice = 0;
        self.combo_choice = "Standard".to_owned();
        self.slider = 68.0;
        self.progress = 42.0;
        self.active_tab = 0;
        self.theme.palette = Palette::default();
        self.last_action = "Sample values reset".to_owned();
    }
}

fn section(
    ui: &mut Ui,
    palette: Palette,
    title: &str,
    badge: &str,
    add_contents: impl FnOnce(&mut Ui),
) {
    BevelFrame::raised()
        .palette(palette)
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(title).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(badge).small().color(palette.shadow));
                });
            });
            ui.add_space(4.0);
            horizontal_separator_with_palette(ui, &palette);
            ui.add_space(6.0);
            add_contents(ui);
        });
    ui.add_space(8.0);
}

fn feature_chip(ui: &mut Ui, palette: Palette, label: &str) {
    BevelFrame::sunken()
        .palette(palette)
        .fill(palette.input)
        .inner_margin(Margin::same(4))
        .show(ui, |ui| {
            ui.label(RichText::new(label).small());
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

#[derive(Clone, Copy)]
enum CaptionGlyph {
    Minimize,
    Maximize,
    Close,
}

fn caption_button(
    ui: &mut Ui,
    rect: Rect,
    id: &'static str,
    glyph: CaptionGlyph,
    palette: Palette,
) -> Response {
    let response = ui.interact(rect, Id::new(id), Sense::click());
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

    let stroke = Stroke::new(1.4, palette.text);
    match glyph {
        CaptionGlyph::Minimize => {
            ui.painter().line_segment(
                [
                    rect.left_bottom() + vec2(5.0, -4.0),
                    rect.right_bottom() + vec2(-5.0, -4.0),
                ],
                stroke,
            );
        }
        CaptionGlyph::Maximize => {
            ui.painter().rect_stroke(
                rect.shrink(4.0),
                CornerRadius::ZERO,
                stroke,
                StrokeKind::Inside,
            );
        }
        CaptionGlyph::Close => {
            ui.painter().line_segment(
                [rect.left_top() + vec2(5.0, 4.0), rect.right_bottom() + vec2(-5.0, -4.0)],
                stroke,
            );
            ui.painter().line_segment(
                [rect.right_top() + vec2(-5.0, 4.0), rect.left_bottom() + vec2(5.0, -4.0)],
                stroke,
            );
        }
    }
    response
}

fn draw_app_icon(painter: &egui::Painter, top_left: egui::Pos2) {
    let size = 6.0;
    for (offset, color) in [
        (vec2(0.0, 0.0), Color32::from_rgb(0xFF, 0x40, 0x40)),
        (vec2(7.0, 0.0), Color32::from_rgb(0x40, 0xC0, 0x40)),
        (vec2(0.0, 7.0), Color32::from_rgb(0x40, 0x60, 0xFF)),
        (vec2(7.0, 7.0), Color32::from_rgb(0xFF, 0xD0, 0x20)),
    ] {
        painter.rect_filled(
            Rect::from_min_size(top_left + offset, vec2(size, size)),
            CornerRadius::ZERO,
            color,
        );
    }
}
