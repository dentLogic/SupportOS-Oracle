# DECISIONS

Key decisions for SupportOS Oracle, newest first. Each entry states the decision,
the reason, and the trade-off. Everything here is verified by CI runs or marked
otherwise.

## Slice 1

- **Template**: the official create-tauri-app Leptos template
  (tauri-apps/create-tauri-app), mirrored exactly in layout: root UI crate
  `supportos-oracle-ui` (Leptos CSR, built by Trunk) as the workspace root with
  `members = ["src-tauri"]`, and `src-tauri` with the standard lib/bin split.
  Reason: the mission requires it and it is the layout the ecosystem tests.
  Trade-off: the frontend and backend live in one workspace from the start; the
  core/ui/shell split into crates/ is deliberately deferred to Slice 2.

- **Identity**: crate `supportos-oracle` / lib `supportos_oracle_lib`
  (create-tauri-app derivation), identifier
  `com.dentlogic.supportos-oracle` (the template's default
  `com.<user>.<package_name>`), `productName` `SupportOS-Oracle` (keeps the DEB
  file name clean; the deb bundler uses the raw product name for the file name
  and kebab-case for the package name), window title `SupportOS Oracle`.

- **Versions pinned**: Rust 1.98.1 via rust-toolchain.toml (with the
  wasm32-unknown-unknown target, rustfmt and clippy components — `rustup show`
  installs it on CI), Tauri 2 (crate requirement `2`, resolved exactly by the
  Cargo.lock the Autofix workflow commits; tauri 2.12.1 was current when this
  was authored), leptos `0.8` (0.8.21 current at authoring time), Trunk
  0.21.14 and tauri-cli 2.12.1 installed on CI with `cargo install --locked`
  (tauri-cli with its default features — stripping TLS from the CLI has broken
  the bundler's downloads before).

- **CI design**: one `ci.yml` with a `build` job on ubuntu-22.04 (Tauri 2
  Linux prerequisites from the official docs, fmt --check, clippy with warnings
  denied, cargo test, trunk build, `cargo tauri build --bundles deb`, artifact
  upload, job summary with failed steps, compiler errors and failed tests) plus
  release jobs that `needs: build`, so a release only happens after green CI.
  Trade-off: workflow_dispatch on CI is required for the Autofix flow because
  pushes made with the workflow token do not trigger workflows.

- **Nightly and tag releases live inside ci.yml** (jobs `nightly` and
  `tag-release`, both gated by `needs: build`) instead of a separate
  workflow_run-triggered workflow. Reason: `needs:` guarantees the "after green
  CI" ordering literally, and workflow_run does not fire for runs started with
  the workflow token (the exact case the Autofix flow creates). Only the two
  release jobs hold `permissions: contents: write`; the workflow level is
  `contents: read`. Nightly replaces all assets of the pre-release named
  `nightly` with the .deb and its SHA-256 checksums and an unsigned-development
  note; `v*` tags create pre-releases with the same assets.

- **Cargo.lock is committed** (generated locally with the pinned toolchain and
  committed with the scaffold; the Autofix workflow refreshes it on demand),
  so the template's `/Cargo.lock` ignore rule was removed from .gitignore and
  the CI cargo steps run with `--locked`. Reason: reproducible CI builds.
  Trade-off: dependency bumps need a deliberate lockfile refresh.

- **`[profile.release]` moved to the workspace root** (the UI crate's
  Cargo.toml). Reason: cargo ignores profiles declared in non-root workspace
  members, so the template's placement in src-tauri/Cargo.toml is dead
  configuration. Same settings as the template otherwise.

- **The one command**: `app_version` returns `env!("CARGO_PKG_VERSION")` of
  src-tauri through a one-line wrapper over `package_version()`, which has
  unit tests. tauri.conf.json's `version` must stay in sync manually; the
  version-consistency CI check is planned for Slice 3. Trade-off: two places
  hold the version until that check exists.

- **Deviations from the template** (all to honor the never-bend rules):
  `run()` returns `Result<(), tauri::Error>` and main propagates instead of
  `.expect(...)` (no expect in production paths); the frontend `invoke`
  binding is declared with `#[wasm_bindgen(..., catch)]` and a
  `Result<JsValue, JsValue>` return so a rejected invoke surfaces as an
  error instead of the template's panic-on-rejection, and the UI shows
  "unknown" on error; the greet example screen is replaced by the title plus
  the app version display (the mission's Slice 1 requirement). The window
  title is "SupportOS Oracle" instead of the package name.

- **Local pre-verification before the first CI run** (the sandbox has no
  system webkit/GTK libraries, so the Tauri crate cannot compile locally):
  `cargo fmt --all --check` green on the whole workspace,
  `cargo clippy -p supportos-oracle-ui --all-targets -- -D warnings` green,
  `cargo generate-lockfile` and a full `trunk build` green (wasm32 target,
  wasm-bindgen 0.2.129 downloaded and applied, dist/ produced with the
  frontendDist path resolving from src-tauri). Local green does not replace
  the CI run; the src-tauri clippy/test, the `cargo tauri build --bundles
  deb` step and the release jobs are still only verifiable on GitHub
  Actions.

- **DEB runtime dependencies declared** in tauri.conf.json
  (`libwebkit2gtk-4.1-0`, `libgtk-3-0`). Reason: the mission requires the
  package manager to install runtime dependencies; the tauri-bundler does not
  add them automatically. Trade-off: the list is hand-maintained and will be
  verified by the install smoke tests in Slice 3.

- **Runner image**: ubuntu-22.04 (mission requirement; still a hosted image at
  authoring time). Trade-off: older system libraries than 24.04; the DEB stays
  compatible with both because webkit2gtk 4.1 exists on 22.04 and 24.04.
