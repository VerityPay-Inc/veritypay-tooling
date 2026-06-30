---
id: ADR-0002
title: Cargo Workspace Architecture
status: accepted
version: 1.0.0
authors:
  - VerityPay Core Team
reviewers: []
related_docs:
  - docs/adrs/0001-tooling-implementation-language.md
  - docs/VALIDATION_ENGINE.md
  - docs/REGISTRY_VALIDATION.md
  - ROADMAP.md
decision_date: 2026-06-29
superseded_by: null
---

# ADR-0002 — Cargo Workspace Architecture

**Status:** accepted · **Version:** 1.0.0 · **Date:** 2026-06-29

**Related:** [ADR-0001](0001-tooling-implementation-language.md) · [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) · [REGISTRY_VALIDATION.md](../REGISTRY_VALIDATION.md) · [ROADMAP.md](../../ROADMAP.md)

---

## Context

`veritypay-tooling` now has:

- Repository [ARCHITECTURE.md](../../ARCHITECTURE.md) and Milestone A scaffold
- [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) — shared validator lifecycle and composition model
- [REGISTRY_VALIDATION.md](../REGISTRY_VALIDATION.md) — first validator scope (Milestone B)
- [ADR-0001](0001-tooling-implementation-language.md) — **Rust** as implementation language

Milestone B implementation is about to begin. Before any source files land, the project must record **how the Rust codebase decomposes** into crates.

A single binary crate would tempt:

- Monolithic modules mixing CLI, orchestration, diagnostics, and VP-TERM rules
- Hidden coupling between validators
- Circular imports as `vp-crossref` and `vp-edition` arrive in later milestones
- A catch-all `utils` or `common` crate that becomes unmaintainable

[VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) requires **independent validators** composed by an engine that aggregates diagnostics without re-running rule logic. The workspace layout must **enforce** that architecture at compile time.

This ADR records crate boundaries, dependency direction, and expansion rules. It does **not** create `Cargo.toml` files or code—that follows in Milestone B implementation PRs.

---

## Decision

**Implement `veritypay-tooling` as a Cargo workspace of independent crates**, not a single binary package.

Initial workspace members (Milestone B):

| Crate | Role |
|-------|------|
| **`vp-cli`** | Binary entrypoint (`vp`); argument parsing; human/JSON output; process exit code |
| **`vp-engine`** | Validation orchestration; validator scheduling; report aggregation |
| **`vp-core`** | Shared types, validation context, validator interface (traits) |
| **`vp-diagnostics`** | Diagnostic model, severity, rule IDs, report structures |
| **`vp-registry`** | Registry validator (VP-TERM, VP-RFC) |

Future workspace members (post–Milestone B): **`vp-crossref`**, **`vp-edition`**, **`vp-docs`** — each a separate validator crate following the same boundaries.

**Composition rule:** `vp-engine` orchestrates validators through the **public interface defined in `vp-core`**. It does **not** depend on validator crates (`vp-registry`, etc.). **`vp-cli`** links the engine and registers which validators are available at runtime (compile-time wiring for Milestone B; extension via new crates + CLI registration, not engine edits to rule logic).

---

## Crate responsibilities

### `vp-cli`

| Field | Definition |
|-------|------------|
| **Purpose** | User-facing **`vp`** binary for local development and CI |
| **Responsibilities** | Parse arguments; build `ValidationContext`; invoke `vp-engine`; format reports (human / JSON); map aggregated result to process exit code; `--help` and version output per [CLI_PHILOSOPHY.md](../../CLI_PHILOSOPHY.md) |
| **Does not belong** | Validation rules; registry parsing logic; diagnostic type definitions; protocol semantics |
| **Public interface (conceptual)** | Binary only—no library API required for external consumers in Milestone B |
| **Depends on** | `vp-engine`, `vp-core`, `vp-diagnostics`, validator crates (`vp-registry`, …) for **registration wiring only** |

The CLI is a **thin shell**. Substantive logic belongs in engine, validators, or shared crates.

---

### `vp-engine`

| Field | Definition |
|-------|------------|
| **Purpose** | **Validation engine** — shared runtime from [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) |
| **Responsibilities** | Accept validation request + registered validator list; run lifecycle (discovery → validation → diagnostics collection); invoke each validator through `vp-core` interface; merge diagnostics via report aggregator; apply exit-code policy; support subset execution (e.g. registry-only) |
| **Does not belong** | VP-TERM/VP-RFC rule implementations; CLI argument definitions; Markdown/YAML parsing specific to one validator; reading validator-internal structs |
| **Public interface (conceptual)** | `run_validation(context, validators) -> ValidationResult` (names illustrative); validator registration types consumed from `vp-core` |
| **Depends on** | `vp-core`, `vp-diagnostics` only — **not** `vp-registry` or future validator crates |

The engine knows **that** a validator runs and **what** diagnostics it returns. It does **not** know **how** registry rules are evaluated.

---

### `vp-core`

| Field | Definition |
|-------|------------|
| **Purpose** | **Stable contracts** between engine, CLI, and validators |
| **Responsibilities** | `ValidationContext` (spec root, scope, edition pin, output mode, strictness); validator trait(s) (`name`, `category`, `validate(&context) -> Vec<Diagnostic>`); shared identifiers (validator id, category enum); path resolution helpers that are engine/validator neutral |
| **Does not belong** | Rule implementations; CLI types (`clap` structs); report formatting; serde schemas for YAML registries |
| **Public interface (conceptual)** | Traits and context types documented as the **only** coupling surface between engine and validators |
| **Depends on** | `vp-diagnostics` (validators emit diagnostics) |

`vp-core` stays **small**. Resist expanding it with every shared helper—prefer placing logic in the validator that owns the rule.

---

### `vp-diagnostics`

| Field | Definition |
|-------|------------|
| **Purpose** | **Structured findings model** — audit artifact for CI and contributors |
| **Responsibilities** | `Diagnostic`, `Severity`, `RuleId`, `Location`, `Category`; `Report` and summary counts; exit-code policy helpers (map severities → pass/fail); stable rule ID types |
| **Does not belong** | Validation logic; file I/O; CLI printing; engine orchestration |
| **Public interface (conceptual)** | Diagnostic builder API; report merge; severity ordering; optional JSON serialization boundary (format version documented when implemented) |
| **Depends on** | Standard library only (no `vp-core`, no engine) — **leaf crate** |

Diagnostics are the **lingua franca** of the workspace. Every validator speaks this type system.

---

### `vp-registry`

| Field | Definition |
|-------|------------|
| **Purpose** | **Registry validator** — Milestone B implementation of [REGISTRY_VALIDATION.md](../REGISTRY_VALIDATION.md) |
| **Responsibilities** | Discover `spec/terminology/registry.yaml` and `spec/rfcs/registry.yaml`; validate structure and internal consistency; emit `vp-diagnostics` with `vp-registry-*` / `vp-term-*` / `vp-rfc-*` rule IDs; read-only access to spec tree |
| **Does not belong** | Cross-reference scanning of full Markdown corpus; Edition manifests; CLI; engine aggregation; defining normative registry policy (spec governs; this crate implements checks) |
| **Public interface (conceptual)** | Implements validator trait from `vp-core`; exposes constructor/factory for CLI registration |
| **Depends on** | `vp-core`, `vp-diagnostics` — **not** `vp-engine`, **not** `vp-cli` |

All registry-specific parsing and rule knowledge stays **inside this crate**.

---

## Dependency graph

```
                    ┌─────────────┐
                    │   vp-cli    │  (binary)
                    └──────┬──────┘
           ┌───────────────┼───────────────┐
           ▼               ▼               ▼
    ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
    │  vp-engine  │  │ vp-registry │  │ (future     │
    └──────┬──────┘  └──────┬──────┘  │  validators)│
           │                │         └──────┬──────┘
           │                │                │
           └────────┬───────┴────────────────┘
                    ▼
             ┌─────────────┐
             │   vp-core   │
             └──────┬──────┘
                    ▼
             ┌─────────────┐
             │vp-diagnostics│  (leaf)
             └─────────────┘
```

**Allowed dependency direction (summary):**

| From | May depend on |
|------|----------------|
| `vp-diagnostics` | *(none in workspace)* |
| `vp-core` | `vp-diagnostics` |
| `vp-registry`, `vp-crossref`, … | `vp-core`, `vp-diagnostics` |
| `vp-engine` | `vp-core`, `vp-diagnostics` |
| `vp-cli` | `vp-engine`, `vp-core`, `vp-diagnostics`, validator crates (wiring) |

**Forbidden:**

- Validator → `vp-engine` or `vp-cli`
- `vp-engine` → validator crates
- Any cycle in the graph

---

## Cyclic dependencies are forbidden

| Reason | Explanation |
|--------|-------------|
| **Architectural honesty** | Cycles hide boundaries the ADR and [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) exist to preserve |
| **Independent validators** | Milestone C must ship `vp-crossref` without refactoring `vp-registry` |
| **Testability** | Validators test against `vp-core` fixtures without loading the engine |
| **Compile times** | Cycles force full-workspace rebuilds and obscure ownership |
| **Review clarity** | Dependency direction makes "where does this rule live?" answerable in one sentence |

If two crates appear to need each other, **extract a smaller type or trait downward** (usually into `vp-core` or `vp-diagnostics`)—never introduce a mutual dependency.

---

## No `utils` or `common` crates

Intentionally **avoid** workspace members named `vp-utils`, `vp-common`, or `shared`.

| Problem | Consequence |
|---------|-------------|
| **Magnet for ambiguity** | Every helper lands there; ownership disappears |
| **Hidden coupling** | Validators share implicit state through grab-bag modules |
| **Review fatigue** | PRs touch `common` for unrelated reasons |
| **Violates Design Philosophy** | Complexity accumulates in the center instead of the edge |

**Instead:**

- **Cross-cutting types** → `vp-core` (minimal) or `vp-diagnostics` (findings only)
- **Validator-specific helpers** → stay inside that validator crate as private modules
- **Engine orchestration** → `vp-engine` only
- **CLI concerns** → `vp-cli` only

If `vp-core` grows too large, split by **conceptual boundary** (e.g. extract path resolution into `vp-spec-paths`) with a documented ADR—not into `common`.

---

## Engine composition without validator internals

The engine composes validators through **interface segregation**:

1. **`vp-core`** defines a validator trait: given `ValidationContext`, return diagnostics.
2. Each validator crate implements the trait and owns its discovery + rule logic internally.
3. **`vp-cli`** constructs a list of validator instances (e.g. `RegistryValidator::new()`) and passes them to **`vp-engine`**.
4. **`vp-engine`** iterates the list, calls `validate`, collects `Diagnostic` values, merges reports—**never** imports registry-specific types.

The engine does **not**:

- Match on validator enum variants tied to rule implementations
- Call into `vp-registry` module functions directly
- Re-interpret or transform rule-specific error payloads beyond aggregation

Adding Milestone C **`vp-crossref`** requires:

- New crate implementing the same trait
- One registration line in `vp-cli`
- **No change** to `vp-engine` orchestration logic (unless engine API evolves via ADR)

This mirrors [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md): *Report Aggregator merges lists; does not re-run validation logic.*

---

## Future expansion

| Crate | Milestone | Purpose |
|-------|-----------|---------|
| **`vp-crossref`** | C | Cross-reference and link validation |
| **`vp-edition`** | D | Edition Manifest validation and draft assistance |
| **`vp-docs`** | F | Non-normative documentation generation |

Each future crate:

- Implements the `vp-core` validator trait
- Depends only on `vp-core` + `vp-diagnostics` (and its own private modules)
- Registers from `vp-cli`
- Documents architecture in `docs/` when scope is non-trivial

Optional future split (not Milestone B): **`vp-metadata`** for front-matter-only checks if `vp-crossref` grows too large—decision deferred until evidence exists.

---

## Rationale

| Factor | Workspace decomposition |
|--------|-------------------------|
| **ADR-0001 (Rust)** | Cargo workspaces are idiomatic for multi-crate platforms |
| **Validation Engine doc** | Independent validators + aggregator maps directly to crate boundaries |
| **Milestone delivery** | B ships `vp-registry` without stubbing cross-ref |
| **Contributor clarity** | "Fix VP-TERM duplicate ID" → `vp-registry` only |
| **CI** | Single `vp` binary from `vp-cli`; internal crates are implementation detail |
| **Long-term maintenance** | Engine and diagnostics stable; validators evolve independently |

A monolith would compile faster initially and fail structurally later. The institution optimizes for **year-five maintainability**, not week-one LOC.

---

## Consequences

### Positive

- Compile-time enforcement of validator independence
- Clear ownership per milestone and per rule namespace
- Engine and CLI remain stable as validator count grows
- `vp-diagnostics` reusable by tests, JSON export, and future library consumers
- Aligns with [Design Philosophy](../../ARCHITECTURE.md) — complexity at the edges, understandable core

### Negative

- More crates to bootstrap before first green CI
- Contributors must learn workspace layout and dependency rules
- Initial PRs touch multiple `Cargo.toml` files when workspace is created

**Acceptable** because boundaries are cheaper to establish **before** Milestone B code than to extract after validators entangle.

---

## Future reconsideration

Revisit this ADR only if:

- Workspace member count creates measurable maintainer burden **without** corresponding validator isolation benefit
- A proposed crate violates acyclic rules and cannot be resolved by trait extraction
- An accepted ADR supersedes this layout (e.g. merging crates with evidence)

Splitting or merging crates requires **ADR-0003** (or successor)—not drive-by refactors.

---

## Related decisions

| Document | Relationship |
|----------|--------------|
| [ADR-0001](0001-tooling-implementation-language.md) | Rust + recommended cargo workspace |
| [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) | Lifecycle implemented by `vp-engine` |
| [REGISTRY_VALIDATION.md](../REGISTRY_VALIDATION.md) | Implemented in `vp-registry` |
| [CLI_PHILOSOPHY.md](../../CLI_PHILOSOPHY.md) | `vp-cli` behavior |
| [ROADMAP.md](../../ROADMAP.md) | Milestone B follows this ADR |

---

## Follow-up

- [ ] Add workspace `Cargo.toml` and crate manifests (Milestone B — separate PR)
- [ ] Document workspace layout in README when directories exist
- [ ] Add `vp-core` validator trait documentation in crate rustdoc when implemented

---

## Conclusion

The Verity Specification Platform treats **structure as a feature**. Crate boundaries are architecture made visible: the engine composes, validators verify, diagnostics report, the CLI delivers.

Implementation may begin once the workspace is scaffolded according to this ADR.

---

*Accepted ADRs are historical records. Supersede with a new ADR; do not silently rewrite this decision.*
