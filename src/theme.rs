use egui::style::HandleShape;
use egui::{
    Color32, Context, CornerRadius, FontId, Id, Margin, Stroke, Style, TextStyle, Theme, vec2,
};

use crate::Palette;

/// Configurable Windows 95-inspired egui theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Classic95 {
    /// The semantic color palette used by the theme and custom widgets.
    pub palette: Palette,
}

impl Classic95 {
    /// Create a theme with a custom palette.
    pub const fn new(palette: Palette) -> Self {
        Self { palette }
    }

    /// Return the egui style configured with this theme's colors and metrics.
    pub fn style(&self) -> Style {
        let p = self.palette;
        let mut style = Style::default();
        let visuals = &mut style.visuals;

        visuals.dark_mode = false;
        visuals.override_text_color = Some(p.text);
        visuals.hyperlink_color = p.accent;
        visuals.faint_bg_color = p.light;
        visuals.extreme_bg_color = p.input;
        visuals.text_edit_bg_color = Some(p.input);
        visuals.window_fill = p.face;
        visuals.window_stroke = Stroke::new(1.0, p.dark_shadow);
        visuals.window_corner_radius = CornerRadius::ZERO;
        visuals.menu_corner_radius = CornerRadius::ZERO;
        visuals.panel_fill = p.face;
        visuals.selection.bg_fill = p.selection;
        visuals.selection.stroke = Stroke::new(1.0, p.selection_text);
        visuals.button_frame = true;
        visuals.collapsing_header_frame = true;
        visuals.striped = false;
        visuals.handle_shape = HandleShape::Rect { aspect_ratio: 0.7 };

        set_widget(&mut visuals.widgets.noninteractive, p.face, p.shadow, p.text);
        set_widget(&mut visuals.widgets.inactive, p.face, p.dark_shadow, p.text);
        set_widget(&mut visuals.widgets.hovered, p.face, p.text, p.text);
        set_widget(&mut visuals.widgets.active, p.face, p.dark_shadow, p.text);
        set_widget(&mut visuals.widgets.open, p.face, p.dark_shadow, p.text);

        style.spacing.item_spacing = vec2(8.0, 6.0);
        style.spacing.button_padding = vec2(8.0, 3.0);
        style.spacing.window_margin = Margin::same(8);
        style.spacing.menu_margin = Margin::same(2);
        style.spacing.interact_size = vec2(20.0, 22.0);
        style.spacing.slider_width = 140.0;
        style.spacing.scroll = egui::style::ScrollStyle::solid();
        style.animation_time = 0.0;
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(13.0));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(13.0));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(11.0));
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::proportional(17.0));
        style
            .text_styles
            .insert(TextStyle::Monospace, FontId::monospace(12.0));

        style
    }

    /// Install the theme for the current egui context.
    ///
    /// This selects egui's light theme and also updates the style used by new
    /// menus and popups. Call it during setup, or again after changing the
    /// palette.
    pub fn install(&self, ctx: &Context) {
        let style = self.style();
        ctx.data_mut(|data| {
            data.insert_temp(Id::new("egui-classic95-palette"), self.palette);
        });
        ctx.set_style_of(Theme::Light, style.clone());
        ctx.set_theme(Theme::Light);
        ctx.set_global_style(style);
    }
}

pub(crate) fn palette_for(ctx: &Context) -> Palette {
    ctx.data(|data| {
        data.get_temp::<Palette>(Id::new("egui-classic95-palette"))
            .unwrap_or_default()
    })
}

fn set_widget(
    widget: &mut egui::style::WidgetVisuals,
    fill: Color32,
    outline: Color32,
    text: Color32,
) {
    widget.bg_fill = fill;
    widget.weak_bg_fill = fill;
    widget.bg_stroke = Stroke::new(1.0, outline);
    widget.fg_stroke = Stroke::new(1.0, text);
    widget.corner_radius = CornerRadius::ZERO;
    widget.expansion = 0.0;
}
