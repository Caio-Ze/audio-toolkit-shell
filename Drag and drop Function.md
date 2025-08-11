# Drag and drop Function - Technical Audit

**Date**: 2025-08-10

---

## Executive Summary

- Add per-terminal drag-and-drop so dropping files/folders inserts their shell-escaped paths into that terminal's input buffer (like a regular terminal).
- Hit-test drops against each terminal's rect; multi-file selection supported; robust quoting/escaping for spaces and special characters.
- Optional behaviors (to be decided): auto-cd when a folder is dropped; auto-run on drop; join multiple paths with spaces.
- **Status**: Planning
- **Feature Flag**: [Proposed] `ATS_DND_AUTO_CD_DIRS` (default: `false`)

<br>

<details>
<summary><strong>View Detailed Audit</strong></summary>

<br>

## Main Goal (TL;DR)

- Provide reliable drag-and-drop of files/folders onto any of the four terminals.
- Insert shell-escaped path(s) into that terminal's input field at the caret, without affecting other panes.

---

## Current Broken Behavior

### No drag-and-drop handling
- **Symptom**: Dropping files/folders onto terminals has no effect.
- **Likely Cause**: App does not currently read/process `dropped_files` from egui input, nor route them per-terminal.
- **Status**: Not implemented.

---

## Expected Behavior

- Dropping a file or folder over a terminal highlights that panel and captures the drop.
- Insert behavior:
  - Files: insert the shell-escaped absolute path (or leave as provided by egui) into the input buffer, followed by a space.
  - Folders: insert the path; if `ATS_DND_AUTO_CD_DIRS=1`, insert `cd <path>` plus a trailing space.
- Multi-select: multiple items are inserted space-separated, each properly quoted.
- No focus misrouting: only the hovered/target terminal processes the drop.
- Visual feedback: hover highlight while dragging over a panel.

---

## Code & Component Analysis

### Terminal Panels & Hit-Testing
- **File**: `audio-toolkit-shell/src-tauri/src/app.rs`
- **Function/Class**: `render_terminal_panel(...)` and main UI layout where panel rects are computed.
- **Notes**: We already have per-panel rects for focus/headers/splitters. Extend to compute a content drop zone (exclude header and splitter handles) and test drops within it.

### Input Handling
- **File**: `audio-toolkit-shell/src-tauri/src/app.rs`
- **Function/Class**: App frame update; access egui input to read dropped files.
- **Notes**: Read dropped items once per frame; route to the panel whose rect contains the pointer when the drop occurs.

### Terminal Input Integration
- **File**: `audio-toolkit-shell/src-tauri/src/app.rs`
- **Function/Class**: `TerminalTab` input buffer management (caret/append) and PTY write logic.
- **Notes**: Provide a helper for inserting text at the caret. Keep insertion local (do not auto-send unless configured).

### Quoting/Escaping
- **File**: `audio-toolkit-shell/src-tauri/src/app.rs` (utility helper)
- **Function**: `shell_escape_path(path: &str) -> String` (to add)
- **Notes**: On macOS, wrap in single quotes and escape existing single quotes using POSIX style: replace `'` with `'\''` (e.g., `'` becomes `'\''`).

---

## Implementation Plan

### A) Drop detection & routing
- **Action**: Read egui dropped files; compute per-panel drop zones; if pointer release occurs over a zone, mark that terminal as drop target and capture the list.
- **Rationale**: Ensures isolation—only the hovered terminal receives the drop.

### B) Insert behavior (files/folders)
- **Action**: For each dropped item, derive a path string; apply `shell_escape_path`; build insertion string.
- **Rationale**: Matches regular terminal behavior; avoids shell injection and broken paths.

### C) Multi-select & options
- **Action**: Join multiple escaped paths by a single space. If `ATS_DND_AUTO_CD_DIRS=1`, detect directories and transform to `cd <path>` (single or first item), append space.
- **Rationale**: Smooth UX for both single and bulk drops; optional auto-cd mirrors common workflows.

### D) Visual feedback
- **Action**: While dragging, draw a subtle highlight over the terminal under the pointer; optionally show a tooltip "Drop to insert path".
- **Rationale**: Confidence that the drop will target the correct pane.

### E) Focus & events
- **Action**: On drop, keep focus on the target terminal; do not interfere with splitters; ensure z-order so handles don't eat drop events.
- **Rationale**: Prevents the historical focus/input routing regressions.

### F) Configuration (optional)
- **Action**: Add env flag `ATS_DND_AUTO_CD_DIRS` (default `false`). Consider TOML `[app]` option later if needed.
- **Rationale**: Allows teams to opt into auto-cd for directory drops.

---

## Acceptance Criteria & Test Cases

### Drag-and-Drop Basics
- [ ] Dropping a single file inserts its quoted absolute path plus a trailing space.
- [ ] Dropping a single folder inserts its quoted path; with `ATS_DND_AUTO_CD_DIRS=1`, inserts `cd <path> `.
- [ ] Dropping multiple mixed items inserts all quoted paths space-separated.

### Per-Terminal Isolation
- [ ] Dropping onto Terminal 2 does not affect Terminal 1/3/4.
- [ ] Focus remains on the drop target terminal; keyboard input goes there.
- [ ] Splitter handles do not intercept the drop; drop zones exclude headers/handles.

### Quoting & Edge Cases
- [ ] Paths containing spaces, quotes, unicode, and emoji are correctly inserted and preserved.
- [ ] Nonexistent paths (if any) are ignored or inserted as-is (decision: insert as provided).
- [ ] Drag-hover shows visual highlight only on the hovered panel.

### Stability
- [ ] No panics on large multi-select drops.
- [ ] Behavior validated at 100% and 125% display scales with `ATS_DEBUG_OVERLAY=1`.

</details>
