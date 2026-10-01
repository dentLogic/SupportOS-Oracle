# PROGRESS

IN PROGRESS: Slice 2 workspace restructure on branch slice2/workspace-restructure.
Approach: move the UI crate into crates/ui (index.html, styles.css, public/ and
Trunk.toml move with it; CI's frontend step gains working-directory), add
crates/core with the version helper and its tests, make the src-tauri
`app_version` command a one-line wrapper over the core function, then push the
branch and dispatch the Autofix workflow on it to regenerate the now-stale
Cargo.lock. The push-triggered CI run on the restructure commit is expected to
be red at `clippy --locked` until Autofix commits the fresh lockfile — that is
the designed A7 flow, and main stays green meanwhile. Merge to main only after
the branch is green.

Current slice: S2 (Foundation) — workspace restructure step (A32/A5)
Current task: push the prepared branch, run Autofix on it, iterate CI to green
Last commit: 2ad04d3 on main (owner added docs/SPEC.md and docs/SPEC-AMENDMENTS
via upload); restructure + docs commits prepared on the branch above
Latest CI result: SUCCESS — Run 4 of CI on 2ad04d3 (fmt, clippy -D warnings,
test, trunk build, DEB packaging, artifact upload, nightly release all green)

What the green run proves (verified evidence, not claims):
- The S1 workspace compiles, is rustfmt-clean and clippy-clean with warnings
  denied on the pinned toolchain (Rust 1.98.1); `cargo test --all` passes.
- `trunk build` produces the WASM frontend and `cargo tauri build --bundles
  deb` produces the DEB, uploaded as a CI artifact and attached to the nightly
  pre-release with a SHA-256 checksum (re-verified after download in Slice 1).
- The restructure branch is PREPARED but NOT pushed: the preparing session had
  no GitHub token (redacted in the mission document), so nothing after 2ad04d3
  is verified by any CI run yet. Do not treat the branch as tested.

Known issues:
- docs/SPEC.md and docs/SPEC-AMENDMENTS.md are present since 2ad04d3: Slice 2
  is unblocked.
- Push credentials: the session that prepared the restructure could not push.
  The branch exists only in that sandbox; if it is lost, redo it from this
  file's description (git mv of the five frontend entries, virtual root
  manifest with resolver "2", crates/core with app::version, thin wrapper,
  tauri.conf.json build block, Trunk working directory, .gitignore/.taurignore
  path updates).
- Depends lists libwebkit2gtk-4.1-0 and libgtk-3-0 twice each in the DEB
  (explicit declaration plus bundler auto-detection). Harmless; cleanup stays
  scheduled for the packaging slice.

Next 3 tasks:
1. Push slice2/workspace-restructure, dispatch Autofix on the branch (lockfile
   regeneration), fix CI first-error-only until green, then merge to main.
2. Slice 2 CI gates: grep gate for TODO/history comments, unused-dependency
   check, cargo audit, cargo deny, and the "CI report" issue comment (A18/A19).
3. Slice 2 foundation: crates/core modules (SQLite via rusqlite "bundled" —
   FTS5 verified enabled by libsqlite3-sys build flags — with migrations and
   WAL, settings, job queue, error type, logging facade, command-and-event
   pattern) and the crates/ui shell (leptos_router 0.8, theming, navigation
   for all 20 reference pages as honest "Not built yet" screens, 404).

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
displayed through the working `app_version` command (launches via the desktop
entry the .deb installs).
Pages remaining: all reference pages (S2 creates the navigation shell with
honest "Not built yet" screens).

History of this slice:
- (none yet; the branch is the first Slice 2 work)

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
