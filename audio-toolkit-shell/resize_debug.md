# Resize Debug: Horizontal Split Behavior

Objective
- Provide a single horizontal divider between two terminals. Dragging the divider resizes the terminals inversely: increasing one decreases the other.
- Avoid overlap or hidden content while keeping the UI responsive and visually consistent with the theme.

Constraints and UX
- One divider only; no nested splitters.
- Smooth drag behavior with predictable limits.
- Terminal content must never force a minimum panel width that blocks resizing.
- Optional: allow collapse-to-zero or enforce a small minimum width for stability.

Simplified Approach (egui built-ins)
1) Use built-in panels:
   - Left: `SidePanel::left("terminal_1").resizable(true).min_width(MIN_LEFT).width_range(MIN_LEFT..=f32::INFINITY)`
   - Right: `CentralPanel` with `ui.set_min_width(MIN_RIGHT)`
   - Result: a single divider; egui guarantees inverse resizing automatically.

2) Remove hidden constraints from content:
   - Wrap headers and input rows in `ScrollArea::horizontal()` and call `.truncate(true)` on long `Label`s.
   - Render terminal output in `ScrollArea::both()`; this prevents long lines from enforcing panel width.
   - Call `ui.set_min_width(0.0)` inside panel contents to allow shrinking.

3) Theme and clipping:
   - Apply `Frame::fill(ctx.style().visuals.panel_fill)` to SidePanel to keep Terminal 1 background consistent.
   - Keep terminal text in monospace and avoid painting outside allocations.

4) Practical, stable limits:
   - Choose defaults: `MIN_LEFT = 120.0`, `MIN_RIGHT = 120.0` (tunable). This avoids edge-case overlap while keeping flexibility.
   - If zero-collapse is desired, set both to `0.0` once stability is verified.

Optional configuration
- Add to app config:
  - `resize.min_left_width: f32` (default 120.0)
  - `resize.min_right_width: f32` (default 120.0)
  - `resize.allow_zero_collapse: bool` (default false)

Acceptance criteria
- Dragging the divider left grows Terminal 2 and shrinks Terminal 1 until the left minimum.
- Dragging the divider right grows Terminal 1 and shrinks Terminal 2 until the right minimum.
- No overlap; terminals remain usable; colors match theme.

Future enhancements
- Persist divider position between runs.
- Animate handle hover/drag for better UX.
- Dynamic PTY sizing (cols/rows) based on visible area.
