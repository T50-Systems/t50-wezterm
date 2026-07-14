# Crate Build Optimization Baseline

Date: 2026-07-14  
Platform: Windows MSVC  
Package: `wezterm-gui`  
Profile: `release`  
Cargo jobs: `2`

## Harness

```bash
python tools/measure-cargo-build.py clean \
  --package wezterm-gui --profile release --jobs 2 \
  --output-dir C:/t50bt
```

The harness never runs `cargo clean`. Clean runs use a unique target directory;
no-op and representative edit runs reuse a stable incremental target directory.
Timing HTML and logs remain local and are not committed.

## Baseline Results

| Scenario | Representative action | Elapsed |
|---|---|---:|
| Clean release | Empty isolated target | 1044.937 s |
| No-op release | No source changes | 1.500 s |
| Terminal implementation edit | Touch `term/src/terminalstate/performer.rs` | 204.563 s |

Local timing artifacts:

- `C:/t50bt/runs/20260714T053639Z-clean/cargo-timing.html`
- `C:/t50bt/runs/20260714T055421Z-noop/cargo-timing.html`
- `C:/t50bt/runs/20260714T055423Z-touch/cargo-timing.html`

## Baseline Discovery

`wezterm-gui/build.rs` assumed that artifacts always lived under the repository
`target/<profile>` directory. This made isolated `CARGO_TARGET_DIR` builds fail
while copying Windows runtime files. The build script now derives the profile
output directory from Cargo's `OUT_DIR`.

## Decision Thresholds

A retained crate boundary should normally meet at least one criterion without
violating the other:

- improve a representative incremental rebuild by at least 10%; or
- shorten/expose parallelism on the clean critical path without regressing the
  clean release build by more than 5%.

All comparisons must use the same package, profile, job count, toolchain, and
machine.
