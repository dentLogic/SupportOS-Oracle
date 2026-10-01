# PROGRESS

IN PROGRESS: none (between tasks; the clean-tree gates landed and the next
concern is the cargo audit/deny gates plus the CI report issue comment).

Current slice: S2 (Foundation) — clean-tree gates step (A18) LANDED
Current task: next A18 gates (cargo audit 0.22.2, cargo deny 0.20.2) and the
A19 "CI report" issue comment
Last commit: e04e0ae on slice2/clean-tree-gates (merged to main after CI run
36880166840 went green)
Latest CI result: SUCCESS — run 36880166840 on e04e0ae: fmt, banned-comment
gate, cargo-machete, clippy -D warnings, test, trunk build, DEB packaging,
artifact upload all green with the pipefail enforcement active.

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
1. Land slice2/clean-tree-gates (grep gate, cargo-machete 0.9.2, unused
   template dependencies removed) through the Autofix flow.
2. cargo audit (0.22.2) and cargo deny (0.20.2) CI steps with a tuned
   deny.toml, then the "CI report" issue-comment job (A18/A19).
3. Foundation modules in crates/core (error type first, then logging facade,
   SQLite via rusqlite "bundled" with migrations/WAL/FTS5, settings, job
   queue, command-and-event pattern) and the crates/ui shell (leptos_router
   0.8, theming, navigation for all 20 reference pages as honest "Not built
   yet" screens, 404).

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
