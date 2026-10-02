# PROGRESS

IN PROGRESS: Slice 3 (packaging). S2 is CLOSED — the slice review
PASSED on 2026-10-02 (owner, reported directly to the agent) after the
launch check passed and every A32 exit criterion held with evidence.
Slice 3 now adds the A33 packaging work: RPM, AppImage, smoke tests,
the verifications, the v* release workflow and the version consistency
check, plus the scheduled Depends deduplication.

Current slice: S3 (Packaging)
Current task: AppImage packaging (add the appimage bundle target, attach
it to the nightly); after that the install/uninstall smoke tests and the
remaining A33 items
Last commit: 9037830 on main (slice3/rpm fast-forward merged: the RPM is
built in CI and attached to the nightly with its sha256; the RPM carries
explicit Fedora package Requires plus the rpm crate's soname Requires)
Latest CI result: SUCCESS — 36982949308 on main (9037830) with the Nightly
job publishing four assets at 2026-10-02T08:18:25Z: DEB 3,968,672 bytes
(sha256 39776bad...22f9, re-downloaded and verified against the published
.sha256 asset) and RPM 3,969,769 bytes (sha256 9929db08...15e3, likewise
re-downloaded and verified). The RPM header was inspected off-line:
name support-os-oracle, version 0.1.0, release 1, arch x86_64, Requires
webkit2gtk4.1 + gtk3 (explicit Fedora names) and libwebkit2gtk-4.1.so.0()(64bit)
+ libgtk-3.so.0()(64bit) (auto-generated sonames), with the same
file layout as the DEB (/usr/bin/supportos-oracle, .desktop, hicolor
icons); 31 core tests (db, settings, jobs, logging, error) plus 2 nav
tests. NOTE: every merge to main replaces the nightly assets, so a digest
recorded here ages immediately — when installing, verify the download
against the .sha256 assets published next to the packages in the Nightly
release, which always match the current bytes.

What the green run proves (verified evidence, not claims):
- The Leptos router shell builds for wasm32 through trunk: leptos_router
  0.8.16 compiled into the UI crate, and the DEB packaged green with the
  new frontend. The nav inventory test holds: 20 unique paths and labels.
- The shell is structurally complete: a Router wraps the persistent sidebar
  (all 20 reference links via the A component) and the routed content; the
  root and every reference route render the honest Not-built-yet page; any
  unknown route renders the 404 fallback. What CI cannot prove: on-screen
  rendering — the owner's launch check of the nightly DEB is the exit
  criterion evidence.
- The command-and-event pattern stays demonstrated by app_version: the UI
  invokes the one-line shell command over the tested core function and
  renders "unknown" on failure rather than a guess.
- Process note: the first push accidentally committed to local main (branch
  creation failed in the same command); nothing was pushed and the commit
  was moved to the branch with local main reset to origin/main — remote
  main never moved.

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

What the green run proves for the settings wiring (runs 36951341830 on
03e9d9e and 36951821709 on ad017ec):
- core::db::SharedDb compiles and its tests pass on the pinned toolchain:
  one caller at a time behind the mutex (a write is visible to the next
  read), and a poisoned lock surfaces as a Database error instead of a
  panic (verified with catch_unwind, not assumed).
- The theme boundary tests pass: set_theme rejects unknown names with
  nothing stored, theme rejects a tampered stored name, and valid names
  round-trip and replace. 31 core tests total.
- The shell compiles with the four settings commands as one-line wrappers
  over tested core functions; tauri's State/Manager/manage path-resolver
  usage was verified against the tauri 2.12.1 source before the push (the
  same for the js-sys/web-sys APIs the theme toggle uses).
- The UI crate compiles for wasm32 with the new web-sys/js-sys edges (same
  versions the Leptos graph already resolved, lockfile regenerated by
  Autofix as ad017ec), and the DEB packaged green with the theme toggle in
  the frontend. cargo machete/audit/deny accepted the manifest additions.
- What CI cannot prove: the on-screen behavior — that the toggle saves and
  reloads the preference and the palette follows it. That evidence is the
  owner's launch check of the nightly DEB (the slice exit criterion).

End-of-slice verification — agent-side evidence (2026-10-02, sandbox
without root; method recorded in DECISIONS.md):
- Asset integrity: the nightly DEB downloaded through the release API and
  its sha256 matches the published digest (the full hex is in the header
  above). dpkg-deb -I shows the expected control fields; the payload carries
  usr/bin/supportos-oracle, the .desktop entry (Name/Exec/Icon,
  StartupWMClass supportos-oracle) and hicolor icons — the structure an
  application-menu launch needs.
- Smoke launch: with the WebKitGTK runtime obtained through `apt-get
  download` (no root) and an LD_PRELOAD path shim, the extracted binary
  launches under Xvfb and stays alive 20+ seconds. The window tree (xwininfo)
  shows "SupportOS Oracle" at exactly 800x600+0+0 with the matching
  WM_CLASS — the configured window, not a crash dialog.
- The fail-loud setup hook completed in that launch: the app data directory
  was created, supportos-oracle.db appeared with journal_mode wal (live -wal
  and -shm files), BOTH migrations applied on a fresh profile (the
  schema_migrations rows show v1 settings_key_value and v2 jobs_queue; the
  settings and jobs tables exist), and the tauri-plugin-log LogDir sink
  wrote the startup record — "SupportOS Oracle starting (version 0.1.0)" —
  to both stdout and logs/SupportOS-Oracle.log. The WebKit webview
  initialized (WebKitCache, storage and hsts files created).
- What the sandbox cannot prove: on-screen rendering of the Leptos UI. The
  webview painted no content pixels in this container, and a negative
  control proved this is environmental, not an app defect: WebKitGTK's own
  MiniBrowser loading a trivial local page (plain HTML + a script that
  fills the page and instantiates a minimal wasm module) ALSO paints
  nothing and reports "WebProcess CRASHED" under the same Xvfb — the
  GPU-less container aborts the WebKit EGL/GPU path for every WebKit
  process (software-rendering env vars, a software mesa EGL vendor, and
  DMABUF/compositing-mode disable all tried; no EGL display is creatable
  without a DRI2 X server or a /dev/dri device). During the app smoke run
  the WebKitWebProcess and WebKitNetworkProcess do spawn (sampled
  continuously for 15s), so the webview pipeline itself starts; only
  painting is impossible here. CI proves the frontend builds into the DEB,
  and the launch above proves the app side of the stack runs. The
  on-screen criterion stays with the owner's launch check.
- The owner's launch check (2026-10-02): PASSED, reported directly to the
  agent. The owner installed the nightly DEB on real hardware, launched
  SupportOS Oracle from the application menu, and confirmed: the
  application launches, every navigation item opens, and the theme
  preference persists across a restart. This is the on-screen evidence
  the sandbox could not produce, and it closes the last open A32 exit
  criterion (CI green; installed nightly launches; every navigation
  item opens). The S2 slice review is requested next — Slice 3 starts
  only after it passes.

Known issues:
- The DEB Depends duplication (libwebkit2gtk-4.1-0 and libgtk-3-0 listed
twice each) is RESOLVED by 70b069b: the rebuilt nightly DEB's control
field now lists each package exactly once, provided by the bundler's
own dependency detection.
- proc-macro-error2 2.0.1 (transitive) prints a future-incompat note on
  clippy; informational only, tracked for the cargo-audit/deny step.

Next 3 tasks:
1. AppImage packaging: add the appimage bundle target, upload and attach
   the AppImage (plus its sha256) to the nightly release.
2. Install/uninstall smoke tests: Ubuntu apt install/remove of the nightly
   DEB and Fedora dnf install/remove of the nightly RPM in containers.
3. The remaining A33 verifications: Ubuntu apt verification, Fedora dnf
   container verification, AppImage xvfb verification, the v* release
   workflow and the version consistency check.

Reference page inventory (extracted from kimpearce888/supportos App.tsx, S4
will formalize): Dashboard /, Inbox /inbox, Notifications /notifications,
Search /search, Customers /customers, Organizations /organizations, AI Center
/ai, Issues /issues, Incidents /incidents, Knowledge /knowledge, Docs /docs,
Objects /custom-objects, Connectors /connectors, Graph /graph, Operations
/operations, Reports /reports, Outreach /outreach, Automation /automation,
Sync Health /sync-health, Settings /settings; plus /onboarding and detail
routes (/inbox/conversation/:id, /customers/:id, /organizations/:id,
/incidents/:id) and the 404 fallback.

Pages done: the main window — the persistent shell with sidebar navigation
for all 20 reference pages, honest "Not built yet" pages on every route,
the 404 fallback, and the app version displayed through the working
`app_version` command (a one-line wrapper over core::app::version()).
Pages remaining: all reference pages' real screens (later slices); the
shell itself is built, themed and wired to the settings store (light/dark
follows the system color-scheme preference until a persisted choice
takes over, saved through the theme toggle).

History of the previous slice (S2, closed 2026-10-02):
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
- 77c2174 + a18db56 (autofix fmt; no lockfile change): settings store on
  slice2/settings — core gains `settings` (generic get/set/remove/list over
  the v1 table with empty-key rejection, the typed log.level boundary
  validating on write and read) with eight tests. Push-triggered CI#34
  failed at the format check only; run CI#35 on the branch tip went fully
  green with 23 core tests.
- 5ac5d85 + b578b8c (autofix fmt) + 517ed80 + 06e3435: durable job queue on
  slice2/jobs — migration v2 creates the jobs table with a state CHECK
  constraint; core gains `jobs` (enqueue, claim_next in a transaction,
  complete with state validation, fail_attempt with a RetryPolicy,
  queued-only cancel, per-state lists) with eight tests. CI#39 caught
  clippy::needless_borrow (26 test call sites), fixed in 517ed80; CI#40
  caught one refit line, collapsed in 06e3435; CI#41 fully green with 31
  core tests.
- d15d534 + e3fbbf1 (autofix lockfile): navigation shell on slice2/ui-shell
  — the UI crate gains leptos_router 0.8 (no extra features needed; the
  crate has no csr feature flag), a nav inventory module (20 reference
  entries, tested), the NotBuiltYet/NotFound pages, and the App shell
  (Router, sidebar, routed content, version footer). CI#44 green on the
  first push (no fmt drift), CI#45 green on the branch tip with the
  regenerated lockfile.
- 0a88f8c: theming on slice2/theming — styles.css switches to theme tokens
  (custom properties) with light and dark palettes, the color-scheme
  property on :root makes native widgets follow the theme, and index.html
  declares the supported schemes to the webview before the CSS loads. No
  Rust change; CI#48 green on the first push.
- 07ad235: SharedDb on slice2/shell-settings — core::db gains the
  mutex-backed connection handle (with() runs one operation at a time; a
  poisoned lock is a Database error, proven by a catch_unwind test). Run
  36949903823 green on the first push.
- 4cc98bf + 796b751: the theme setting boundary on the same branch — the
  app.theme key with THEME_NAMES (system/light/dark), set_theme validating
  on write and theme validating on read, three tests. Run 36950278307
  failed at the format check only (a short format! line rustfmt collapses);
  run 36950456462 on the fix went green.
- cdef512 + c6044a0 + 03e9d9e: the shell opens the database — setup
  resolves the app data directory, creates it, opens the DB (WAL plus
  migrations) and manages SharedDb as state; the invoke handler gains
  settings_get/settings_set and the typed settings_theme/settings_set_theme
  as one-line wrappers. Run 36950856749 failed the format check (rustfmt
  splits closure chains), run 36951093697 caught clippy::redundant_closure
  on the theme wrapper; run 36951341830 on the fixes went green.
- 83a8c19 + ad017ec (autofix lockfile): the persisted theme toggle — a ui
  theme module (cycle order, invoke arguments via js-sys, apply/remove the
  data-theme attribute on the document root, readable command errors), the
  sidebar button that saves first and applies after, and the CSS pinning
  the light/dark token sets on html[data-theme]. Both the push-triggered
  run 36951795383 and the tip run 36951821709 went green (31 core tests,
  2 nav tests, web-sys/js-sys edges recorded in the regenerated lockfile).
- 7f02c28: the settings-wiring docs (PROGRESS/DECISIONS updates),
  fast-forward merged slice2/shell-settings to main; main run 36952644692
  green with the Nightly job rebuilding the DEB (3,968,672 bytes) at
  01:52:10Z.
- The S2 exit-evidence pass (13efb16): nightly assets verified by
  sha256, package structure inspected, and an agent-side smoke launch under
  Xvfb proved the launch path end-to-end (window 800x600 with the right
  WM_CLASS, DB with WAL and both migrations on a fresh profile, the startup
  log record in place, the WebKitWebProcess and WebKitNetworkProcess
  alive). The webview painted no content pixels in the GPU-less sandbox —
  and a negative control (WebKit's own MiniBrowser with a trivial JS page
  failing identically) proved that is a container limitation, not an app
  defect; the on-screen criterion remains the owner's launch check.
- The current-artifact re-verification pass (this commit): the nightly DEB
  the owner will actually download — the 04:18:59Z rebuild at e327d70,
  3,968,664 bytes, sha256 d9dda2f9...9d38 — was downloaded, digest-verified
  against the release, and smoke-launched on a clean profile: window
  title/WM_CLASS/geometry, WAL database with both migrations applied, and
  the startup log record all reconfirmed on those exact bytes. This pass
  also fixed a handoff defect found in it: the header still pointed the
  owner at the sha256 of the superseded 01:52:10Z build (two rebuilds
  earlier), so a check against the header would have shown a false
  mismatch; the owner instruction now points at the .sha256 asset in the
  release, which always matches the current DEB (decision recorded in
  DECISIONS).
- The owner's launch-check result (09a8b47): PASSED, reported directly
  to the agent on 2026-10-02 — installed from the nightly DEB, launched
  from the application menu, every navigation item opens, and the theme
  preference persists across a restart. All three A32 exit criteria were
  then met with recorded evidence (CI green on main; the installed nightly
  launches; every navigation item opens). The S2 slice review PASSED on
  2026-10-02 (owner, reported directly to the agent); no deviations were
  pending approval, the nightly at 09a8b47 was verified
  (5f5c847e...fe5), and the slice closed.

History of this slice (S3):
- Slice 3 opened (492bf37): PROGRESS records the S2 closure (launch
  check passed, review passed) and the A33 scope: RPM, AppImage,
  install/uninstall smoke tests, Ubuntu apt verification, Fedora dnf
  container verification, AppImage xvfb verification, the v* release
  workflow, the version consistency check, and the scheduled DEB Depends
  deduplication. First concern: the Depends dedup, then the RPM target.
- 70b069b (slice3/depends-dedup): dropped the explicit
  tauri.conf.json linux.deb.depends entries; the bundler's own detection
  keeps providing both runtime packages. Verified on the rebuilt nightly
  (07:35:50Z, sha256 10c8e938...d1fff, digest-checked against the
  release's .sha256 asset): dpkg-deb -f now reports
  "Depends: libwebkit2gtk-4.1-0, libgtk-3-0" — each package exactly
  once (the pre-dedup 05:56:27Z build listed both twice). CI
  36979031768 green on main. The docs commit b843486 recorded that result.
- 9037830 (slice3/rpm): the CI package step now runs
  `cargo tauri build --bundles deb,rpm`; the packages artifact keeps
  deb/ and rpm/ subdirectories (multi-path upload roots at the least
  common ancestor), and both the nightly and the v* tag-release jobs
  attach the RPM beside the DEB with per-package .sha256 assets, failing
  if either format is missing. tauri.conf.json declares the Fedora names
  webkit2gtk4.1 and gtk3 in bundle.linux.rpm.depends (the RPM bundler
  has no package-level auto-detection, unlike the DEB bundler — see
  DECISIONS.md). Verified on the nightly (08:18:25Z): four assets,
  both digests re-checked, RPM header inspected (name support-os-oracle,
  0.1.0-1 x86_64, package + soname Requires, DEB-equivalent file layout).
  Branch CI 36982470703 green; main CI 36982949308 green. This commit
  records that result.

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
