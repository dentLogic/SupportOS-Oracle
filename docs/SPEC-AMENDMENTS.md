# SupportOS (Oracle) — Specification Amendments

This document overrides `docs/SPEC.md` wherever the two documents conflict.

These amendments are part of the mission contract and must be followed by every implementation session.

## A1. Product identity and repository

The product name is:

**SupportOS (Oracle)**

The target public repository is:

`dentLogic/SupportOS-Oracle`

The reference product is:

`https://github.com/kimpearce888/supportos`

The target is a reimplementation of the reference product, not an opportunity to redesign the product around unrelated ideas.

---

## A2. Operating system scope

SupportOS (Oracle) is **Linux-only**.

Do not add:

- Apple/macOS packaging;
- Windows packaging;
- Apple-specific code;
- Windows-specific code;
- cross-platform UI work whose only purpose is supporting those systems.

The first production package is a Linux `.deb`.

RPM and AppImage are added later according to the slice plan.

---

## A3. Frontend/backend implementation restriction

The application must use:

- Tauri 2;
- Rust backend;
- Leptos CSR;
- Rust/WASM;
- Trunk.

There must be **no hand-written JavaScript or TypeScript**.

Do not introduce a JavaScript/TypeScript frontend merely because a library or example is easier that way.

---

## A4. Slice 1 template fidelity

Slice 1 must mirror the official `tauri-apps/create-tauri-app` Leptos template.

Do not invent a custom Tauri/Leptos project layout in Slice 1.

Preserve the template's relevant Tauri build settings, including the template's `withGlobalTauri` setting.

`frontendDist` must point to the actual Trunk output directory relative to `src-tauri`.

A wrong frontend distribution path is considered a build-blocking defect.

---

## A5. Workspace timing

Do **not** restructure the application into a multi-crate workspace during Slice 1.

The workspace restructure happens in Slice 2.

The intended Slice 2 structure is:

```text
crates/core
crates/ui
src-tauri
```

`crates/core` must not depend on Tauri or GTK.

This allows core Rust tests to run without Linux GUI system libraries.

---

## A6. CI environment

CI uses:

- Ubuntu 22.04;
- Rust cache;
- `wasm32-unknown-unknown`;
- Trunk;
- Tauri CLI with its default features.

Install the Linux prerequisites required by Tauri 2, including the WebKitGTK/GTK development libraries and the other packages specified by the mission.

CI must run:

- `cargo fmt --check`;
- clippy with warnings denied;
- `cargo test`;
- `trunk build`;
- `tauri build` producing the DEB.

The CI workflow must support `workflow_dispatch`.

---

## A7. Lockfile and formatting constraint

The implementation environment cannot run Cargo commands locally.

Therefore:

- do not claim to have generated `Cargo.lock` locally;
- do not depend on an unverified lockfile;
- provide a manually triggered `autofix` workflow;
- that workflow may run `cargo generate-lockfile` and `cargo fmt`;
- it commits the generated changes;
- it then explicitly dispatches the CI workflow.

Do not use `--locked` until a lockfile has actually been committed.

---

## A8. One-change build loop

Implementation must use this loop:

1. Make one small change addressing one concern.
2. Commit it.
3. Wait for GitHub Actions.
4. Inspect the failing step.
5. Fix the first error only.
6. Commit again.
7. Repeat.

Do not stack unverified changes.

If the same error survives three attempts:

- stop that item;
- report the exact error;
- report what was tried;
- give the best current diagnosis.

Do not thrash.

Do not delete tests/checks to obtain green CI.

Do not use `continue-on-error` for required checks.

---

## A9. Verification language

Never state that something works merely because the code looks correct.

Use evidence-based language.

A green CI run proves only what that run actually tested.

A feature is not considered working until its required screen/action has been exercised in the installed application.

Never use the words:

- complete;
- done;
- perfect;
- 100%;

as unsupported claims of project completion.

---

## A10. Dependencies and API versions

Look up Tauri 2, Leptos, Trunk, and other external APIs in their official documentation before relying on an API.

Pin the versions used by the project.

At minimum, maintain:

- `rust-toolchain.toml`;
- pinned GitHub Action versions;
- pinned tool versions where practical.

Do not guess an API from memory when official documentation is available.

---

## A11. Qdrant Edge is mandatory

The vector database layer is:

**Qdrant Edge**

It must be hidden behind:

```text
VectorStore
```

Do not replace Qdrant Edge with:

- SQLite vectors;
- another embedded vector database;
- a hosted Qdrant service;
- another vector engine.

If the pinned Qdrant Edge version cannot build or operate correctly on Linux x64:

1. stop;
2. preserve the evidence;
3. report the exact failure;
4. do not silently substitute another engine.

The pinned Qdrant Edge version must be recorded in the appropriate architecture documentation.

---

## A12. AI policy

AI is advisory only.

The application may:

- retrieve;
- summarize;
- classify;
- analyze;
- suggest;
- draft.

The application may **not** automatically send a customer reply.

There must always be a user-controlled boundary before a customer-facing reply is sent.

`unknown` is a valid AI result and must not be converted into a confident answer without evidence.

---

## A13. Data egress and telemetry

There is:

- no telemetry;
- no hidden analytics;
- no cloud AI;
- no undisclosed data export.

Network communication is limited to:

- Help Scout;
- local AI providers;
- user-configured connectors;
- the explicitly permitted Help Scout OAuth/webhook localhost listener.

Secrets and customer content must not leak into logs, UI diagnostics, commits, workflow output, or release notes.

---

## A14. Localhost listener restriction

Only one small localhost listener is permitted.

Its uses are limited to:

1. Help Scout webhook reception.
2. Help Scout OAuth callback.

Polling remains the baseline.

Do not turn the application into a general-purpose local HTTP server.

---

## A15. No fake functionality

The UI must never contain:

- dead buttons;
- fake success messages;
- fake records;
- placeholder records presented as real data;
- controls that appear functional but do nothing;
- stubs hidden behind production-looking screens.

When a planned page has not yet been built, the page must say clearly:

**Not built yet**

and contain no misleading controls.

---

## A16. Error-handling rules

Production paths must not use:

- `unwrap`;
- `expect`;

for recoverable/runtime operations.

Errors must be handled explicitly.

The application must never report success before the underlying operation has succeeded.

---

## A17. Code cleanliness

Do not leave:

- TODO comments;
- commented-out implementation;
- history comments;
- duplicated business logic;
- unused dependencies;
- unnecessary wrappers.

Use the clean-tree gates defined by the mission.

These rules apply to implementation code and project configuration, not merely to application UI code.

---

## A18. CI quality gates

From Slice 2 onward, CI progressively adds:

- clippy with warnings denied;
- grep gate for TODO/history comments;
- unused-dependency checking;
- `cargo audit`;
- `cargo deny`;
- CI summaries;
- `CI report` issue comments.

Required checks must fail the workflow when they fail.

Do not hide failures.

---

## A19. CI summary

Every workflow must write a concise job summary containing useful information about:

- failed steps;
- failed tests;
- compiler errors;
- relevant package/build failures.

From Slice 2 onward, the CI report is also posted as a comment on the GitHub issue titled:

**CI report**

---

## A20. Packaging

Slice 1 produces a `.deb`.

Slice 3 adds:

- RPM;
- AppImage;
- install/uninstall smoke tests.

Package smoke tests must verify the actual package rather than merely inspecting build output.

Where applicable, tests must:

1. install the package;
2. launch the application under xvfb;
3. verify the process;
4. verify the application data directory;
5. uninstall the package.

RPM testing uses a Fedora container with `dnf`.

AppImage testing must account for the Linux FUSE/runtime requirements documented by Tauri.

---

## A21. Nightly release

On every push to `main`:

1. CI must pass.
2. The nightly release is updated.
3. The release contains the generated package assets.
4. SHA-256 checksums are included.
5. The release note states plainly that it is an unsigned development build.

Only the release job receives `contents: write`.

Do not grant broad write permissions to ordinary CI jobs.

---

## A22. Versioned releases

Tags matching:

```text
v*
```

create real releases.

They remain pre-releases until explicitly changed.

Milestone markers are not release tags.

Do not create a `v1.0.0` release merely because the code compiles.

The final release level must be supported by the audit and verification evidence.

---

## A23. Slice 2 navigation rule

Slice 2 creates the UI shell and navigation for every reference page identified by the current inventory.

If the page's real implementation belongs to a later slice, the navigation target must still open and show:

**Not built yet**

This is preferable to:

- hiding the route;
- creating a dead link;
- inventing placeholder functionality.

---

## A24. Reference-product authority

Where the specification is silent about exact existing SupportOS behavior, do not invent a new product decision.

Use this order:

1. existing SupportOS implementation;
2. existing project documentation/configuration;
3. Help Scout API capabilities and official documentation;
4. explicit approved amendments.

Record important discoveries in the parity/audit documents.

---

## A25. Feature-scope protection

The mission is to make the existing intended product correct and production-ready.

Do not:

- add unrelated features;
- redesign existing workflows for preference;
- add speculative upgrades;
- replace existing functionality merely because a different implementation is fashionable;
- expand scope to create work that is not required by the reference product.

Technical implementation improvements are allowed when they preserve the specified product behavior.

---

## A26. Deviations

Any intentional deviation from this specification or its amendments must be recorded in:

```text
docs/DEVIATIONS.md
```

A deviation is pending until explicitly approved by the project owner.

The implementation agent must never approve its own deviation.

---

## A27. Progress tracking

After every task commit, update `PROGRESS.md` with:

- current slice;
- current task;
- last commit;
- latest CI result;
- next three tasks;
- known issues;
- pages completed;
- pages remaining.

Before a long/risky task, write:

```text
IN PROGRESS: <task>, <approach>
```

Do not leave `main` red.

Risky work must happen on a named branch and be recorded in progress notes.

---

## A28. Session start

Every session starts by reading:

1. `AGENTS.md`;
2. `PROGRESS.md`;
3. latest GitHub Actions run;
4. only the specification sections required for the current slice.

The session opening status must be:

```text
Resuming at <slice/task>. Last commit <hash>. CI <status>. Next <task>.
```

If CI is red, fix it before starting unrelated work.

---

## A29. Session end

Every session report must state:

- what the green run proves;
- pages completed;
- pages remaining;
- CI status;
- each package built or failed;
- exact error text for failures;
- blocked items;
- deviations awaiting approval;
- what could not be verified;
- anything the owner must do personally.

Examples of owner-only work include:

- granting permissions;
- providing a real Help Scout account;
- performing clean-machine verification.

---

## A30. Slice progression

Slices are sequential.

A slice is not complete until its exit criteria are met.

After a slice becomes green and `PROGRESS.md` is updated, the agent may continue to the next slice without waiting for another instruction.

The agent must stop when:

- the three-attempt stuck rule triggers;
- an item is blocked by credentials/permissions/owner decision;
- session context becomes unsafe to continue;
- the slice exit criteria cannot be met.

---

## A31. Slice 1 exact deliverables

Slice 1 consists of:

- `README.md` — three lines, Linux-only, install the `.deb`;
- MIT license;
- `.gitignore`;
- `rust-toolchain.toml`;
- template-based empty Tauri/Leptos application;
- application title;
- one working command that displays the application version;
- CI workflow;
- autofix workflow;
- nightly `.deb` release;
- `AGENTS.md` under 40 lines;
- `PROGRESS.md`;
- `docs/DECISIONS.md`.

Slice 1 exit criteria:

- CI green;
- `.deb` attached to nightly release.

---

## A32. Slice 2 exact foundation

Slice 2 adds:

- workspace restructure;
- `crates/core`;
- `crates/ui`;
- thin Tauri shell;
- SQLite;
- migrations;
- WAL;
- FTS5;
- settings;
- job queue;
- application error type;
- logging;
- command-and-event pattern;
- theming;
- UI shell;
- navigation for every known reference page;
- honest `Not built yet` pages;
- 404;
- CI summary;
- `CI report` issue comments;
- clean-tree gates;
- unused dependency check;
- cargo audit;
- cargo deny.

Slice 2 exit criteria:

- CI green;
- installed nightly application launches;
- every navigation item opens.

---

## A33. Slice 3 exact packaging

Slice 3 adds:

- RPM;
- AppImage;
- install/uninstall smoke tests;
- Ubuntu apt verification;
- Fedora dnf container verification;
- AppImage xvfb verification;
- `v*` release workflow;
- version consistency check.

Exit criteria:

- DEB, RPM, and AppImage attached to nightly;
- smoke tests green.

---

## A34. Slice 4 discovery

Slice 4 creates a repeatable discovery job that clones the reference repository and extracts inventories for:

- routes;
- tables;
- migrations;
- settings;
- vocabularies;
- UI pages.

It must produce:

- `docs/PARITY-MATRIX.md`;
- `docs/audit/UI-GAP.md`;
- `docs/original-notes/`;
- `docs/REFERENCE-VERSION.md`;
- `docs/MANUAL-VERIFICATION.md`.

This discovery is the authoritative detailed inventory for later parity implementation.

---

## A35. Slice 5 end-to-end harness

Slice 5 adds the real UI verification harness.

For every existing implemented page in scope, it must:

- launch under xvfb;
- use Tauri/WebKit WebDriver;
- be written in Rust;
- record visible text;
- record controls;
- click every control;
- produce a plain-text report;
- fail on dead controls;
- fail on error banners;
- fail on placeholder data where real data is required.

---

## A36. Slice 6 Qdrant Edge proof

Slice 6 is performed first on a separate branch.

It must prove, inside the real application:

- build;
- persistence;
- reopen;
- dense search;
- sparse search where supported;
- metadata filters.

Merge only after green evidence.

Update:

```text
docs/architecture/VECTORSTORE.md
```

with the exact pinned version and implementation details.

If Qdrant Edge fails on Linux x64, stop and report instead of substituting another engine.

---

## A37. Slice 7 Help Scout mirror

Slice 7 implements the Help Scout mirror foundation:

- `HelpScoutProvider` trait;
- real provider;
- fake provider;
- stand-in HTTP server tests;
- OAuth;
- rate-limited queue;
- checkpointed sync;
- Beacon chat integration where part of the reference scope;
- Docs mirror;
- ratings;
- webhook listener.

Required screens include:

- onboarding wizard;
- Help Scout connection;
- Sync Health;
- webhook push state;
- demo mode;
- demo tools.

---

## A38. Slice 8 Inbox

Slice 8 implements:

- activity engine;
- response states;
- filters;
- saved views;
- ticket operations through the write-protection pipeline;
- lexical search;
- command palette.

Required screens include:

- Inbox;
- conversation detail;
- Search;
- Customers;
- Organizations.

---

## A39. Later slices

Later slices implement the existing planned SupportOS families in order, including:

- team operations;
- VectorStore and AI providers;
- hybrid search;
- AI features;
- intelligence;
- Issue Radar;
- incidents;
- SLA;
- knowledge;
- graph;
- timeline;
- reports;
- quality;
- outreach;
- data tools;
- custom objects;
- connectors;
- backup/restore;
- encrypted synchronization;
- settings.

Each family must have its screens, tests, and CI evidence before its slice is considered complete.

---

## A40. Final audit and release

Before a final production release:

1. perform a fresh independent audit;
2. perform a clean-build pass;
3. verify the installed application;
4. verify release packages;
5. verify parity evidence;
6. verify unresolved deviations;
7. publish `v1.0.0` only if the evidence supports it.

If evidence does not support `v1.0.0`, use an appropriate release-candidate version instead.

The final README must explain in plain English:

- what SupportOS (Oracle) is;
- how to install it;
- why it was built;
- important architectural decisions;
- each key decision as:
  **we chose X because Y, and the trade-off is Z**.

---

## A41. Credentials

GitHub credentials supplied for implementation are operational secrets.

They must never be written to:

- source files;
- configuration files;
- repository files;
- commits;
- workflows;
- logs;
- issues;
- comments;
- release notes.

If credentials are accidentally exposed, stop using the exposed credential and report the exposure rather than copying it elsewhere.

---

## A42. Final interpretation rule

If there is a conflict:

1. these amendments win over `SPEC.md`;
2. an explicitly approved owner decision wins over both;
3. an undocumented preference does not override the written specification;
4. the safest interpretation is the one that preserves existing product behavior, prevents data loss/unsafe communication, and avoids unapproved scope expansion.
