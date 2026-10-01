# PROGRESS

IN PROGRESS: none (between tasks; the SQLite foundation landed; the next
concern is the settings store, then the job queue, then the UI shell).

Current slice: S2 (Foundation)
Current task: settings store next (typed get/set over the settings table on
top of core::db, with tests and one-line shell commands), then the job queue,
then theming and the UI shell
Last commit: aee6f9e on slice2/sqlite (the SQLite foundation plus the
regenerated lockfile and rustfmt pass; code commit 78f57da)
Latest CI result: SUCCESS — run 36941866117 (aee6f9e) fully green; core
tests now 15 (version, three error-type, four logging, seven db).

What the green run proves (verified evidence, not claims):
- The SQLite foundation works on the pinned toolchain (Rust 1.98.1) with
  rusqlite 0.40.2 and the bundled libsqlite3-sys 0.38.2: the runner compiled
  the vendored SQLite, clippy denied warnings over the workspace, and
  cargo machete/audit/deny accepted the new subtree (bans ok, licenses ok,
  sources ok).
- `core::db::open` produces a database whose journal mode is WAL (asserted
  from the pragma's own answer, not assumed), foreign keys enforced, and all
  migrations applied; reopening the same file applies nothing new and keeps
  data. Migration determinism is tested: a modified applied migration, a
  database newer than the code, a non-ascending list and a failing migration
  (rolled back completely, no record left) are all rejected with database
  errors.
- FTS5 is compiled into the bundled SQLite and usable: the test creates an
  fts5 virtual table, inserts and matches (the lexical-search layer of SPEC
  4.2 has its engine available).
- The push-triggered run on the code commit (36941836510, CI#30) failed at
  the format check only — hand-formatted code drifted from rustfmt 1.98.1
  (import ordering, closure collapsing); the Autofix pass corrected it and
  the tip run is green. Known rustfmt shapes recorded for future commits.

What the green run proves for the logging facade (run 36938147864 on 6392268):
- The logging facade compiles and passes its tests on the pinned toolchain
  (Rust 1.98.1): level-name mapping (known names, unknown rejection, Info
  default) and the capture-logger test proving the startup record reaches
  whatever logger the host installs. `cargo fmt --check` passed on the first
  push without autofix reformatting.
- The shell builds with tauri-plugin-log 2.10.0 (stdout + LogDir targets,
  level Info from core) — clippy denied warnings over the whole workspace,
  the DEB packaged green, and cargo machete/audit/deny accepted the new
  dependency subtree (fern 0.7.1, num_threads, android_log-sys) on the
  committed lockfile (run 36938147864).
- The tauri-plugin-log API usage was verified against the official 2.10.0
  source before the push, and the setup-hook ordering claim against
  tauri 2.12.1's app.rs: initialize_plugins (build) runs before the setup
  callback, so the startup record reaches the installed sinks.
- New toolchain finding recorded in DECISIONS: cargo-deny's internal cargo
  metadata re-resolves and rewrites a stale lock on the runner before the
  clippy/test steps, so `--locked` no longer fails for additive registry
  dependency changes (it did in the scaffold era, when clippy was the first
  resolving step). The committed lock is still regenerated through Autofix,
  and until it lands only cargo audit scans the old package set.

Known issues:
- The DEB's Depends still lists libwebkit2gtk-4.1-0 and libgtk-3-0 twice each
  (explicit declaration plus bundler auto-detection). Harmless; cleanup stays
  scheduled for the packaging slice.
- proc-macro-error2 2.0.1 (transitive) prints a future-incompat note on
  clippy; informational only, tracked for the cargo-audit/deny step.

Next 3 tasks:
1. Settings store on the SQLite foundation: typed get/set/keys operations
   over the settings table in crates/core with tests (including the logging
   level name round-trip), wired to the shell as one-line commands.
2. The job queue on top of the database with tests, then the command-and-
   event pattern through the thin shell.
3. crates/ui shell: leptos_router 0.8, theming, navigation for all 20
   reference pages as honest "Not built yet" screens, 404, and the
   command-and-event pattern through the thin shell.

Reference page inventory (extracted from kimpearce888/supportos App.tsx, S4
will formalize): Dashboard /, Inbox /inbox, Notifications /notifications,
Search /search, Customers /customers, Organizations /organizations, AI Center
/ai, Issues /issues, Incidents /incidents, Knowledge /knowledge, Docs /docs,
Objects /custom-objects, Connectors /connectors, Graph /graph, Operations
/operations, Reports /reports, Outreach /outreach, Automation /automation,
Sync Health /sync-health, Settings /settings; plus /onboarding and detail
routes (/inbox/conversation/:id, /customers/:id, /organizations/:id,
/incidents/:id) and the 404 fallback.

Pages done: the main window — title "SupportOS Oracle" and the app version
displayed through the working `app_version` command (now a one-line wrapper
over core::app::version()).
Pages remaining: all reference pages (S2 creates the navigation shell with
honest "Not built yet" screens).

History of this slice (S2):
- 2e34dc4 + dade1e6 + 449c1ec (autofix lockfile): workspace restructure on
  slice2/workspace-restructure, CI run 36878092716 green, fast-forward merged
  to main (449c1ec). The push-triggered run on dade1e6 revealed the pipe
  masking defect (its clippy/test steps printed lockfile errors yet stayed
  green).
- d0eac7c: pipefail fix (shell: bash on the piped clippy/test steps) on main;
  CI run 36879294137 green with real exit-code enforcement.
- 763e624 + a9ff6ec (autofix lockfile) + 8e656c9 + e04e0ae (autofix lockfile):
  clean-tree gates on slice2/clean-tree-gates. First run (36879605492) failed
  at clippy: the removed wasm-bindgen-futures is used by the #[wasm_bindgen]
  async-fn expansion (E0433) — restored with a cargo-machete ignored entry
  and a manifest comment. CI run 36880166840 green on the branch tip, merged
  to main.
- 9da5388 (cargo audit 0.22.2), ce3b16e + 1a5d579 (cargo deny 0.20.2 with
  deny.toml; first run rejected BSL-1.0 from xxhash-rust and warned about
  unused allow entries — BSL-1.0 allowed, list trimmed to the tree), 01de1cf
  + eefe78a (ci-report job; first attempt failed because gh could not
  resolve the repository without a checkout — now passes -R) on
  slice2/supply-chain-gates. CI run 36889519391 green; issue #1 "CI report"
  created with the first report comment.
- 65c656f + 937f740 (autofix lockfile): application error type in
  crates/core (thiserror 2, one variant per S2 subsystem plus invalid input
  and I/O, Result alias, three tests). CI run 36890453528 green, merged to
  main.
- aa39692 + 6392268 (autofix lockfile): logging facade on slice2/logging —
  core gains `logging` (log 0.4 facade: level-name mapping with the Info
  default, log_app_start) with four tests including a capture-logger test;
  the shell installs tauri-plugin-log 2.10.0 (Stdout + LogDir targets, level
  from core's default filter) and emits the startup record from the setup
  hook. Push-triggered run 36938079798 passed all gates even before the
  lockfile commit (cargo-deny's metadata step had re-resolved the runner's
  lock; see DECISIONS), autofix landed the regenerated lock as 6392268, and
  run 36938147864 on the branch tip went fully green.
- 78f57da + aee6f9e (autofix lockfile + fmt): SQLite foundation on
  slice2/sqlite — core gains `db` (open with WAL verified from the pragma's
  answer, foreign keys on, deterministic migration machinery with exact-SQL
  bookkeeping, migration v1 creating the settings table, seven tests incl.
  FTS5 capability). Push-triggered run 36941836510 failed only at the format
  check (hand-formatting drift; no code error), autofix corrected it and
  landed the lockfile, and run 36941866117 on the branch tip went fully
  green with 15 core tests. rusqlite 0.40.2 / libsqlite3-sys 0.38.2
  (bundled, FTS5 flag verified in the upstream build.rs) entered the tree.

History of the previous slice (S1, for the record):
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
- 044e184: record Slice 1 completion in PROGRESS.md (Run 3 green).
- 2ad04d3: owner added docs/SPEC.md + docs/SPEC-AMENDMENTS.md (Run 4 green).
