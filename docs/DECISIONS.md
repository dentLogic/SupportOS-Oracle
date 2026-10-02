# DECISIONS

Key decisions for SupportOS Oracle, newest first. Each entry states the decision,
the reason, and the trade-off. Everything here is verified by CI runs or marked
otherwise.

## Slice 2

- **The S2 exit evidence includes an agent-side smoke launch of the nightly
  DEB, done rootless in the sandbox, and it is recorded as supporting
  evidence only — never as the "installs and launches" exit criterion**
  (slice2/s2-exit-evidence, 2026-10-02): the DEB is downloaded through the
  release API and its sha256 checked; the payload is inspected (binary,
  .desktop, icons); the WebKitGTK runtime closure is obtained with
  `apt-get download` + `dpkg-deb -x` (no root, no system mutation); a small
  LD_PRELOAD shim rewrites execve/posix_spawn/dlopen paths from
  /usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/ (compile-time fixed in
  WebKitGTK; no env override exists in 2.52) to the extracted tree, and the
  binary runs under Xvfb. That launch proved: the 800x600 window titled
  "SupportOS Oracle" with WM_CLASS supportos-oracle, the fail-loud setup
  hook completing (app data dir, DB with journal_mode wal, migrations v1
  and v2 applied, the startup record in the LogDir sink and stdout), and a
  stable 20+ second run. Reason: AGENTS.md says "GitHub Actions is the
  compiler", but a launch-path proof is stronger than a build proof, and
  the sandbox cannot run root installs. Trade-offs: the shim and the
  extracted webkit tree are sandbox artifacts, deliberately NOT committed
  to the repo; the GPU-less container aborts WebKitGPUProcess (EGL has no
  display), so no webview content pixels appear — on-screen rendering
  stays unproven and belongs to the owner's launch check, exactly as the
  exit criteria require. The owner's check remains the criterion that
  closes S2.

- **Theming follows the system color-scheme preference** (slice2/theming,
  branch tip 0a88f8c after CI#48 went green): styles.css defines every
  color as a custom property token; :root holds the light palette plus
  color-scheme: light, and a prefers-color-scheme: dark media query swaps
  the tokens for the dark palette (color-scheme: dark), so native widgets
  and scrollbars follow the theme too. index.html declares
  <meta name="color-scheme" content="light dark"> so the webview picks the
  right scheme before the CSS loads. Reason: SPEC 13 requires theme
  support; following the system preference is real, honest support without
  any control (A15 forbids dead or fake controls). Trade-off: no in-app
  preference yet — the persisted theme choice and its toggle arrive with
  the settings-wired commands, which is the deliberate next commit.

- **The navigation shell** (slice2/ui-shell, branch tip e3fbbf1 after CI#44
  and CI#45 went green): the UI crate gains leptos_router 0.8 (plain
  dependency — the crate needs no feature flags for client-side routing;
  its only features are ssr/nightly/tracing) and the shell structure:
  `<Router>` wraps the persistent sidebar (one `<A>` link per reference
  entry, driven by a const nav inventory module with tests: 20 unique
  paths/labels) and the routed content; the root and each reference route
  render the honest NotBuiltYet page (title plus "Not built yet", no fake
  controls or data), unknown routes render the 404 fallback, and the
  version footer keeps the app_version command demonstration (failure
  renders "unknown", never a guess). Reason: A23 requires navigation for
  every reference page with honest targets, and SPEC 13 requires the
  persistent shell, 404 and readable accessible text (semantic nav/list
  elements, aria-label). Trade-off: on-screen rendering is not CI-provable
  — the exit-criterion evidence is the owner launching the nightly DEB;
  theming and settings-wired commands land as their own commits.

- **The durable job queue** (slice2/jobs, branch tip 06e3435 after CI#41
  went green): migration v2 creates `jobs` with a CHECK constraint keeping
  the state set closed at the database level (queued, running, succeeded,
  failed, cancelled — tested by inserting an unknown state directly).
  `core::jobs` exposes enqueue, `claim_next` (a transaction: select the
  oldest job whose run-at time has passed, ordered by run_at then id, and
  mark it running), `complete` (requires running; finishing a non-running or
  unknown job is a job-queue error, never a silent no-op), `fail_attempt`
  (attempts counted, last error stored, requeue with a fixed backoff while
  the RetryPolicy budget lasts, then terminal failure), `cancel` (queued
  jobs only; running jobs answer false — the operation is in flight — and
  unknown ids are errors), and per-state listing. All time inputs are caller
  parameters: the queue never reads a clock, so tests and workers stay
  deterministic. Rows are never deleted, preserving failure visibility.
  Reason: SPEC 10 demands explicit states, retries, failure visibility and
  cancellation where supported; the caller-supplied time keeps the queue
  testable without a clock mock (A5). Trade-off: no worker loop or executor
  yet — the queue is the durable substrate, and the first real executors
  (sync, indexing) arrive with their slices. The build loop caught two
  real issues: clippy::needless_borrow across the tests (CI#39, fixed in
  517ed80) and one line that fit again after that fix (CI#40, collapsed in
  06e3435).

- **The settings store** (slice2/settings, branch tip a18db56 after CI#35
  went green): `core::settings` exposes generic `get`/`set` (SQLite upsert so
  a write replaces), `remove` (idempotent) and `list` (ordered by key) over
  the v1 `settings` table, plus the first typed boundary: `log.level` is
  validated against the logging facade's levels on write AND on read —
  storing an unknown name is invalid input, and a stored name that no longer
  maps (a hand-edited database) is a settings error rather than a silent
  fallback. Empty or whitespace-only keys are invalid input: keys are the
  store's API surface and blank keys are bugs, not data. Values stay raw
  strings; structured settings (Help Scout connection, providers, theme)
  arrive with their own subsystem commits and define their own typed
  boundaries on top of the generic layer. Reason: SPEC 12 wants settings
  local and persisted; validating at the typed boundary keeps invalid state
  out of the database (A16) while the generic layer stays trivially reusable.
  Trade-off: no JSON serialization of structured values yet — each subsystem
  commits to its own encoding discipline, which is a conscious deferral, not
  an omission. No new dependencies; the same lockfile ran green.

- **The SQLite foundation** (slice2/sqlite, branch tip aee6f9e after CI run
  36941866117 went green): core::db opens the database with rusqlite 0.40.2
  using the bundled libsqlite3-sys 0.38.2 — the vendored SQLite is compiled
  with -DSQLITE_ENABLE_FTS5 (verified in the upstream build.rs), so FTS5
  needs no system library and is proven at runtime by a test that creates an
  fts5 virtual table and matches. `open` sets `PRAGMA journal_mode=WAL` and
  asserts the pragma's own answer is "wal" (a filesystem that silently
  rejects WAL becomes a database error instead of a silent mode change),
  turns foreign keys on, and applies the migration list. The migration
  machinery records each migration's exact SQL in schema_migrations and is
  deliberately strict: a modified applied migration, a database holding
  versions the build cannot provide, a non-ascending list, or a failing
  migration (whole transaction rolled back, no record) all fail with
  database errors — deterministic migration behavior per SPEC 4.1. Migration
  v1 creates the settings key-value table, which the next commit's settings
  store builds on. Reason: SPEC 4.1 mandates SQLite + migrations + WAL + FTS5
  with persistence in core; exact-SQL bookkeeping makes divergence loud
  rather than silent (A16). Trade-off: the machinery rejects
  source-modified migrations instead of rewriting history, so any schema
  change after release must be a new version, never an edit — the intended
  discipline. The first push (78f57da) failed only rustfmt (import ordering
  and closure shapes; the Autofix pass corrected it), which is the designed
  healing path for hand-formatted commits.

- **Logging facade and the log sink** (slice2/logging, branch tip 6392268
  after CI runs 36938079798 and 36938147864 went green): core emits through
  the plain `log` 0.4 facade and never installs a sink — `core::logging`
  carries the level plumbing (a strict `level_filter_from_name` that returns
  None for unknown names so a future settings integration must fall back
  explicitly, plus the `Info` default filter) and `log_app_start`, the
  version-bearing startup record. The shell installs tauri-plugin-log 2.10.0
  (3.0.0-alpha.2 exists and is deliberately avoided) with Stdout + LogDir
  targets and the level taken from core's default — the plugin's own default
  is Trace, which would flood the log file, so the override is functional,
  not cosmetic. The startup record is emitted from the Tauri setup hook;
  that ordering was verified in the tauri 2.12.1 source (app.rs:
  `initialize_plugins` inside `build()` runs before the setup callback) and
  the plugin API shapes (Builder::new/level/targets, TargetKind::LogDir's
  file_name field) against the published 2.10.0 source before the push.
  LogDir was chosen over TargetKind::Folder because Folder takes a required
  `path: PathBuf` that only exists after an app handle does; LogDir resolves
  the platform directory (`$XDG_DATA_HOME/{bundleIdentifier}/logs` or
  `$HOME/.local/share/...` on Linux) itself. Reason: A5 keeps core sink-free
  and testable (the capture-logger test proves records reach an installed
  logger), SPEC 19 forbids secrets in logs, so the facade carries only level
  plumbing and version-bearing records. Trade-off: nothing user-visible is
  logged from the UI yet — records start flowing when real subsystems land.

- **A stale lockfile no longer fails `--locked` in the current CI step
  order** (observed in run 36938079798 on aa39692): cargo-deny runs
  `cargo metadata` internally, which re-resolves a manifest that gained
  registry dependencies and rewrites the runner's Cargo.lock before the
  clippy/test steps execute — deny's own output on that run already listed
  tauri-plugin-log v2.10.0, and the subsequent `--locked` builds passed with
  the regenerated runner lock. The scaffold-era failure ("error: cannot
  update the lock file ... --locked", run 36878040457) happened when clippy
  was the first resolving step and the workspace itself had changed. Net
  effect: the committed-lock discipline is enforced by the Autofix dispatch
  (which commits the regenerated lock) rather than by the `--locked` flag,
  and until that commit lands, cargo audit is the one gate still reading the
  old package set. Recorded so future red/green triage reads the right
  symptom.

- **Application error type** (slice2/error-type, merged to main as 937f740
  after CI run 36890453528 went green): `core::error::Error` carries one
  variant per Slice 2 subsystem (Database, Settings, JobQueue) plus
  InvalidInput and Io, with thiserror 2 providing Display and
  std::error::Error, and a `Result<T>` alias used across core operations.
  The message format is "subsystem: detail" so the UI can render a
  human-readable explanation while internals stay behind the type (SPEC 20,
  A16). Tests assert every variant renders a non-empty subsystem-prefixed
  message and the std trait implementation. Reason: the S2 module list names
  the error type explicitly, and every later module (SQLite, settings, jobs)
  returns it. Trade-off: variants exist before their subsystems land — they
  map exactly to the S2 mandate, nothing beyond it.

- **Supply-chain gates and the CI report comment** (slice2/supply-chain-gates,
  merged to main as eefe78a after CI run 36889519391 went green): CI adds
  cargo-audit 0.22.2 (fails on RustSec vulnerabilities; maintenance-level
  advisories stay warnings) and cargo-deny 0.20.2 running
  `cargo deny check bans licenses sources` — advisories are deliberately not
  repeated by deny, so the RustSec database is fetched once. deny.toml pins
  the policy: duplicate versions warn (the Tauri tree carries unavoidable
  duplicates), the license allow list holds exactly the licenses the current
  tree uses plus BSL-1.0 (xxhash-rust, reached through leptos's server_fn —
  the first run rejected it, and the never-encountered entries BSD-2-Clause,
  CDLA-Permissive-1.0, ISC and Unicode-DFS-2016 were trimmed so a new license
  entering the tree fails for explicit review), and only crates.io is an
  allowed source. The workspace crates declare license = MIT to match the
  repository LICENSE. A new ci-report job (needs: build, nightly,
  tag-release; always()) posts the step-outcome table plus collected
  compiler errors and failed test names as a comment on the issue titled
  "CI report", creating it on first use; it is the only job with
  issues: write, and it passes -R ${GITHUB_REPOSITORY} to every gh call
  because it has no checkout step (the first attempt failed on
  "not a git repository"). Trade-off: one comment per CI run, including the
  transiently red runs during the Autofix flow — that is deliberate
  visibility, not spam.

- **Clean-tree gates** (slice2/clean-tree-gates, merged to main as e04e0ae
  after CI run 36880166840 went green): CI gains two steps
  after the format check — a grep gate that fails on TODO/FIXME/XXX/HACK
  markers in Rust sources (crates/ and src-tauri/), and cargo-machete 0.9.2
  (prebuilt via taiki-e/install-action, same pinned step as Trunk and the
  Tauri CLI) failing on unused dependencies. To make the tree pass the new
  checks, the unused template dependencies are removed: the UI crate keeps
  only leptos, wasm-bindgen and console_error_panic_hook (the sources
  reference exactly those), and src-tauri keeps tauri, tauri-plugin-opener
  and the core path dependency. serde/serde-wasm-bindgen and friends return
  with the commits whose code actually uses them (the UI shell and the
  command payloads). The first run (36879605492) caught a false negative in
  that plan: wasm-bindgen-futures is used through the #[wasm_bindgen]
  extern-block expansion of `async fn` (E0433 once removed), a usage textual
  scanners cannot see, so it is restored with a package.metadata.cargo-machete
  `ignored` entry and a manifest comment stating exactly why. Reason: A17/A18
  demand the gates, and shipping them
  together with the fix keeps one concern per commit: a clean tree. History
  comments ("why X changed") have no deterministic grep signature, so they
  stay a review responsibility rather than a gate. Trade-off: the marker list
  is fixed (TODO/FIXME/XXX/HACK) and .rs-only for now; the grep can be
  widened when a real need appears.

- **Pipefail for piped CI steps** (d0eac7c): the clippy and test steps run
  with `shell: bash`, which GitHub Actions executes with `-eo pipefail`, so
  `cargo ... | tee log` can now actually fail the step. Before the fix, the
  default shell (`bash -e`, no pipefail) took the pipeline status from tee:
  run 36878040457 printed `error: cannot update the lock file ... --locked`
  in both steps while they stayed green — clippy and test were never
  exit-code-enforced in any earlier run. The restructure run's logs (36878092716)
  show clippy finished clean and the core test passed, so no hidden breakage
  was being masked, and every later green run enforces the real verdicts.

- **Workspace restructure** (slice2/workspace-restructure, merged to main as
  449c1ec after CI run 36878092716 went green): the repo root becomes a virtual workspace
  manifest with `members = ["crates/core", "crates/ui", "src-tauri"]` and
  `resolver = "2"` (the value the leptos workspace itself uses; edition-2024
  members do not require v3 and v2 is the most widely exercised). The release
  profile stays in the root manifest — cargo ignores profiles in non-root
  members. The UI crate moves to crates/ui together with index.html,
  styles.css, public/ and Trunk.toml, so Trunk always runs with its working
  directory at crates/ui: CI's frontend step uses `working-directory:
  crates/ui`, and tauri.conf.json's beforeDevCommand/beforeBuildCommand are
  prefixed with `cd crates/ui &&`. `frontendDist` becomes
  `../crates/ui/dist`, still relative to src-tauri. Reason: the mission's
  Slice 2 structure (A5) while keeping every path relative and explicit.
  Trade-off: one directory level more in frontend paths; a second Trunk build
  still happens inside `cargo tauri build` (its beforeBuildCommand), costing
  CI a little time in exchange for a bundler that cannot see a stale dist.

- **crates/core is the version source of truth**: `core::app::version()`
  returns the core crate's `CARGO_PKG_VERSION`, and the src-tauri
  `app_version` command is a one-line wrapper over it — the A5 thin-shell
  pattern (each command wraps a core function that has a Rust test; the
  semver-shape test moved into core). Reason: `env!` cannot read another
  crate's version, and the shell must stay thin. Trade-off: the version now
  lives in lockstep in four places (core, ui, src-tauri, tauri.conf.json)
  until the Slice 3 version-consistency check lands.

- **Lockfile regeneration after the restructure goes through Autofix on the
  branch**: the restructure leaves the committed Cargo.lock stale (package
  set changed), so the push-triggered CI run is expected to fail at
  `clippy --locked`; the Autofix workflow is dispatched on the branch, where
  it regenerates the lockfile, formats, commits and dispatches CI. Merge to
  main happens only after the branch is green. Reason: A7 forbids local cargo
  and A27 requires risky work on a named branch with main never red.

- **The ui crate manifest keeps the exact S1 dependency list** through the
  restructure (leptos csr 0.8, wasm-bindgen stack, serde, console panic hook);
  leptos_router and the foundation dependencies (rusqlite with the `bundled`
  feature — its libsqlite3-sys build defines `SQLITE_ENABLE_FTS5`, verified in
  the upstream build.rs — plus thiserror, the log facade and tauri-plugin-log
  in the shell) arrive with their own feature commits. Reason: one concern
  per commit (A8). Trade-off: the restructure commit is larger than a pure
  file move because crates/core carries the moved version helper, but it is
  one coherent concern: the new shape itself.

- **The database state is core's `SharedDb`, not a shell struct**:
  `crates/core/src/db.rs` exposes a `Mutex<Connection>` handle whose `with()`
  runs one operation at a time, and the Tauri shell only manages it as
  state. Reason: the lock discipline and poison handling are logic, and A5
  keeps every piece of logic in core where it has a test (the poison path is
  exercised with catch_unwind). Trade-off: core carries a
  concurrency-aware type, though it is persistence infrastructure by nature;
  commands stay one-line wrappers with no unwraps.

- **A launch without the database fails the launch**: the shell's setup hook
  resolves `app.path().app_data_dir()` (verified in tauri 2.12.1's
  `path/desktop.rs`: `dirs::data_dir()/{identifier}`, so
  `~/.local/share/com.dentlogic.supportos-oracle/supportos-oracle.db` on
  Linux), creates the directory, opens the DB with WAL and migrations, and
  any failure aborts startup with the error. Reason: A16 (no silent
  failure) — an app that lost its local state has nothing honest to show.
  Trade-off: a broken data directory takes the whole window down; the error
  message names the failing step.

- **The theme preference applies through a `data-theme` attribute on the
  document root** rather than a JS-injected class or a stylesheet swap:
  `html[data-theme="light"|"dark"]` re-declares the token sets next to the
  existing `:root` and media-query blocks, and attribute specificity beats
  `:root` in both, so a pinned palette wins over the system preference and
  "system" (attribute removed) falls back to the media query. Reason: no
  duplicated stylesheets, no hand-written JS (Rust sets the attribute
  through web-sys), and the CSS stays declarative. Trade-off: the token
  lists appear twice each (default light, media dark, plus the two pinned
  selectors) — plain CSS custom properties without `light-dark()`, which
  WebKitGTK cannot be assumed to support.

- **The persisted log.level is stored and validated but not yet applied at
  startup**: tauri-plugin-log's builder runs before the setup hook, so
  reading the level would require opening the database before the app
  exists and duplicating Tauri's path resolution. Reason: least code that
  works; the level has a validated boundary and an honest default (Info).
  Trade-off: editing log.level in a future Settings page takes effect on
  the next launch; the application lands with the Settings screen slice.

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

- **Versions pinned**: Rust 1.98.1 via rust-toolchain.toml (profile minimal,
  wasm32-unknown-unknown target, rustfmt and clippy components — CI runs
  `rustup toolchain install`, which installs exactly that), Tauri 2 (crate
  requirement `2`, resolved exactly by the committed Cargo.lock; tauri 2.12.1
  was current when this was authored), leptos `0.8` (0.8.21 current at
  authoring time). Trunk 0.21.14 and tauri-cli 2.12.1 are installed on CI via
  taiki-e/install-action@v2.87.22, which fetches the tools' official prebuilt
  release binaries (tauri-cli with its default features — stripping TLS from
  the CLI has broken the bundler's downloads before; the prebuilt binaries
  keep them). Trade-off: prebuilt binaries are trusted instead of compiling
  from source with `cargo install --locked` (~10 minutes saved per run).
  Trunk resolves the matching wasm-bindgen-cli version from Cargo.lock and
  downloads it itself.

- **CI design**: one `ci.yml` with a `build` job on ubuntu-22.04 (Tauri 2
  Linux prerequisites from the official docs plus libgtk-3-dev, fmt --check,
  clippy with warnings denied over the whole workspace, cargo test, trunk
  build, `cargo tauri build --bundles deb`, artifact upload (both
  target/release/bundle/deb/ and the legacy src-tauri/target/... path, since
  cargo places workspace output at the workspace root), job summary with
  failed steps, compiler errors and failed tests) plus
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
