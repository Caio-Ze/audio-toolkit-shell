# Resizer and Layout Audit (Audio Toolkit Shell)

This archive documents where all resizing and dimensioning logic currently lives, explains why the current layout feels broken, and proposes a robust interactive resizing design for the four-terminal UI.

Date: 2025-08-08

---

## Main Goal (TL;DR)

- **Left column fixed to 40% width**: Terminal 1 (left `SidePanel`) occupies exactly 40% of the window width (non-resizable for now). This was validated in-app.
- **Buttons aligned to Terminal 1**: The buttons container lives under Terminal 1 and occupies 35% of page height and 40% of page width (aligned to the left column). Terminal 1 uses the upper 65% of the left column.
- **Right cluster bottom fixed to 35% height**: Terminal 4 (bottom row) occupies exactly 35% of the total page height. The top row (Terminals 2 & 3) uses the remaining 65% height.
- **Top-row split (T2 vs T3)**: Must be dimensionable (interactive) with a visible divider. Likewise, the split between top (2/3) and bottom (4) must be dimensionable with a visible divider.

## Current Broken Behavior (What you are seeing)

- **Left divider feels locked or unreliable**:
  - The handle is hard to grab near the bottom because the action button grid sits close to the panel edge; the margin may be too small in practice.
  - The decorative 1px divider line is only painted (non-interactive) and can mislead visually.
- **Right cluster dividers do not work at all**:
  - The layout uses `egui_extras::StripBuilder` which, in this version, has no interactive splitters. So the visual boundaries for 2↔3 and (2/3)↔4 are not draggable.
- **Buttons container is not resizable**:
  - It is currently fixed (~68px height) and cannot be adjusted, which forces awkward layouts, early scrolling, and makes the left edge harder to grab near the bottom. The buttons container must itself be resizable to be usable.
- **Buttons look too small/imbalanced**:
  - Compact styling made buttons hard to parse and may contribute to the left-edge grab being unreliable.

### New findings (after introducing custom splitters)

- **Right-cluster divider not draggable due to Z-order/interaction precedence**:
  - In `src-tauri/src/app.rs`, helpers `split_vertical()` and `split_horizontal()` register the handle interaction before the terminal content is rendered in `update()`.
  - `render_terminal_panel()` adds a large click-to-focus `interact` region that spans most of each pane. Because egui gives precedence to the last-added interactive widget in overlapping areas, the terminal click zones steal the pointer from the handles.
  - Result: the visual bar appears but dragging does nothing. Fix by registering/painting splitter handles after the pane content (so handles are on top), or by using an overlay pass.

- **Left/right divider felt immovable**:
  - The 1px painter line between left and right is decorative and non-interactive; attempting to drag it won’t resize.
  - Even though `SidePanel::left(...).resizable(true)` is enabled, the large click zones on the right cluster can overlap at the boundary and win the interaction, making the main divider feel locked. Increasing the right gutter on the left panel helps, but the core fix is to ensure splitter/edge interactions are registered after content so they take precedence.

- **Terminal 1 appears too small**:
  - The left width default is currently ~35% of `screen_w` (see `update()` around width calculations). Combined with the divider interaction issue, it appears “stuck small.” The new strategy intentionally fixes it at 35% width.

- **Mouse wheel scrolling synchronized across Terminals 2/3/4**:
  - Symptom: Scrolling over any right-cluster terminal scrolls all three at once.
  - Likely cause: `egui::ScrollArea` identity collision (or shared auto id) across panels, so scroll state is shared.
  - Fix applied: give each terminal output its own stable id via `.id_source(ui.id().with(("terminal_output_scroll", tab.title())))` inside `render_terminal_panel()` so hover/scroll only affects that pane.
  - Status: Implemented in `src-tauri/src/app.rs` (`render_terminal_panel`); pending validation.

### Current regression (after switching to fixed rects for validation)

- The right-cluster divisions between Terminals 2, 3, and 4 are currently missing in the UI. This happened because we temporarily replaced interactive splitters with fixed rect computations to validate sizing. We must restore:
  - A horizontal, interactive handle (and visible divider) between Terminal 2 and Terminal 3.
  - A vertical, interactive handle (and visible divider) between the top row (2/3) and the bottom row (4).
  - Proper Z-order so handles reliably capture pointer input.

## Expected Behavior (How it should work)

- **Left SidePanel divider** (`src-tauri/src/app.rs`, `SidePanel::left("terminal_1")`):
  - `.resizable(true)` with a comfortable grab radius (~12px) and a small content gutter (8–12px) so no widgets sit on top of the edge.
  - Always leaves at least a minimum width for the right cluster; the divider can be dragged freely along the whole height.
- **Right cluster** (`CentralPanel`):
  - Two custom interactive splitters (one vertical, one horizontal) implemented with `ui.interact(handle_rect, id, Sense::drag())` and handle thickness ~10–12px.
  - Fractions update live in app state and are clamped by pixel-based minimums so panes never collapse.
  - No overlap between handles; handles highlight on hover/drag for clear affordance.
- **Buttons container resizable**: The container holding the action buttons under Terminal 1 is vertically resizable within the left `SidePanel` (its own internal splitter), with sensible min heights for both Terminal 1 and the buttons area. The buttons content scrolls inside that container and never overlaps the panel edge.
- **Input routing**: Clicking any terminal focuses it; only the focused tab’s PTY receives keystrokes. Ctrl+C/D work as signals; platform shortcuts are not hijacked.

### 1) Where the resizer and dimensioning code live today

This section maps the exact code locations that define the main split and the internal right-cluster layout, as well as any dimensioning constants and user-facing hitboxes affecting resize behavior.

#### 1) Main left/right split (SidePanel vs right area)

- File: `src-tauri/src/app.rs`
- Function: `AudioToolkitApp::update()`
- Lines (approx): 954–1008 and 981–992
  - Computes constraints and defaults for the left panel width:
    - `min_left_width`, `min_right_width`, `allow_zero_collapse` read from `AppSettings`.
    - Ensures right cluster visibility: `min_visible_right` and clamps `max_left`.
    - Sets `default_left` for initial panel width.
  - Builds the left panel:
    - `let left_panel = egui::SidePanel::left("terminal_1")`
    - `.resizable(true)`
    - `.default_width(default_left)`
    - `.min_width(min_left)`
    - `.width_range(min_left..=max_left)`
    - `.frame(... .inner_margin(egui::Margin { right: 14.0, .. }) ...)`
      - The 14px right inner margin is intended to keep content off the resizer handle so the handle remains easy to grab.

- Divider line between left and right:
  - Lines (approx): 1243–1253
  - Painter: `ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, Id::new("divider_left_right")))`
  - Draw: `painter.vline(x, ctx.screen_rect().y_range(), Stroke { width: 1.0, color: CatppuccinTheme::FRAPPE.surface1 })`
  - Note: This is decorative only and does not accept input; it should not block the resizer.

- Global grab handle size:
  - Lines (approx): 930–934
  - `style.interaction.resize_grab_radius_side = 12.0;` (wider hitbox for panel edges)

### 2) Right cluster layout (terminals 2/3/4)

- File: `src-tauri/src/app.rs`
- Function: `AudioToolkitApp::update()`
- Lines (approx): 1171–1241
- Implementation:
  - Uses `egui::CentralPanel::default()` to host the right cluster.
  - Inside the CentralPanel, uses `egui_extras::StripBuilder` with relative sizes:
    - Vertical split: top (relative: `top_frac`) vs bottom (relative: `1.0 - top_frac`).
    - Top row uses a horizontal `StripBuilder` split between left and right via `hsplit` and `1.0 - hsplit`.
  - Fractions come from `AppSettings` and are clamped:
    - `right_top_fraction` (height of the top row)
    - `right_top_hsplit_fraction` (width of the top-left top-right split)

- Important limitation today:
  - `StripBuilder` here is not interactive in the current dependency version (no `.resizable(true)` available). As a result, the “divider” for terminals 2 vs 3 and the divider for top vs bottom (3 vs 4) do not accept drag interactions.
  - This matches your observation that “terminal 3 and 4 divider [is] totally not working.” The dividers exist visually by relative layout but are not draggable.

### 3) Buttons area (under Terminal 1)

- File: `src-tauri/src/app.rs`
- Function: `AudioToolkitApp::update()`
- Lines (approx): 997–1168
- Notes:
  - Terminal 1 UI consumes `ui.available_height() - 68.0`, leaving 68px for actions.
  - A compact 2-column grid is rendered inside a vertical `ScrollArea`.
  - The right inner margin of 14px on the SidePanel (see main split) is intended to keep content away from the resizer handle; however, if the margin is too small or alignment is off, the SidePanel edge can feel hard to grab.

### 4) Application settings that affect dimensions

- File: `src-tauri/src/config.rs`
- Struct: `AppSettings` (lines ~65–84), defaults at ~153–163
  - `min_left_width: f32` — default 120.0
  - `min_right_width: f32` — default 120.0
  - `allow_zero_collapse: bool` — default false
  - `right_top_fraction: f32` — default 0.6 (top row height share)
  - `right_top_hsplit_fraction: f32` — default 0.5 (top-left width share)

### 5) Where things likely need to move/change

- Keep the main left/right split as a `SidePanel::left(...).resizable(true)`. That part is appropriate and should stay in `update()`.
- Replace ad‑hoc resizing with the new fixed-percentage layout for predictability:
  - Left column fixed at 35% width (non-resizable for now) and buttons container fixed to 35% height within it.
  - Right cluster bottom fixed at 35% height; top row uses 65%.
  - Top-row split (T2 vs T3) may remain interactive or be fixed (default 50%). If interactive, register handle interactions after content to avoid Z-order issues.
  - Ensure decorative dividers are painted but never intercept input (painters are fine). Only handle rects should be pointer targets.

---
### Plan v2 — fixed layout + interactive right cluster

#### Targets
- **Left column**: width = 0.40 × window width; non-resizable for now.
- **Buttons container**: height = 0.35 × window height within the left column; width aligned (0.40 × window width). Terminal 1 uses remaining 65% height of the left column.
- **Right cluster**: bottom (Terminal 4) height = 0.35 × window height; top row height = 0.65 × window height.
- **Right-cluster splits**: both splits are interactive and have visible dividers:
  - Horizontal split: Terminal 2 vs Terminal 3.
  - Vertical split: Top row (2/3) vs Bottom row (4).

#### Implementation tasks
1) Left column
   - Compute `left_w = ctx.screen_rect().width() * 0.40` each frame.
   - Lock the panel width with `.width_range(left_w..=left_w)` and consider `.resizable(false)` so no grab affordance is shown.
   - Keep a small right inner margin (8–12px) for visual breathing room only; it won’t be a resizer anymore.

2) Buttons container (inside left column)
   - Set `left_buttons_frac = 0.35` of total page height (or compute pixel height directly from `ctx.screen_rect().height()` and the panel’s available rect).
   - Render Terminal 1 in the upper 65% of the left column; render the buttons container in the lower 35% with a vertical `ScrollArea`.
   - Remove/disable the internal draggable handle for the buttons area (non-interactive in v2).

3) Right cluster (CentralPanel)
   - Set defaults: `right_top_frac = 0.65` (top row) and `right_hsplit_frac = 0.50` (T2 vs T3). Bottom is 0.35 (Terminal 4).
   - Implement two interactive splitters with visible dividers:
     - Vertical: top (2/3) vs bottom (4).
     - Horizontal: T2 vs T3.
   - Register/paint handles after pane content (or in an overlay pass) to guarantee pointer capture. Handle thickness 10–12 px. Maintain min px clamps (e.g., top/bottom ≥ 140 px; left/right ≥ 160 px).

4) Interaction ordering (critical fix)
   - For any remaining interactive splitter, call `ui.interact(handle_rect, ...)` and paint the handle after drawing the pane content (`render_terminal_panel(...)`). This prevents click zones from stealing pointer events.
   - Alternatively, render handles in a dedicated overlay pass at the end of `update()`.

5) Persistence
   - For v2 (fixed layout), runtime fractions can be recomputed each frame. If you later re-enable interactivity, persist fractions to `config.toml` on exit.

6) Validation
   - Verify left column is exactly 40% width, buttons exactly 35% of page height within it (aligned to 40% width), and bottom terminal exactly 35% height.
   - Verify right-cluster splits are dimensionable with visible dividers (T2↔T3 and (2/3)↔4) and capture pointer reliably.
   - Re-check focus/input routing and confirm no visual or pointer overlaps.

---
### Proposed implementation (detailed plan + example code)

Below is a concrete, low-risk approach to add interactive splitters without changing your main architecture.

### A) Add runtime fields for split state

- File: `src-tauri/src/app.rs`
- Struct: `AudioToolkitApp`
- Add fields (initialized from `AppSettings` in `new()`):
  - `right_top_frac: f32` — starts at `config.app.right_top_fraction`
  - `right_hsplit_frac: f32` — starts at `config.app.right_top_hsplit_fraction`
- Rationale: `AppSettings` provides defaults, but interactive dragging should mutate runtime state. Optionally persist back to config on exit.

### B) Implement custom splitter helpers

Implement two helpers that take a `Ui`, a mutable fraction, minimum pixel sizes, and draw/handle a draggable bar:

- `split_vertical(ui, fraction, min_top_px, min_bottom_px, handle_thickness_px, id)` returns `(top_rect, handle_rect, bottom_rect)` and mutates `fraction` by drag events.
- `split_horizontal(ui, fraction, min_left_px, min_right_px, handle_thickness_px, id)` returns `(left_rect, handle_rect, right_rect)` and mutates `fraction` by drag events.

Key details:
- Use `let rect = ui.available_rect_before_wrap_finite();`
- Compute child rects from `fraction` and total size.
- Create a handle rect of thickness 8–12px centered on the split line.
- Register interaction: `let resp = ui.interact(handle_rect, id, egui::Sense::drag());`
- On drag, update fraction with `delta / total_dim`, clamp using min px converted to min fraction so neither side collapses below e.g. 120px.
- Paint the handle (rounded rect + subtle 1px stroke) with theme colors (e.g., `surface1` stroke, `overlay0` on hover/active).

This guarantees reliable, smooth resizing independent of `StripBuilder` limitations.

### C) Replace right cluster `StripBuilder` with custom splitters

Inside the `egui::CentralPanel::default().show(ctx, |ui| { ... })` block:

1) Reserve the full right-panel `rect`.
2) Call `split_vertical(ui, &mut self.right_top_frac, min_top_px, min_bottom_px, 10.0, Id::new("right_vsplit"))` to split top vs bottom.
3) For the top rect, create a temporary child `Ui` via `ui.child_ui(top_rect, Layout::left_to_right(Align::Min))` and inside it call `split_horizontal` with `self.right_hsplit_frac` and a unique `Id`.
4) Allocate UIs for the three terminal rects and render panels using the existing `render_terminal_panel(...)` for tabs 1, 2, 3 (indices 1,2,3).

Suggested pixel minimums:
- `min_top_px = 140.0`, `min_bottom_px = 140.0` (adjust to taste)
- `min_left_px = 160.0`, `min_right_px = 160.0`
- Handle thickness: 10–12px for easy capture.

### D) Keep the main left/right SidePanel as-is, but tune ergonomics

- The left resizer should work with `.resizable(true)`; to improve reliability:
  - Keep (or reduce) `.inner_margin(... right: 14.0 ...)` to 8–12px.
  - Ensure no `ui.allocate_ui_at_rect` or child widgets spill over the exact border.
  - The decorative 1px divider via painter is safe (non-interactive). If it visually misleads, keep it but don’t rely on it for hit testing.

### E) Example: vertical splitter helper (sketch)

Below is a compact example to illustrate the idea. This lives in `app.rs` near other UI helpers.

```rust
fn split_vertical(ui: &mut egui::Ui,
                  fraction: &mut f32,
                  min_top_px: f32,
                  min_bottom_px: f32,
                  handle_px: f32,
                  id: egui::Id) -> (egui::Rect, egui::Rect, egui::Rect) {
    let rect = ui.available_rect_before_wrap_finite();
    let total_h = rect.height();

    // Clamp based on pixel minimums
    let min_f = (min_top_px / total_h).clamp(0.0, 0.9);
    let max_f = 1.0 - (min_bottom_px / total_h).clamp(0.0, 0.9);
    *fraction = (*fraction).clamp(min_f, max_f);

    let split_y = rect.top() + total_h * (*fraction);
    let handle_top = split_y - handle_px * 0.5;
    let handle_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left(), handle_top),
        egui::pos2(rect.right(), handle_top + handle_px),
    );

    // Interaction
    let resp = ui.interact(handle_rect, id, egui::Sense::drag());
    if resp.dragged() {
        let dy = ui.input(|i| i.pointer.delta().y);
        *fraction = ((*fraction) + dy / total_h).clamp(min_f, max_f);
    }

    // Paint handle (subtle)
    let visuals = ui.style().visuals.clone();
    let color = if resp.hovered() || resp.dragged() { CatppuccinTheme::FRAPPE.overlay0 } else { CatppuccinTheme::FRAPPE.surface1 };
    ui.painter().rect_filled(handle_rect, 2.0, color.linear_multiply(0.35));

    let top_rect = egui::Rect::from_min_max(rect.min, egui::pos2(rect.right(), handle_top));
    let bottom_rect = egui::Rect::from_min_max(egui::pos2(rect.left(), handle_top + handle_px), rect.max);
    (top_rect, handle_rect, bottom_rect)
}
```

A similar `split_horizontal` computes an `x` split and uses `ui.input(|i| i.pointer.delta().x)`.

### F) Wiring the splitters in `CentralPanel`

Pseudocode inside `CentralPanel::show`:

```rust
let (top_rect, _vhandle, bottom_rect) = split_vertical(ui, &mut self.right_top_frac, 140.0, 140.0, 10.0, Id::new("right_vsplit"));

// Top row
let mut top_ui = ui.child_ui(top_rect, egui::Layout::left_to_right(egui::Align::Min));
let (left_rect, _hhandle, right_rect) = split_horizontal(&mut top_ui, &mut self.right_hsplit_frac, 160.0, 160.0, 10.0, Id::new("right_hsplit"));

let mut top_left_ui = ui.child_ui(left_rect, egui::Layout::top_down(egui::Align::Min));
if self.tabs.len() >= 2 { /* render tab[1] */ }

let mut top_right_ui = ui.child_ui(right_rect, egui::Layout::top_down(egui::Align::Min));
if self.tabs.len() >= 3 { /* render tab[2] */ }

// Bottom full width
let mut bottom_ui = ui.child_ui(bottom_rect, egui::Layout::top_down(egui::Align::Min));
if self.tabs.len() >= 4 { /* render tab[3] */ }
```

This keeps your existing `render_terminal_panel()` while making the right-cluster splits fully draggable.

### G) Button sizing and aesthetics

- Increase button height back to ~20–22px for readability.
- Keep grid width-capped to avoid over-stretching.
- Maintain a narrow but consistent gap between the button grid and the SidePanel’s right edge so the panel handle remains easy to grab.

### H) Left panel internal vertical splitter (Terminal 1 vs Buttons)

- Add runtime field on `AudioToolkitApp`:
  - `left_buttons_frac: f32` — fraction of the left panel’s height devoted to the buttons container (e.g., start around 0.18–0.25), initialized from config or derived from current fixed px height.
- In the `SidePanel::left("terminal_1")` UI, replace the fixed `ui.available_height() - 68.0` with a vertical splitter:
  - Use the same `split_vertical` helper to divide the left-panel content area into `top_rect` (Terminal 1) and `bottom_rect` (buttons container).
  - Clamp with pixel minimums, e.g., `min_top_px = 200.0` for Terminal 1, `min_bottom_px = 56.0` for buttons, and `handle_px = 8.0–10.0` with `Id::new("left_buttons_vsplit")`.
  - Render Terminal 1 in `top_rect` and render the scrollable button grid inside `bottom_rect`.
- Keep the SidePanel’s right inner margin (8–12px) so buttons never sit on the outer edge; this preserves the left/right divider’s grab reliability.

Pseudocode:

```rust
let (t1_rect, _btn_handle, btn_rect) = split_vertical(ui, &mut self.left_buttons_frac,
    200.0, 56.0, 9.0, Id::new("left_buttons_vsplit"));

let mut t1_ui = ui.child_ui(t1_rect, egui::Layout::top_down(egui::Align::Min));
render_terminal_panel(&mut t1_ui, &mut self.tabs[0], /* ... */);

let mut btn_ui = ui.child_ui(btn_rect, egui::Layout::top_down(egui::Align::Min));
egui::ScrollArea::vertical().show(&mut btn_ui, |ui| {
    render_action_buttons(ui);
});
```

This makes the buttons area explicitly resizable, avoids content pressing against the panel edge, and eliminates competition with the main left/right divider.

### I) Acceptance checklist (what to test)

1. Left column measures exactly 40% of window width; no resize affordance shown.
2. Right cluster bottom (Terminal 4) measures exactly 35% of window height.
3. Buttons container occupies the lower 35% of the left column and scrolls correctly; Terminal 1 uses the upper 65%.
4. Right cluster shows visible dividers, and both splits are dimensionable: T2↔T3 (horizontal) and (2/3)↔4 (vertical). Handles capture pointer reliably.
5. Focus and input always route to the clicked terminal; Ctrl+C/D still function as signals.
6. If any splitter remains interactive, handles capture pointer reliably (registered after content).
7. Scrolling is isolated: mouse wheel only scrolls the hovered terminal; other terminals remain unchanged. Sticky-to-bottom behavior preserved per terminal.

---

## Summary

- Today, the left resizer is implemented via `SidePanel::left(...).resizable(true)` and should work; the perceived failure appears tied to ergonomics (button area and margins) rather than logic.
- The right cluster now uses custom interactive splitters with visible dividers (T2↔T3 and (2/3)↔4), registered after content to ensure reliable pointer capture.
- New bug found: mouse wheel scrolling was synchronized across 2/3/4 due to `ScrollArea` identity sharing; we gave each terminal output a unique `id_source` to isolate scrolling.

## Objectives to Fulfill and How to Achieve Them

- **[Objective 1] Left/Right main resizer is reliable along full height**
  - Achieve:
    - Keep `SidePanel::left("terminal_1").resizable(true)` with `style.interaction.resize_grab_radius_side ≈ 12.0`.
    - Maintain a right inner margin of 8–12px inside the SidePanel so content never sits on the edge.
    - Ensure no widgets (including the buttons scroll area) overlap the outer edge.
    - Keep the decorative 1px divider painter-only (non-interactive) so it never steals input.

- **[Objective 2] Buttons container is itself resizable**
  - Achieve:
    - Add `left_buttons_frac: f32` to `AudioToolkitApp` (init ≈ 0.18–0.25 or derived from current px).
    - Inside the left `SidePanel`, use `split_vertical(...)` to divide Terminal 1 (top) and Buttons container (bottom).
    - Clamp min sizes (e.g., top ≥ 200 px, bottom ≥ 56 px) and give the handle ~8–10 px thickness.
    - Render the buttons within a `ScrollArea` inside the bottom rect; never press content against the panel edge.
    - Optional: persist `left_buttons_frac` on exit.

- **[Objective 3] Right-cluster splitters are fully interactive (2↔3 and (2/3)↔4)**
  - Achieve:
    - Add `right_top_frac: f32` and `right_hsplit_frac: f32` to `AudioToolkitApp` (init from `AppSettings`).
    - Implement `split_vertical` and `split_horizontal` helpers with `ui.interact(handle_rect, id, Sense::drag())`.
    - In `CentralPanel`, first split top vs bottom (vertical), then split the top row left vs right (horizontal).
    - Use min px clamps for all panes (e.g., top/bottom ≥ 140 px; left/right ≥ 160 px); handle thickness 10–12 px.
    - Paint subtle hover/active visuals for handles; ensure no overlap between handles.
    - Optional: persist fractions on exit.

- **[Objective 4] Focus and input routing remain correct**
  - Achieve:
    - Click-to-focus must update the focused tab id; only that PTY receives keystrokes.
    - Verify arrows, Backspace, Ctrl+C (SIGINT), and Ctrl+D (EOF) continue to route to the focused terminal only.
    - Regression-test Shift+Tab cycling and mouse focus changes after introducing new splitters.

- **[Objective 5] Visual separation without interfering with input**
  - Achieve:
    - Keep 1px divider lines via foreground painters for aesthetics only.
    - Ensure Z-order: interactive handle rects are the only pointer targets on the splits.
    - Maintain small but consistent gaps between content and edges so hitboxes are reliable.

- **[Objective 6] Configuration and persistence**
  - Achieve:
    - Load initial fractions from `AppSettings` as defaults.
    - Clamp invalid/legacy values on load (e.g., < min px equivalent or > 1 - min).
    - Optionally save updated fractions on app exit back to `config.toml`.

- **[Objective 7] Validation via acceptance tests**
  - Achieve:
    - Re-run the acceptance checklist in this document (sections “I) Acceptance checklist”).
    - Manually test dragging each handle from extreme to extreme, verify no overlaps, and confirm focus routing remains correct.


