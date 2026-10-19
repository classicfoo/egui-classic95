use egui::emath::GuiRounding;
use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, InnerResponse, Margin, Painter, Rect, Response,
    RichText, Sense, Stroke, Ui, UiBuilder, Vec2, Widget, pos2, vec2,
};
use std::ops::RangeInclusive;
use std::{cell::Cell, rc::Rc};

use crate::theme::palette_for;
use crate::{Bevel, BevelFrame, Palette, classic_popup_style, paint_bevel};

/// An egui text editor inside the library's classic sunken input frame.
pub struct ClassicTextEdit<'a> {
    edit: egui::TextEdit<'a>,
    size: Option<Vec2>,
    palette: Option<Palette>,
}

impl<'a> ClassicTextEdit<'a> {
    /// Wrap an egui text editor in a reusable classic input frame.
    pub fn new(edit: egui::TextEdit<'a>) -> Self {
        Self {
            edit,
            size: None,
            palette: None,
        }
    }

    /// Set the outside size of the framed text editor in egui points.
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = Some(size.max(Vec2::ZERO));
        self
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Show the text editor and return its egui response.
    pub fn show(self, ui: &mut Ui) -> Response {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        BevelFrame::sunken()
            .palette(palette)
            .fill(palette.input)
            .inner_margin(Margin::same(2))
            .show(ui, |ui| {
                let edit = self.edit.frame(Frame::NONE.inner_margin(Margin {
                    left: 2,
                    right: 2,
                    top: 0,
                    bottom: 0,
                }));
                if let Some(size) = self.size {
                    ui.add_sized([(size.x - 4.0).max(1.0), (size.y - 4.0).max(1.0)], edit)
                } else {
                    ui.add(edit)
                }
            })
            .inner
    }
}

impl Widget for ClassicTextEdit<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// A classic dialog group box with a label interrupting its top edge.
#[derive(Clone, Debug)]
pub struct ClassicGroupBox {
    title: String,
    palette: Option<Palette>,
    fill: Option<Color32>,
    inner_margin: Margin,
    stretch: bool,
}

impl ClassicGroupBox {
    /// Create a group box with application-provided heading text.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            palette: None,
            fill: None,
            inner_margin: Margin {
                left: 10,
                right: 10,
                top: 3,
                bottom: 10,
            },
            stretch: true,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set the group box background color.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Set the padding around the heading and contents.
    pub fn inner_margin(mut self, margin: Margin) -> Self {
        self.inner_margin = margin;
        self
    }

    /// Keep the group box at the width available from its parent UI.
    pub fn stretch(mut self, stretch: bool) -> Self {
        self.stretch = stretch;
        self
    }

    /// Show the labeled group and return its inner response.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let frame = Frame::new()
            .fill(self.fill.unwrap_or(palette.face))
            .stroke(Stroke::NONE)
            .inner_margin(self.inner_margin)
            .show(ui, |ui| {
                if self.stretch {
                    ui.set_min_width(ui.available_width());
                }
                let heading = ui.label(RichText::new(self.title).strong());
                ui.add_space(5.0);
                let inner = add_contents(ui);
                (inner, heading.rect)
            });
        paint_group_box(ui, frame.response.rect, frame.inner.1, palette);
        InnerResponse {
            inner: frame.inner.0,
            response: frame.response,
        }
    }
}

/// A row of selectable classic tabs joined to a raised content panel.
#[derive(Debug)]
pub struct ClassicTabs<'a> {
    titles: Vec<String>,
    selected: &'a mut usize,
    palette: Option<Palette>,
    id_salt: egui::Id,
    panel_margin: Margin,
}

/// Results from showing a [`ClassicTabs`] control.
pub struct ClassicTabsResponse<R> {
    /// Selected tab index after processing input.
    pub selected: usize,
    /// Whether clicking a tab changed the selected index.
    pub changed: bool,
    /// Responses for each tab in the same order as the labels.
    pub tabs: Vec<Response>,
    /// The raised panel and its application-defined contents.
    pub content: InnerResponse<R>,
}

impl<'a> ClassicTabs<'a> {
    /// Create tabs from application-provided labels and selected index.
    pub fn new<I, S>(titles: I, selected: &'a mut usize) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            titles: titles.into_iter().map(Into::into).collect(),
            selected,
            palette: None,
            id_salt: egui::Id::NULL.with("classic95-tabs"),
            panel_margin: Margin::same(8),
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set a stable identifier when multiple tab controls share the same UI.
    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_salt = egui::Id::NULL.with(salt);
        self
    }

    /// Set the padding inside the raised content panel.
    pub fn panel_margin(mut self, margin: Margin) -> Self {
        self.panel_margin = margin;
        self
    }

    /// Show the tab row and panel, passing the selected index to app content.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui, usize) -> R,
    ) -> ClassicTabsResponse<R> {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let mut selected = if self.titles.is_empty() {
            0
        } else {
            (*self.selected).min(self.titles.len() - 1)
        };
        *self.selected = selected;

        let old_spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        let mut tab_rects = Vec::with_capacity(self.titles.len());
        let mut tab_responses = Vec::with_capacity(self.titles.len());
        let tab_text_color = palette.text;
        ui.horizontal(|ui| {
            for (index, title) in self.titles.iter().enumerate() {
                let width = ui
                    .painter()
                    .layout_no_wrap(title.clone(), FontId::proportional(13.0), tab_text_color)
                    .size()
                    .x
                    .ceil()
                    + 14.0;
                let (_, rect) = ui.allocate_space(vec2(width, 19.0));
                let hit_rect = if index == selected {
                    rect
                } else {
                    Rect::from_min_max(pos2(rect.left(), rect.top() + 2.0), rect.right_bottom())
                };
                let mut response = ui.interact(hit_rect, self.id_salt.with(index), Sense::click());
                if response.clicked() && selected != index {
                    selected = index;
                    *self.selected = index;
                    response.mark_changed();
                }
                tab_rects.push(rect);
                tab_responses.push(response);
            }
        });

        let content = BevelFrame::raised()
            .palette(palette)
            .fill(palette.face)
            .inner_margin(self.panel_margin)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                add_contents(ui, selected)
            });
        ui.spacing_mut().item_spacing = old_spacing;
        paint_classic_tabs(
            ui,
            palette,
            &self.titles,
            &tab_rects,
            content.response.rect,
            selected,
        );

        ClassicTabsResponse {
            selected,
            changed: selected != *self.selected || tab_responses.iter().any(Response::changed),
            tabs: tab_responses,
            content,
        }
    }
}

/// A checkbox with the library's crisp pixel-drawn Win95 mark.
pub struct ClassicCheckbox<'a> {
    checked: &'a mut bool,
    label: String,
    palette: Option<Palette>,
}

impl<'a> ClassicCheckbox<'a> {
    /// Create a checkbox that updates `checked` when clicked.
    pub fn new(checked: &'a mut bool, label: impl Into<String>) -> Self {
        Self {
            checked,
            label: label.into(),
            palette: None,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Show the checkbox and return its egui response.
    pub fn show(self, ui: &mut Ui) -> Response {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let response = ui.checkbox(self.checked, self.label);
        let origin = pos2(
            response.rect.left(),
            response.rect.center().y - CHECKBOX_PIXELS.len() as f32 * 0.5,
        )
        .round_to_pixels(ui.pixels_per_point());
        paint_bitmap_backdrop(
            ui,
            origin,
            vec2(
                CHECKBOX_PIXELS[0].len() as f32,
                CHECKBOX_PIXELS.len() as f32,
            ),
            palette.face,
        );
        paint_palette_bitmap(ui, origin, &CHECKBOX_PIXELS, palette);
        if *self.checked {
            paint_pixel_bitmap(
                ui.painter(),
                origin + vec2(3.0, 3.0),
                &CHECK_MARK_PIXELS,
                palette.text,
            );
        }
        response
    }
}

impl Widget for ClassicCheckbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// A radio button with the library's crisp pixel-drawn Win95 mark.
pub struct ClassicRadioButton<'a, Value> {
    selected: &'a mut Value,
    value: Value,
    label: String,
    palette: Option<Palette>,
}

impl<'a, Value> ClassicRadioButton<'a, Value> {
    /// Create a radio option that selects `value` when clicked.
    pub fn new(selected: &'a mut Value, value: Value, label: impl Into<String>) -> Self {
        Self {
            selected,
            value,
            label: label.into(),
            palette: None,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }
}

impl<Value: PartialEq + Copy> ClassicRadioButton<'_, Value> {
    /// Show the radio option and return its egui response.
    pub fn show(self, ui: &mut Ui) -> Response {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let response = ui.radio_value(self.selected, self.value, self.label);
        let (_, icon_rect) = ui.spacing().icon_rectangles(response.rect);
        let icon_size = vec2(RADIO_PIXELS[0].len() as f32, RADIO_PIXELS.len() as f32);
        let origin = (icon_rect.center() - icon_size * 0.5).round_to_pixels(ui.pixels_per_point());
        ui.painter().rect_filled(
            Rect::from_center_size(icon_rect.center(), icon_rect.size() + vec2(4.0, 4.0)),
            CornerRadius::ZERO,
            palette.face,
        );
        paint_palette_bitmap(ui, origin, &RADIO_PIXELS, palette);
        if *self.selected == self.value {
            paint_pixel_bitmap(
                ui.painter(),
                origin + vec2(4.0, 4.0),
                &RADIO_MARK_PIXELS,
                palette.text,
            );
        }
        response
    }
}

impl<Value: PartialEq + Copy> Widget for ClassicRadioButton<'_, Value> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// A classic rail and rectangular thumb over egui's standard slider interaction.
pub struct ClassicSlider<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    palette: Option<Palette>,
    show_value: bool,
    trailing_fill: bool,
    handle_aspect_ratio: f32,
}

impl<'a> ClassicSlider<'a> {
    /// Create a slider over a floating point range.
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            palette: None,
            show_value: false,
            trailing_fill: false,
            handle_aspect_ratio: 0.55,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Choose whether egui's formatted value appears beside the slider.
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    /// Choose whether the slider paints the selected range fill.
    pub fn trailing_fill(mut self, trailing_fill: bool) -> Self {
        self.trailing_fill = trailing_fill;
        self
    }

    /// Set the egui handle aspect ratio while retaining the classic painted thumb.
    pub fn handle_aspect_ratio(mut self, aspect_ratio: f32) -> Self {
        self.handle_aspect_ratio = aspect_ratio.max(0.01);
        self
    }

    /// Show the slider and return its egui response.
    pub fn show(self, ui: &mut Ui) -> Response {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let response = ui.add(
            egui::Slider::new(self.value, self.range.clone())
                .show_value(self.show_value)
                .trailing_fill(self.trailing_fill)
                .handle_shape(egui::style::HandleShape::Rect {
                    aspect_ratio: self.handle_aspect_ratio,
                }),
        );
        paint_classic_slider(
            ui,
            palette,
            response.rect,
            *self.value,
            response.is_pointer_button_down_on(),
            self.range,
        );
        response
    }
}

impl Widget for ClassicSlider<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// A segmented classic progress display for application-provided progress values.
#[derive(Clone, Copy, Debug)]
pub struct ClassicProgressBar {
    value: f32,
    size: Vec2,
    palette: Option<Palette>,
    segment_width: f32,
    gap: f32,
}

impl ClassicProgressBar {
    /// Create a segmented progress bar with a percentage value from 0 to 100.
    pub fn new(value: f32) -> Self {
        Self {
            value,
            size: vec2(120.0, 18.0),
            palette: None,
            segment_width: 8.0,
            gap: 2.0,
        }
    }

    /// Set the display size in egui points.
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set the width of each filled segment.
    pub fn segment_width(mut self, width: f32) -> Self {
        self.segment_width = width.max(1.0);
        self
    }

    /// Set the gap between segments.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    /// Paint the progress bar and return its hover response.
    pub fn show(self, ui: &mut Ui) -> Response {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, palette.face);
        paint_bevel(ui.painter(), rect, Bevel::Sunken, &palette);
        let inner = rect.shrink(3.0);
        let count = ((inner.width() + self.gap) / (self.segment_width + self.gap)).floor() as usize;
        let filled = ((self.value.clamp(0.0, 100.0) / 100.0) * count as f32).round() as usize;
        for index in 0..filled.min(count) {
            let x = inner.left() + index as f32 * (self.segment_width + self.gap);
            ui.painter().rect_filled(
                Rect::from_min_size(
                    pos2(x, inner.top()),
                    vec2(self.segment_width.min(inner.right() - x), inner.height()),
                ),
                CornerRadius::ZERO,
                palette.selection,
            );
        }
        response
    }
}

impl Widget for ClassicProgressBar {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// Paint the raised button and arrow used at the end of a classic combo box.
#[derive(Clone, Copy, Debug, Default)]
pub struct ClassicComboArrow {
    palette: Option<Palette>,
}

impl ClassicComboArrow {
    /// Create a combo-box arrow using the palette installed in the egui context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Paint the arrow into a rectangle supplied by egui's combo-box icon callback.
    pub fn show(self, ui: &Ui, rect: Rect) {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let enabled = ui.is_enabled();
        let pressed = enabled
            && ui.input(|input| {
                input.pointer.button_down(egui::PointerButton::Primary)
                    && input
                        .pointer
                        .interact_pos()
                        .is_some_and(|pointer_pos| rect.contains(pointer_pos))
            });
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, palette.face);
        paint_bevel(
            ui.painter(),
            rect,
            if pressed {
                Bevel::Pressed
            } else {
                Bevel::Raised
            },
            &palette,
        );
        let center = rect.center() + if pressed { vec2(1.0, 1.0) } else { Vec2::ZERO };
        ui.painter().add(egui::Shape::convex_polygon(
            vec![
                pos2(center.x - 4.0, center.y - 2.0),
                pos2(center.x + 4.0, center.y - 2.0),
                pos2(center.x, center.y + 2.0),
            ],
            if enabled {
                palette.text
            } else {
                palette.disabled_text
            },
            Stroke::NONE,
        ));
    }
}

/// A classic framed combo box that uses egui for its popup interaction.
pub struct ClassicComboBox {
    selected_text: String,
    id_salt: egui::Id,
    width: f32,
    arrow_width: f32,
    enabled: bool,
    palette: Option<Palette>,
}

impl ClassicComboBox {
    /// Create a combo box with the text shown for its current value.
    pub fn new(selected_text: impl Into<String>) -> Self {
        Self {
            selected_text: selected_text.into(),
            id_salt: egui::Id::NULL.with("classic95-combo-box"),
            width: 120.0,
            arrow_width: 22.0,
            enabled: true,
            palette: None,
        }
    }

    /// Set a stable identifier when multiple combo boxes share a UI.
    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_salt = egui::Id::NULL.with(salt);
        self
    }

    /// Set the outside width of the combo box in egui points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(0.0);
        self
    }

    /// Set the width of the raised arrow button in egui points.
    pub fn arrow_width(mut self, width: f32) -> Self {
        self.arrow_width = width.max(1.0);
        self
    }

    /// Enable or disable interaction with the combo box.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Show the combo box and return egui's response and optional popup contents.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<Option<R>> {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let enabled = self.enabled;
        ui.add_enabled_ui(enabled, |ui| {
            let arrow_rect = Rc::new(Cell::new(None));
            let icon_arrow_rect = Rc::clone(&arrow_rect);
            let id_salt = self.id_salt;
            let selected_text = self.selected_text;
            let width = self.width;
            let arrow_width = self.arrow_width;
            let frame = BevelFrame::sunken()
                .palette(palette)
                .fill(if enabled { palette.input } else { palette.face })
                .inner_margin(Margin::same(1))
                .show(ui, |ui| {
                    let combo_id = ui.make_persistent_id(id_salt);
                    let popup_is_open = egui::ComboBox::is_open(ui.ctx(), combo_id);
                    let combo_has_focus = ui
                        .ctx()
                        .read_response(combo_id)
                        .is_some_and(|response| response.has_focus());
                    let arrow_key_pressed = ui.input(|input| {
                        input.key_pressed(egui::Key::ArrowUp)
                            || input.key_pressed(egui::Key::ArrowDown)
                    });
                    if !popup_is_open && combo_has_focus && arrow_key_pressed {
                        egui::Popup::open_id(ui.ctx(), combo_id.with("popup"));
                    }

                    ui.spacing_mut().icon_width = arrow_width;
                    ui.visuals_mut().button_frame = false;
                    egui::ComboBox::from_id_salt(id_salt)
                        .selected_text(selected_text)
                        .width(width)
                        .icon(move |_ui, rect, _visuals, _is_open| {
                            icon_arrow_rect.set(Some(rect));
                        })
                        .popup_style(egui::style::StyleModifier::new(move |style| {
                            classic_popup_style(style, &palette);
                            style.visuals.popup_shadow = egui::Shadow::NONE;
                            style.visuals.window_fill = palette.input;
                            style.visuals.window_stroke = Stroke::new(1.0, palette.text);
                            style.visuals.window_corner_radius = CornerRadius::ZERO;
                            style.visuals.menu_corner_radius = CornerRadius::ZERO;
                            style.spacing.menu_margin = Margin::ZERO;
                            style.spacing.item_spacing.y = 0.0;
                            style.spacing.interact_size.y = 20.0;
                            style.spacing.button_padding = vec2(3.0, 0.0);
                        }))
                        .show_ui(ui, add_contents)
                });
            if let Some(rect) = arrow_rect.get() {
                ClassicComboArrow::new().palette(palette).show(ui, rect);
            }
            frame.inner
        })
        .inner
    }
}

/// Direction for a classic scrollbar arrow button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollButtonDirection {
    /// Draw an up arrow.
    Up,
    /// Draw a down arrow.
    Down,
}

/// A raised or pressed classic scrollbar arrow button.
#[derive(Clone, Copy, Debug)]
pub struct ClassicScrollButton {
    direction: ScrollButtonDirection,
    palette: Option<Palette>,
    id: Option<egui::Id>,
}

impl ClassicScrollButton {
    /// Create an arrow button with the requested direction.
    pub fn new(direction: ScrollButtonDirection) -> Self {
        Self {
            direction,
            palette: None,
            id: None,
        }
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set a stable identifier when using a custom rectangle.
    pub fn id(mut self, id: egui::Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Draw the button at a caller-provided rectangle.
    pub fn show_at(self, ui: &mut Ui, rect: Rect) -> Response {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let id = self.id.unwrap_or_else(|| ui.next_auto_id());
        let response = ui.interact(rect, id, Sense::click());
        let pressed = response.is_pointer_button_down_on();
        paint_scrollbar_bevel(
            ui.painter(),
            rect,
            if pressed {
                Bevel::Pressed
            } else {
                Bevel::Raised
            },
            &palette,
        );
        let center = rect.center() + if pressed { vec2(1.0, 1.0) } else { Vec2::ZERO };
        let points = if self.direction == ScrollButtonDirection::Down {
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
}

/// A reusable vertical classic scrollbar for application-managed offsets.
///
/// It draws end arrows, a stippled shaft, and a proportional draggable thumb.
pub struct ClassicScrollBar<'a> {
    offset: &'a mut f32,
    max_offset: f32,
    viewport_length: f32,
    height: f32,
    scroll_step: f32,
    palette: Option<Palette>,
    id: egui::Id,
}

impl<'a> ClassicScrollBar<'a> {
    /// Width of the classic vertical scrollbar, including its arrow buttons.
    pub const WIDTH: f32 = 16.0;

    /// Create a scrollbar for an application-managed scroll offset.
    pub fn new(offset: &'a mut f32, max_offset: f32, viewport_length: f32, height: f32) -> Self {
        Self {
            offset,
            max_offset: max_offset.max(0.0),
            viewport_length: viewport_length.max(0.0),
            height: height.max(0.0),
            scroll_step: 32.0,
            palette: None,
            id: egui::Id::NULL.with("classic95-scrollbar"),
        }
    }

    /// Set the distance moved by one arrow-button click.
    pub fn scroll_step(mut self, step: f32) -> Self {
        self.scroll_step = step.max(1.0);
        self
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set a stable identifier when multiple scrollbars share a UI.
    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id = egui::Id::NULL.with(salt);
        self
    }

    /// Show the scrollbar and update the supplied offset in response to input.
    pub fn show(self, ui: &mut Ui) -> Response {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let (bar_rect, bar_response) =
            ui.allocate_exact_size(vec2(Self::WIDTH, self.height), Sense::hover());
        *self.offset = self.offset.clamp(0.0, self.max_offset);
        if self.max_offset <= 0.0 || self.height < 36.0 {
            return bar_response;
        }

        let arrow_size = Self::WIDTH.min(self.height * 0.5);
        let up_rect = Rect::from_min_size(bar_rect.min, vec2(Self::WIDTH, arrow_size));
        let down_rect = Rect::from_min_size(
            pos2(bar_rect.left(), bar_rect.bottom() - arrow_size),
            vec2(Self::WIDTH, arrow_size),
        );
        if ClassicScrollButton::new(ScrollButtonDirection::Up)
            .palette(palette)
            .id(self.id.with("up"))
            .show_at(ui, up_rect)
            .clicked()
        {
            *self.offset = (*self.offset - self.scroll_step).max(0.0);
        }
        if ClassicScrollButton::new(ScrollButtonDirection::Down)
            .palette(palette)
            .id(self.id.with("down"))
            .show_at(ui, down_rect)
            .clicked()
        {
            *self.offset = (*self.offset + self.scroll_step).min(self.max_offset);
        }

        let track_rect = Rect::from_min_max(
            pos2(bar_rect.left(), up_rect.bottom()),
            pos2(bar_rect.right(), down_rect.top()),
        );
        let track_response = ui.interact(track_rect, self.id.with("track"), Sense::click());
        ui.painter()
            .rect_filled(track_rect, CornerRadius::ZERO, palette.light);
        for y in (track_rect.top() as i32..track_rect.bottom() as i32).step_by(2) {
            for x in (track_rect.left() as i32..track_rect.right() as i32).step_by(2) {
                if ((x / 2) + (y / 2)) & 1 == 0 {
                    ui.painter().rect_filled(
                        Rect::from_min_size(pos2(x as f32, y as f32), vec2(1.25, 1.25)),
                        CornerRadius::ZERO,
                        palette.highlight,
                    );
                }
            }
        }

        let track_height = track_rect.height().max(1.0);
        let content_height = self.viewport_length + self.max_offset;
        let thumb_height = ((track_height * self.viewport_length / content_height.max(1.0))
            .max(18.0))
        .min(track_height);
        let thumb_travel = (track_height - thumb_height).max(0.0);
        let thumb_top =
            track_rect.top() + thumb_travel * (*self.offset / self.max_offset).clamp(0.0, 1.0);
        let thumb_rect = Rect::from_min_size(
            pos2(bar_rect.left(), thumb_top),
            vec2(track_rect.width(), thumb_height),
        );
        let thumb = ui.interact(thumb_rect, self.id.with("thumb"), Sense::drag());
        if track_response.clicked() && !thumb.hovered() {
            let pointer_y = track_response
                .interact_pointer_pos()
                .map_or(thumb_rect.center().y, |pos| pos.y);
            let direction = if pointer_y < thumb_rect.top() {
                -1.0
            } else {
                1.0
            };
            *self.offset = (*self.offset + direction * self.viewport_length * 0.85)
                .clamp(0.0, self.max_offset);
        }
        let drag_start_id = self.id.with("drag-start");
        if thumb.drag_started_by(egui::PointerButton::Primary) {
            ui.ctx()
                .data_mut(|data| data.insert_temp(drag_start_id, *self.offset));
        }
        if thumb.dragged_by(egui::PointerButton::Primary) {
            let start = ui
                .ctx()
                .data(|data| data.get_temp::<f32>(drag_start_id))
                .unwrap_or(*self.offset);
            let drag_distance = thumb.total_drag_delta().map_or(0.0, |delta| delta.y);
            *self.offset = (start + drag_distance * self.max_offset / thumb_travel.max(1.0))
                .clamp(0.0, self.max_offset);
        }
        if thumb.drag_stopped_by(egui::PointerButton::Primary) {
            ui.ctx()
                .data_mut(|data| data.remove_temp::<f32>(drag_start_id));
        }

        paint_scrollbar_bevel(
            ui.painter(),
            thumb_rect,
            if thumb.is_pointer_button_down_on() {
                Bevel::Pressed
            } else {
                Bevel::Raised
            },
            &palette,
        );
        bar_response
    }
}

fn paint_scrollbar_bevel(painter: &Painter, rect: Rect, bevel: Bevel, palette: &Palette) {
    painter.rect_filled(rect, CornerRadius::ZERO, palette.face);
    paint_bevel(painter, rect, bevel, palette);
}

/// A classic status strip with an inset border and pixel-drawn resize grip.
#[derive(Clone, Copy, Debug)]
pub struct ClassicStatusBar {
    height: f32,
    palette: Option<Palette>,
}

impl ClassicStatusBar {
    /// Create a status strip with a 25 point height.
    pub fn new() -> Self {
        Self {
            height: 25.0,
            palette: None,
        }
    }

    /// Set the status strip height in egui points.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(1.0);
        self
    }

    /// Use a local palette instead of the one installed in the egui context.
    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Show application-defined status content and return its outer response.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui, Rect) -> R,
    ) -> InnerResponse<R> {
        let palette = self.palette.unwrap_or_else(|| palette_for(ui.ctx()));
        let visible_width = (ui.clip_rect().right() - ui.cursor().left()).max(0.0);
        let width = ui.available_width().min(visible_width);
        let (rect, response) = ui.allocate_exact_size(vec2(width, self.height), Sense::hover());
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, palette.face);
        paint_status_border(ui.painter(), rect, palette);
        let content = ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            ui.shrink_clip_rect(rect);
            add_contents(ui, rect)
        });
        paint_size_grip(ui, rect, palette);
        InnerResponse {
            inner: content.inner,
            response,
        }
    }
}

impl Default for ClassicStatusBar {
    fn default() -> Self {
        Self::new()
    }
}

fn paint_group_box(ui: &Ui, rect: Rect, heading: Rect, palette: Palette) {
    let painter = ui.painter();
    let top = heading.center().y;
    let left_gap = (heading.left() - 4.0).max(rect.left() + 4.0);
    let right_gap = (heading.right() + 4.0).min(rect.right() - 4.0);
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
    titles: &[String],
    tabs: &[Rect],
    panel: Rect,
    selected: usize,
) {
    let mut tab_palette = palette;
    tab_palette.light = Color32::from_rgb(0xDF, 0xDF, 0xDF);
    tab_palette.dark_shadow = Color32::BLACK;
    let panel = panel.round_to_pixels(ui.pixels_per_point());
    paint_bevel(ui.painter(), panel, Bevel::Raised, &tab_palette);
    for (index, tab) in tabs.iter().copied().enumerate() {
        if index != selected {
            paint_classic_tab(ui, palette, &tab_palette, tab, panel, &titles[index], false);
        }
    }
    if let Some(tab) = tabs.get(selected).copied() {
        paint_classic_tab(
            ui,
            palette,
            &tab_palette,
            tab,
            panel,
            &titles[selected],
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
    let top = slot.top() + if selected { 0.0 } else { 2.0 };
    let bottom = if selected {
        panel.top() + 2.0
    } else {
        panel.top()
    };
    let tab = Rect::from_min_max(pos2(slot.left(), top), pos2(slot.right(), bottom))
        .round_to_pixels(ui.pixels_per_point());
    ui.painter()
        .rect_filled(tab, CornerRadius::ZERO, palette.face);
    let left = tab.left();
    let right = tab.right();
    let side_end = if selected {
        panel.top() + 1.0
    } else {
        panel.top()
    };
    paint_tab_pixels(
        ui.painter(),
        left + 2.0,
        top,
        right - left - 4.0,
        1.0,
        edge_palette.highlight,
    );
    paint_tab_pixels(
        ui.painter(),
        left + 1.0,
        top + 1.0,
        right - left - 3.0,
        1.0,
        edge_palette.highlight,
    );
    paint_tab_pixels(
        ui.painter(),
        left + 2.0,
        top + 1.0,
        right - left - 4.0,
        1.0,
        edge_palette.light,
    );
    paint_tab_pixels(
        ui.painter(),
        right - 2.0,
        top + 1.0,
        2.0,
        1.0,
        edge_palette.dark_shadow,
    );
    paint_tab_pixels(ui.painter(), right - 1.0, top + 1.0, 1.0, 2.0, palette.face);
    let side_height = side_end - top - 2.0;
    paint_tab_pixels(
        ui.painter(),
        left,
        top + 2.0,
        1.0,
        side_height,
        edge_palette.highlight,
    );
    paint_tab_pixels(
        ui.painter(),
        left + 1.0,
        top + 2.0,
        1.0,
        side_height,
        edge_palette.light,
    );
    paint_tab_pixels(
        ui.painter(),
        right - 2.0,
        top + 2.0,
        1.0,
        side_height,
        edge_palette.shadow,
    );
    paint_tab_pixels(
        ui.painter(),
        right - 1.0,
        top + 2.0,
        1.0,
        side_height,
        edge_palette.dark_shadow,
    );
    ui.painter().text(
        pos2(slot.center().x, (top + panel.top()) * 0.5),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(13.0),
        palette.text,
    );
}

fn paint_tab_pixels(painter: &Painter, x: f32, y: f32, width: f32, height: f32, color: Color32) {
    painter.rect_filled(
        Rect::from_min_size(pos2(x, y), vec2(width, height)),
        CornerRadius::ZERO,
        color,
    );
}

fn paint_classic_slider(
    ui: &Ui,
    palette: Palette,
    rect: Rect,
    value: f32,
    pressed: bool,
    range: RangeInclusive<f32>,
) {
    let half_handle = (rect.height() / 2.5) * 0.55;
    let left = rect.left() + half_handle;
    let right = rect.right() - half_handle;
    let range_start = *range.start();
    let range_end = *range.end();
    let amount = if range_start == range_end {
        0.0
    } else {
        ((value - range_start) / (range_end - range_start)).clamp(0.0, 1.0)
    };
    let center_x = egui::lerp(left..=right, amount);
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

fn paint_bitmap_backdrop(ui: &Ui, origin: egui::Pos2, size: Vec2, color: Color32) {
    let rect = Rect::from_min_size(origin - vec2(1.0, 1.0), size + vec2(2.0, 2.0));
    ui.painter().rect_filled(rect, CornerRadius::ZERO, color);
}

fn paint_palette_bitmap(ui: &Ui, origin: egui::Pos2, rows: &[&str], palette: Palette) {
    let origin = origin.round_to_pixels(ui.pixels_per_point());
    let mut mesh = egui::epaint::Mesh::default();
    for (row, pixels) in rows.iter().enumerate() {
        for (column, pixel) in pixels.bytes().enumerate() {
            let color = match pixel {
                b's' => palette.shadow,
                b'#' => palette.text,
                b'W' => palette.highlight,
                _ => palette.face,
            };
            let rect =
                Rect::from_min_size(origin + vec2(column as f32, row as f32), vec2(1.0, 1.0));
            mesh.add_colored_rect(rect, color);
        }
    }
    ui.painter()
        .add(egui::Shape::Mesh(std::sync::Arc::new(mesh)));
}

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

fn paint_status_border(painter: &Painter, rect: Rect, palette: Palette) {
    let left = rect.left() + 0.5;
    let top = rect.top() + 0.5;
    let right = rect.right() - 0.5;
    let bottom = rect.bottom() - 0.5;
    painter.line_segment(
        [pos2(left, top), pos2(right, top)],
        Stroke::new(1.0, palette.shadow),
    );
    painter.line_segment(
        [pos2(left, top), pos2(left, bottom)],
        Stroke::new(1.0, palette.shadow),
    );
    painter.line_segment(
        [pos2(left, bottom), pos2(right, bottom)],
        Stroke::new(1.0, palette.highlight),
    );
    painter.line_segment(
        [pos2(right, top), pos2(right, bottom)],
        Stroke::new(1.0, palette.highlight),
    );
}

const STATUS_SIZE_GRIP_PIXELS: [&str; 13] = [
    ".............",
    "............W",
    "...........WW",
    "..........WWs",
    ".........WWss",
    "........WWssW",
    ".......WWssWW",
    "......WWssWWs",
    ".....WWssWWss",
    "....WWssWWssW",
    "...WWssWWssWW",
    "..WWssWWssWWs",
    ".WWssWWssWWss",
];

fn paint_size_grip(ui: &Ui, rect: Rect, palette: Palette) {
    let origin =
        pos2(rect.right() - 14.0, rect.bottom() - 14.0).round_to_pixels(ui.pixels_per_point());
    let mut mesh = egui::epaint::Mesh::default();
    for (row, pixels) in STATUS_SIZE_GRIP_PIXELS.iter().enumerate() {
        for (column, pixel) in pixels.bytes().enumerate() {
            let color = match pixel {
                b's' => palette.shadow,
                b'W' => palette.highlight,
                _ => continue,
            };
            mesh.add_colored_rect(
                Rect::from_min_size(origin + vec2(column as f32, row as f32), vec2(1.0, 1.0)),
                color,
            );
        }
    }
    ui.painter()
        .add(egui::Shape::Mesh(std::sync::Arc::new(mesh)));
}

const CHECKBOX_PIXELS: [&str; 13] = [
    "ssssssssssssW",
    "s##########.W",
    "s#WWWWWWWWW.W",
    "s#WWWWWWWWW.W",
    "s#WWWWWWWWW.W",
    "s#WWWWWWWWW.W",
    "s#WWWWWWWWW.W",
    "s#WWWWWWWWW.W",
    "s#WWWWWWWWW.W",
    "s#WWWWWWWWW.W",
    "s#WWWWWWWWW.W",
    "s...........W",
    "WWWWWWWWWWWWW",
];
const CHECK_MARK_PIXELS: [&str; 7] = [
    "......#", ".....##", "#...###", "##.###.", "#####..", ".###...", "..#....",
];
const RADIO_PIXELS: [&str; 13] = [
    "....ssss.....",
    "..s######s...",
    ".s#sWWWWW#s..",
    ".#sWWWWWWWs..",
    "s#WWWWWWWWWsW",
    "s#WWWWWWWWWsW",
    "s#WWWWWWWWWsW",
    "s#WWWWWWWWWsW",
    ".#WWWWWWWWWsW",
    ".s#WWWWWWWsW.",
    "..ssWWWWWs.W.",
    "....sssssWW..",
    "....WWWWW....",
];
const RADIO_MARK_PIXELS: [&str; 5] = ["#####", "#####", "#####", "#####", "#####"];
