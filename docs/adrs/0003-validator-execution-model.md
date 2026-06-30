---
id: ADR-0003
title: Validator Execution Model
status: accepted
version: 1.0.0
authors:
  - VerityPay Core Team
reviewers: []
related_docs:
  - docs/adrs/0002-workspace-architecture.md
  - docs/VALIDATION_ENGINE.md
  - docs/REGISTRY_VALIDATION.md
  - ROADMAP.md
decision_date: 2026-06-29
superseded_by: null
---

# ADR-0003 — Validator Execution Model

**Status:** accepted · **Version:** 1.0.0 · **Date:** 2026-06-29

**Related:** [ADR-0002](0002-workspace-architecture.md) · [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) · [REGISTRY_VALIDATION.md](../REGISTRY_VALIDATION.md) · [ROADMAP.md](../../ROADMAP.md)

---

## Context

`veritypay-tooling` now has:

- [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) — conceptual validator lifecycle and composition
- [ADR-0001](0001-tooling-implementation-language.md) — Rust implementation language
- [ADR-0002](0002-workspace-architecture.md) — Cargo workspace decomposition (`vp-cli`, `vp-engine`, `vp-core`, `vp-diagnostics`, `vp-registry`)

The next implementation milestone introduces the **Registry Validator** ([REGISTRY_VALIDATION.md](../REGISTRY_VALIDATION.md)). Before code begins, the project must record **how validators execute** inside the workspace—not **what rules** they enforce.

Validation rules belong in architecture docs and spec governance. **Execution architecture** belongs here: who runs when, what data flows where, and what validators are forbidden from doing.

Without an explicit execution model, implementers will:

- Call one validator from another when a shared check is convenient
- Couple output formatting to rule logic
- Introduce order-dependent behavior that breaks CI reproducibility
- Block future parallel execution with hidden shared state

This ADR defines **validator interaction**. It does **not** define VP-TERM rules, registry schemas, or CLI flag syntax.

---

## Decision

**All validators execute through a single, engine-orchestrated pipeline** with strict separation of roles:

| Layer | Owns |
|-------|------|
| **Validators** | Discovery (within scope), rule evaluation, diagnostic emission |
| **`vp-engine`** | Orchestration, scheduling, aggregation, exit-code policy |
| **`vp-cli`** | Presentation (human text, JSON), process arguments, wiring validators into the engine |

Validators are **plugins to the engine**, not coordinators of each other. The platform handles everything except rule logic.

---

## Execution pipeline

Every validation run follows this pipeline:

```
Specification Root
        ↓
    Discovery
        ↓
Validation Context
        ↓
Independent Validators
        ↓
    Diagnostics
        ↓
   Aggregation
        ↓
    Exit Code
```

### Specification Root

The **read-only root** of a `veritypay-spec` checkout (local path or pinned revision). All file access is relative to this root. Nothing in the pipeline writes back to the spec tree during validation.

### Discovery

Each validator discovers **its own** in-scope artifacts (registry YAML paths, Markdown globs, manifest files—per validator architecture doc). Discovery runs as part of validator execution under engine supervision, not as a separate global pass that mutates shared caches consumed ambiguously by other validators.

### Validation Context

An **immutable snapshot of run configuration** constructed once per invocation and shared with every validator in the run:

- Spec root location
- Scope (which validators are enabled)
- Optional Edition pin
- Output mode hint (for validators that might adjust advisory verbosity—not for formatting)
- Strictness policy reference

Validators **read** context; they do not modify it or attach mutable run state visible to peers.

### Independent Validators

The engine invokes each registered validator **without** passing results from other validators. Each validator:

1. Receives immutable context
2. Performs discovery + validation internally
3. Returns a diagnostic list

Order of invocation is **engine-defined for reporting stability** but must **not** affect correctness (see principles below).

### Diagnostics

Structured findings only—severity, rule ID, location, message, suggestion—using the shared model in `vp-diagnostics`. Validators emit diagnostics; they do not print to stdout, format JSON, or assign process exit codes.

### Aggregation

The engine merges diagnostic lists, applies de-duplication policy if configured, computes summary counts, and determines pass/fail per severity policy. Aggregation **does not re-run rules** or reinterpret validator-specific payloads.

### Exit Code

The engine maps aggregated outcome to a process exit code. **`vp-cli`** surfaces that code to the shell. Validators never set exit codes directly.

---

## Principles

| Principle | Meaning |
|-----------|---------|
| **Validators are independent** | No validator requires another validator's output to produce correct results |
| **Validators are read-only** | Spec tree and Edition inputs are not modified during validation |
| **Validators never modify specification files** | No auto-fix, no silent rewrite, no "helpful" patches |
| **Validators never invoke each other** | All coordination flows through `vp-engine` |
| **Validators never depend on execution order** | Same inputs → same diagnostic set regardless of invocation order |
| **Validators receive immutable context** | Shared configuration is frozen for the duration of the run |
| **Validators emit diagnostics only** | No side channels (logging to stdout as primary output, writing reports to disk unless explicitly documented as a generator milestone) |
| **The engine owns orchestration** | Scheduling, validator list, lifecycle |
| **The engine owns aggregation** | Merge, summarize, exit-code mapping |
| **The CLI owns presentation** | Formatting, `--help`, human vs machine output |

---

## Role boundaries (ADR-0002 alignment)

```
┌──────────────────────────────────────────────────────────┐
│  vp-cli          presentation + validator registration   │
└────────────────────────────┬─────────────────────────────┘
                             │ invokes
┌────────────────────────────▼─────────────────────────────┐
│  vp-engine       orchestration + aggregation + exit code │
└────────────────────────────┬─────────────────────────────┘
                             │ invokes (interface only)
        ┌────────────────────┼────────────────────┐
        ▼                    ▼                    ▼
   vp-registry          vp-crossref (future)   vp-edition (future)
        │                    │                    │
        └────────────────────┴────────────────────┘
                             │ emit
                             ▼
                      vp-diagnostics
```

Validators implement validation logic only. They do not import `vp-engine` or `vp-cli`.

---

## Why this architecture

### Deterministic execution

Given the same spec root, context, validator set, and tool version, the **set of diagnostics** must be identical regardless of environment. Independence from execution order and absence of shared mutable state make runs **reproducible**—required for CI merge gates and grant audit evidence.

### Future parallel execution

Because validators do not call each other and do not rely on order, the engine may invoke them **concurrently** in a later release without changing validator implementations. Parallelism is an **engine optimization**, not a validator concern.

### Isolated testing

Each validator tests against fixture spec trees and immutable context fixtures **without** spinning up the full engine or other validators. Rule tests assert diagnostics; engine tests assert aggregation—separate failure domains.

### Plugin-like validator expansion

Milestone C adds `vp-crossref` by implementing the same execution contract and registering with `vp-cli`. No engine fork, no validator chain edits. Expansion resembles **plugins**, not **pipeline stages**.

### Stable CI behavior

CI invokes one entrypoint; the engine runs a declared validator set; exit code reflects aggregated policy. Local and CI paths differ only in presentation flags—not in which rules fire or how findings merge.

---

## Forbidden patterns

The following are **architectural violations**, not style preferences:

| Forbidden | Why |
|-----------|-----|
| **Validator calling another validator** | Hides coupling; breaks independence and test isolation |
| **Shared mutable state between validators** | Introduces order dependence and race hazards under parallel execution |
| **Hidden global caches** (cross-run or cross-validator) | Stale results; non-reproducible CI; undebuggable failures |
| **Validator-specific output formatting** | Splits presentation from rules; breaks JSON contract owned by CLI |
| **Business logic inside CLI** | Rules drift out of testable validator crates; violates layer ownership |
| **Validators setting process exit codes** | Exit policy must be centralized in engine |
| **Validators reading peer diagnostic buffers** | Creates execution-order dependence |
| **Engine re-running or rewriting rule logic** | Aggregator merges; it does not validate |
| **Writing into spec root during validation** | Breaks read-only trust model |

Convenience is not an exception. If two validators need the same file parse, **duplicate read-only parsing** or extract a **pure, side-effect-free shared function** in a documented crate—not a mutable cache with validator-specific keys.

---

## Future considerations

### Parallel execution

The engine **may** run validators in parallel when safe. Requirements:

- Validator behavior remains **deterministic** (same diagnostic set)
- Aggregator applies **stable sort** for report output (e.g. by file, line, rule ID) so CI diffs do not flap
- Parallelism is invisible to validator authors

Sequential execution is valid for Milestone B; parallel execution is an **optional engine enhancement**, not a validator API change.

### Incremental validation

Future optimization may skip validators based on changed paths. Such skipping is **engine policy** with explicit scope rules—validators remain stateless with respect to prior runs unless a future ADR defines a audited cache contract.

### Streaming diagnostics

Future CLI may stream findings as validators complete. Streaming affects **presentation timing**, not validator contracts—they still return complete diagnostic lists to the engine.

### Generator milestones

`vp-docs` (Milestone F) may write **non-normative generated output** to designated output paths—not into the spec tree. That behavior is scoped by ADR when implemented; it is not general validation execution.

---

## Consequences

### Positive

- Validator authors focus on **rules only**
- Engine and CLI evolve without forking registry logic
- CI and local runs stay aligned
- Parallel and incremental execution remain available without validator rewrites
- Matches [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) composition model

### Negative

- Some duplicated read-only I/O across validators until shared pure helpers are justified
- Stricter discipline than ad hoc script pipelines
- Validator authors cannot "quickly call" another validator for a shared check—they duplicate or extract pure functions

**Acceptable** because specification tooling optimizes for **audit trust**, not minimal LOC.

---

## Future reconsideration

Revisit this ADR if:

- A requirement **cannot** be met without controlled validator-to-validator communication (must be justified with new ADR)
- Parallel execution exposes determinism failures requiring contract changes
- A superseding ADR records new evidence

Changing execution boundaries requires **ADR-0004** (or successor)—not undocumented engine shortcuts.

---

## Related decisions

| Document | Relationship |
|----------|--------------|
| [ADR-0002](0002-workspace-architecture.md) | Crate roles map to execution layers |
| [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) | Conceptual lifecycle this ADR operationalizes |
| [REGISTRY_VALIDATION.md](../REGISTRY_VALIDATION.md) | First validator under this model |
| [CLI_PHILOSOPHY.md](../../CLI_PHILOSOPHY.md) | CLI presentation ownership |

---

## Follow-up

- [ ] Implement engine orchestration per this model (Milestone B)
- [ ] Registry validator tests use immutable context fixtures only
- [ ] Document stable diagnostic sort policy when aggregation is implemented

---

## Conclusion

**Validator authors implement validation logic.** Discovery within scope, rule evaluation, diagnostic emission—nothing else.

The **platform** handles specification root resolution, context construction, orchestration, aggregation, exit codes, and presentation. That separation keeps the Verity Specification Platform honest: rules are testable, runs are reproducible, and CI means the same thing everywhere.

---

*Accepted ADRs are historical records. Supersede with a new ADR; do not silently rewrite this decision.*
