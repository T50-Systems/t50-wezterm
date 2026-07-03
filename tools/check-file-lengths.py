#!/usr/bin/env python3
"""Fail when maintained source files exceed a line-count threshold.

This intentionally ignores vendored dependencies, generated data, binary assets,
large docs assets, and lockfiles. It is meant to guard hand-maintained project
source after large-file refactors land.
"""

from __future__ import annotations

import argparse
import fnmatch
import subprocess
import sys
from pathlib import Path

DEFAULT_LIMIT = 500

EXCLUDED_PREFIXES = (
    "assets/",
    "bidi/data/",
    "deps/",
    "docs/screenshots/",
    "test-data/",
)

EXCLUDED_FILES = {
    "Cargo.lock",
    "docs/colorschemes/data.json",
    "docs/config/lua/wezterm/nerdfonts.md",
    "docs/asciinema-player.css",
    "docs/changelog.md",
    "docs/config/appearance.md",
    "wezterm-char-props/data/emoji-data.txt",
    "wezterm-char-props/data/emoji-variation-sequences.txt",
    "color-types/src/rgb.txt",
    "wezterm-char-props/src/emoji_variation.rs",
    "wezterm-char-props/src/nerdfonts_data.rs",
    "wezterm-gui/src/unicode_names.rs",
}

EXCLUDED_GLOBS = (
    "*.dll",
    "*.dylib",
    "*.exe",
    "*.icns",
    "*.mp4",
    "*.png",
    "*.ttf",
)

# Temporary migration allowlist. Remove entries as each phase splits a file.
TEMP_ALLOWED = {
    "bidi/src/bidi_class.rs",
    "bidi/src/lib.rs",
    "bintree/src/lib.rs",
    "ci/generate-docs.py",
    "ci/generate-workflows.py",
    "codec/src/lib.rs",
    "config/src/color.rs",
    "config/src/config.rs",
    "config/src/font.rs",
    "config/src/keyassignment.rs",
    "config/src/lib.rs",
    "config/src/lua.rs",
    "config/src/scheme_data.rs",
    "filedescriptor/src/unix.rs",
    "filedescriptor/src/windows.rs",
    "lfucache/src/lib.rs",
    "mux/src/domain.rs",
    "mux/src/lib.rs",
    "mux/src/localpane.rs",
    "mux/src/pane.rs",
    "mux/src/ssh.rs",
    "mux/src/tab.rs",
    "mux/src/termwiztermtab.rs",
    "mux/src/tmux_commands.rs",
    "pty/src/cmdbuilder.rs",
    "term/src/screen.rs",
    "term/src/terminalstate/kitty.rs",
    "term/src/terminalstate/mod.rs",
    "term/src/terminalstate/performer.rs",
    "term/src/test/mod.rs",
    "termwiz/src/caps/mod.rs",
    "termwiz/src/input.rs",
    "termwiz/src/lineedit/mod.rs",
    "termwiz/src/render/terminfo.rs",
    "termwiz/src/terminal/unix.rs",
    "termwiz/src/terminal/windows.rs",
    "termwiz/src/widgets/layout.rs",
    "termwiz/src/widgets/mod.rs",
    "vtparse/src/lib.rs",
    "wezterm-cell/src/image.rs",
    "wezterm-cell/src/lib.rs",
    "wezterm-char-props/src/widechar_width.rs",
    "wezterm-client/src/client.rs",
    "wezterm-client/src/domain.rs",
    "wezterm-client/src/pane/clientpane.rs",
    "wezterm-client/src/pane/renderable.rs",
    "wezterm-escape-parser/src/apc.rs",
    "wezterm-escape-parser/src/csi.rs",
    "wezterm-escape-parser/src/lib.rs",
    "wezterm-escape-parser/src/osc.rs",
    "wezterm-escape-parser/src/parser/mod.rs",
    "wezterm-escape-parser/src/tmux_cc/mod.rs",
    "wezterm-font/src/ftwrap.rs",
    "wezterm-font/src/hbwrap.rs",
    "wezterm-font/src/lib.rs",
    "wezterm-font/src/parser.rs",
    "wezterm-font/src/rasterizer/colr.rs",
    "wezterm-font/src/rasterizer/freetype.rs",
    "wezterm-font/src/shaper/harfbuzz.rs",
    "wezterm-gui/src/commands.rs",
    "wezterm-gui/src/customglyph.rs",
    "wezterm-gui/src/frontend.rs",
    "wezterm-gui/src/glyphcache.rs",
    "wezterm-gui/src/inputmap.rs",
    "wezterm-gui/src/main.rs",
    "wezterm-gui/src/overlay/copy.rs",
    "wezterm-gui/src/overlay/launcher.rs",
    "wezterm-gui/src/overlay/quickselect.rs",
    "wezterm-gui/src/renderstate.rs",
    "wezterm-gui/src/shapecache.rs",
    "wezterm-gui/src/termwindow/background.rs",
    "wezterm-gui/src/termwindow/box_model.rs",
    "wezterm-gui/src/termwindow/charselect.rs",
    "wezterm-gui/src/termwindow/keyevent.rs",
    "wezterm-gui/src/termwindow/mouseevent.rs",
    "wezterm-gui/src/termwindow/palette.rs",
    "wezterm-gui/src/termwindow/render/fancy_tab_bar.rs",
    "wezterm-gui/src/termwindow/render/mod.rs",
    "wezterm-gui/src/termwindow/render/pane.rs",
    "wezterm-gui/src/termwindow/render/screen_line.rs",
    "wezterm-gui/src/termwindow/resize.rs",
    "wezterm-gui/src/termwindow/webgpu.rs",
    "wezterm-mux-server-impl/src/sessionhandler.rs",
    "wezterm-ssh/src/config.rs",
    "wezterm-ssh/src/sessioninner.rs",
    "wezterm-ssh/tests/e2e/sftp.rs",
    "wezterm-ssh/tests/sshd.rs",
    "wezterm-surface/src/lib.rs",
    "wezterm-surface/src/line/line.rs",
    "wezterm-surface/src/line/test.rs",
    "wezterm/src/asciicast.rs",
    "wezterm/src/main.rs",
    "window/src/egl.rs",
    "window/src/os/macos/window.rs",
    "window/src/os/wayland/frame.rs",
    "window/src/os/wayland/window.rs",
    "window/src/os/windows/window.rs",
    "window/src/os/x11/connection.rs",
    "window/src/os/x11/cursor.rs",
    "window/src/os/x11/keyboard.rs",
    "window/src/os/x11/window.rs",
}


def git_files() -> list[str]:
    output = subprocess.check_output(["git", "ls-files"], text=True)
    return output.splitlines()


def excluded(path: str) -> bool:
    if path in EXCLUDED_FILES:
        return True
    if path.startswith(EXCLUDED_PREFIXES):
        return True
    return any(fnmatch.fnmatch(path, pattern) for pattern in EXCLUDED_GLOBS)


def line_count(path: Path) -> int:
    data = path.read_bytes()
    return data.count(b"\n") + (1 if data and not data.endswith(b"\n") else 0)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--limit", type=int, default=DEFAULT_LIMIT)
    parser.add_argument(
        "--strict",
        action="store_true",
        help="also fail temporary allowlist entries; intended for final cleanup",
    )
    args = parser.parse_args()

    offenders: list[tuple[int, str, bool]] = []
    for path in git_files():
        if excluded(path):
            continue
        p = Path(path)
        if not p.is_file():
            continue
        count = line_count(p)
        if count > args.limit:
            allowed = path in TEMP_ALLOWED and not args.strict
            offenders.append((count, path, allowed))

    offenders.sort(reverse=True)
    blocking = [(n, p) for n, p, allowed in offenders if not allowed]
    allowed = [(n, p) for n, p, allowed in offenders if allowed]

    if blocking:
        print(f"Files over {args.limit} lines:", file=sys.stderr)
        for count, path in blocking:
            print(f"{count:>6}  {path}", file=sys.stderr)
    if allowed:
        print(f"Temporary allowlist over {args.limit} lines ({len(allowed)}):")
        for count, path in allowed[:50]:
            print(f"{count:>6}  {path}")
        if len(allowed) > 50:
            print(f"... {len(allowed) - 50} more")

    return 1 if blocking else 0


if __name__ == "__main__":
    raise SystemExit(main())
