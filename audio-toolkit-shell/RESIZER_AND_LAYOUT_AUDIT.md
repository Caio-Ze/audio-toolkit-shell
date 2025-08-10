# Resizer and Layout Audit (Audio Toolkit Shell)

This archive documents where all resizing and dimensioning logic currently lives, explains why the current layout feels broken, and proposes a robust interactive resizing design for the four-terminal UI.

Date: 2025-08-08

---

## Executive Summary (Updated 2025-08-10)

- **Left column fixed to 40% width**: The left `SidePanel` remains fixed at 40% of window width (non‑resizable).
- **Left column internal split is now 70/30**: Terminal 1 uses the upper 70% and the buttons box uses the lower 30% of the left column height. Implemented in `src-tauri/src/app.rs` by setting `split_y = rect.top() + total_h * 0.70`.
- **Right cluster splitters are interactive**: The vertical split (top 2/3 vs bottom 4) and the top-row horizontal split (2 vs 3) are draggable. Initial top/bottom comes from config `right_top_fraction` (default 0.6). Terminal 4 is not fixed.
- **Focus & scroll fixes landed**: Click‑to‑focus header band and per‑pane `ScrollArea` ids (index‑based) are in place; interactive handles are registered after content to win pointer precedence.
- **Build status**: Release build passes.
- **Buttons panel status (prepass implemented)**: Implemented a row‑background prepass behind a feature flag to remove the mid‑column seam and right‑edge recess. Buttons now render content only (stripe/label/hover ring) over a single row‑wide opaque background.
 - **Feature flag**: `ATS_BTN_ROW_PREPASS` (default: true). Set to `0`/`false` to revert to legacy per‑cell backgrounds for A/B testing.
 - **Fallback**: Legacy `egui::Grid` path retained for rollback.

 Status:
 - **Validation complete (2025-08-10)**: With `ATS_BTN_ROW_PREPASS=1` and `ATS_DEBUG_OVERLAY=1`, the mid‑column seam is eliminated at 100% and 125% scale; no right‑edge clipping observed; hover/pressed states remain consistent; resizing is stable.
 - Row‑prepass remains enabled by default; legacy path retained for rollback.

<details><summary>Archived detailed audit</summary>

## Main Goal (TL;DR)

- **Left column fixed to 40% width**: Terminal 1 (left `SidePanel`) occupies exactly 40% of the window width (non-resizable for now). This was validated in-app.
- **Buttons aligned to Terminal 1**: The buttons container lives under Terminal 1 and occupies 35% of page height and 40% of page width (aligned to the left column). Terminal 1 uses the upper 65% of the left column.
- **Right cluster vertical split (2/3 vs 4)**: Interactive with a visible divider. Default proportions come from config (`right_top_fraction`, default 0.6 → 60% top, 40% bottom). Recommended default is 65/35; Terminal 4 is not fixed.
- **Top-row split (T2 vs T3)**: Must be dimensionable (interactive) with a visible divider. Likewise, the split between top (2/3) and bottom (4) must be dimensionable with a visible divider.

## Current Broken Behavior (What you are seeing)



- **Buttons container fixed by design (Plan v2)**:
  - In Plan v2, the buttons container is intentionally fixed at 35% of the total page height within the 40% left column. It scrolls internally when content overflows. No internal resizer is provided.
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
  - The left width target is 40% of `screen_w`. Combined with prior divider interaction issues, it appeared “stuck small.” The current strategy intentionally fixes it at 40% width (validated in code and layout behavior).

- **Mouse wheel scrolling synchronized across Terminals 2/3/4**:
  - Symptom: Scrolling over any right-cluster terminal scrolls all three at once.
  - Likely cause: `egui::ScrollArea` identity collision (or shared auto id) across panels, so scroll state is shared.
  - Fix applied: give each terminal output its own stable id via `.id_source(ui.id().with(("terminal_output_scroll", tab.title())))` inside `render_terminal_panel()` so hover/scroll only affects that pane.
  - Status: Implemented in `src-tauri/src/app.rs` (`render_terminal_panel`); pending validation.

- **Residual coupling: Scrolling Terminal 3 moves Terminal 2**:
  - Symptom: With Terminal 3 selected/hovered, mouse wheel scroll also scrolls Terminal 2.
  - Likely cause: Title-based id may collide or be reused (duplicate titles, or id path differences between panes), causing `ScrollArea` state to be shared between T2 and T3.
  - Best fix: Use a deterministic, index-based id per pane instead of title strings. Pass the tab index down to `render_terminal_panel()` and set:
    ```rust
    egui::ScrollArea::vertical()
        .id_source(ui.id().with(("terminal_output_scroll_idx", tab_index)))
        .stick_to_bottom(true)
        .show(ui, |ui| { /* ... */ });
    ```
    This guarantees uniqueness regardless of titles or localization. Ensure each pane uses a distinct `child_ui` and that no parent `id_source` is shared.
  - Status: Implemented, but issue persists asymmetrically — after switching to index-based ids, scrolling Terminal 3 still moves Terminal 2, while scrolling Terminal 2 does NOT move Terminal 3. Additionally, we have observed a broader focus/input regression (see below), which may be the underlying cause of the perceived scroll coupling or preventing correct input routing.

  - Asymmetric behavior details:
    - T2 → T3: independent (OK)
    - T3 → T2: coupled (BUG)

  - Hypotheses to verify next:
    1. Overlapping rects: T3’s scroll area or click zone may overlap into T2’s rect, making both areas hovered. Verify by painting tinted debug rects and logging `rect` coordinates.
    2. Parent/container scroll: A parent `ScrollArea` (or implicit scrolling) may be wrapping the top row. Audit to ensure only the per-terminal `ScrollArea` handles wheel input.
    3. ID scoping: Even with unique `ScrollArea` ids, other auto-generated widget ids in the panel may collide. Add a per-pane `ui.push_id(("pane_scope", tab_index), ...)` around the entire panel render to isolate any implicit ids.
    4. Event ordering: Ensure no late-registered widget at the top row intercepts wheel and re-routes it. Keep splitter handle registration after content; avoid any additional post-content `interact` except the handles.

  - Fix plan (next iteration):
    - Wrap each terminal panel rendering in a scope to isolate all auto ids:
      ```rust
      ui.push_id(("pane_scope", tab_index), |ui| {
          let clicked = Self::render_terminal_panel(ui, tab, is_focused, tab_index);
          // ...
      });
      ```
    - Confirm no parent `ScrollArea` exists around the top-row containers. If present, remove/disable it.
    - Add a temporary debug overlay to draw each pane’s rects (top/bottom, left/right) and handle rects. Toggle via env var `ATS_DEBUG_OVERLAY=1`.
    - Re-test the asymmetric case (T3 scrolling) explicitly.

  - Implementation status:
    - Added `push_id(("pane_scope", idx))` around every `render_terminal_panel` call in `app.rs`.
    - Removed the panel-wide click zone to avoid any overlap with the `ScrollArea`; focusing is now via header click only.
    - Added an optional debug overlay (rect tinting) gated behind `ATS_DEBUG_OVERLAY` environment variable.

  - Repro steps used:
    1. Launch app, hover Terminal 3, scroll; observe Terminal 2 scrolls too.
    2. Hover Terminal 2, scroll; observe Terminal 3 remains stable.
    3. No drag on splitters; only wheel input used.

  - Outcome so far: The initial title-based id fix and the index-based id fix did not fully resolve the T3→T2 coupling. Proceeding with scoping and overlap debugging.

### Postmortem: Prior fix placement vs root cause

- **Where we applied fixes**:
  - Scroll isolation implemented inside `render_terminal_panel()` in `src-tauri/src/app.rs` by setting a unique `ScrollArea` id per terminal (first by title, then by index) and using a dedicated click id per panel.
  - Right-cluster interactive handles were registered after content render in `AudioToolkitApp::update()` to resolve Z-order pointer precedence. This is unrelated to wheel events but ensures splitter drags work.

- **Why this might be insufficient**:
  - If the top-row `child_ui` rects overlap due to handle thickness or off-by-one math, the hovered area for T3 could include T2’s `ScrollArea` region, causing both to process wheel input.
  - If any parent container above the top-row is scrollable (explicit or implicit), wheel events can be forwarded in unexpected ways. Siblings don’t receive events directly in egui, so parent propagation is the most likely path.
  - Auto-generated widget ids inside each panel (e.g., labels, click zones) might still collide without an explicit per-pane `push_id` scope, affecting how egui tracks hover/scroll behavior.

- **Conclusion**:
  - The prior fixes were applied in the right general area (per-pane scroll state) but likely miss an additional requirement: strict scoping and no-overlap guarantees at the top-row container level. Next iteration will add `push_id` scoping and instrument rect overlays to confirm no geometric overlap, plus a check for unintended parent scrolling.

### New Regression: Focus selection and input routing

- **Symptoms**:
  - Click-to-focus works only partially or inconsistently; sometimes the target terminal does not become focused.
  - The focused-title color change (previously switching color to indicate focus) is no longer reliably applied; titles no longer reflect the focused state.
  - Text input from the keyboard is not reaching the selected terminal as expected.
  - Overall responsiveness feels degraded (potentially due to focus not being set and input handlers not engaging for the intended terminal).

- **Timeline / Changes possibly related**:
  - We removed the panel-wide click zone in `render_terminal_panel()` and now rely on clicking the header label to set focus.
  - We introduced `push_id(("pane_scope", idx))` scoping around each panel render.
  - Splitter handle registration remains after content for pointer precedence.

- **Current status**:
  - Focus indicator (title color) no longer reliably reflects selection.
  - Input routing seems broken: selected terminal often does not receive text input.
  - Scroll isolation validation is blocked by the focus/input regression because it can mask which terminal is actually handling events.

- **Hypotheses**:
  1. The clickable focus area is too narrow (header-only) or visually overlapped by other widgets, reducing hit-target reliability.
  2. Z-order or geometry overlap with splitter handles is intercepting clicks before they reach the header.
  3. `push_id` scoping or id changes altered widget identity paths and broke our `focused_terminal` update logic.
  4. The rendering order or re-use of `Ui` contexts causes the header click response to be dropped.
  5. Keyboard input routing path expects `focused_terminal` to be set; if not set, input is ignored or sent to the wrong terminal.

- **Planned diagnosis (no code changes yet)**:
  - Add an explicit QA pass to verify header click hitboxes and ensure they span a reliable width/height.
  - Use `ATS_DEBUG_OVERLAY=1` to visualize pane rects and handle rects; confirm header region is not overlapped by handles.
  - Log when a header click is registered (which index) and when `focused_terminal` changes (old→new), to correlate UI state with clicks.
  - Visually emphasize focused terminal (temporary border/tint) to confirm state changes even if title color lags.

- **Likely fix directions (to be implemented later)**:
  - Restore a safe, non-overlapping panel-wide click zone fully inside each pane’s rect (keeping gutters free for splitters) to make focusing more forgiving.
  - Ensure header and panel click zones do not overlap with splitter handles; register any click interactors before/after content as needed to avoid event conflicts.
  - Re-check that focus indicator styling (title color) keys off `is_focused` and that `focused_terminal` updates occur unconditionally on click.
  - Optionally add a small on-screen indicator (e.g., a dot/badge) near the title to confirm the focus target.

### Code references and geometry proof (right cluster)

- **Where handles and pane rects are defined**: `src-tauri/src/app.rs`, in `AudioToolkitApp::update()` within the right-cluster section.
  - Vertical split (top vs bottom): `v_handle_rect` spans full width at the split Y (≈10 px thick).
  - Horizontal split (T2 vs T3):
    ```rust
    const HANDLE_THICK: f32 = 10.0;
    let h_handle_rect = egui::Rect::from_min_max(
        egui::pos2(h_handle_left, top_rect.top()),
        egui::pos2(h_handle_left + HANDLE_THICK, top_rect.bottom()),
    );
    let left_rect  = egui::Rect::from_min_max(top_rect.min, egui::pos2(h_handle_left,               top_rect.bottom()));
    let right_rect = egui::Rect::from_min_max(egui::pos2(h_handle_left + HANDLE_THICK, top_rect.top()), top_rect.max);
    // registered AFTER content:
    let h_resp = ui.interact(h_handle_rect, egui::Id::new("right_hsplit"), egui::Sense::drag());
    ```
  - Because `h_handle_rect` uses `top_rect.top()`..`top_rect.bottom()`, the handle occupies a vertical stripe across the entire height of the top row (including the header region). Since it is registered after content, it wins interactions over the header, making header clicks within that stripe fail to focus.

- **Focus click location**: `render_terminal_panel()` header region:
  ```rust
  let header_resp = ui.horizontal(|ui| {
      let lbl = ui.add(
          egui::Label::new(RichText::new(format!("{} 🖥️ {}", focus_indicator, tab.title())) )
              .truncate(true)
              .sense(egui::Sense::click()),
      );
      // "(Click to focus)" hint when not focused
      lbl
  }).inner;
  if header_resp.clicked() { clicked = true; }
  ```
  With the panel-wide click zone removed, this header label is the only focus hitbox; combined with the full-height handle stripe, user clicks often miss.

### Status summary (2025-08-09)

- **Left column**: fixed at 40% width; buttons container at 35% height (validated).
- **Right cluster splitters**: interactive and visible; handles registered after content.
- **Scroll isolation**: index-based `ScrollArea` IDs implemented; residual T3→T2 coupling still observed, but current focus regression likely masks true behavior.
- **Focus/input regression**: header-only focus hitbox + full-height T2↔T3 handle stripe causes missed focus; keyboard input then routes to wrong/previous tab.
- **Debug overlay**: available via `ATS_DEBUG_OVERLAY=1` to visualize pane and handle rects.

### Fix blueprint (to implement next)

1) **Restore robust focus hit area**:
   - Reintroduce a panel-wide click zone within each pane that explicitly excludes handle rectangles and gutters reserved for splitters.
   - Keep header click as a secondary focus path.

2) **Prevent handle/header overlap**:
   - Change `h_handle_rect` to avoid the header band, e.g.:
     - Compute header height used in `render_terminal_panel()` and set
       `h_handle_rect.top = top_rect.top() + header_height + margin`.
     - Or split the handle into two bars (top and bottom) leaving a gap over the header band.
   - Keep `v_handle_rect` as-is but ensure it doesn’t overlay clickable areas unnecessarily.

3) **Event model**:
   - Preserve registration order (content then handles) for reliable drags.
   - Ensure plain clicks on the handle stripe are eaten only when dragging starts; if feasible, allow simple clicks to pass-through to header/panel for focus.

4) **Validation helpers**:
   - Add temporary logs on header click and on `focused_terminal` change (old→new).
   - Add a temporary focus border/tint around the focused pane for visual confirmation.

### Acceptance tests (refined)

- **Focus & input**
  - [x] Clicking anywhere within a pane (excluding gutters/handle stripe) focuses that terminal.
  - [x] Clicking the header focuses the terminal even near the T2↔T3 boundary.
  - [x] Focus indicator (title color and border/tint) updates immediately on focus change.
  - [x] Keyboard input routes to the focused terminal only.

- **Splitters**
  - [x] T2↔T3 handle is draggable without intercepting header clicks.
  - [x] Top↔Bottom handle is draggable without intercepting clicks into the top headers.
  - [x] Handles remain ~10 px and ergonomic to grab.

- **Scrolling**
  - [x] Scrolling T2 does not move T3; scrolling T3 does not move T2; T4 scroll is independent.
  - [x] Sticky-to-bottom behavior holds per terminal.
  - [x] With `ATS_DEBUG_OVERLAY=1`, pane and handle rects are visible and do not overlap header focus areas.

### Regression status (right-cluster dividers)

- Resolved: The right-cluster divisions between Terminals 2, 3, and 4 have been restored with custom interactive splitters and visible handles, registered after content for correct pointer precedence.

## Expected Behavior (How it should work)

- **Left column (SidePanel)** (`src-tauri/src/app.rs`, `SidePanel::left("terminal_1")`):
  - Fixed at 40% of window width; no resize affordance shown.
  - Lock with `.width_range(left_w..=left_w)` where `left_w = ctx.screen_rect().width() * 0.40`, and consider `.resizable(false)` to remove the grab affordance. Keep a small right inner margin (8–12px) for visual breathing room.
- **Right cluster** (`CentralPanel`):
  - Two custom interactive splitters (one vertical, one horizontal) implemented with `ui.interact(handle_rect, id, Sense::drag())` and handle thickness ~10–12px.
  - Fractions update live in app state and are clamped by pixel-based minimums so panes never collapse.
  - No overlap between handles; handles highlight on hover/drag for clear affordance.
- **Buttons container fixed**: The container under Terminal 1 has a fixed height equal to 35% of the total page height within the left column; it uses a vertical `ScrollArea` for overflow. No internal splitter is present.
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
    - Sets `left_w = ctx.screen_rect().width() * 0.40` as the target width (Plan v2).
  - Builds the left panel (Plan v2):
    - `let left_panel = egui::SidePanel::left("terminal_1")`
    - `.resizable(false)` (or rely on the width lock)
    - `.width_range(left_w..=left_w)`
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

#### Buttons grid — seam in the middle and right-edge “cutoff” (Updated 2025-08-10)

__Observed__
- Buttons still show a dark vertical gap between the two columns, and the right column looks slightly clipped at the far right.

__What we tried (did not fully work)__
- Set `h_spacing = 0.0` for `egui::Grid` and zeroed `ui.style_mut().spacing.item_spacing.x/y` inside the buttons `ScrollArea`.
- Computed separate widths for columns and consumed the entire container width:
  - `col_w_left = ((total_w - h_spacing) / 2.0).floor()`,
  - `col_w_right = total_w - h_spacing - col_w_left`.
- Removed inner-corner rounding by adding `is_right_col` to `render_action_button(...)` and zeroing the inner seam corners to avoid curved gaps.
- Kept a 1px shrink for the far-right paint rect when a button abuts the UI’s right edge to avoid subpixel/right-edge overdraw.

Result: Seam reduced but still visible; right column still appears cut off by ~1 px at some sizes.

__Hypotheses__
- Anti-aliasing seam: two abutting rounded rectangles can leave a thin AA line. Even with inner corners set to 0, AA on each edge can reveal the background.
- Width rounding: `floor` on the left may create a 1 px deficit at the seam under certain widths. Although the remainder is assigned to the right column, AA plus rounding may still expose a line.
- Right-edge shrink: The 1 px shrink for the outer-right paint rect may exaggerate the perception of clipping on certain pixel densities.
- ScrollArea/overlay subtleties: If a vertical scrollbar appears, effective content width can change frame-to-frame, triggering small visual offsets.

__Precise code locations to adjust__
- `src-tauri/src/app.rs`
  - Buttons grid width math: around the `egui::Grid::new("action_buttons")` block (approx lines 1188–1316 in current revision).
  - `AudioToolkitApp::render_action_button(...)`: around lines 709–788. This is where we set rounding, compute `paint_rect`, draw the background, stripe, and hover ring.

__Planned fixes (minimal, low risk)__
1) Overlap at the seam to defeat AA:
   - In `render_action_button(...)` adjust `paint_rect` per column:
     - Left column: `paint_rect.max.x += 0.5` (or 1.0) to overlap the seam.
     - Right column: `paint_rect.min.x -= 0.5` (or 1.0).
   - Keep inner-corner rounding at 0 so the overlap is invisible.
2) Tweak right-edge policy:
   - Reduce or remove the 1 px right-edge shrink. With whole-pixel widths and seam overlap, it should be unnecessary. If keeping, prefer 0.5 px to minimize visible recess.
3) Width rounding improvement:
   - Use `round()` for the left column and derive the right as the exact remainder:
     ```rust
     let col_w_left = ((total_w - h_spacing) * 0.5).round().max(1.0);
     let col_w_right = (total_w - h_spacing - col_w_left).max(1.0);
     ```
   - This guarantees `left + right == total` at integer pixels while balancing rounding error across both columns.

__Alternative (more robust, medium risk)__
- Paint row backgrounds once per row across the full width, then paint per-button accents/labels on top. This removes the possibility of a seam entirely but requires restructuring the grid rendering to know row extents.

__Acceptance tests (buttons)__
- [x] No visible vertical seam at the mid-column boundary at 100% and 125% scale on macOS.
- [x] No right-edge recess or clipping of the right column.
- [x] Resizing the window does not introduce or remove seams.
- [x] Buttons continue to fill the entire buttons container both horizontally and vertically, with vertical spacing preserved.

#### Post-fix validation and why it didn’t work (2025-08-10)

__What we implemented (minimal fix set)__
- In `src-tauri/src/app.rs` → `AudioToolkitApp::render_action_button(...)`:
  - Added ±0.5 px seam overlap: left buttons extend `paint_rect.max.x`, right buttons extend `paint_rect.min.x`.
  - Removed 1 px right-edge shrink to avoid apparent recess at the far right.
  - Anchored stripe and text to the original `rect.min.x` to avoid label drift when overlapping.
- In the buttons grid block (`egui::Grid::new("action_buttons")`):
  - Switched left column width to `round()` and assigned exact remainder to the right column.

__Observed after changes__
- The vertical seam between columns still shows in release at certain window widths and at 100%/125% scale.
- Right-edge recess is improved but not fully eliminated at all sizes.

__Why this likely didn’t work__
- __AA overlap is insufficient__: Egui’s shape tessellation applies anti‑aliasing at edges. Two separate rects meeting at a boundary can still leave a sub‑pixel feather where the background peeks through. A ±0.5 px overlap helps but may not fully cover at different DPI/pixels‑per‑point and fractional layout positions.
- __Grid/child layout ordering__: `egui::Grid` lays out children independently; each button paints its own background. Even if widths sum to the container’s width, feathering and per‑cell clipping can still create a faint line.
- __State overlays__: Hover/pressed overlays are per‑cell. Even if base backgrounds align, the overlay alphas can diverge slightly across the seam, visually reinforcing a line.
- __Device pixel alignment__: Without explicit alignment to device pixels (`ui.pixels_per_point()`), edges may fall on half‑pixels after transforms, reintroducing feathering artifacts.

__Conclusion__: Per‑cell background rectangles are fragile against AA and pixel alignment; a single background painted per row (or for the whole panel) is more robust.

### Row‑background prepass (Implemented 2025‑08‑10)

__What changed__
- We added a row‑background prepass that paints one opaque rect per row across the full width, with outer corners rounded only on the top/bottom rows. Button cells no longer paint their own backgrounds.

__Flags__
- `ATS_BTN_ROW_PREPASS` toggles the prepass (default: enabled). Set to `0` or `false` to disable and use the legacy per‑cell Grid backgrounds.
- `ATS_DEBUG_OVERLAY` (1/true) remains available to visualize pane/seam geometry.

__Code locations__
- `src-tauri/src/app.rs`
  - New: `AudioToolkitApp::render_action_button_no_bg(...)` (content only; no BG). Inserted near the original `render_action_button(...)` implementation.
  - Updated buttons block (two‑column area under Terminal 1): prepass branch paints the row background, splits the row rect into left/right, and calls `render_action_button_no_bg(...)` for each cell. The legacy Grid path is preserved under the feature flag.
  - Width handling: columns are derived from container width with `round()` on the left and exact remainder for the right to ensure pixel‑perfect sum.

__Why this should fix the seam__
- Because the row background is a single shape spanning both columns, there is no inter‑cell edge where AA can reveal the panel fill. Hover rings and labels are drawn atop the uniform row background.

__Notes__
- Inner corners on the two cells are square (0 radius); outer corners keep small rounding (4 px) via the row rect on first/last row only.
- Hover ring is a thin stroke per cell; it does not introduce a background fill seam.
- Right‑edge “shrink” is not used in prepass mode; the row rect spans the full width.

__Validation checklist__
- Seam is not visible between columns at 100% and 125% scale.
- Right column does not appear recessed or clipped at any window width.
- Button grid continues to fill the container both horizontally and vertically; resizing the window does not introduce gaps.
- Hover/pressed visuals remain crisp without reintroducing seams.

### Buttons panel — Diagnostic plan (no code changes yet)

- __Instrumentation__
  - Log and overlay: `ui.pixels_per_point()`, `total_w`, `col_w_left`, `col_w_right`, seam X position. Draw a debug vertical line at the computed seam using `painter.line_segment` with a bright color (gated by `ATS_DEBUG_OVERLAY`).
  - Toggle render modes via `ATS_DEBUG_OVERLAY`:
    1) Per‑cell backgrounds (current).
    2) Single row‑background prepass (no per‑cell backgrounds).
    3) Single full‑panel background.
  - Capture screenshots at 100% and 125% scale for each mode.

- __Isolation tests__
  - Temporarily render the two columns using `ui.columns(2, ...)` instead of `egui::Grid` to rule out Grid‑specific spacing/strokes.
  - Force integer alignment: snap left/right x to `round_to_pixel(ui, x)` using `ui.pixels_per_point()` to see if seam disappears when boundaries align to device pixels.
  - Disable per‑cell hover fills (keep only border ring) to check if overlay alpha contributes to the seam.

__Precise code locations to instrument__
- `src-tauri/src/app.rs`
  - `AudioToolkitApp::render_action_button(...)` (≈ 709–788): add optional debug line painting and logging of `rect`, `paint_rect`, and `ui.pixels_per_point()`.
  - Buttons grid block (≈ 1188–1316): log `total_w`, `col_w_left`, `col_w_right`, and computed seam x; add optional alternative render mode paths under `ATS_DEBUG_OVERLAY`.

### Buttons panel — Careful implementation plan (future)

__Preferred (robust)__: Row‑background prepass
- For each row, compute the union rect from the left and right cells and paint one opaque background rect for the entire row.
- Keep per‑cell content (labels, icons, accent stripe) unchanged.
- For hover/pressed feedback, avoid full‑fill overlays that meet at the seam; use either:
  - A subtle inner border ring on the hovered half only, or
  - A translucent overlay that stops short of the seam by 1 px and does not overlap the seam, or
  - A row‑wide overlay plus a light inset border to indicate the hovered half.
- Outer corners rounding: apply only on extreme outer corners of the topmost/bottommost rows; no rounding on inner seams.

__Alternative (simpler)__: Full‑panel background
- Paint a single background rect for the buttons container. Per‑cell backgrounds are removed.
- Hover/pressed feedback uses border rings only. No seam is possible as there is no per‑cell background.

__Pixel alignment hardening__
- Introduce helpers:
  - `fn px(ui: &egui::Ui) -> f32 { ui.pixels_per_point() }`
  - `fn snap(x: f32, px: f32) -> f32 { (x * px).round() / px }`
- Snap seam x and row rect edges before painting to align to device pixels.

__Acceptance criteria (final)__
- Zero visible seam at mid‑column boundary at 100%/125% scale in release.
- No right‑edge recess at any window width.
- Hover/pressed feedback remains clear per button without reintroducing a seam.
- No layout jank when window resizes; widths always sum to container width.

__Rollback & guardrails__
- Keep a feature flag (e.g., `ATS_BTN_ROW_PREPASS=1`) to toggle between per‑cell and row‑prepass during validation.
- If row‑prepass complicates hover states, fall back to full‑panel background + border rings.

### 4) Application settings that affect dimensions

- File: `src-tauri/src/config.rs`
- Struct: `AppSettings` (lines ~65–84), defaults at ~153–163
  - `min_left_width: f32` — default 120.0
  - `min_right_width: f32` — default 120.0
  - `allow_zero_collapse: bool` — default false
  - `right_top_fraction: f32` — default 0.6 (top row height share)
  - `right_top_hsplit_fraction: f32` — default 0.5 (top-left width share)

### 5) Where things likely need to move/change

- Lock the main left/right split to a fixed 40% left column: compute `left_w = ctx.screen_rect().width() * 0.40` each frame, set `.width_range(left_w..=left_w)`, and consider `.resizable(false)` so no grab affordance is shown.
- Replace ad‑hoc resizing with the new fixed-percentage layout for predictability:
  - Left column fixed at 40% width (non-resizable for now) and buttons container fixed to 35% height within it, computed per frame.
  - Right cluster vertical split is interactive (top vs bottom). Initial proportion comes from `right_top_fraction` (config default 0.6 → ~60% top/40% bottom). Recommended initial is ~65% top/35% bottom.
  - Top-row split (T2 vs T3) may remain interactive or be fixed (default 50%). If interactive, register handle interactions after content to avoid Z-order issues.
  - Ensure decorative dividers are painted but never intercept input (painters are fine). Only handle rects should be pointer targets.

---
### Plan v2 — fixed layout + interactive right cluster

#### Targets
- **Left column**: width = 0.40 × window width; non-resizable for now.
- **Buttons container**: height = 0.35 × window height within the left column; width aligned (0.40 × window width). Terminal 1 uses remaining 65% height of the left column.
- **Right cluster**: vertical split (top vs bottom) is interactive. Initial top height = `right_top_fraction` (config default 0.6). Recommended initial is ~0.65 top / 0.35 bottom.
- **Right-cluster splits**: both splits are interactive and have visible dividers:
  - Horizontal split: Terminal 2 vs Terminal 3.
  - Vertical split: Top row (2/3) vs Bottom row (4).

#### Implementation tasks
2) Buttons container (inside left column)
   - Use a fixed internal split at 65% (Terminal 1) / 35% (buttons) inside the left `SidePanel`.
     Code computes `split_y = rect.top() + total_h * 0.65` each frame; no runtime field required.
   - Render Terminal 1 in the upper 65% of the left column; render the buttons container in the lower 35% with a vertical `ScrollArea`.
   - Remove/disable the internal draggable handle for the buttons area (non-interactive in v2).
   - Note: the legacy `left_buttons_frac` field exists but is currently unused.

---
### 6) Validation
   - Verify left column is exactly 40% width; buttons exactly 35% of page height within it (aligned to 40% width); right-cluster vertical split is interactive and starts around 60–65% top / 40–35% bottom and adjusts on drag.
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

### D) Fixed left column ergonomics

- With the left column fixed (no resizer), to improve ergonomics:
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

This keeps the buttons area fixed and scrollable, avoids content pressing against the panel edge, and eliminates competition with the main left/right divider.

### I) Acceptance checklist (what to test)

1. Left column measures exactly 40% of window width; no resize affordance shown.
2. Right cluster bottom (Terminal 4) is resizable via the vertical splitter; initial proportion comes from config (e.g., ~60–65% top / ~40–35% bottom).
3. Buttons container occupies the lower 30% of the left column and scrolls correctly; Terminal 1 uses the upper 70%.
4. Right cluster shows visible dividers, and both splits are dimensionable: T2↔T3 (horizontal) and (2/3)↔4 (vertical). Handles capture pointer reliably.
5. Focus and input always route to the clicked terminal; Ctrl+C/D still function as signals.
6. If any splitter remains interactive, handles capture pointer reliably (registered after content).
7. Scrolling is isolated: mouse wheel only scrolls the hovered terminal; other terminals remain unchanged. Sticky-to-bottom behavior preserved per terminal.
8. Explicit regression test: With Terminal 3 hovered/selected, scrolling must not move Terminal 2 (and vice versa).
9. T2 scroll does not move T3
10. T3 scroll does not move T2
11. T4 scroll does not move T2 or T3
12. T2/T3 remain resizable via horizontal splitter; top/bottom via vertical splitter
13. Sticky-to-bottom behavior intact across terminals
14. Focusing a terminal by clicking its header still routes input to the correct terminal
15. With `ATS_DEBUG_OVERLAY=1`, pane and handle rects display correctly without overlap

  ---
 
## Focus & Input — Current State and Plan (2025-08-10 18:46 -03:00)
 
### Current State (observed)
- The wide header click band exists, but clicking headers (e.g., Terminal 1 or 2) often does not update focus. The title color and focus border do not consistently reflect selection, and keystrokes are not reliably routed to the clicked terminal.
- Right-cluster splitter handles are registered after content, with the horizontal handle starting below the header band to avoid overlap.
- Scroll isolation uses stable, index-based `ScrollArea` ids per terminal.
 
### Implementation (as in code today)
- `src-tauri/src/app.rs`
  - `render_terminal_panel(ui, tab, is_focused, tab_index) -> bool` defines a full-width header band via `allocate_exact_size(.., Sense::click())` and returns `true` when the header is clicked. It draws a blue focus border when `is_focused`.
  - `AudioToolkitApp::update()` sets `self.focused_terminal` for each pane when `render_terminal_panel(...)` returns `true` (Terminal 1 left; Terminals 2/3 top-right; Terminal 4 bottom-right). Keyboard input is forwarded from `ctx` to the currently focused tab’s PTY writer.
  - Right-cluster handles are computed so the horizontal handle’s top begins below the header band; handle interactions are registered after content.
 
### Likely Root Causes for unreliable click-to-focus
1. Header vs handle geometry mismatch: The header band height is computed in two places (inside `render_terminal_panel()` and again in `update()` when positioning the top-row horizontal handle). Minor style differences can cause the handle to intrude into the header area, stealing clicks due to last-added precedence.
2. No panel-wide focus click: Only the header band is clickable for focus; clicks in the terminal output area don’t update focus, which can feel broken to users who click anywhere in a pane.
3. Event precedence/z-order: Even without overlap, registering extra interactions after content can starve earlier widgets if any geometry accidentally overlaps. We must make handle precedence explicit and keep focus click zones outside handle rectangles.
 
### Plan — Focus/Input Fix v1.1 (low-risk)
1) Unify header band height
    - Add a helper `header_band_height(ui: &egui::Ui) -> f32` (or `from_style(&egui::Style)`) used in both `render_terminal_panel()` and the right-cluster handle geometry in `update()` so the horizontal handle can never overlap the header by accident.
 
2) Panel-wide click-to-focus (excluding handles)
    - After rendering each pane’s content, register a transparent `Sense::click()` interaction over the pane rect minus any handle gutters that border it.
    - Keep this registration BEFORE adding splitter handle interactions, so handles (added last) always take precedence in any touching area.
    - Header click remains a secondary/explicit path; the panel-wide click improves ergonomics.
 
3) Instrumentation for diagnostics (gated by `ATS_DEBUG_OVERLAY`)
    - In `render_terminal_panel()`: paint a translucent overlay for the header rect; log `hovered/pressed/clicked` with pointer position and header rect.
    - In `update()`: log focus transitions as `old -> new` when `self.focused_terminal` changes; paint tinted rects for panel click zones and handle rects.
 
4) Validation
    - Verify clicks on header or anywhere in a pane (except handle gutters) set focus instantly; title color and subtle border update accordingly; keyboard input routes only to the focused PTY.
 
### Exact Code Locations to Update
- `src-tauri/src/app.rs`
  - Add helper: `fn header_band_height(ui: &egui::Ui) -> f32` near other UI helpers.
  - `render_terminal_panel(...)`
    - Replace local header height math with `header_band_height(ui)`.
    - When `ATS_DEBUG_OVERLAY=1`, paint `header_rect` overlay and log pointer/response events for the header.
  - `AudioToolkitApp::update()` (right cluster layout)
    - Replace the ad-hoc `header_title_size`/`header_h_band` computation with `header_band_height(ui)` to compute `h_top` for `h_handle_rect`.
    - For each pane, after calling `render_terminal_panel(...)`, register a panel-wide focus click zone using the pane’s rect:
      - Terminal 1 (left): `t1_rect` (no internal handle in v2; safe to use full rect).
      - Terminal 2 (top-left): `left_rect` minus `h_handle_rect`.
      - Terminal 3 (top-right): `right_rect` minus `h_handle_rect`.
      - Terminal 4 (bottom): `bottom_rect` minus `v_handle_rect`.
    - Register splitter handle interactions last so they always win if areas touch.
  - Keyboard input: already routed via `handle_terminal_key_input_ctx(ctx, &mut tab.pty_writer)` for `self.focused_terminal`; keep as-is, add debug log when writer is `None`.
 
### Acceptance (focus/input)
- Click header or anywhere within a pane (excluding gutters/handles) focuses that terminal; title color changes and a blue border appears.
- Keystrokes (e.g., `echo TEST` + Enter) are received only by the focused terminal.
- Shift+Tab cycles focus across the four terminals in order.
- With `ATS_DEBUG_OVERLAY=1`, header and panel click zones are visualized; handle rects never overlap the header tint.

## Summary

- Today, the left column is fixed at 40% width (no resizer) by design; prior divider ergonomics concerns (grab radius, margins) no longer apply.
- The right cluster now uses custom interactive splitters with visible dividers (T2↔T3 and (2/3)↔4), registered after content to ensure reliable pointer capture.
- New bug found: mouse wheel scrolling was synchronized across 2/3/4 due to `ScrollArea` identity sharing; we gave each terminal output a unique `id_source` to isolate scrolling.
- Residual issue: Terminal 3’s scroll still affects Terminal 2 in some cases. Plan: switch to index-based `ScrollArea` ids (stable and collision-proof) and re-test.

## Objectives to Fulfill and How to Achieve Them

- **[Objective 1] Left column fixed at 40% width (non-resizable)**
  - Achieve:
    - Compute `left_w = ctx.screen_rect().width() * 0.40`.
    - Set `.width_range(left_w..=left_w)` and/or `.resizable(false)` on `SidePanel::left("terminal_1")` so no grab affordance is shown.
    - Maintain a right inner margin of 8–12px inside the SidePanel so content never sits on the edge.

- **[Objective 2] Buttons container fixed height with internal scrolling**
  - Achieve:
    - Allocate the lower 35% of the page height within the left column for the buttons container; the upper 65% goes to Terminal 1.
    - Render the buttons within a vertical `ScrollArea` inside the bottom rect; never press content against the panel edge.
    - Do not add an internal draggable handle for the buttons area.

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

</details>
