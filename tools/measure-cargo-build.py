#!/usr/bin/env python3
"""Measure reproducible Cargo clean and incremental build scenarios.

The script never runs `cargo clean`. Clean measurements use a unique target
folder, while incremental/no-op measurements reuse a stable target folder.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import cast


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "scenario",
        choices=("clean", "noop", "touch"),
        help="clean uses an isolated target; noop/touch reuse the incremental target",
    )
    parser.add_argument("--package", default="wezterm-gui")
    parser.add_argument("--profile", choices=("dev", "release"), default="release")
    parser.add_argument(
        "--jobs",
        type=int,
        help="limit Cargo parallel jobs for memory-constrained machines",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path(".t50-build-timings"),
    )
    parser.add_argument(
        "--touch",
        type=Path,
        help="representative source file to invalidate for the touch scenario",
    )
    parser.add_argument(
        "--no-default-features",
        action="store_true",
        help="pass --no-default-features to cargo build",
    )
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def utc_stamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def target_dir(output: Path, scenario: str, stamp: str) -> Path:
    if scenario == "clean":
        return output / f"target-clean-{stamp}"
    return output / "target-incremental"


def cargo_command(args: argparse.Namespace) -> list[str]:
    command = ["cargo", "build", "--package", args.package, "--timings"]
    jobs = cast(int | None, args.jobs)
    if jobs is not None:
        if jobs < 1:
            raise SystemExit("--jobs must be at least 1")
        command.extend(("--jobs", str(jobs)))
    if args.profile == "release":
        command.append("--release")
    if args.no_default_features:
        command.append("--no-default-features")
    return command


def find_timing_html(target: Path) -> Path | None:
    timing_dir = target / "cargo-timings"
    preferred = timing_dir / "cargo-timing.html"
    if preferred.exists():
        return preferred
    matches = sorted(timing_dir.glob("cargo-timing*.html"))
    return matches[-1] if matches else None

def display_path(path: Path | None, root: Path) -> str | None:
    if path is None:
        return None
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def main() -> int:
    args = parse_args()
    root = repo_root()
    output_arg = cast(Path, args.output_dir)
    output = output_arg if output_arg.is_absolute() else root / output_arg
    scenario = cast(str, args.scenario)
    touch_arg = cast(Path | None, args.touch)
    stamp = utc_stamp()
    run_dir = output / "runs" / f"{stamp}-{scenario}"
    run_dir.mkdir(parents=True, exist_ok=False)

    if scenario == "touch" and touch_arg is None:
        raise SystemExit("--touch is required for the touch scenario")
    if scenario != "touch" and touch_arg is not None:
        raise SystemExit("--touch is only valid for the touch scenario")

    target = target_dir(output, scenario, stamp)
    target.mkdir(parents=True, exist_ok=True)
    command = cargo_command(args)
    environment = os.environ.copy()
    environment["CARGO_TARGET_DIR"] = str(target)

    touched_path: Path | None = None
    original_times: tuple[int, int] | None = None
    if touch_arg is not None:
        candidate = (root / touch_arg).resolve()
        if not candidate.is_file() or root not in candidate.parents:
            raise SystemExit(f"touch path is not a repository file: {touch_arg}")
        touched_path = candidate
        stat = touched_path.stat()
        original_times = (stat.st_atime_ns, stat.st_mtime_ns)
        os.utime(touched_path, None)

    started = time.monotonic()
    try:
        with (run_dir / "build.log").open("w", encoding="utf-8") as log:
            result = subprocess.run(
                command,
                cwd=root,
                env=environment,
                stdout=log,
                stderr=subprocess.STDOUT,
                check=False,
            )
    finally:
        if touched_path is not None and original_times is not None:
            os.utime(touched_path, ns=original_times)
    elapsed = time.monotonic() - started

    timing = find_timing_html(target)
    copied_timing: Path | None = None
    if timing is not None:
        timing_path = cast(Path, timing)
        copied_timing = run_dir / timing_path.name
        shutil.copy2(timing_path, copied_timing)

    summary = {
        "scenario": scenario,
        "package": args.package,
        "profile": args.profile,
        "command": command,
        "target_dir": display_path(target, root),
        "touch": str(touch_arg) if touch_arg else None,
        "elapsed_seconds": round(elapsed, 3),
        "exit_code": result.returncode,
        "timing_html": display_path(copied_timing, root),
        "timestamp_utc": stamp,
    }
    (run_dir / "summary.json").write_text(
        json.dumps(summary, indent=2) + "\n", encoding="utf-8"
    )
    print(json.dumps(summary, indent=2))
    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
