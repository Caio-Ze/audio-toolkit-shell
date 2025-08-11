# Setting the Default Window Size and Panel Splits

This guide shows how to pick your preferred window size and panel split fractions in the app, then make them the defaults for next runs.


## 1) Run with resize/split debug logs

- Option A (overlay + resize logs):
  ```bash
  ATS_DEBUG_OVERLAY=1 cargo run --release
  ```
- Option B (only window logs):
  ```bash
  ATS_WINDOW_TRACE=1 cargo run --release
  ```

Notes:
- `ATS_DEBUG_OVERLAY=1` prints both window resize logs and splitter drag logs.
- `ATS_WINDOW_TRACE=1` prints window resize logs even without the overlay.


## 2) Interactively choose your layout

- Resize the window to your target default size.
  - The terminal prints lines like:
    ```
    [WINDOW] resized: inner_size_pts=1458.0x713.0 ppp=2.00 inner_size_px=2916x1426
    [CONFIG] app.window_width = 1458.0, app.window_height = 713.0
    ```
- Drag the right cluster splitter(s) to your desired fractions.
  - The overlay prints lines like:
    ```
    [SPLIT] Vertical drag dy=... new_top_frac=0.617
    [SPLIT] Horizontal drag dx=... new_hsplit_frac=0.500
    ```


## 3) Persist to the runtime config.toml

The app reads a `config.toml` next to the executable (or from `ATS_CONFIG_DIR` if set).

- For release builds, the file typically lives at:
  - `src-tauri/target/release/config.toml`
- Update these keys under the `[app]` section:
  - `window_width = <value from [CONFIG] log>`
  - `window_height = <value from [CONFIG] log>`
  - `right_top_fraction = <value from [SPLIT] Vertical drag>`
  - Optionally: `right_top_hsplit_fraction = <value from [SPLIT] Horizontal drag>`

Example:
```toml
[app]
name = "Audio Toolkit Shell"
window_width = 1458.0
window_height = 713.0
right_top_fraction = 0.617
right_top_hsplit_fraction = 0.500
```

Tip:
- To store the config elsewhere during testing, set `ATS_CONFIG_DIR` to a folder. The file will be created there on first run.


## 4) (Optional) Update first-run defaults in code

If you want new installs/first-run templates to use your chosen defaults, update the code defaults in `src-tauri/src/config.rs`:

- In `DEFAULT_CONFIG_TEMPLATE` (the TOML template string), change the `[app]` values:
  - `window_width = ...`
  - `window_height = ...`
  - optionally add `right_top_fraction = ...` and `right_top_hsplit_fraction = ...`
- In `default_config()`, set the same values on `AppSettings`:
  - `window_width`, `window_height`, `right_top_fraction`, `right_top_hsplit_fraction`
- If there’s a unit test asserting default sizes (e.g., `test_default_config()`), update the expected numbers accordingly.

After edits:
```bash
cd src-tauri
cargo test
cargo run --release
```


## 5) Validate

- Re-run with the overlay to visually confirm the window starts at your chosen size and splits:
  ```bash
  ATS_DEBUG_OVERLAY=1 cargo run --release
  ```
- If everything looks good, run normally without the flags.


## Quick reference

- Enable overlay + resize logs:
  ```bash
  ATS_DEBUG_OVERLAY=1 cargo run --release
  ```
- Enable only resize logs:
  ```bash
  ATS_WINDOW_TRACE=1 cargo run --release
  ```
- Release config file location:
  - `src-tauri/target/release/config.toml` (unless `ATS_CONFIG_DIR` is set)
- Code defaults and template:
  - `src-tauri/src/config.rs`


## Notes

- Window size logs are in logical points (same units as `window_width`/`window_height` in the TOML). Pixels are shown for reference.
- The left column remains fixed to 40% width by design; right cluster splitters are interactive and persist via the fractions above.
