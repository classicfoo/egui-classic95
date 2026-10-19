//! A small Windows 95-inspired theme and widget toolkit for [`egui`].
//!
//! Install [`Classic95`] once during application setup, then use the reusable
//! widgets and bevel primitives wherever you need classic desktop chrome.
//!
//! ```no_run
//! # fn main() {
//! let ctx = egui::Context::default();
//! egui_classic95::Classic95::default().install(&ctx);
//!
//! let _ = ctx.run_ui(Default::default(), |ui| {
//!     if egui_classic95::ClassicButton::new("Open")
//!         .show(ui)
//!         .clicked()
//!     {
//!         // Open a document.
//!     }
//! });
//! # }
//! ```

mod bevel;
mod chrome;
mod controls;
mod listbox;
mod listview;
mod menu;
mod palette;
mod theme;
mod toolbar;
mod tooltip;
mod treeview;
mod widgets;

pub use bevel::{Bevel, paint_bevel};
pub use chrome::{
    CaptionButton, CaptionButtonKind, ClassicSystemMenu, ClassicTitleBar, ClassicTitleBarResponse,
    ClassicWindow, ClassicWindowFrame,
};
pub use controls::{
    ClassicCheckbox, ClassicComboArrow, ClassicComboBox, ClassicGroupBox, ClassicProgressBar,
    ClassicRadioButton, ClassicScrollBar, ClassicScrollButton, ClassicSlider, ClassicStatusBar,
    ClassicStatusPane, ClassicTabs, ClassicTabsResponse, ClassicTextEdit, ScrollButtonDirection,
};
pub use listbox::{ClassicListBox, ClassicListBoxResponse};
pub use listview::{ClassicListView, ClassicListViewColumn, ClassicListViewResponse};
pub use menu::{
    ClassicMenuBar, ClassicMenuItem, ClassicMenuPopup, ClassicMenuSeparator, ClassicToolbarButton,
    classic_menu_style, classic_popup_style, vertical_separator, vertical_separator_with_palette,
};
pub use palette::Palette;
pub use theme::Classic95;
pub use toolbar::ClassicToolbar;
pub use tooltip::ClassicTooltip;
pub use treeview::{ClassicTreeNode, ClassicTreeView, ClassicTreeViewResponse};
pub use widgets::{
    BevelFrame, ClassicButton, horizontal_separator, horizontal_separator_with_palette,
};
