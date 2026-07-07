# Clean Code / SOLID maturity analysis

Branch: `analysis/clean-code-solid`
Scope: T50 WezTerm fork, with emphasis on the pane-label / secondary-bar changes currently carried on top of upstream WezTerm.

## Executive summary

Overall maturity: **medium**.

The fork is built on top of a mature upstream codebase with established domain boundaries (`config`, `mux`, `termwindow`, `render`, Lua scripting). The T50 changes fit the existing style reasonably well and keep the feature behind opt-in configuration (`enable_secondary_bar`). However, the current implementation is still prototype-shaped in a few places: UI layout policy, rendering, hit-testing, Lua API state, and bar positioning are spread across several large modules without a single feature-level abstraction.

That is acceptable for a personal fork, but before treating pane labels / secondary bar as a stable foundation, the feature needs more encapsulation, tests around layout math, and clearer ownership of UI state.

## Maturity scorecard

| Dimension | Score | Notes |
| --- | ---: | --- |
| Readability | 3/5 | Names are understandable, but feature behavior is distributed across `tabbar.rs`, `termwindow/mod.rs`, `render/tab_bar.rs`, `render/fancy_tab_bar.rs`, and `mouseevent.rs`. |
| Single Responsibility | 2.5/5 | Existing modules already do a lot; the new feature adds more layout, state, rendering, and interaction rules to those modules. |
| Open/Closed Principle | 3/5 | The feature extends existing enums and render paths, but each new bar/status type requires changes in multiple match statements. |
| Liskov Substitution | N/A | Mostly procedural Rust/domain structs rather than inheritance/subtyping. No meaningful LSP concern observed. |
| Interface Segregation | 3/5 | Lua API surface is small (`set_secondary_bar`), but `TabBarState::new` has grown into a broad constructor that handles many modes. |
| Dependency Inversion | 2.5/5 | UI generation depends directly on config, mux-derived pane info, terminal line formatting, and render-layer details. This follows upstream style but limits isolated testing. |
| Testability | 2/5 | No local unit tests were found for tabbar layout, pane label truncation, secondary bar positioning, or hit-testing changes. |
| Stability risk | Medium | The feature touches render/layout/mouse hot paths. Bugs can surface as visual glitches, wrong click targets, or resize/title update loops. |

## What is clean already

### 1. Feature is opt-in

`config/src/config.rs` adds `enable_secondary_bar` with the default dynamic value, so stable behavior should remain unchanged unless the user opts in.

Why this is good:

- Lowers risk for the default terminal experience.
- Makes it easier to bisect feature regressions.
- Fits the personal-fork model: experimental UI can exist without destabilizing daily use.

### 2. Lua API is small

`wezterm-gui/src/scripting/guiwin.rs` adds a compact `SecondaryBarState` and `window:set_secondary_bar(...)` method.

Why this is good:

- Clear user-facing API boundary.
- Avoids leaking internal render types to Lua.
- Keeps the feature easy to script from `.wezterm.lua`.

Concern:

- The API shape is currently only `{ left, center, right }`; if future needs include per-pane labels, style, visibility, or click actions, this shape may become cramped.

### 3. Existing render model is reused

The implementation reuses `TabBarState`, `TabEntry`, `Line`, and existing fancy/non-fancy tabbar render paths instead of introducing an unrelated renderer.

Why this is good:

- Less new infrastructure.
- Visual style naturally follows existing tabbar colors.
- Lower chance of making a completely separate UI system.

## Main Clean Code concerns

### Concern 1: `TabBarState::new` is doing too much

`wezterm-gui/src/tabbar.rs` now handles:

- tab title computation,
- status text parsing,
- integrated title buttons,
- pane label generation,
- left/center/right zone placement,
- content modes (`Full`, `StatusOnly`),
- UI hit-test item generation.

This is the largest maintainability risk. The function is a central coordinator, but it now also contains feature policy.

Suggested direction:

- Extract a small `TabBarLayoutBuilder` or `BarLayout` helper.
- Extract pane label formatting into a dedicated function/struct with tests.
- Keep `TabBarState::new` as orchestration only.

Example target shape:

```rust
struct PaneLabelFormatter {
    zero_based: bool,
    max_title_graphemes: usize,
}

impl PaneLabelFormatter {
    fn label_for(&self, pane: &PaneInformation) -> String { ... }
}
```

### Concern 2: feature behavior is spread across too many modules

The secondary bar affects:

- `config/src/config.rs` — flag,
- `scripting/guiwin.rs` — Lua setter,
- `termwindow/mod.rs` — notification and status state,
- `tabbar.rs` — model/layout,
- `render/tab_bar.rs` — positioning and non-fancy rendering,
- `render/fancy_tab_bar.rs` — fancy rendering,
- `mouseevent.rs` and related callers — coordinate offsets / UI item handling.

This is normal for a UI feature, but there is no single local document or module that defines the invariant:

> When secondary bar is enabled, exactly one bar remains primary, a second status-only bar is laid out on the opposite edge, pane content offsets account for both bars, and UI hit-testing uses the same geometry as rendering.

Suggested direction:

- Add a small internal comment/doc near `secondary_tab_bar_enabled`, `top_bar_pixel_height`, `bottom_bar_pixel_height`, and `secondary_tab_bar_y` describing the invariant.
- Consider a `BarGeometry` struct that computes all bar positions once.

### Concern 3: string width / truncation is heuristic

`shorten_pane_title` truncates to 12 characters using `chars().count()` and `take(11)`.

Risk:

- Unicode graphemes and double-width characters may display incorrectly.
- This is a terminal UI; width should usually be cell-width-aware.

Suggested direction:

- Use grapheme-aware and/or `unicode_column_width` based truncation.
- Add tests for ASCII, emoji, CJK, empty title, long path, and whitespace-heavy title.

### Concern 4: click/hit-test behavior needs explicit tests

The feature adds `PaneStatus` as a `TabBarItem`, modifies tabbar UI items, and changes coordinate offsets for bars. These are easy to regress because render geometry and hit-test geometry must agree.

Suggested direction:

- Add focused tests for `TabBarState::compute_ui_items` with pane labels.
- Add tests that secondary bar top/bottom positions match `tab_bar_at_bottom` combinations.
- If full integration tests are hard, add pure tests for extracted geometry helpers.

### Concern 5: config naming is slightly ambiguous

`enable_secondary_bar` says there is a secondary bar, but not what kind of bar. It currently behaves like a second status/tabbar line, not a generic arbitrary UI strip.

Possible clearer names if still early:

- `enable_secondary_tab_bar`
- `enable_secondary_status_bar`
- `enable_pane_label_bar`

If the name is already part of your local config and you like it, keep it; this is a personal fork. But if you expect the API to evolve, now is the best time to rename it.

## SOLID-oriented findings

### Single Responsibility Principle

Current state: **weak-to-medium**.

The upstream architecture already centralizes many responsibilities in `TermWindow` and tabbar rendering modules. The T50 changes follow that pattern, but they increase the burden.

Most important SRP improvement:

- Extract secondary-bar geometry and pane-label formatting from rendering/event modules.

### Open/Closed Principle

Current state: **medium**.

Adding `PaneStatus` and secondary bars required touching several matches. This is idiomatic Rust for explicit enums, but it means every new tabbar item type will require coordinated edits.

This is not necessarily bad. In Rust UI code, explicit match exhaustiveness is often preferable to hidden dynamic dispatch. The key is to keep each match small and policy-free.

### Interface Segregation Principle

Current state: **medium**.

The Lua interface is small, which is good. Internal APIs are b
