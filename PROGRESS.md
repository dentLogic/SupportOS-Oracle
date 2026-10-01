# PROGRESS

IN PROGRESS: none (between tasks; the supply-chain gates and the CI report
comment landed; the next concern is the foundation modules in crates/core
and the UI shell).

Current slice: S2 (Foundation) — A18/A19 CI gates LANDED
Current task: foundation modules (error type, logging, SQLite, settings,
job queue, command-and-event) and the UI shell with navigation
Last commit: eefe78a on slice2/supply-chain-gates (merged to main after CI
run 36889519391 went green)
Latest CI result: SUCCESS — run 36889519391 on eefe78a: fmt, banned-comment
gate, cargo-machete, cargo audit, cargo deny, clippy -D warnings, test,
trunk build, DEB packaging, artifact upload green; the ci-report job posted
the run's report as issue comment #1 (issue "CI report").

What the green run proves (verified evidence, not claims):
- The restructured workspace (crates/core + crates/ui + thin src-tauri)
  compiles, is rustfmt-clean and clippy-clean with warnings denied on the
  pinned toolchain (Rust 1.98.1); `cargo test --all` passes (core's semver
  test: 1 passed; the clippy/test logs contain no errors).
- `trunk build` produces the WASM frontend from crates/ui and
  `cargo tauri build --bundles deb` produces the DEB through the new
  `cd crates/ui && trunk build` beforeBuildCommand and the
  ../crates/ui/dist frontendDist; the artifact uploads green.
- Runs 36878040457/36878092716 both passed BEFORE the pipefail fix, so their
  clippy/test step outcomes were masked; the defect is documented and fixed
  in d0eac7c, and every later green run enforces the real exit codes.

Known issues:
- The DEB's Depends still lists libwebkit2gtk-4.1-0 and libgtk-3-0 twice each
  (explicit declaration plus bundler auto-detection). Harmless; cleanup stays
  scheduled for the packaging slice.
- proc-macro-error2 2.0.1 (transitive) prints a future-incompat note on
  clippy; informational only, tracked for the cargo-audit/deny step.

Next 3 tasks:
1. crates/core foundation: application error type (thiserror) with tests,
   then the logging facade (log) wired to tauri-plugin-log in the shell.
2. crates/core SQLite layer (rusqlite bundled, migrations, WAL, FTS5) with
   tests, then settings and the job queue behind it.
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
