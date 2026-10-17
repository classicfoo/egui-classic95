use egui::Color32;

/// Named colors used by the classic desktop palette.
///
/// The fields are public so applications can recolor the theme without
/// replacing its geometry or widget behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    /// Desktop background (`#008080`).
    pub desktop: Color32,
    /// Standard window and control face (`#C0C0C0`).
    pub face: Color32,
    /// Outer top and left highlight (`#FFFFFF`).
    pub highlight: Color32,
    /// Inner top and left highlight (`#DFDFDF`).
    pub light: Color32,
    /// Inner bottom and right shadow (`#808080`).
    pub shadow: Color32,
    /// Outer bottom and right shadow (`#0A0A0A`).
    pub dark_shadow: Color32,
    /// Normal text (`#000000`).
    pub text: Color32,
    /// Disabled text (`#808080`).
    pub disabled_text: Color32,
    /// Active caption bar (`#000080`).
    pub active_title: Color32,
    /// Inactive caption bar (`#808080`).
    pub inactive_title: Color32,
    /// Caption text (`#FFFFFF`).
    pub title_text: Color32,
    /// Text entry background (`#FFFFFF`).
    pub input: Color32,
    /// Selection background (`#000080`).
    pub selection: Color32,
    /// Selection text (`#FFFFFF`).
    pub selection_text: Color32,
    /// Small emphasis color, used for links and selection marks.
    pub accent: Color32,
}

impl Palette {
    /// The familiar Windows 95 silver, navy, and teal system colors.
    pub const fn windows_95() -> Self {
        Self {
            desktop: Color32::from_rgb(0x00, 0x80, 0x80),
            face: Color32::from_rgb(0xC0, 0xC0, 0xC0),
            highlight: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            light: Color32::from_rgb(0xDF, 0xDF, 0xDF),
            shadow: Color32::from_rgb(0x80, 0x80, 0x80),
            dark_shadow: Color32::from_rgb(0x0A, 0x0A, 0x0A),
            text: Color32::from_rgb(0x00, 0x00, 0x00),
            disabled_text: Color32::from_rgb(0x80, 0x80, 0x80),
            active_title: Color32::from_rgb(0x00, 0x00, 0x80),
            inactive_title: Color32::from_rgb(0x80, 0x80, 0x80),
            title_text: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            input: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            selection: Color32::from_rgb(0x00, 0x00, 0x80),
            selection_text: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            accent: Color32::from_rgb(0x00, 0x00, 0x80),
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::windows_95()
    }
}
