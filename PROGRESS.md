# PROGRESS

Current slice: S1 (Pipeline) — COMPLETE per its exit criteria (green CI, .deb on the nightly release)
Current task: Slice 1 done; waiting for docs/SPEC.md and docs/SPEC-AMENDMENTS.md before Slice 2
Last commit: 50d3f2b (plus this PROGRESS.md update on top)
Latest CI result: SUCCESS — run 36865558666
(https://github.com/dentLogic/SupportOS-Oracle/actions/runs/36865558666)
on commit 50d3f2bd: fmt, clippy --workspace -D warnings, test, trunk build,
cargo tauri build --bundles deb, artifact upload, nightly release all green.

What the green run proves (verified evidence, not claims):
- The whole workspace (UI crate + src-tauri) compiles, is rustfmt-clean and
  clippy-clean with warnings denied on the pinned toolchain (Rust 1.98.1).
- `cargo test --workspace` passes (app_version command tests).
- `trunk build` produces the WASM frontend and `cargo tauri build --bundles deb`
  produces SupportOS-Oracle_0.1.0_amd64.deb, uploaded as a CI artifact.
- The "nightly" pre-release exists with the .deb and its SHA-256 checksum; the
  checksum was re-downloaded and verified to match the .deb.
- The .deb was inspected: Package support-os-oracle, Depends libwebkit2gtk-4.1-0
  and libgtk-3-0, desktop entry at usr/share/applications/SupportOS-Oracle.desktop,
  hicolor icons, binary at usr/bin/supportos-oracle.

Known issues:
- docs/SPEC.md and docs/SPEC-AMENDMENTS.md are missing from the repo: per the
  mission, ONLY Slice 1 is in scope until they are added. Waiting on the owner.
- Depends lists libwebkit2gtk-4.1-0 and libgtk-3-0 twice each (explicit
  declaration in tauri.conf.json plus the bundler's auto-detection). Harmless;
  will be cleaned up when the packaging slice revisits it.

Next 3 tasks (blocked on the specs being added):
1. Owner adds docs/SPEC.md and docs/SPEC-AMENDMENTS.md to the repo.
2. Slice 2 Foundation: workspace restructure into crates/core + crates/ui + the
   Tauri shell, CI green before and after.
3. Slice 2 continues: SQLite (WAL, FTS5, migrations), settings, job queue,
   error type, logging, theming, UI shell with navigation and honest
   "not built yet" screens for every reference page.

Pages done: the main window — title "SupportOS Oracle" and the app version
displayed through the working `app_version` command (launches via the desktop
entry the .deb installs).
Pages remaining: all reference pages (S2 creates the navigation shell with
honest "not built yet" screens).

History of this slice (for the record):
- a614bf0, def3e75, c7b7b65, 434905e: scaffold, CI, autofix, docs (locally
  pre-verified: fmt, clippy on the UI crate, lockfile, a full trunk build).
- 79a87a7: CI hardening (workspace-wide clippy, prebuilt pinned tools,
  explicit toolchain install, DEB path coverage, libgtk-3-dev).
  First CI run (36864306949): build job green; nightly job failed — the
  artifact nested the .deb under target/release/bundle/deb/, so the flat
  *.deb glob in the release job found no files.
- 50d3f2b: fix the artifact path (single search path; the DEB lives in the
  workspace-root target directory). Second CI run (36865558666): fully green,
  nightly assets in place.
