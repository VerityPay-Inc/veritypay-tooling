# Roadmap

**Capability-based roadmap for `veritypay-tooling`.**

This roadmap is **not date-driven**. Milestones complete when their success criteria are met—not when a quarter ends. Progress aligns with [Phase II Platform Plan — Milestone A–G (tooling spine)](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/PHASE_II_PLATFORM_PLAN.md).

**Current milestone:** **A — Tooling scaffold** *(complete when this document and sibling scaffold files are merged)*

---

## Overview

| Milestone | Name | Status |
|-----------|------|--------|
| **A** | Tooling scaffold | **Complete** (documentation) |
| **B** | Registry validation | Not started |
| **C** | Cross-reference validation | Not started |
| **D** | Edition validation | Not started |
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
- [docs/adrs/0002-workspace-architecture.md](docs/adrs/0002-workspace-architecture.md) — ADR-0002: Cargo workspace (Accepted)
- [docs/adrs/0001-tooling-implementation-language.md](docs/adrs/0001-tooling-implementation-language.md) — ADR-0001: Rust (Accepted)
- Validator for `spec/terminology/registry.yaml`
- Validator for `spec/rfcs/registry.yaml`
- Front matter checks for RFC registry alignment (minimal)
- Integration path documented for `veritypay-spec` PR CI

**Success criteria:**

- [ ] Intentionally broken registry entries fail validation with actionable messages
- [ ] Valid registries in current `veritypay-spec` main pass
- [ ] Duplicate IDs and invalid status values are detected
- [ ] Validation runnable locally against a spec checkout path

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

- [ ] Reference to non-existent VP-TERM-9999 fails with file and line citation
- [ ] Broken internal markdown links fail validation
- [ ] Valid `veritypay-spec` main passes (or documents known exceptions with tracking)
- [ ] Checks composable independently of registry-only validation

**Not included:**

- Edition Manifest validation (Milestone D)
- External URL crawling as a required merge gate
- Semantic review of prose correctness
- Documentation generation (Milestone F)
- Protocol semantics or claim evaluation

---

## Milestone D — Edition validation

**Goal:** **Edition Manifests** can be validated before publication.

**Outputs:**

- Edition Manifest schema validator
- Pin existence checks (paths, versions, registry snapshots)
- Edition builder draft mode (optional artifact generation—non-normative)
- Alignment with [SPECIFICATION_RELEASE_PROCESS](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_RELEASE_PROCESS.md) checklist

**Success criteria:**

- [ ] Illustrative Genesis manifest draft validates or reports concrete gaps
- [ ] Invalid manifest (missing pin, bad document reference) fails with structured errors
- [ ] Edition validation does not mutate `veritypay-spec`
- [ ] Maintainers can run edition checks as part of release candidate workflow

**Not included:**

- Publishing or declaring an Edition (governance in `veritypay-spec`)
- Reference interpreter or conformance execution
- Public website or SPECIFICATION_STATUS auto-publish
- Normative changes to manifest policy without spec governance

---

## Milestone E — CLI polish

**Goal:** Unified **`vp`** CLI suitable for local development and CI orchestration.

**Outputs:**

- Subcommands per [CLI_PHILOSOPHY.md](CLI_PHILOSOPHY.md) (validate, lint, registry, edition, docs, release)—final names via ADR
- Consistent exit codes and `--help` discoverability
- Machine-readable output mode for CI (format documented)
- Version command reporting tooling and supported spec layout version

**Success criteria:**

- [ ] `vp validate` (or agreed equivalent) runs registry + cross-ref + front matter (+ edition when configured)
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
