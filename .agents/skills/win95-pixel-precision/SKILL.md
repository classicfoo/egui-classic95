---
name: win95-pixel-precision
description: Compare and refine egui-classic95 controls against original Windows 95 screenshots using measured pixels. Use for requests about Win95 visual authenticity or a control that looks wrong; not for general visual redesign.
---

# Win95 Pixel Precision

Use this workflow when a user asks for a pixel-accurate Windows 95 control or points out a visual mismatch.

## Compare the right reference

- Find a genuine Windows 95 screenshot or original UI resource showing the same kind of control and state. Prefer an original-resolution image and record its source URL, OS/version, and visible state.
- Keep the source image unchanged. Inspect the control at native scale, then make a nearest-neighbor enlargement for edge and corner inspection. Treat resampling, display scaling, palette changes, and later Windows versions as possible sources of mismatch.
- Measure the target control's bounds, text margins, gaps, baseline, connected edges, corner steps, and one-pixel color paths. Sample RGB values directly where the reference permits it.
- Do not assume every Win95 control uses the same bevel recipe. Compare the specific control: tabs, title buttons, sliders, and ordinary buttons may use different edge colors or geometry.

## Make and verify the change

- Preserve explicit project choices such as the system sans-serif font. Fix geometry and painting in egui code; derive text-fitting dimensions from the font actually used instead of unrelated hard-coded widths.
- Keep interaction rectangles aligned with the visible control. For an interactive control, verify its click, drag, or selected state along with its pixels.
- Build and run `widget_factory`. If no display is attached, use the project's 1200×850 Xvfb and Pillow `ImageGrab` workflow. Capture the same state as the reference and compare aligned crops at 1× and with nearest-neighbor enlargement.
- Report which details match exactly and which remain approximate because the reference uses different text, font rasterization, scaling, or a changed system palette. Keep preview crops in `/tmp`; do not add reference images to the repository unless asked.
- Follow the repository's `AGENTS.md` workflow for diff checks, commits, and pushes.
