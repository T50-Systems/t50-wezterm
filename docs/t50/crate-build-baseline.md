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

## Post-refactor Results

The post-refactor measurements use the same machine, package, profile, and
two-job limit. The pre-refactor scenarios were rerun from commit `475d1a99e`
against its existing incremental target to provide equivalent comparisons.

| Scenario | Before | After | Change |
|---|---:|---:|---:|
| Clean release | 1044.937 s | 1095.532 s | +4.84% |
| No-op release | 1.500 s | 1.594 s | +6.27% |
| Terminal implementation edit | 204.563 s | 187.828 s | -8.18% |
| Config runtime edit | 283.796 s | 224.672 s | -20.83% |
| Mux backend edit | 154.078 s | 145.593 s | -5.51% |
| GUI render edit | 87.485 s | 83.516 s | -4.54% |
| Custom-glyph mapping edit | 86.875 s | 83.625 s | -3.74% |
| GUI command derivation edit | 87.453 s | 87.562 s | +0.12% |

Local post-refactor timing artifacts are under `C:/t50bt-post/runs/`.

## Interpretation

- The strongest measured gain is config runtime isolation at 20.83%.
- Terminal and mux implementation edits also improved, although they remain
  below the preferred 10% threshold individually.
- Moving custom glyph and GUI input code reduces the main GUI compilation unit
  and exposes independent clean-build jobs, but final GUI codegen/linking still
  dominates their representative edit scenarios.
- Clean release time regressed 4.84%, remaining inside the ADR's 5% limit.
- The boundaries remain accepted for this iteration because the combined change
  materially improves a high-cost config edit, reduces multiple rebuild paths,
  and exposes explicit parallel units without crossing the clean-build stop
  threshold. Future work should target final GUI codegen/link time rather than
  adding finer crates indiscriminately.
