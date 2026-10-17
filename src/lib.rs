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
mod palette;
mod theme;
mod widgets;

pub use bevel::{Bevel, paint_bevel};
pub use palette::Palette;
pub use theme::Classic95;
pub use widgets::{
    BevelFrame, ClassicButton, horizontal_separator, horizontal_separator_with_palette,
};
