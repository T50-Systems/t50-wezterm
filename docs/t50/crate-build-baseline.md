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

## Codegen and Link Follow-up

The final GUI unit was probed separately after the crate work. A failed MSVC
link-timing probe reached the linker only after about 86 seconds, while a complete
rust-lld build took 85.5 seconds. This indicates that backend code generation, not
the final linker alone, accounts for most of the roughly 82–86-second GUI unit.
rust-lld was therefore not adopted as the default linker.

An opt-in `release-fast` profile now retains release optimization level 3 while
enabling incremental compilation and 256 codegen units:

```bash
cargo fast -p wezterm-gui

python tools/measure-cargo-build.py touch \
  --package wezterm-gui --profile release-fast --jobs 2 \
  --touch wezterm-gui/src/commands/palette.rs
```

Shipping and CI artifacts continue to use the unchanged `release` profile.

Measured on the same Windows MSVC machine:

| Scenario | Standard release | `release-fast` | Change |
|---|---:|---:|---:|
| Clean build | 1095.532 s | 1096 s | +0.04% |
| No-op build | 1.594 s | 1.519 s | -4.71% |
| Semantic GUI edit | 83.516 s baseline GUI edit | 13.312 s | -84.06% |
| Final GUI unit during clean build | 85.6 s | 76.0 s | -11.21% |

The clean `release-fast` run also confirmed that removing the build-time `git2`
dependency from `wezterm-version` eliminates one host `libgit2-sys` build. The
previous clean timing contained two `libgit2-sys` build-script runs (91.2 and
64.7 seconds); the follow-up contains one 91.6-second run needed by the runtime
plugin path. Wall-clock clean time remains dominated by the vendored static
OpenSSL build, which measured 509.2 seconds in the follow-up run.

Local timing artifacts are under `C:/t50bt-fast/`.

## Windows OpenSSL CI Cache

Windows CI restores a dedicated cache of the OpenSSL installation produced by
the locked `openssl-src` dependency. On a cache hit,
`ci/windows-openssl-cache.ps1` sets `OPENSSL_NO_VENDOR=1`, `OPENSSL_DIR`, and
`OPENSSL_STATIC=1`; `openssl-sys` then links the restored static libraries instead
of rebuilding OpenSSL. A cache miss in the trusted warmer preserves the existing
vendored build and publishes its installation for later consumer runs.

The cache fingerprint includes `Cargo.lock`, every workspace manifest, the
static-CRT Cargo configuration, the cache helper, rustc identity, hosted-runner
image identity, and the visible MSVC compiler version. The cache itself carries a
manifest that is checked before activation; an invalid restore is discarded and
falls back to the vendored build. A dedicated workflow on `main` and `dev`, plus
a daily default-branch refresh, seeds the cache so pull requests and tagged builds
can restore it. Consumers only restore caches; the trusted warmer saves repaired
or newly keyed archives, allowing an invalid immutable cache to self-heal.

Production `release`, static CRT linkage, package contents, and the local fallback
remain unchanged. The expected warm-cache ceiling is the roughly 509-second
native OpenSSL step; CI measurements must confirm the realized saving.

## Native C/C++ sccache Coverage

The measured clean timings for `cairo-sys-rs` (48.6 seconds),
`libssh-rs-sys` (25.1 seconds), and `libgit2-sys` (91.6 seconds) came from
the local isolated harness without `RUSTC_WRAPPER`. Windows CI already sets
`RUSTC_WRAPPER=sccache` with the GitHub Actions backend. The locked `cc 1.2.63`
uses a compatible `RUSTC_WRAPPER` automatically as the C/C++ compiler wrapper,
and all three native build scripts compile through `cc::Build`. Their unchanged
MSVC object files are therefore already eligible for the shared sccache.

The generated Windows workflows reset sccache statistics immediately before the
four release builds and report them immediately afterward. This separates release
cache evidence from the later test-profile build and makes C/C++ hit rates visible
before introducing a more fragile prebuilt-library or Cargo-target cache.
