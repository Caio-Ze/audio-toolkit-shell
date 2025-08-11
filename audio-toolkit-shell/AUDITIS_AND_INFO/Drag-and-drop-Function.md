# Drag and drop Function - Technical Audit

**Date**: 2025-08-11

---

## Executive Summary

- Deterministic DnD: all drops are routed to the currently focused terminal tab (drop location on the window no longer matters).
- Dropped items are POSIX single-quote shell-escaped and inserted into the focused terminal with a trailing space; multi-item drops are space-joined.
- Per-tab configurable folder-drop behavior via TOML in each tab's config: `[tabs.dnd]` with `auto_cd_on_folder_drop` and `auto_run_on_folder_drop`.
- Precedence (single directory drop): `auto_cd_on_folder_drop` > `auto_run_on_folder_drop` > default (quoted path with trailing space, no Enter).
- Visuals: the focused terminal shows a crisp 2px blue border; while dragging files anywhere over the app, it also shows a subtle 4px blue glow.
- Removed: legacy pointer/rect hit-testing, pointer caching, session state, and DnD debug/tracing flags.
- Status: Implemented, built in release, and manually validated; working as expected.

## Simplified Implementation (How we did it)

- File: `audio-toolkit-shell/src-tauri/src/app.rs`
- Function: `AudioToolkitApp::handle_dnd_single_pass(...)`
  - Reads `i.raw.hovered_files`/`i.raw.dropped_files` from egui input once per frame.
  - Ignores pointer position and per-rect hit-testing; always targets `self.focused_terminal` when a drop occurs.
  - Shell-quotes dropped items and writes them to the focused terminal PTY; multi-item drops are space-joined with a trailing space.
  - For a single directory drop, reads the focused tab's per-tab DnD settings (`tab.config.dnd`) and applies precedence:
    - If `auto_cd_on_folder_drop = true`: insert `cd '<dir>'` and press Enter.
    - Else if `auto_run_on_folder_drop = true`: insert `'<dir>'` and press Enter.
    - Else: insert `'<dir>'` with a trailing space (no Enter).
- Visuals live in `render_terminal_panel(...)`: 2px blue focus border; during drag-hover, a subtle 4px blue glow on the focused panel.
- Code cleanup:
  - Removed legacy DnD fields (pointer cache/session, tracing flags) from `AudioToolkitApp`.
  - Deleted unused helpers: `handle_dnd_for_rect(...)`, `paint_dnd_hover(...)`, `paint_crosshair(...)`.

## Status and Testing

- Built with `cargo build --release` and launched.
- Manual validation:
  - Drops anywhere over the window go to the focused terminal only.
  - Multi-file drops are correctly quoted and space-joined with a trailing space.
  - Single directory: behavior matches per-tab config (auto-cd, auto-run, or default).
  - Focus remains on the target terminal; keyboard input continues there.
  - Verified at 100% and 125% scale; no panics or regressions observed.

## Configuration

- Per-tab TOML keys under each `[[tabs]]` block:
  - `auto_cd_on_folder_drop` (bool, default: false)
  - `auto_run_on_folder_drop` (bool, default: false)
- Legacy env var support removed: `ATS_DND_AUTO_CD_DIRS` is no longer used.
- Deprecated/removed: `ATS_DND_TRACE`, `ATS_DND_POINTER_XHAIRS`, `ATS_DND_CACHED_MS`, `ATS_DND_CACHE_OUTSIDE`, `ATS_DND_NEAREST_FALLBACK`, `ATS_DND_HOVER_LOG_MS`.

<br>

<details>
<summary><strong>Detailed Notes</strong></summary>

<br>

## Main Goal (TL;DR)

- Simple, deterministic DnD: every drop targets the currently focused terminal tab.
- No pointer hit-testing or per-rect heuristics; no timing-dependent routing.

---

## How It Works

- File: `audio-toolkit-shell/src-tauri/src/app.rs`
- Function: `AudioToolkitApp::handle_dnd_single_pass(...)`
  - Reads `i.raw.hovered_files` and `i.raw.dropped_files` once per frame.
  - During hover, requests repaint so the focused terminal shows a subtle blue glow (border + glow painted in `render_terminal_panel(...)`).
  - On drop, always targets `self.focused_terminal` and inserts shell-escaped paths into its PTY input.
- Multi-item drops are space-joined with a trailing space.
- For a single directory, the focused tab's per-tab DnD settings determine whether to auto-cd, auto-run, or default insert.

---

## Removed Legacy Logic

- Pointer position caching, session tracking, and nearest-rect fallbacks.
- Per-rect handler `handle_dnd_for_rect(...)` and paint helpers `paint_dnd_hover(...)`, `paint_crosshair(...)`.
- DnD runtime flags: `ATS_DND_TRACE`, `ATS_DND_POINTER_XHAIRS`, `ATS_DND_CACHED_MS`, `ATS_DND_CACHE_OUTSIDE`, `ATS_DND_NEAREST_FALLBACK`, `ATS_DND_HOVER_LOG_MS`.

---

## Behavior

- Drop anywhere over the window; the focused terminal receives the insertion.
- Focus remains on that terminal before/after the drop.
- Visuals: focused terminal has a crisp 2px blue border; during hover, an additional subtle 4px blue glow.

---

## Quoting & Edge Cases

- POSIX single-quote quoting; embedded `'` becomes `'\''`.
- Mixed selections (files/folders) supported; names with spaces/unicode/emoji preserved.
- Nonexistent paths (if any) are inserted as provided.

---

## Configuration

- All behavior is configured per tab in TOML; there is no environment-variable control.

---

## Testing & Results

- Built and run in release; manual validation performed.
- Verified multi-file quoting, trailing space behavior, and single-dir auto-cd.
- Confirmed focused-tab routing across 100% and 125% scales; no panics or regressions observed.

---

## Acceptance Criteria

- [x] All drops route to the focused terminal tab.
- [x] Visual highlight reflects the focused terminal (not hovered panel).
- [x] Multi-file/mixed drops are correctly shell-escaped and space-joined with trailing space.
- [x] Single-directory behavior follows per-tab config: auto-cd when `auto_cd_on_folder_drop = true`; auto-run when `auto_run_on_folder_drop = true`; default otherwise.

</details>

## Proposed Configurable Enhancements (TOML)

- __auto_cd_on_folder_drop__ (bool, default: false)
  - When a single directory is dropped, insert `cd '<dir>'` and simulate Enter so the command executes immediately.

- __auto_run_on_folder_drop__ (bool, default: false)
  - When a single directory is dropped, insert the quoted directory path only and simulate Enter. This does not change directories; it simply types the path and presses Enter.

- __Precedence when both are true__
  - auto_cd_on_folder_drop takes priority.
  - If `auto_cd_on_folder_drop` is true, perform auto-cd + Enter.
  - Else if `auto_run_on_folder_drop` is true, insert quoted dir path + Enter.
  - Else fall back to current behavior (insert quoted items with trailing space; no Enter).

### TOML schema (example)

```toml
[[tabs]]
title = "Terminal 1"
command = "bash"
[tabs.dnd]
auto_cd_on_folder_drop = true
auto_run_on_folder_drop = false
```

### Detailed Implementation Plan

- __config.rs__ (structure and parsing)
  - Add `DndSettings` and embed per terminal: `TabConfig { dnd: DndSettings, ... }`.
  - Parse under each tab's `[tabs.dnd]` table in the app’s TOML config (per-terminal).
  - Defaults: both false when keys are omitted.

- __app.rs__ (`AudioToolkitApp::handle_dnd_single_pass()`)
  - Detect single-item directory drops (we already detect directories for the env-flag flow).
  - Apply precedence:
    1) If `tab.config.dnd.auto_cd_on_folder_drop` is true:
       - Build `cmd = format!("cd {}", quoted_dir)`.
       - Write `cmd` to the target PTY, then write a newline to simulate Enter.
    2) Else if `tab.config.dnd.auto_run_on_folder_drop` is true:
       - Write `quoted_dir` to the target PTY, then write a newline to simulate Enter.
    3) Else: keep existing insertion behavior (quoted path + trailing space; no Enter).
  - Multi-item or non-directory drops: unchanged from current logic.

- __terminal/PTY write__ (simulate Enter)
  - After insertion, send a newline (`\n`) to PTY (or `\r` if needed by platform). Reuse existing PTY write helpers.

- __Telemetry/Logging__ (optional)
  - Log which mode executed (`auto_cd`, `auto_run`, or `default`).

### Acceptance Criteria

- Config parsing: Missing keys default to false; toggling in TOML takes effect on next app start.
- Auto-cd: Dropping a single directory inserts `cd '<dir>'` and executes (Enter simulated).
- Auto-run: Dropping a single directory inserts `'<dir>'` and executes (Enter simulated) when auto-cd is disabled.
- Precedence: When both flags are true, auto-cd path is executed; auto-run is ignored.
- Non-dir/multi-item: Behavior remains unchanged (quoted items, space-joined, trailing space, no Enter).

### Testing Plan

- Toggle each flag independently and together; verify precedence and outcomes in the focused terminal.
- Verify at 100% and 125% scale; ensure no regressions to multi-file or file drops.
- Confirm newline writes reach PTY and commands execute as expected.
