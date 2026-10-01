# SupportOS (Oracle) — Product Specification

## 1. Purpose

SupportOS (Oracle) is a Linux-only desktop customer-support application that reimplements the existing SupportOS project as a production Tauri 2 application.

The product is intended to provide a local, durable working environment for support teams while integrating with Help Scout and user-configured local/external providers only where explicitly supported.

The reference implementation is:

- Repository: `https://github.com/kimpearce888/supportos`

The reference repository is the behavioral/product baseline. Exact inventories of routes, screens, data structures, settings, and terminology must be established during Slice 4 discovery and recorded in `docs/PARITY-MATRIX.md`.

This specification defines the product contract. It does not authorize adding unrelated features.

---

## 2. Product principles

1. **Production application, not a demo.**
   Every implemented feature must work through the installed desktop application.

2. **Existing-product parity first.**
   The goal is to reproduce the existing SupportOS feature set and behavior, then implement the planned SupportOS (Oracle) architecture around it.

3. **Local-first operation.**
   Local application data, indexing, analysis, and AI provider integrations must remain local unless a user-configured integration explicitly requires external communication.

4. **Help Scout is the external support system.**
   Help Scout remains the authoritative external source for Help Scout data and operations.

5. **AI is advisory.**
   AI may analyze, summarize, retrieve, classify, recommend, or draft. It must never autonomously send a customer reply.

6. **Unknown is valid.**
   The application must not invent answers when evidence is insufficient.

7. **No fake functionality.**
   A visible control must either perform its documented action or clearly state that the feature is not built yet.

8. **No silent data egress.**
   The application must not transmit support data, telemetry, analytics, prompts, or customer content to an undeclared service.

---

## 3. Platform and technology contract

### 3.1 Supported platform

- Linux desktop is the only supported platform for this product.
- The primary distributable is a `.deb`.
- RPM and AppImage packaging are added in Slice 3.
- macOS and Windows are out of scope.

### 3.2 Application stack

The application uses:

- Tauri 2
- Rust backend
- Leptos CSR frontend compiled to WebAssembly
- Trunk for the frontend build
- SQLite for application persistence
- Qdrant Edge behind a `VectorStore` abstraction
- Rust tests for backend/core behavior
- Rust-based UI end-to-end tests when the real screens exist

There must be no hand-written JavaScript or TypeScript application code.

### 3.3 Architecture

Starting in Slice 2, the application is structured as:

```text
crates/core
  Pure Rust domain, persistence, services, providers, jobs, search and business logic.
  No Tauri or GTK dependency.

crates/ui
  Leptos CSR frontend compiled to WASM.

src-tauri
  Thin Tauri shell.
  Commands are one-line adapters to core functions.
```

The core must be testable without Linux GUI/system libraries.

---

## 4. Data and persistence

### 4.1 SQLite

The local database must:

- use SQLite;
- use migrations;
- enable WAL mode;
- have deterministic schema migration behavior;
- keep persistence logic in `crates/core`;
- expose typed domain/service operations rather than spreading SQL throughout UI code.

### 4.2 Search

The application supports two complementary search layers:

1. **Lexical search**
   - SQLite FTS5.
   - Used for deterministic text search and filtering.

2. **Vector search**
   - Qdrant Edge.
   - Accessed only through the `VectorStore` abstraction.
   - The exact pinned version is documented in the architecture documentation.
   - Qdrant Edge must not be silently replaced by another vector engine.

### 4.3 Local application state

The application stores the minimum local state needed for:

- settings;
- Help Scout connection state;
- synchronization checkpoints;
- cached/mirrored Help Scout data;
- jobs;
- search indexes;
- AI/provider configuration;
- application preferences;
- local reports and derived intelligence where applicable.

Secrets must not be rendered into the UI and must not be written to logs.

---

## 5. Help Scout integration

SupportOS (Oracle) integrates with Help Scout through a provider abstraction.

### 5.1 Provider contract

The Help Scout integration must have:

- a `HelpScoutProvider` abstraction;
- a real Help Scout implementation;
- a fake/test implementation;
- deterministic error handling;
- rate limiting;
- retry behavior appropriate to transient failures;
- checkpointed synchronization;
- OAuth support;
- webhook support through the single permitted localhost listener;
- polling as the baseline synchronization mechanism.

### 5.2 Help Scout data areas

The product must support the Help Scout resources required by the existing SupportOS feature set, including as applicable:

- mailboxes/inboxes;
- conversations/tickets;
- conversation threads;
- customers;
- organizations;
- users/team members;
- tags;
- custom properties/fields where supported by the API;
- saved views or equivalent application-side views where supported;
- Help Scout Docs content where required by the reference product;
- ratings;
- webhooks.

Exact endpoint/resource coverage must be confirmed during Slice 4 against the reference implementation and Help Scout API capabilities.

### 5.3 Synchronization

The sync system must:

- have an explicit connection state;
- maintain checkpoints;
- handle pagination;
- respect Help Scout rate limits;
- retry transient failures;
- surface permanent failures clearly;
- avoid silently dropping records;
- make synchronization health visible to the user;
- support webhook push where configured;
- retain polling as the fallback/baseline.

### 5.4 Demo mode

The application includes a demo mode for the Help Scout integration.

Demo mode must use clearly identified local/fake data and must never pretend that fake data came from a real Help Scout account.

---

## 6. Inbox and ticket operations

The Inbox is the primary support workspace.

It must support the existing SupportOS ticket workflow, including:

- conversation/ticket listing;
- status filtering;
- mailbox filtering;
- assignee/team filtering;
- tag filtering;
- date-based filtering;
- search;
- saved views where present in the reference product;
- opening a conversation;
- reading the complete conversation context;
- replying;
- adding internal notes;
- changing supported ticket status;
- assigning/reassigning;
- applying/removing tags where supported;
- customer and organization context;
- attachments where supported;
- refresh/synchronization state;
- clear success and error states.

### 6.1 Write-protection pipeline

All write operations must pass through a write-protection pipeline.

The pipeline must prevent accidental autonomous customer communication.

The application must distinguish:

- reading data;
- preparing a draft;
- user-confirmed customer reply;
- internal note;
- other Help Scout write operations.

Automatic customer-reply sending is permanently disabled.

### 6.2 Response states

The application must represent the relevant response states required by the reference product, including at minimum the distinction between:

- unsent/draft content;
- user-confirmed outgoing reply;
- internal note.

No UI may imply that an action was sent when it was only drafted.

---

## 7. Customers and organizations

The application provides customer-focused views independent of merely displaying ticket lists.

Users must be able to:

- find customers;
- inspect customer information available through Help Scout;
- inspect relevant conversation history;
- identify related organizations where available;
- filter customer-focused data using supported properties;
- move from customer context to related conversations.

The exact fields and filters are established by the reference implementation and Help Scout API parity work.

---

## 8. Search

Search is a first-class application capability.

### 8.1 Lexical search

The application must provide deterministic local search over indexed support content using SQLite FTS5.

Search must support the filters and entities required by the existing SupportOS product.

### 8.2 Semantic/vector search

Where implemented by the relevant slice, semantic search uses the `VectorStore` abstraction backed by Qdrant Edge.

The vector layer must support:

- indexing;
- persistence;
- reopening after application restart;
- dense search;
- sparse search where supported by the selected Qdrant Edge implementation;
- metadata filtering;
- deterministic error reporting.

### 8.3 Similar historical questions

The product includes retrieval of previously handled/similar support questions where the existing SupportOS feature set provides that capability.

Results must show enough source context for the user to verify why a result was returned.

---

## 9. AI-assisted support

AI features are advisory tools for support work.

The application must support the provider architecture needed for the existing SupportOS AI functionality, including local providers such as:

- LM Studio;
- Ollama;
- generic/local-compatible providers where required by the existing product.

AI capabilities may include:

- ticket/conversation analysis;
- summaries;
- suggested internal notes;
- suggested customer replies;
- retrieval-assisted answers;
- similar-question lookup;
- classification;
- support intelligence.

### 9.1 AI safety and provenance

AI output must:

- be clearly identified as AI-generated/advisory;
- never be presented as guaranteed truth;
- allow the user to inspect supporting context where applicable;
- preserve an `unknown` result when evidence is insufficient;
- never automatically send a customer response.

### 9.2 Provider failures

If an AI provider is unavailable, misconfigured, or returns an error:

- the application must show a useful error;
- the UI must remain usable;
- no fake AI response may be displayed;
- credentials must not be exposed.

---

## 10. Activity, workflow and jobs

The application includes an activity/job engine required by the existing SupportOS workflows.

It must support:

- queued background jobs;
- explicit job states;
- retries for appropriate transient failures;
- failure visibility;
- cancellation where supported;
- synchronization work;
- indexing work;
- other long-running local operations.

The UI must never report a successful operation before the underlying operation has actually succeeded.

---

## 11. Command and event model

The application uses a command-and-event pattern for domain operations.

Commands represent requested changes/actions.

Events represent completed domain changes or meaningful state transitions.

This pattern must:

- keep business logic out of UI components;
- make important state changes observable;
- allow synchronization and UI refresh to react to state changes;
- remain testable in pure Rust.

---

## 12. Settings

Settings must provide the configuration required by the existing SupportOS feature set, including where applicable:

- Help Scout connection;
- synchronization behavior;
- AI providers;
- local AI endpoints;
- vector/search configuration;
- application preferences;
- theme;
- demo mode;
- other existing product configuration discovered during parity analysis.

Secrets must be stored and handled securely and must never be rendered into ordinary UI logs or diagnostic output.

---

## 13. Navigation and UI shell

The application has a persistent UI shell with navigation.

Every reference page identified by the current specification/reference inventory must have a navigation entry.

For a page not yet implemented:

- navigation must work;
- the destination must load;
- the page must explicitly say `Not built yet`;
- it must not contain fake controls or fake data.

The shell must also provide:

- a 404/not-found page;
- application-level error presentation;
- consistent loading/empty/error states;
- theme support;
- accessible controls and readable visible text.

---

## 14. Command palette

The application includes the command palette required by the existing SupportOS feature set.

Commands must correspond to real application actions.

A command that cannot currently execute must not be presented as functional.

---

## 15. Reports and quality

The product includes reporting and quality capabilities from the existing SupportOS feature set.

These may include:

- support activity reporting;
- response/handling metrics;
- quality-related analysis;
- Help Scout ratings;
- customer/support trends;
- AI-derived support intelligence.

Exact report definitions, fields, formulas, and pages must be taken from the reference implementation and parity inventory rather than invented during implementation.

---

## 16. Intelligence

The later product slices include the existing intelligence capabilities planned for SupportOS, including:

- Issue Radar;
- incidents;
- SLA-related intelligence;
- knowledge intelligence;
- graph relationships;
- customer/conversation timelines.

These are implemented only according to the existing product requirements and parity findings.

The system must distinguish measured facts from AI-derived interpretation.

---

## 17. Outreach and data tools

The later product scope includes the existing SupportOS capabilities for:

- outreach;
- bulk/customer-targeted communication workflows where supported;
- custom objects;
- connectors;
- backup and restore;
- encrypted synchronization;
- application settings/data administration.

Exact behavior is determined from the existing SupportOS implementation, API capabilities, and parity documentation.

No new unrelated workflow is authorized by this section.

---

## 18. Localhost listener

One small localhost HTTP listener is permitted only for:

1. Help Scout webhook delivery.
2. Help Scout OAuth callback.

It must not become a general-purpose network server.

All other synchronization uses the normal provider/polling mechanisms.

---

## 19. Security and privacy

The application must:

- keep secrets out of source code;
- keep secrets out of commits;
- keep secrets out of logs;
- avoid telemetry;
- avoid undisclosed analytics;
- avoid undisclosed cloud AI;
- avoid undisclosed data export;
- communicate only with Help Scout, configured local AI providers, and explicitly user-configured connectors;
- use least privilege for external credentials;
- clearly report connection failures;
- avoid exposing customer data in diagnostics.

---

## 20. Error handling

Errors are user-visible when they affect an operation.

Every important operation must have:

- loading state;
- success state where appropriate;
- failure state;
- useful human-readable error;
- recovery path where one exists.

Production paths must not use `unwrap` or `expect`.

There must be no silent failure.

---

## 21. Testing and verification

The product is verified at several levels.

### 21.1 Core tests

Business logic and services must have Rust tests.

### 21.2 Build checks

CI must verify:

- formatting;
- clippy with warnings denied;
- Rust tests;
- frontend compilation;
- Tauri application build.

### 21.3 UI verification

Once real pages exist, UI tests must:

- run under xvfb;
- use the Tauri/WebKit WebDriver path;
- be written in Rust;
- record visible text and controls;
- exercise every control on each covered page;
- produce a plain-text report;
- fail on dead controls, error banners, or placeholder data.

### 21.4 Human-level acceptance

A slice is accepted only when a person can install the produced package, launch it from the application menu, and use every feature of that slice on screen.

---

## 22. Release and packaging contract

The application must ultimately provide:

- `.deb`;
- `.rpm`;
- AppImage.

Packages must:

- contain the application correctly;
- declare required runtime dependencies;
- ship desktop integration;
- ship an icon;
- be installable and removable;
- pass package smoke tests.

Nightly builds are development builds and are clearly identified as unsigned.

Version tags create actual releases according to the release policy in the amendments.

---

## 23. Slice acceptance

A slice is complete only when:

1. its code is committed;
2. CI is green;
3. the required application package is produced;
4. the installed application launches;
5. every feature of the slice works visibly on screen;
6. `PROGRESS.md` is updated;
7. no unresolved blocker prevents the slice exit criteria;
8. no unapproved deviation is silently accepted.

A green compiler run alone does not prove that a screen works.

---

## 24. Reference and parity authority

When this specification does not define an exact field, route, screen, interaction, or calculation:

1. inspect the existing SupportOS reference implementation;
2. inspect the Help Scout API documentation when integration behavior is involved;
3. record the discovered behavior in the parity/audit documentation;
4. do not invent a replacement behavior merely because it is easier to implement.

Slice 4 is the formal discovery point for producing the complete parity inventory.

---

## 25. Out of scope

Unless explicitly introduced by an approved amendment, the following are not product requirements:

- Windows support;
- macOS support;
- a browser-only version;
- a cloud-hosted SupportOS backend;
- automatic customer-reply sending;
- hidden telemetry;
- undisclosed cloud AI;
- replacement of Qdrant Edge with another vector engine;
- unrelated new features not present in the existing SupportOS scope.

