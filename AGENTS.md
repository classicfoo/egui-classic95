# Project workflow

## Library parity

- Treat `src/` and the public exports in `src/lib.rs` as the source of reusable behavior, interaction, accessibility, layout, styling, defaults, and API documentation. This applies to feature and bug fixes as well as visual refinements.
- Keep the Widget Factory as a real consumer of the library. Use public `egui-classic95` APIs for effects the library owns; do not leave library-relevant behavior or styling as an example-only implementation.
- When a showcase change exposes a missing reusable capability, add or extend the public library API, document it, and update the Widget Factory to use that API before committing.
- Review each change from a consumer's perspective: an otherwise standard egui app should get the same library-owned look and behavior by installing `Classic95` and using the public components, without copying code from the Widget Factory.

## Widget Factory previews

- Treat `examples/widget_factory.rs` as the visual showcase for the library.
- For requests to match original Windows 95 pixels, use `.agents/skills/win95-pixel-precision/SKILL.md`.
- For requested UI or theme changes, build it with `cargo build --example widget_factory`, run it, capture a fresh screenshot, and show the screenshot in the response.
- If no display is attached, run the example under Xvfb at 1200×850 and capture the screen with Pillow's `ImageGrab`. Keep preview images in `/tmp` unless the user asks to add them to the repository.
- Keep exploratory font and style comparisons isolated to previews. When the user chooses a variant, update the project default.

## Git workflow

- Before editing or pushing, inspect `git status` and synchronize with `origin` so existing work is preserved.
- After each completed code change, inspect the diff, run `git diff --check`, and make a concise commit when applicable.
- Push completed commits to `origin main` so the user's clone can update with `git pull`. Never force-push or discard remote changes.
- In the final response, report the commit hash and branch, and say whether the build and screenshot succeeded.
