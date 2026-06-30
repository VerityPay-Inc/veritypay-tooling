# Roadmap

**Capability-based roadmap for `veritypay-tooling`.**

This roadmap is **not date-driven**. Milestones complete when their success criteria are met—not when a quarter ends. Progress aligns with [Phase II Platform Plan — Milestone A–G (tooling spine)](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/PHASE_II_PLATFORM_PLAN.md).

**Current milestone:** **A — Repository scaffold**

---

## Overview

| Milestone | Name | Status |
|-----------|------|--------|
| **A** | Repository scaffold | **In progress** |
| **B** | Registry validation | Not started |
| **C** | Cross-reference validation | Not started |
| **D** | Edition validation | Not started |
| **E** | CLI polish | Not started |
| **F** | Documentation generation | Not started |
| **G** | Public automation | Not started |

---

## Milestone A — Repository scaffold

**Goal:** Establish `veritypay-tooling` as a mature **engineering project** before validators exist—clear purpose, architecture, CLI philosophy, and contribution rules.

**Outputs:**

- [README.md](README.md) — purpose, boundaries, links to `veritypay-spec`
- [ARCHITECTURE.md](ARCHITECTURE.md) — component model (conceptual)
- [ROADMAP.md](ROADMAP.md) — this document
- [CLI_PHILOSOPHY.md](CLI_PHILOSOPHY.md) — future `vp` CLI principles
- [CONTRIBUTING.md](CONTRIBUTING.md) — contributor expectations
- Repository maturity declared: **Scaffold**

**Success criteria:**

- [ ] A new contributor can explain what tooling does and does not do in five minutes
- [ ] Dependency on `veritypay-spec` is explicit and one-directional
- [ ] Milestones B–G have defined scope without implementation commitment
- [ ] No validator code merged under the pretense of "early MVP"

---

## Milestone B — Registry validation

**Goal:** CI can fail on invalid or inconsistent **VP-TERM** and **VP-RFC** registries.

**Outputs:**

- Registry schema definitions (documented; implementation in tooling)
- Validator for `spec/terminology/registry.yaml`
- Validator for `spec/rfcs/registry.yaml`
- Front matter checks for RFC registry alignment (minimal)
- Integration path documented for `veritypay-spec` PR CI

**Success criteria:**

- [ ] Intentionally broken registry entries fail validation with actionable messages
- [ ] Valid registries in current `veritypay-spec` main pass
- [ ] Duplicate IDs and invalid status values are detected
- [ ] Validation runnable locally against a spec checkout path

---

## Milestone C — Cross-reference validation

**Goal:** Unknown **VP-TERM**, **VP-RFC**, architecture anchors, and broken internal links fail CI.

**Outputs:**

- Cross-reference scanner for Markdown corpus
- ID resolution against registries and documented anchor conventions for `depends_on` / `required_by`
- Documentation validation for relative links
- Front matter validation expanded for spec documents

**Success criteria:**

- [ ] Reference to non-existent VP-TERM-9999 fails with file and line citation
- [ ] Broken internal markdown links fail validation
- [ ] Valid `veritypay-spec` main passes (or documents known exceptions with tracking)
- [ ] Checks composable independently of registry-only validation

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

---

## Milestone E — CLI polish

**Goal:** Unified **`vp`** CLI suitable for local development and CI orchestration.

**Outputs:**

- Subcommands per [CLI_PHILOSOPHY.md](CLI_PHILOSOPHY.md) (validate, lint, registry, edition, docs, release)
- Consistent exit codes and `--help` discoverability
- Machine-readable output mode for CI (format documented)
- Version command reporting tooling and supported spec pin range

**Success criteria:**

- [ ] `vp validate` runs registry + cross-ref + front matter (+ edition when configured)
- [ ] `vp --help` documents command groups without reading source
- [ ] CI in `veritypay-spec` can invoke a single entrypoint
- [ ] Errors are readable by contributors unfamiliar with codebase

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
2. For structural CLI or validation contract changes, write an ADR in this repo
3. If validation rules imply **new normative spec requirements**, propose RFC in `veritypay-spec` first

---

*Capability before calendar. Validation before execution.*
