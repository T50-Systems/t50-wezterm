# Plan to reach 8.5/10 Clean Code / SOLID maturity

Branch: `analysis/clean-code-solid`  
Target: raise the T50 pane-label / secondary-bar work from **medium maturity** to **8.5/10** while minimizing regressions through TDD and focused unit tests.

## Goal

Make the T50-specific UI additions stable enough for daily Windows use. Fedora remains a future portability concern, but the active regression guard for this plan is Windows-only.

The plan focuses on the custom surface area:

- `enable_secondary_bar`
- `window:set_secondary_bar(...)`
- pane labels in tab/status UI
- primary/secondary bar geometry
- tabbar hit-testing and mouse interaction
- fancy and non-fancy tabbar render paths

## Definition of 8.5/10 maturity

The work is considered 8.5/10 when:

1. Secondary bar geometry is centralized and unit-tested.
2. Pane label formatting is extracted, Unicode-aware, and unit-tested.
3. `TabBarState::new` is reduced to orchestration rather than feature policy.
4. Fancy and non-fancy rendering consume the same layout model.
5. Hit-test coordinates are derived from the same geometry used by rendering.
6. Lua API behavior is documented and covered by at least shape-level tests where practical.
7. Default behavior remains unchanged when `enable_secondary_bar = false`.
8. Tests cover Windows-only stability scenarios for now: 1 pane, 2 panes, 4 panes, tabbar top/bottom, fancy/non-fancy mode, Windows config variants.
9. Refactors are split into small commits where each commit has tests first or test updates first.
10. No broad upstream merge is required to achieve the cleanup.

## TDD rules for this effort

For each implementation phase:

1. Write or update failing unit tests first.
2. Run the narrowest relevant test command.
3. Implement the smallest change that passes.
4. Refactor only after tests pass.
5. Run the phase-level regression command before committing.

Preferred test commands:

```bash
cargo test -p wezterm-gui tabbar
cargo test -p wezterm-gui pane_label
cargo test -p wezterm-gui bar_geometry
cargo test -p config secondary_bar
```

If compile time is high, start with module-filtered tests, then run the broader package test before merging back.

## Phase 0 — baseline guard

### Purpose

Capture current behavior before refactoring so the cleanup has a safety net.

### Add tests first

Create targeted tests in or near `wezterm-gui/src/tabbar.rs` and extracted helper modules once introduced.

Baseline cases:

- primary tabbar with one tab and one pane is unchanged when `enable_secondary_bar = false`.
- pane labels are not emitted when there is only one pane.
- pane labels are emitted when there are multiple panes.
- active pane label is distinguishable from inactive pane labels.
- tab index remains one-based by default.
- `tab_and_split_indices_are_zero_based = true` affects pane/tab labels consistently.

### Success criteria

- Tests fail against intentionally changed expectations.
- Tests pass against current behavior.
- No production refactor yet.

## Phase 1 — extract PaneLabelFormatter

### Problem

Pane label string policy currently lives inside tabbar construction. It mixes title normalization, indexing, truncation, and display formatting with layout assembly.

### New unit under test

Add a small formatter, likely in `wezterm-gui/src/tabbar.rs` first or a new `wezterm-gui/src/tabbar/pane_label.rs` if module layout allows.

Target shape:

```rust
struct PaneLabelFormatter {
    zero_based: bool,
    max_cell_width: usize,
}

impl PaneLabelFormatter {
    fn format(&self, pane_index: usize, title: &str) -> String;
}
```

### Tests to add

- empty title becomes `shell` or the chosen fallback.
- path-like title keeps basename only.
- whitespace-heavy title is normalized.
- long ASCII title truncates with ellipsis.
- CJK/double-width title respects cell width.
- emoji/grapheme title does not split incorrectly.
- zero-based and one-based indexing both work.

### SOLID gain

- Improves Single Responsibility.
- Improves testability.
- Reduces accidental rendering regressions from label policy changes.

## Phase 2 — extract BarGeometry

### Problem

Primary and secondary bar positions are computed across render, title update, and mouse offset logic. This increases the chance that drawing and hit-testing disagree.

### New unit under test

Create a pure geometry helper that does not depend on window/render internals beyond primitive inputs.

Target shape:

```rust
struct BarGeometryInput {
    pixel_height: usize,
    border_top: usize,
    border_bottom: usize,
    tab_bar_height: f32,
    show_tab_bar: bool,
    tab_bar_at_bottom: bool,
    secondary_enabled: bool,
}

struct BarGeometry {
    primary_y: Option<f32>,
    secondary_y: Option<f32>,
    content_top_offset: f32,
    content_bottom_offset: f32,
}
```

### Tests to add

Matrix:

| Case | Expected |
| --- | --- |
| no tabbar | no primary/secondary, zero offsets |
| top primary, no secondary | primary top, content top offset one bar |
| bottom primary, no secondary | primary bottom, content bottom offset one bar |
| top primary + secondary | primary top, secondary bottom, both offsets |
| bottom primary + secondary | secondary top, primary bottom, both offsets |
| small window height | y positions clamp safely, no underflow |

### SOLID gain

- Centralizes geometry policy.
- Improves Dependency Inversion by making the logic pure.
- Reduces render/mouse divergence.

## Phase 3 — slim TabBarState construction

### Problem

`TabBarState::new` currently handles full tabbar construction and status-only secondary bar construction via `TabBarContentMode`.

### Refactor target

Split public constructors:

```rust
impl TabBarState {
    pub fn new_primary(...);
    pub fn new_status_bar(...);
}
```

Keep a private shared builder if needed.

### Tests to add

- `new_primary` includes tabs/new-tab/window buttons according to config.
- `new_status_bar` never includes tab activation items.
- left/center/right status zones are preserved.
- pane labels appear only in the intended bar/mode.

### SOLID gain

- Improves Interface Segregation.
- Makes invalid combinations harder to express.
- Reduces branching inside one constructor.

## Phase 4 — unify fancy and non-fancy consumption

### Problem

Fancy and non-fancy rendering both consume `TabBarState`, but feature-specific rendering decisions are duplicated or mirrored in separate paths.

### Refactor target

Keep renderer-specific drawing separate, but make both renderers consume the same precomputed layout entries and UI item types.

### Tests to add

Unit-test what can be pure:

- `TabBarState::compute_ui_items` returns same item count/geometry independent of fancy vs non-fancy when given same line widths.
- `PaneStatus` UI item width and x-position match its line entry.
- Secondary bar UI items use secondary y-coordinate.

### SOLID gain

- Reduces duplicate policy.
- Keeps renderer differences focused on drawing, not layout decisions.

## Phase 5 — mouse hit-testing guard

### Problem

Pane labels and secondary bars add more clickable UI. Mouse routing must not steal terminal mouse events accidentally and must keep tab/pane interactions correct.

### Tests to add

Pure tests around `UIItem::hit_test` and generated UI items:

- click within pane label hits `TabBarItem::PaneStatus`.
- click one cell outside pane label does not hit it.
- secondary bar y-coordinate is respected.
- terminal content y-coordinate starts below top bars.
- bottom bar does not affect top content offset.

If `mouseevent.rs` is too integrated for direct unit tests, extract coordinate conversion into a pure helper and test that.

### SOLID gain

- Separates coordinate conversion from event dispatch.
- Lowers risk of regressions in mouse-heavy TUIs.

## Phase 6 — Lua API hardening

### Problem

`set_secondary_bar` currently accepts a full state object. Future changes could accidentally break user config without tests or docs.

### Tests / checks to add

- Validate dynamic conversion for `{ left, center, right }`.
- Decide whether missing fields should default to empty string or error.
- Ensure disabling `enable_secondary_bar` ignores secondary state visually but does not crash.

### Documentation to add

Create:

```text
docs/t50/secondary-bar.md
```

Include:

```lua
config.enable_secondary_bar = true
wezterm.on('update-status', function(window, pane)
  window:set_secondary_bar({
    left = '',
    center = '...',
    right = '',
  })
end)
```

### SOLID gain

- Clearer interface contract.
- Less accidental API drift.

## Phase 7 — regression suite and branch policy

### Minimum guard before merging cleanup to `main`

Run:

```bash
cargo test -p config secondary_bar
cargo test -p wezterm-gui pane_label
cargo test -p wezterm-gui bar_geometry
cargo test -p wezterm-gui tabbar
cargo test -p wezterm-gui
```

If full `wezterm-gui` tests are too slow locally, record that in the PR/commit notes and at least run all new focused tests.

### Manual smoke matrix

Windows only for this plan:

- stable Windows config with `front_end = 'Software'.
- future Windows config with secondary bar enabled.
- 1 pane, 2 panes, 4 panes.
- horizontal and vertical splits.
- tabbar top and bottom.
- fancy and non-fancy tabbar.
- close panes/tabs repeatedly.
- resize window repeatedly.
- sleep/resume once if practical.
- run a noisy TUI/log output workload inside a pane.

Fedora/Wayland validation is intentionally deferred until that migration starts. Do not block this cleanup branch on Linux compositor behavior.

## Commit plan

Recommended commits:

1. `test: capture pane label and secondary bar baseline`
2. `refactor: extract pane label formatting`
3. `test: cover bar geometry matrix`
4. `refactor: centralize tab bar geometry`
5. `refactor: split primary and status tab bar builders`
6. `test: guard tab bar UI item hit testing`
7. `docs: document secondary bar lua api`
8. `chore: record clean code maturity guard commands`

## Risk management

- Do not refactor render and mouse handling in the same commit.
- Do not rename config/Lua APIs after tests are written unless the rename is the entire commit.
- Keep `enable_secondary_bar = false` behavior as the default compatibility baseline.
- Prefer pure helper extraction over introducing trait abstractions.
- Avoid large upstream merges during this cleanup branch.

## Expected final score

If all phases are completed:

| Dimension | Expected score |
| --- | ---: |
| Readability | 4/5 |
| Single Responsibility | 4/5 |
| Open/Closed | 3.5/5 |
| Interface Segregation | 4/5 |
| Dependency Inversion | 3.5/5 |
| Testability | 4.5/5 |
| Stability guard | 4.5/5 |

Overall target: **8.5/10** for the T50-specific feature slice.

This does not mean the entire upstream WezTerm codebase becomes 8.5/10. It means the custom T50 pane-label / secondary-bar layer has enough boundaries and tests to be safe for personal daily use.
