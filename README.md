# egui-classic95

A small reusable theme engine for building Windows 95-inspired interfaces with [egui](https://github.com/emilk/egui). Its default system palette uses silver control faces, navy active captions, teal desktop color, and square corners. Raised and recessed edges are painted from the classic light-and-shadow bevel convention.

The **Widget Factory** is the project centerpiece: a runnable control gallery with editable fields, live buttons, selection controls, report and tree views, a reusable toolbar, tooltips, sliders, progress, containers, palette editing, and an interaction inspector.

On native platforms, `Classic95::install` uses the system's generic sans-serif font for proportional text, so the app follows the host's font configuration without bundling a font. Font discovery runs in the background; egui's built-in fonts remain available as fallbacks. On WebAssembly, it keeps egui's built-in fonts.

## Run the Widget Factory

```sh
cargo run --example widget_factory
```

## Add the theme to an egui app

```toml
[dependencies]
egui = "0.37"
egui-classic95 = "0.1"
eframe = "0.37" # needed when using eframe
```

Installing `Classic95` sets egui's theme, palette, font preferences, and standard widget styling. `ClassicWindow` draws the classic frame and title bar, supports dragging and the standard window controls, and provides the standard system-menu actions. Apps supply the title, icon, client contents, and any app-specific system-menu items.

Install the theme once during startup, then compose the reusable controls and window chrome:

```rust
use egui_classic95::{
    BevelFrame, Classic95, ClassicButton, ClassicListBox, ClassicListView,
    ClassicListViewColumn, ClassicMenuBar, ClassicMenuItem, ClassicMenuPopup,
    ClassicWindow, ClassicToolbar, ClassicToolbarButton,
    ClassicTooltip, ClassicTreeNode, ClassicTreeView,
};

fn setup(ctx: &egui::Context) {
    Classic95::default().install(ctx);
}

// eframe apps disable native decorations because ClassicWindow draws them.
fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_decorations(false),
        ..Default::default()
    }
}

fn app_window(ui: &mut egui::Ui, show_toolbar: &mut bool) {
    ClassicWindow::new("My application").show_with_system_menu(
        ui,
        |_painter, _icon_rect| {
            // Draw an application-specific icon here.
        },
        |ui| {
            if ui.add(ClassicMenuItem::button("About")).clicked() {
                // Run an application-specific action.
            }
        },
        |ui| {
            ClassicMenuBar::new().show(ui, |ui| {
                ui.menu_button("View", |ui| {
                    ClassicMenuPopup::new().show(ui, |ui| {
                        ui.add(ClassicMenuItem::check("Show toolbar", show_toolbar));
                    });
                });
            });

            if ui.add(ClassicButton::new("Apply")).clicked() {
                // Apply changes.
            }
            BevelFrame::sunken().show(ui, |ui| {
                ui.label("A recessed content well");
            });
        },
    );
}
```

Keep the selected list index in app state to use the reusable list box:

```rust
let list = ClassicListBox::new(["General", "Details", "Advanced"], &mut self.selected_page)
    .id_salt("property-pages")
    .max_height(160.0)
    .show(ui);
if let Some(index) = list.clicked {
    // Respond to the user's selection.
}
```

Wrap egui text editing and combo-box behavior in reusable classic controls:

```rust
let name = ClassicTextEdit::new(
    egui::TextEdit::singleline(&mut self.name).hint_text("Enter a name"),
)
.size(egui::vec2(ui.available_width(), 24.0))
.show(ui);

let display_mode = ClassicComboBox::new(self.display_mode.clone())
    .id_salt("display-mode")
    .width(156.0)
    .show(ui, |ui| {
        for option in ["Standard", "Large icons", "Details"] {
            ui.selectable_value(&mut self.display_mode, option.to_owned(), option);
        }
    });
```

Use a report view for tabular data and keep the selected source row in application state:

```rust
let view = ClassicListView::new(
    [
        ClassicListViewColumn::new("Name").width(150.0),
        ClassicListViewColumn::new("Type"),
    ],
    [["Readme.txt", "Text document"], ["Setup.exe", "Application"]],
    &mut self.selected_file,
)
.max_height(180.0)
.show(ui);
```

Build a tree from `ClassicTreeNode`s. The selection is a path of child indexes; expansion state is kept by egui using the supplied salt:

```rust
let nodes = [
    ClassicTreeNode::new("Desktop")
        .expanded(true)
        .child(ClassicTreeNode::new("My Computer")),
];
let tree = ClassicTreeView::new(nodes, &mut self.selected_folder)
    .id_salt("folders")
    .max_height(180.0)
    .show(ui);
```

Compose a toolbar from reusable buttons, separators, and application-defined content:

```rust
ClassicToolbar::new().show(ui, |ui| {
    if ClassicToolbarButton::new("Open")
        .show(ui, |painter, icon_rect| {
            // Paint the application's icon.
        })
        .clicked()
    {
        // Open the selected item.
    }
});
```

Attach a themed tooltip to any egui response. It keeps egui's hover delay, follows the pointer, and adds an accessible description:

```rust
let response = ui.button("Help");
let response = ClassicTooltip::new("Explains what this command does.")
    .show_for(response);
```

The title, icon, menu labels, menu actions, and page contents remain application-owned. Caption button responses are returned from `ClassicTitleBar::show`; the application sends the matching viewport commands or uses them for its own window actions.

`ClassicButton`, `BevelFrame`, and `horizontal_separator` read the palette installed in the egui context. Pass `.palette(my_palette)` to a component when you want a local override.

## Public pieces

- `Classic95` installs a square-cornered light style with compact spacing and a rectangular slider handle.
- `Palette` exposes the classic system colors and can be edited or replaced.
- `Bevel` and `paint_bevel` paint raised, pressed, sunken, and window edge conventions.
- `ClassicButton` draws a keyboard-focusable push button and supports a default-button outline.
- `BevelFrame` wraps arbitrary egui content in a raised, sunken, or window-style panel.
- `horizontal_separator` and `horizontal_separator_with_palette` draw an etched divider.
- `ClassicWindowFrame`, `ClassicTitleBar`, and `CaptionButton` provide the raised clipped window frame and pixel-drawn title controls.
- `ClassicSystemMenu`, `ClassicMenuBar`, `ClassicMenuPopup`, `ClassicMenuItem`, and `ClassicMenuSeparator` supply classic menu styling while leaving menu content to the application.
- `ClassicGroupBox` and `ClassicTabs` provide the classic labeled group and raised tabbed panel.
- `ClassicScrollBar` provides app-managed vertical scrolling with end arrows, a stippled shaft, and a proportional draggable thumb. `ClassicListView` and `ClassicTreeView` reuse it when their rows overflow.
- `ClassicListBox` provides a recessed list with full-width selectable rows and optional scrolling.
- `ClassicListView` provides a single-selection report view with beveled column headers, full-width row selection, keyboard navigation, and optional scrolling.
- `ClassicTreeNode` and `ClassicTreeView` provide indented, selectable trees with dotted connectors, plus/minus expanders, keyboard navigation, and optional scrolling.
- `ClassicToolbar` provides a reusable toolbar panel for application-defined controls. Its raised `ClassicToolbarButton`s can show a pressed selected state.
- `ClassicTooltip` adds delayed, pointer-positioned pale-yellow tooltips to egui responses, including disabled controls.
- `ClassicCheckbox`, `ClassicRadioButton`, `ClassicSlider`, `ClassicProgressBar`, `ClassicComboArrow`, `ClassicScrollButton`, `ClassicStatusBar`, and `ClassicStatusPane` provide reusable control visuals.
- `ClassicTextEdit` wraps egui text editing in a classic sunken input frame; `ClassicComboBox` applies a classic field, arrow, and popup style while keeping egui's selection behavior.

`ClassicTextEdit`, `ClassicComboBox`, `ClassicCheckbox`, `ClassicRadioButton`, and `ClassicSlider` keep egui's editing and interaction behavior while drawing reusable classic visuals. Collapsible groups remain egui controls; `ClassicTabs` provides a reusable classic tab row and panel.

## Design references

The visual language is informed by *The Windows Interface Guidelines for Software Design* (Microsoft Press, 1995, ISBN 1-55615-679-0), including its list view, tree view, toolbar, and tooltip guidance. Control comparisons also use period Windows 95 Explorer screenshots ([split tree and file panes](https://www.cs.oberlin.edu/~rms/mmcc/a/mod6/mod6411.html), [report list and toolbar](https://www.tech2u.com.au/training/tech2u/win95_1/copy.html)), [BevelDesk](https://github.com/marchildmann/BevelDesk), [98.css](https://github.com/jdan/98.css), egui's [custom window frame example](https://github.com/emilk/egui/tree/main/examples/custom_window_frame), and [egui_styled](https://github.com/dspray95/egui_styled).

This is an independent homage. It ships no Microsoft assets or proprietary fonts.

## License

MIT. See [LICENSE](LICENSE).
