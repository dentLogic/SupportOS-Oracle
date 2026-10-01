# PROGRESS

Current slice: S1 (Pipeline)
Current task: Slice 1 pushed with a working token; first CI run in progress
Last commit: (see `git log -1` — CI hardening commit on top of the four Slice 1 commits)
Latest CI result: pending (first run triggered by the initial push)

Local pre-verification done before the push (sandbox has no webkit/GTK system
libraries, so the Tauri crate cannot compile locally; CI remains the authority):
- `cargo fmt --all --check` green on the whole workspace.
- `cargo clippy -p supportos-oracle-ui --all-targets -- -D warnings` green
  (both wasm32 and host targets).
- `cargo generate-lockfile` produced Cargo.lock (534 packages: tauri 2.12.1,
  leptos 0.8.21, wasm-bindgen 0.2.129); it is committed with the scaffold.
- `trunk build` green: wasm32 build, wasm-bindgen applied, dist/ produced and
  `frontendDist: ../dist` resolves from src-tauri.
- `rustup toolchain install` (from rust-toolchain.toml) verified locally.

CI hardening applied on top of the scaffold (one commit):
- clippy runs `--workspace` so src-tauri is linted too (it previously only
  covered the root UI crate).
- The DEB upload path covers both `target/release/bundle/deb/` (workspace-root
  target, where cargo puts it for this workspace layout) and the legacy
  `src-tauri/target/...` path, with `if-no-files-found: error`.
- Trunk and the Tauri CLI install via taiki-e/install-action (pinned
  trunk@0.21.14, tauri-cli@2.12.1 — official prebuilt binaries with default
  features) instead of `cargo install` (saves ~10 min per run).
- `rustup toolchain install` instead of `rustup show` (rustup warns that
  auto-install on proxy invocation is deprecated).
- `libgtk-3-dev` added to the prerequisites (mission lists GTK explicitly).
- rust-toolchain.toml gained `profile = "minimal"`.

Next 3 tasks:
1. Watch the first CI run; fix the FIRST error only; repeat until green.
2. Verify the "nightly" pre-release carries the .deb and its SHA-256 checksum.
3. Report the session results; wait for docs/SPEC.md and docs/SPEC-AMENDMENTS.md
   before starting Slice 2 (only Slice 1 is in scope until they exist).

Known issues:
- docs/SPEC.md and docs/SPEC-AMENDMENTS.md are missing from the repo: per the
  mission, only Slice 1 is in scope until they are added.
- Still unverified by any CI run: src-tauri clippy/test, `cargo tauri build
  --bundles deb`, and the nightly/tag release jobs.

Pages done: none (Slice 1 has no application pages; the window shows the title
and the app version from the `app_version` command)
Pages remaining: all reference pages (S2 creates the navigation shell with
honest "not built yet" screens)
