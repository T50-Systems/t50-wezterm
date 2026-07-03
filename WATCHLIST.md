# T50 WezTerm watchlist

This fork is personal/T50-focused. It is **not** intended to continuously track
upstream WezTerm. The goal is to keep a stable terminal for my Windows workflow
and future Fedora workflow, while selectively borrowing fixes or ideas from
upstream only when they match my needs.

## When to check upstream

Check this list when:

- WezTerm crashes, hangs, or loses pane state.
- I update my local build or rebase this fork.
- I move to Fedora/Wayland.
- I work on pane labels, secondary bar, mux, or rendering.
- An upstream issue below is closed with a fix that applies cleanly to this fork.

## High priority: crashes, stack overflow, panes, mux

| Issue | Why it matters |
| --- | --- |
| [#7861 — wezterm-mux-server.exe crashes with STATUS_STACK_OVERFLOW when 26+ panes](https://github.com/wezterm/wezterm/issues/7861) | Same exception class as my Windows crash: `0xc00000fd` / stack overflow. Watch for root cause or mux recursion fixes. |
| [#7527 — mux: Unbounded PDU memory allocation causes OOM crashes and stack overflow](https://github.com/wezterm/wezterm/issues/7527) | Has minimal repro/root cause; relevant to mux stability and crashes under heavy pane/session traffic. |
| [#7847 — Crash when using pane:move_to_new_window when single pane in window](https://github.com/wezterm/wezterm/issues/7847) | Pane/Lua/UI crash with root cause; relevant because this fork touches pane/tab UI. |
| [#4390 — Switching panes in multiple directions rapidly switches focus](https://github.com/wezterm/wezterm/issues/4390) | Pane focus/mux behavior; relevant if pane labels or navigation regress. |
| [#6094 — CloseCurrentTab can crash in certain sequences/timings](https://github.com/wezterm/wezterm/issues/6094) | Timing-sensitive tab/pane close crash; relevant to UI state changes. |

## Windows stability

| Issue | Why it matters |
| --- | --- |
| [#5992 — Windows crash: OpenGL context was lost; should reinit](https://github.com/wezterm/wezterm/issues/5992) | Relevant to Windows GPU/OpenGL stability. My stable config avoids forced OpenGL. |
| [#7519 — Windows sleep/resume causes wezterm-gui.exe crash in d3d11.dll](https://github.com/wezterm/wezterm/issues/7519) | Relevant on Windows laptops/desktops after sleep/resume. |
| [#7824 — Windows GPU timeout followed by wezterm-gui APPCRASH](https://github.com/wezterm/wezterm/issues/7824) | GPU timeout crash class; watch if using hardware rendering again. |
| [#7819 — WebGPU backend panics in Surface::configure](https://github.com/wezterm/wezterm/issues/7819) | Avoid WebGPU unless this class of issue is fixed. |
| [#7774 — Update bundled Windows ConPTY pair](https://github.com/wezterm/wezterm/issues/7774) | Relevant for PowerShell/TUI apps and Windows console backend stability. |

## Fedora / Linux / Wayland

| Issue | Why it matters |
| --- | --- |
| [#6815 — WebGPU + Wayland fractional scaling crash](https://github.com/wezterm/wezterm/issues/6815) | Important before using Fedora Wayland with fractional scaling. |
| [#7880 — Wayland drag-and-drop panic](https://github.com/wezterm/wezterm/issues/7880) | Recent Wayland crash; avoid or patch before relying on drag/drop. |
| [#6233 — GNOME launcher crash on 200% scaling](https://github.com/wezterm/wezterm/issues/6233) | Relevant for GNOME/Fedora HiDPI setups. |
| [#7725 — Wayland crash on large output](https://github.com/wezterm/wezterm/issues/7725) | Relevant for large command output/log streaming. |
| [#2445 — Sway scaling crash](https://github.com/wezterm/wezterm/issues/2445) | Relevant if using Sway/Wayland scaling. |

## Local policy

- Do not merge upstream just because new commits exist.
- Prefer small cherry-picks or local patches for issues that match my symptoms.
- Keep the stable config conservative first; use experimental renderer/UI only when needed.
- If a crash happens, collect local logs/dumps before changing code.
