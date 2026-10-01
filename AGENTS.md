# AGENTS.md — SupportOS Oracle

Goal: reimplement https://github.com/kimpearce888/supportos as SupportOS Oracle, a
Linux-only Tauri 2 desktop app: Rust backend, Rust/WASM (Leptos) frontend, no
hand-written JavaScript or TypeScript, English only. The requirements live in
docs/SPEC.md, overridden by docs/SPEC-AMENDMENTS.md where they conflict; if they
are missing, do only Slice 1 and wait. "Built" means: installed from the .deb
(later .rpm/AppImage) from the nightly release, launched from the application
menu, and every finished-slice feature works on screen.

Rules that never bend: advisory AI only (automatic customer-reply sending is
permanently off; "unknown" is a legitimate answer; secrets never reach the UI);
no telemetry, no cloud AI, no data egress beyond Help Scout, local AI providers
and user-configured connectors; Qdrant Edge stays behind the VectorStore
abstraction at its pinned version — if it fails on Linux x64, stop and report;
one localhost listener max (Help Scout webhook and OAuth callback); no fake
work (no dead controls, placeholder data or success messages without an
action); least code that works (no unwrap or expect in production paths, no
TODOs, no commented-out code); never claim complete/done/100% — deviations go
to docs/DEVIATIONS.md and stay pending approval; never delete tests or checks
to get green; never use continue-on-error on required steps.

Build loop (GitHub Actions is the compiler — no local cargo):
1. Make one small change (one concern per commit). Wait for the run to finish.
2. Read the failing step's log; fix the FIRST error only; commit; repeat.
3. If the same error survives 3 attempts, stop: report the exact error, what
   you tried, and your best guess. Do not thrash or rewrite unrelated files.
4. Never stack unverified changes; never say something works without a green
   run; look up versions and APIs in the official docs and pin them.

Session start: read AGENTS.md, then PROGRESS.md, then the latest Actions run
(fix red first), then only the spec parts the current slice needs. Announce:
"Resuming at <slice/task>. Last commit <hash>. CI <status>. Next <task>."
Session end: report what a green run proves, pages done and remaining, CI
status, packages built or failed (with error text), BLOCKED items, deviations
awaiting approval, and what could not be verified.
