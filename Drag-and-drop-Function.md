# Drag and drop Function - Technical Audit

**Date**: 2025-08-11

---

## Executive Summary

- Deterministic DnD: all drops are routed to the currently focused terminal tab (drop location on the window no longer matters).
- Dropped items are POSIX single-quote shell-escaped and inserted into the focused terminal with a trailing space; multi-item drops are space-joined.
- Optional: single-directory auto-cd via `ATS_DND_AUTO_CD_DIRS=1`.
- Visuals: the focused terminal shows a crisp 2px blue border; while dragging files anywhere over the app, it also shows a subtle 4px blue glow.
- Removed: legacy pointer/rect hit-testing, pointer caching, session state, and DnD debug/tracing flags.
- Status: Implemented, built in release, and manually validated; working as expected.

## Simplified Implementation (How we did it)

- File: `audio-toolkit-shell/src-tauri/src/app.rs`
- Function: `AudioToolkitApp::handle_dnd_single_pass(...)`
  - Reads `i.raw.hovered_files`/`i.raw.dropped_files` from egui input once per frame.
  - Ignores pointer position and per-rect hit-testing; always targets `self.focused_terminal` when a drop occurs.
  - Shell-quotes dropped items and writes them to the focused terminal PTY; multi-item drops are space-joined with a trailing space.
  - If `ATS_DND_AUTO_CD_DIRS=1` and exactly one directory is dropped, inserts `cd '<dir>' ` instead.
- Visuals live in `render_terminal_panel(...)`: 2px blue focus border; during drag-hover, a subtle 4px blue glow on the focused panel.
- Code cleanup:
  - Removed legacy DnD fields (pointer cache/session, tracing flags) from `AudioToolkitApp`.
  - Deleted unused helpers: `handle_dnd_for_rect(...)`, `paint_dnd_hover(...)`, `paint_crosshair(...)`.

## Status and Testing

- Built with `cargo build --release` and launched.
- Manual validation:
  - Drops anywhere over the window go to the focused terminal only.
  - Multi-file drops are correctly quoted and space-joined with a trailing space.
  - Single directory + `ATS_DND_AUTO_CD_DIRS=1` inserts `cd '<dir>' `.
  - Focus remains on the target terminal; keyboard input continues there.
  - Verified at 100% and 125% scale; no panics or regressions observed.

## Configuration

- `ATS_DND_AUTO_CD_DIRS`: enable auto-cd behavior for single-directory drops (default: off).
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
  - If `ATS_DND_AUTO_CD_DIRS=1` and exactly one directory is dropped, inserts `cd '<dir>' ` instead.

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

- `ATS_DND_AUTO_CD_DIRS=1` enables auto `cd` for a single directory drop.

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
- [x] Single-directory auto-cd works when `ATS_DND_AUTO_CD_DIRS=1`.

</details>
