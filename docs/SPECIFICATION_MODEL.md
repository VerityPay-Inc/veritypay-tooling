# Specification Model

**Architecture for a reusable typed specification model inside `veritypay-tooling`.**

This document defines the **Specification Model**—a shared typed layer between raw files in `veritypay-spec` and consumers such as validators, Edition tooling, future documentation generation, and the reference interpreter. It does not specify parsing libraries, crate layout, or migration timelines for existing validators.

For file access primitives, see `SpecRepository` in `vp-core`. For validator lifecycle and independence, see [VALIDATION_ENGINE.md](VALIDATION_ENGINE.md) and [ADR-0003](adrs/0003-validator-execution-model.md). For Edition Manifest checks, see [EDITION_VALIDATION.md](EDITION_VALIDATION.md).

**Working crate name:** `vp-spec-model`

**Boundary:** The Specification Model **builds and represents** the specification corpus. It does **not** execute protocol behavior, define protocol semantics, publish Editions, or replace validators.

---

## Purpose

`veritypay-tooling` validates and supports publication of the VerityPay specification. Today, each validator discovers files and parses YAML or Markdown **independently**. That approach is acceptable for early milestones, but it will create duplication as Edition validation, documentation generation, reference interpretation, and conformance tooling grow.

The Specification Model should become the **shared typed layer** between raw files and validators:

- One place to parse VP-TERM and VP-RFC registries into stable structures
- One place to represent Edition Manifests, document front matter, and cross-reference graphs
- One vocabulary for future docs generation and the reference interpreter

Validators **remain responsible for rules and diagnostics**. The model **remains responsible for representation**. Separation keeps [ADR-0003](adrs/0003-validator-execution-model.md) intact: validators are independent, emit diagnostics only, and do not depend on each other's execution.

---

## Model identity

A **Specification Model** represents one **immutable view** of a specification checkout.

It is tied to:

| Anchor | Role |
|--------|------|
| **One spec root** | The `veritypay-spec` checkout being represented |
| **One repository state** | The file tree and contents at build time |
| **One configuration** | Session inputs such as Edition Manifest path (from `ValidationConfig`) |
| **One loading session** | Built once per validation or tooling run |

Every validator participating in a validation run **observes the same Specification Model** when the platform provides one. The model is not runtime state that validators mutate—it is a frozen snapshot, analogous to how validators observe immutable `ValidationContext` without changing it.

Do not conflate the model with:

| Concept | Role |
|---------|------|
| **Specification Model** | Typed representation of what was built |
| **Validation configuration** | Inputs that select *how* validation runs |
| **Diagnostics** | Findings emitted by validators |
| **Reports** | Aggregated outcomes from the engine |

---

## Why this exists

| Problem today | Model response |
|---------------|----------------|
| Registry YAML parsed separately in registry, cross-reference, and Edition validators | Load once into `RegistrySet`; consumers read typed data |
| Front matter parsed ad hoc for Edition pin checks | `DocumentFrontMatter` with source location preserved |
| Cross-reference discovery rebuilt per validator | `ReferenceGraph` built from corpus scan, reused by cross-ref and future tools |
| Edition Manifest shape interpreted inline | `EditionManifest` as a typed view of manifest YAML |
| Future docs generation and reference interpreter would re-parse the same files | Same `Specification` snapshot for all downstream consumers |

The model does **not** decide what is valid. It answers: **what did we build, from where, in what shape?**

---

## Position in the platform

```
SpecRepository (vp-core)
       ↓
SpecificationBuilder (vp-spec-model)
       ↓
Specification
       ↓
Validators · Edition tooling · Future docs generation · Future reference interpreter
```

The middle layer **discovers, parses, normalizes, connects, and assembles** typed structures—it is closer to an AST builder than a file reader. Conceptually:

```
Parser  →  AST Builder  →  AST
```

| Layer | Responsibility |
|-------|----------------|
| **`SpecRepository`** | Read-only file access under a spec root; no business semantics |
| **`SpecificationBuilder`** | Discovers artifacts, parses into typed nodes, connects relationships, produces an immutable `Specification` |
| **`Specification`** | Immutable snapshot of loaded artifacts and relationships |
| **Validators** | Apply documented rules; emit diagnostics; may consume the model as input |
| **CLI / engine** | Unchanged contract: aggregate diagnostics, present output, exit codes |

Configuration ([CONFIGURATION_ARCHITECTURE.md](CONFIGURATION_ARCHITECTURE.md)) still resolves **how** a session runs (`spec_root`, `edition`, output). The model consumes **what** is on disk at the configured spec root—it does not parse `.vp.toml` or CLI flags.

---

## Model objects

Each object below is a **conceptual type**. Exact field names and nesting are frozen when the first implementation ADR is accepted, aligned with `veritypay-spec` governance—not invented silently at code time.

### `Specification`

| | |
|--|--|
| **Purpose** | Root aggregate: one loaded view of a spec checkout for a validation or tooling session. |
| **Responsibilities** | Hold spec root identity; expose child collections (`RegistrySet`, optional `EditionManifest`, document corpus, `ReferenceGraph`); provide lookup helpers (e.g. resolve path → `SpecificationDocument`). |
| **Non-responsibilities** | Validate structure; mutate files; authorize publication; cache across runs unless explicitly documented. |
| **Likely source files** | Entire tree under spec root; driven by builder scope and `ValidationConfig` (e.g. whether Edition Manifest path is set). |

### `RegistrySet`

| | |
|--|--|
| **Purpose** | Container for machine-readable registries loaded from `spec/`. |
| **Responsibilities** | Expose `TerminologyRegistry` and `RfcRegistry`; record load provenance (paths, parse success/failure at load time). |
| **Non-responsibilities** | Enforce VP-TERM or VP-RFC rules; deduplicate diagnostics; write registry YAML. |
| **Likely source files** | `spec/terminology/registry.yaml`, `spec/rfcs/registry.yaml` |

### `TerminologyRegistry`

| | |
|--|--|
| **Purpose** | Typed representation of the VP-TERM registry. |
| **Responsibilities** | Expose term entries (id, title, stability, dependencies, normative definition pointers, etc. per spec schema); preserve YAML source locations where practical. |
| **Non-responsibilities** | Validate ID format, duplicate detection, or glossary prose alignment; define term meanings. |
| **Likely source files** | `spec/terminology/registry.yaml` |

### `RfcRegistry`

| | |
|--|--|
| **Purpose** | Typed representation of the VP-RFC registry. |
| **Responsibilities** | Expose RFC entries (id, status, path, dependencies, version, etc. per VP-RFC-0000); support id → entry lookup for other model layers. |
| **Non-responsibilities** | Validate registry rules; accept or reject RFCs; read RFC prose bodies beyond metadata needs. |
| **Likely source files** | `spec/rfcs/registry.yaml`; optionally RFC front matter from `rfcs/*.md` when builder scope includes it. |

### `EditionManifest`

| | |
|--|--|
| **Purpose** | Typed representation of an Edition Manifest YAML file. |
| **Responsibilities** | Expose manifest fields (`edition`, `edition_id`, `protocol_version`, `status`, `specification_documents`, `accepted_rfcs`, `registry_snapshots`, `conformance_baseline`, optional `supersedes`); preserve YAML paths for diagnostics consumers. |
| **Non-responsibilities** | Verify pins, RFC membership, or publication authorization; generate or publish manifests. |
| **Likely source files** | Path from `ValidationConfig.edition` (relative to spec root), e.g. `editions/genesis-edition.yaml` |

### `SpecificationDocument`

| | |
|--|--|
| **Purpose** | One normative or supporting Markdown (or future format) file under the spec tree. |
| **Responsibilities** | Expose relative path, raw text access, parsed `DocumentFrontMatter`, and section structure when loaded. |
| **Non-responsibilities** | Lint prose; enforce required sections; modify content. |
| **Likely source files** | `docs/**/*.md`, `rfcs/*.md`, root-level `*.md` per documented corpus scope |

### `DocumentFrontMatter`

| | |
|--|--|
| **Purpose** | Typed view of YAML front matter at the top of a specification document. |
| **Responsibilities** | Expose documented fields (`spec`, `title`, `status`, `version`, `depends_on`, etc.); record line range in source file. |
| **Non-responsibilities** | Validate field values against governance rules; infer version when front matter is absent. |
| **Likely source files** | Leading `---` block of Markdown files in the document corpus |

### `DocumentSection`

| | |
|--|--|
| **Purpose** | Structural unit within a document (heading hierarchy, optional HTML anchors). |
| **Responsibilities** | Expose heading level, title, slug or id, line span; support anchor resolution for link targets. |
| **Non-responsibilities** | Validate cross-links; enforce section templates. |
| **Likely source files** | Parsed body of `SpecificationDocument` sources |

### `ReferenceGraph`

| | |
|--|--|
| **Purpose** | Typed graph of references discovered in the specification corpus. |
| **Responsibilities** | Record reference kind (VP-TERM, VP-RFC, relative link, anchor); source location (file, line, column); target string or resolved path. |
| **Non-responsibilities** | Resolve whether references are valid; emit broken-link diagnostics. |
| **Likely source files** | Derived from Markdown corpus scan (same discovery scope as [cross-reference validation](CROSS_REFERENCE_VALIDATION.md)) |

### Resolution (future)

Today, `ReferenceGraph` records **symbolic** references—`VP-TERM-009`, `VP-RFC-0000`, relative paths, anchors—as strings with source locations. Future milestones may introduce reference resolution services that map symbolic references (VP-TERM, VP-RFC, VP-CS, document anchors) to typed model objects (`Reference` → `ResolvedReference`) while preserving the immutable Specification Model—so `VP-TERM-009` becomes a handle to `TerminologyEntry` rather than an opaque string. That layer is what the reference interpreter will eventually consume; it is **not** in scope for the first `vp-spec-model` milestone.

---

## Boundaries

### The model

| In scope | Out of scope |
|----------|--------------|
| Read from `SpecRepository` | Write or mutate `veritypay-spec` |
| Expose typed, immutable structures | Apply validation rules or assign severity |
| Preserve source locations where possible | Publish Editions or sign artifacts |
| Parse YAML and Markdown into documented shapes | Execute claims or protocol behavior |
| Support partial load (e.g. registries only) | Replace `vp validate` or the engine |
| Fail build with structured parse errors at the builder boundary | Emit `vp-*` rule diagnostics (validators own those) |

Parse failures at build time are **builder concerns** (e.g. "manifest YAML could not be parsed into `EditionManifest`"). Rule violations are **validator concerns** (e.g. `vp-edition-unknown-rfc`).

### Validators

Per [ADR-0003](adrs/0003-validator-execution-model.md):

| Validators | Model |
|------------|-------|
| May use the model as **read-only input** | Does not call validators |
| **Own** rule logic and diagnostics | Does not own rule IDs or severity policy |
| **Emit** diagnostics via `vp-diagnostics` | Does not aggregate reports or exit codes |
| **Remain independent**—no validator invokes another | Built once per session; no hidden shared mutable state between validators |

Validators may continue to use `SpecRepository` directly until migrated. Migration is **opt-in per validator**, not a big-bang rewrite.

---

## Relationship to existing validators

### Short term

| Validator | Today | With model (not required yet) |
|-----------|-------|-------------------------------|
| **Registry (`vp-registry`)** | Parses `registry.yaml` directly | Unchanged; continues to pass all tests |
| **Cross-reference (`vp-crossref`)** | Discovers references and loads registries inline | Unchanged |
| **Edition (`vp-edition`)** | Loads manifest and registries inline | Unchanged |

Introducing `vp-spec-model` must **not** change CLI behavior or existing diagnostic output until validators explicitly migrate.

### Migration status

| Validator | Registry consumption via `vp-spec-model` |
|-----------|------------------------------------------|
| **Registry (`vp-registry`)** | ✓ migrated |
| **Edition (`vp-edition`)** | ✓ migrated |
| **Cross-reference (`vp-crossref`)** | ✓ migrated |

Validator migration is recorded in [ADR-0006](adrs/0006-spec-model-migration-complete.md).

### Medium term

| Consumer | Model usage |
|----------|-------------|
| **Registry validator** | Validate loaded `RegistrySet` instead of re-parsing YAML |
| **Cross-reference validator** | Validate `ReferenceGraph` against `RegistrySet` |
| **Edition validator** | Validate loaded `EditionManifest` against `RegistrySet` and `SpecificationDocument` pins |

Each migration keeps **rule ownership** in the validator crate. The model only removes duplicate parsing.

### Long term

| Consumer | Model usage |
|----------|-------------|
| **Documentation generation** | Render derived views from `Specification` and registries |
| **Reference interpreter (`veritypay-reference`)** | Consume stable types for normative document graph |
| **Conformance tooling** | Resolve VP-CS baseline IDs against a future VP-CS registry layer on the same model |

---

## Builder design notes (non-normative)

These constraints guide the first implementation but are not themselves normative spec policy:

| Concern | Direction |
|---------|-----------|
| **Immutability** | `Specification` snapshot is frozen after build; matches `ValidationContext` immutability |
| **Incremental build** | Registries-only and document-corpus milestones are complete; `ReferenceGraph` is a later milestone |
| **Parse vs validate** | Builder returns `Result` or optional components; validators interpret missing data as rule failures |
| **Location preservation** | Prefer attaching `Location` or path metadata to model nodes for diagnostic quality |
| **Dependencies** | `vp-spec-model` depends on `vp-core` (`SpecRepository`); does not depend on `vp-engine` or validator crates |
| **Crate home** | `crates/vp-spec-model/` per [ADR-0002](adrs/0002-workspace-architecture.md) |

---

## Success criteria (first milestone)

The first `vp-spec-model` milestone is **complete** when:

| # | Criterion |
|---|-----------|
| 1 | `vp-spec-model` crate exists in the workspace |
| 2 | Can load VP-TERM registry into typed `TerminologyRegistry` structures |
| 3 | Can load VP-RFC registry into typed `RfcRegistry` structures |
| 4 | Exposes stable data structures **without** validation behavior |
| 5 | All existing validators continue passing their test suites |
| 6 | No behavior changes in `vp validate` CLI output or exit codes |
| 7 | Builder reads exclusively via `SpecRepository` |

**Not included in the first milestone:** migrating existing validators; `ReferenceGraph`; `EditionManifest` parsing; docs generation; reference interpreter integration; validation rule diagnostics from the model layer.

### Document corpus milestone

The document corpus milestone is **complete** when:

| # | Criterion |
|---|-----------|
| 1 | `DocumentCorpus` and `SpecificationDocument` types exist in `vp-spec-model` |
| 2 | `SpecificationBuilder::build_documents_only()` and `build_registries_and_documents()` load the Markdown corpus |
| 3 | Front matter, heading anchors, and HTML `id` anchors are extracted without validation rules |
| 4 | Corpus discovery matches the cross-reference scope (including exclusions) |
| 5 | All existing validators and CLI behavior remain unchanged |

**Not included:** `ReferenceGraph`; migrating `vp-crossref`; link validation; required front matter checks.

---

## Governance alignment

| Topic | Authority |
|-------|-----------|
| Registry field definitions | `veritypay-spec` — `spec/terminology/registry.yaml`, `spec/rfcs/registry.yaml`, VP-RFC-0000 |
| Edition Manifest fields | [SPECIFICATION_RELEASE_PROCESS.md](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_RELEASE_PROCESS.md) |
| Document front matter conventions | Spec documents and VP-RFC-0000 |
| Whether a rule is normative | `veritypay-spec` governance; tooling implements checks in validators |

When model shapes should change, update **spec or tooling ADR first**, then implement in `vp-spec-model`.

---

## Related documents

| Document | Relationship |
|----------|--------------|
| [VALIDATION_ENGINE.md](VALIDATION_ENGINE.md) | Validator lifecycle; model is input, not engine |
| [REGISTRY_VALIDATION.md](REGISTRY_VALIDATION.md) | First consumer domain for `RegistrySet` |
| [CROSS_REFERENCE_VALIDATION.md](CROSS_REFERENCE_VALIDATION.md) | Future consumer of `ReferenceGraph` |
| [EDITION_VALIDATION.md](EDITION_VALIDATION.md) | Future consumer of `EditionManifest` |
| [CONFIGURATION_ARCHITECTURE.md](CONFIGURATION_ARCHITECTURE.md) | Session config; model does not parse config |
| [ADR-0002](adrs/0002-workspace-architecture.md) | Crate placement |
| [ADR-0003](adrs/0003-validator-execution-model.md) | Validator independence preserved |
| [ADR-0005](adrs/0005-specification-model.md) | Engineering decision for `vp-spec-model` |
| [ADR-0006](adrs/0006-spec-model-migration-complete.md) | Validator migration to shared model input |
| [ARCHITECTURE.md](../ARCHITECTURE.md) | Platform component model |
| [ROADMAP.md](../ROADMAP.md) | Capability milestones |

---

*Validators ask whether the specification is coherent. The Specification Model asks what the specification contains—and how it was assembled.*
