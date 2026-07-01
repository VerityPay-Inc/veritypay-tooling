# Roadmap

**Capability-based roadmap for `veritypay-tooling`.**

This roadmap is **not date-driven**. Milestones complete when their success criteria are met—not when a quarter ends. Progress aligns with [Phase II Platform Plan — Milestone A–G (tooling spine)](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/PHASE_II_PLATFORM_PLAN.md).

**Current milestone:** **Validation Platform Ready** — Milestones B–D and the `vp-spec-model` shared layer are complete; Milestone E (CLI polish) and beyond remain deferred.

---

## Overview

| Milestone | Name | Status |
|-----------|------|--------|
| **A** | Tooling scaffold | **Complete** |
| **B** | Registry validation | **Complete** |
| **C** | Cross-reference validation | **Complete** |
| **C.3** | CLI output (human, JSON, quiet) | **Complete** |
| **C.4** | Configuration (`.vp.toml`) | **Complete** |
| **D** | Edition validation | **Complete** |
| **—** | `vp-spec-model` shared layer | **Complete** |
| **E** | CLI polish | Not started |
| **F** | Documentation generation | Not started |
| **G** | Public automation | Not started |

Each milestone below includes **Goal**, **Outputs**, **Success criteria**, and **Not included** so scope stays explicit.

---

## Milestone A — Tooling scaffold

**Goal:** Establish `veritypay-tooling` as a mature **engineering project** before validators exist—clear purpose, architecture, CLI philosophy, and contribution rules.

**Outputs:**

- [README.md](README.md) — purpose, boundaries, links to local docs and `veritypay-spec`
- [ARCHITECTURE.md](ARCHITECTURE.md) — component model (conceptual; no implementation language)
- [ROADMAP.md](ROADMAP.md) — this document with milestones A–G
- [CLI_PHILOSOPHY.md](CLI_PHILOSOPHY.md) — future `vp` CLI principles (illustrative commands only)
- [CONTRIBUTING.md](CONTRIBUTING.md) — contributor expectations and specification boundary
- [LICENSE](LICENSE) — license terms
- Repository maturity declared: **Scaffold**

**Success criteria:**

- [x] A new contributor can explain what tooling does and does not do in five minutes
- [x] Dependency on `veritypay-spec` is explicit and one-directional
- [x] Milestones B–G each define goal, outputs, success criteria, and not-included scope
- [x] No validator code merged under the pretense of "early MVP"

**Not included:**

- Validator implementation (registry, cross-ref, edition, or docs)
- CLI binary or library source code
- CI workflows or GitHub Actions
- Language, framework, or package manager choice (deferred to ADR at Milestone B)
- Changes to normative text in `veritypay-spec`

---

## Milestone B — Registry validation

**Goal:** CI can fail on invalid or inconsistent **VP-TERM** and **VP-RFC** registries.

**Outputs:**

- Registry schema definitions (documented; implementation in tooling)
- [docs/VALIDATION_ENGINE.md](docs/VALIDATION_ENGINE.md) — shared validator framework architecture
- [docs/REGISTRY_VALIDATION.md](docs/REGISTRY_VALIDATION.md) — registry validator responsibilities and rule classes
- [docs/adrs/0003-validator-execution-model.md](docs/adrs/0003-validator-execution-model.md) — ADR-0003: Validator execution model (Accepted)
- [docs/adrs/0002-workspace-architecture.md](docs/adrs/0002-workspace-architecture.md) — ADR-0002: Cargo workspace (Accepted)
- [docs/adrs/0001-tooling-implementation-language.md](docs/adrs/0001-tooling-implementation-language.md) — ADR-0001: Rust (Accepted)
- Validator for `spec/terminology/registry.yaml`
- Validator for `spec/rfcs/registry.yaml`
- Front matter checks for RFC registry alignment (minimal)
- Integration path documented for `veritypay-spec` PR CI

**Success criteria:**

- [x] Intentionally broken registry entries fail validation with actionable messages
- [x] Valid registries in current `veritypay-spec` main pass
- [x] Duplicate IDs and invalid status values are detected
- [x] Validation runnable locally against a spec checkout path

**Not included:**

- Cross-reference scanning of full Markdown corpus (Milestone C)
- Edition Manifest validation (Milestone D)
- Unified `vp validate` orchestration (Milestone E)
- Automatic glossary rewrite or normative term definition
- Glossary–registry full sync automation (may report drift only)

---

## Milestone C — Cross-reference validation

**Goal:** Unknown **VP-TERM**, **VP-RFC**, architecture anchors, and broken internal links fail CI.

**Outputs:**

- Cross-reference scanner for Markdown corpus
- ID resolution against registries and documented anchor conventions
- Documentation validation for relative links
- Front matter validation expanded for spec documents

**Success criteria:**

- [x] Reference to non-existent VP-TERM-9999 fails with file and line citation
- [x] Broken internal markdown links fail validation
- [x] Valid `veritypay-spec` main passes (or documents known exceptions with tracking)
- [x] Checks composable independently of registry-only validation

**Not included:**

- Edition Manifest validation (Milestone D)
- External URL crawling as a required merge gate
- Semantic review of prose correctness
- Documentation generation (Milestone F)
- Protocol semantics or claim evaluation
- Edition Manifest validation (Milestone D—after [C.4 Configuration](docs/CONFIGURATION_ARCHITECTURE.md))

---

## Milestone C.3 — CLI output

**Goal:** Human-readable, JSON, and quiet validation output for local development and CI.

**Outputs:**

- Human renderer with rule IDs, locations, and suggestions
- JSON output mode for machine consumption
- `--quiet` summary-only mode
- Exit codes: validation failures (1), configuration errors (2)

**Success criteria:**

- [x] `vp validate --format human` prints actionable diagnostics
- [x] `vp validate --format json` emits structured findings
- [x] `vp validate --quiet` prints summary counts only
- [x] Valid `veritypay-spec` main passes with zero errors in all output modes

**Not included:**

- Subcommand groups and `--help` polish (Milestone E)
- CI entrypoint in `veritypay-spec` (Milestone G)
- Documentation generation output (Milestone F)

---

## Milestone C.4 — Configuration

**Goal:** Centralize validation options in **`.vp.toml`** so CI and local runs share defaults without flag sprawl.

**Outputs:**

- [docs/CONFIGURATION_ARCHITECTURE.md](docs/CONFIGURATION_ARCHITECTURE.md) — config architecture and merge precedence
- [docs/adrs/0004-configuration-model.md](docs/adrs/0004-configuration-model.md) — ADR-0004: Configuration model (Accepted)
- `ValidationConfig` on `ValidationContext` (`spec_root`, `profile`, `output`, `edition`, `strict`)
- Config loader in tooling; CLI flags override file values
- Keys stored for future profile and Edition validators even before those features ship

**Success criteria:**

- [x] `.vp.toml` `[validation]` table loads when present; absent config does not break CLI
- [x] CLI > config > defaults merge precedence tested
- [x] `vp validate --spec X` unchanged when no config file exists
- [x] Edition manifest path available as `ctx.config.edition` for Milestone D

**Not included:**

- Validation profile engine (stores `profile` key only until profiles ship)
- Edition validator (Milestone D)
- Global user config or secrets in TOML
- Normative spec changes

**Prerequisite for:** Milestone D Edition validation.

---

## Milestone D — Edition validation

**Goal:** **Edition Manifests** can be validated before publication.

**Prerequisite:** Milestone C.4 Configuration (manifest path via `ValidationConfig.edition`).

**Outputs:**

- [docs/EDITION_VALIDATION.md](docs/EDITION_VALIDATION.md) — Edition identity, states, validation architecture
- Edition Manifest schema validator (`vp-edition` crate)
- Pin existence checks (paths, versions, registry snapshots)
- Edition builder draft mode (optional artifact generation—non-normative)
- Alignment with [SPECIFICATION_RELEASE_PROCESS](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_RELEASE_PROCESS.md) checklist

**Success criteria:**

- [x] Illustrative Genesis manifest draft validates or reports concrete gaps
- [x] Invalid manifest (missing pin, bad document reference) fails with structured errors
- [x] Edition validation does not mutate `veritypay-spec`
- [x] Maintainers can run edition checks as part of release candidate workflow

**Not included:**

- Publishing or declaring an Edition (governance in `veritypay-spec`)
- Reference interpreter or conformance execution
- Public website or SPECIFICATION_STATUS auto-publish
- Normative changes to manifest policy without spec governance

---

## Specification model — shared layer (complete)

**Goal:** Provide a stable, typed representation of the specification corpus consumed by all validators.

**Outputs:**

- [docs/SPECIFICATION_MODEL.md](docs/SPECIFICATION_MODEL.md) — architecture
- [docs/adrs/0005-specification-model.md](docs/adrs/0005-specification-model.md) — ADR-0005 (Accepted)
- [docs/adrs/0006-spec-model-migration-complete.md](docs/adrs/0006-spec-model-migration-complete.md) — ADR-0006 (Accepted)
- `vp-spec-model` crate — `SpecificationBuilder`, `RegistrySet`, `DocumentCorpus`, `ReferenceGraph`

**Success criteria:**

- [x] `vp-spec-model` loads registries into typed structures
- [x] `vp-spec-model` loads `DocumentCorpus` with front matter and section anchors
- [x] `ReferenceGraph` with node/edge lookup built by `SpecificationBuilder`
- [x] `vp-registry` uses `vp-spec-model` for typed registry loading
- [x] `vp-edition` uses `vp-spec-model` for registry lookup during Edition validation
- [x] `vp-crossref` validates `ReferenceGraph` edges with hybrid fallback on model load failure
- [x] Existing validators and CLI unchanged in diagnostic semantics

**Deferred:**

- Typed `EditionManifest` model
- Reference resolution layer beyond graph edges
- Removing hybrid fallback when malformed input no longer needs raw diagnostics

---

## Specification model — first milestone

**Goal:** Introduce typed registry loading via `vp-spec-model` without migrating validators.

**Outputs:**

- [docs/SPECIFICATION_MODEL.md](docs/SPECIFICATION_MODEL.md) — architecture
- [docs/adrs/0005-specification-model.md](docs/adrs/0005-specification-model.md) — ADR-0005 (Accepted)
- `vp-spec-model` crate — `SpecificationBuilder`, VP-TERM/VP-RFC typed registries

**Success criteria:**

- [x] `vp-spec-model` loads registries into typed structures
- [x] Existing validators and CLI unchanged
- [x] `vp-registry` uses `vp-spec-model` for typed registry loading on valid registries
- [x] `vp-edition` uses `vp-spec-model` for registry lookup during Edition validation

**Not included:** validator migration, `EditionManifest`, `ReferenceGraph`.

---

## Specification model — document corpus milestone

**Goal:** Load the Markdown document corpus into typed structures without migrating validators.

**Success criteria:**

- [x] `vp-spec-model` loads `DocumentCorpus` with front matter and section anchors
- [x] `SpecificationBuilder::build_documents_only()` and `build_registries_and_documents()` implemented
- [x] Existing validators and CLI unchanged

**Not included:** `ReferenceGraph`, link validation.

---

## Specification model — cross-reference migration

**Goal:** Cross-reference validation consumes `DocumentCorpus` and `RegistrySet` from `vp-spec-model`.

**Success criteria:**

- [x] `vp-crossref` uses `SpecificationBuilder::build_registries_and_documents()` with hybrid fallback
- [x] Document scan, anchor lookup, and registry resolution use typed model data when available
- [x] Existing crossref fixture tests unchanged

**ADR:** [docs/adrs/0006-spec-model-migration-complete.md](docs/adrs/0006-spec-model-migration-complete.md) — validator migration complete (Accepted)

---

## Specification model — reference graph milestone

**Goal:** Capture symbolic references across the specification corpus as immutable model data.

**Success criteria:**

- [x] `ReferenceGraph` with node/edge lookup and incoming/outgoing traversal
- [x] Built by `SpecificationBuilder` using shared `ReferenceDiscovery`
- [x] Existing validators and CLI unchanged

**Not included:** migrating validators to validate via `ReferenceGraph`; reference resolution.

---

## Specification model — cross-reference graph validation

**Goal:** Cross-reference validation validates `ReferenceGraph` edges instead of rediscovering references.

**Success criteria:**

- [x] `vp-crossref` iterates `ReferenceGraph` edges when the model loads successfully
- [x] Hybrid fallback preserved when typed model load fails
- [x] Existing crossref fixture tests unchanged

---

## Milestone E — CLI polish

**Goal:** Unified **`vp`** CLI suitable for local development and CI orchestration.

**Outputs:**

- Subcommands per [CLI_PHILOSOPHY.md](CLI_PHILOSOPHY.md) (validate, lint, registry, edition, docs, release)—final names via ADR
- Consistent exit codes and `--help` discoverability
- Machine-readable output mode for CI (format documented)
- Version command reporting tooling and supported spec layout version

**Success criteria:**

- [ ] `vp validate` runs registry + cross-ref (+ edition when configured) — *partial: validate subcommand ships; front matter rules are embedded in registry/crossref validators*
- [ ] `vp --help` documents command groups without reading source
- [ ] CI in `veritypay-spec` can invoke a single entrypoint
- [ ] Errors are readable by contributors unfamiliar with codebase

**Not included:**

- Documentation generation features (Milestone F)
- Organization-wide reusable workflows (Milestone G)
- Stable 1.0 CLI compatibility guarantee before tooling 1.0 release
- Commands that accept/reject RFCs or alter protocol meaning

---

## Milestone F — Documentation generation

**Goal:** Reduce manual duplication via **non-normative** generated views from registries.

**Outputs:**

- `vp docs` (or equivalent) generating index pages from VP-TERM / VP-RFC registries
- Clear "generated—do not edit" markers on output
- Optional dependency graph or status table for maintainers
- Documentation of what may vs may not be generated

**Success criteria:**

- [ ] Generated term index matches registry on clean spec main
- [ ] Generated output is reproducible from same inputs
- [ ] No generated file is treated as normative without spec governance promotion
- [ ] Generation failures do not silently produce partial output

**Not included:**

- Public website deployment pipeline
- Overwriting canonical files in `veritypay-spec` without explicit maintainer workflow
- Normative glossary or RFC text generation
- SPECIFICATION_STATUS full automation (may define hooks only; Milestone G)

---

## Milestone G — Public automation

**Goal:** Organization-wide **reusable automation** for specification hygiene.

**Outputs:**

- Reusable GitHub Actions (or equivalent) consumable by `veritypay-spec` and sibling reference/conformance repos
- Org `.github` integration documented
- Public contributor docs for enabling validation on forks
- SPECIFICATION_STATUS automation hooks defined (implementation may live in spec or tooling—documented)

**Success criteria:**

- [ ] Spec PRs run tooling checks without custom copy-paste workflows per repo
- [ ] Fork contributors can run equivalent checks locally and in CI
- [ ] Automation version pinned and changelog maintained
- [ ] Grant / audit audience can point to public CI evidence of validation

**Not included:**

- SDK release automation
- Reference interpreter CI (owned by `veritypay-reference`)
- Conformance suite execution (owned by `veritypay-conformance`)
- Certification program or vendor badge issuance
- Production deployment or marketing site builds

---

## After Milestone G

Tooling enters **maintenance and extension** mode: new registries (VP-CS, VP-EDITION), stricter Edition rules, and website integration as spec governance defines them.

**Explicitly deferred** (see Phase II plan):

- SDK tooling
- Reference interpreter helpers (minimal shared types only if ADR-approved)
- Certification automation
- Public marketing website build pipeline

---

## How to propose roadmap changes

Roadmap changes are **tooling governance**, not protocol changes.

1. Open an issue describing capability gap and proposed milestone adjustment
2. For structural CLI or validation contract changes, write an ADR in this repository
3. If validation rules imply **new normative spec requirements**, propose RFC in `veritypay-spec` first

---

*Capability before calendar. Validation before execution.*
