# egui-classic95

A small reusable theme engine for building Windows 95-inspired interfaces with [egui](https://github.com/emilk/egui). Its default system palette uses silver control faces, navy active captions, teal desktop color, and square corners. Raised and recessed edges are painted from the classic light-and-shadow bevel convention.

The **Widget Factory** is the project centerpiece: a runnable control gallery with editable fields, live buttons, selection controls, sliders, progress, containers, palette editing, and an interaction inspector.

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
```

Install the theme once during startup, then use the widgets anywhere in the UI:

```rust
use egui_classic95::{BevelFrame, Classic95, ClassicButton};

fn setup(ctx: &egui::Context) {
    Classic95::default().install(ctx);
}

fn controls(ui: &mut egui::Ui) {
    if ui.add(ClassicButton::new("Apply")).clicked() {
        // Apply changes.
    }

    BevelFrame::sunken().show(ui, |ui| {
        ui.label("A recessed content well");
    });
}
```

`ClassicButton`, `BevelFrame`, and `horizontal_separator` read the palette installed in the egui context. Pass `.palette(my_palette)` to a widget when you want a local override.

## Public pieces

- `Classic95` installs a square-cornered light style with compact spacing and a rectangular slider handle.
- `Palette` exposes the classic system colors and can be edited or replaced.
- `Bevel` and `paint_bevel` paint raised, pressed, sunken, and window edge conventions.
- `ClassicButton` draws a keyboard-focusable push button and supports a default-button outline.
- `BevelFrame` wraps arbitrary egui content in a raised, sunken, or window-style panel.
- `horizontal_separator` and `horizontal_separator_with_palette` draw an etched divider.

The library uses egui's standard controls for text editing, checkboxes, radio buttons, combo boxes, sliders, progress bars, tabs, and collapsible groups, so their state and keyboard behavior stay native to egui while inheriting the installed palette.

## Design references

The visual language is informed by *The Windows Interface Guidelines for Software Design* (Microsoft Press, 1995, ISBN 1-55615-679-0), [BevelDesk](https://github.com/marchildmann/BevelDesk), [98.css](https://github.com/jdan/98.css), egui's [custom window frame example](https://github.com/emilk/egui/tree/main/examples/custom_window_frame), and [egui_styled](https://github.com/dspray95/egui_styled).

This is an independent homage. It ships no Microsoft assets or proprietary fonts.

## License

MIT. See [LICENSE](LICENSE).
