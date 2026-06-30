---
id: ADR-0001
title: Implementation Language for veritypay-tooling
status: accepted
version: 1.0.0
authors:
  - VerityPay Core Team
reviewers: []
related_docs:
  - docs/VALIDATION_ENGINE.md
  - docs/REGISTRY_VALIDATION.md
  - ROADMAP.md
  - ARCHITECTURE.md
decision_date: 2026-06-29
superseded_by: null
---

# ADR-0001 — Implementation Language for veritypay-tooling

**Status:** accepted · **Version:** 1.0.0 · **Date:** 2026-06-29

**Related:** [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) · [REGISTRY_VALIDATION.md](../REGISTRY_VALIDATION.md) · [ROADMAP.md](../../ROADMAP.md) · [veritypay-spec — ADR Guide](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/ADR_GUIDE.md)

---

## Context

Milestone A established `veritypay-tooling` as a **documentation-only scaffold**: purpose, architecture, roadmap, CLI philosophy, and validation engine design. Milestone B will implement the **first validators** (VP-TERM and VP-RFC registry validation) on top of [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md).

Before implementation begins, the project must choose an **implementation language**. This is an engineering decision—not a protocol decision. It does not bind independent VerityPay implementers. It does bind how this repository evolves for years.

The choice affects:

| Area | Impact |
|------|--------|
| **CLI architecture** | How `vp` is structured, distributed, and invoked in CI |
| **Library ecosystem** | Crates, modules, and shared types consumed by future validators |
| **Contributor experience** | Onboarding time, review standards, and who can merge |
| **Packaging** | How releases are built, versioned, and installed |
| **CI** | Build times, reproducibility, and runner requirements |
| **Long-term maintainability** | Refactoring safety as validators and rules grow |
| **Future validator ecosystem** | Cross-reference, Edition, documentation generators sharing one core |

`veritypay-tooling` validates **specification structure** for an institution that treats correctness, explicit boundaries, and durable infrastructure as first-order values. The implementation language should reinforce—not undermine—that posture.

Validation domains involve structured YAML registries, Markdown corpora, stable diagnostic models, and composable validators. The language must model these domains **explicitly** and fail **predictably** when inputs are malformed.

---

## Decision

**`veritypay-tooling` will be implemented in Rust.**

The primary reason is **not** raw runtime performance. Validators are I/O-bound on file reads; microseconds of CPU are not the bottleneck.

The primary reasons are:

- **Correctness** — strong static guarantees for parsing, schema handling, and diagnostic construction
- **Maintainability** — explicit types as validators and rule IDs accumulate across milestones B–G
- **Reliability** — predictable behavior in CI and on contributor machines without hidden runtime drift
- **Explicit modeling** — validation domains expressed as types that document intent (registry entry, diagnostic, rule ID)
- **Cross-platform distribution** — single static binaries for macOS, Linux, and Windows without shipping a runtime
- **Long-term sustainability** — a codebase that remains legible as the Specification Platform grows

This choice aligns with institutional engineering values already established across Verity: infrastructure that outlives individuals, explicit behavior over implicit convention, replacement-friendly components, and seriousness appropriate to public specification work—not product sprint velocity.

---

## Alternatives considered

Each option was evaluated for **tooling fit**, not general language popularity.

### Rust

**Advantages**

- Single static binary—ideal for `vp` in CI and local dev without runtime installation
- Excellent CLI ecosystem (`clap` and related patterns)
- Strong typing for registry schemas, diagnostics, and validator composition
- Memory safety without a garbage collector—fewer whole classes of production bugs
- Reliable, structured error handling (`thiserror`, `anyhow`) matching [REGISTRY_VALIDATION.md](../REGISTRY_VALIDATION.md) diagnostic philosophy
- Mature package ecosystem (`cargo`, `serde`, `serde_yaml`)
- Good long-term maintainability as rule surface area grows
- Straightforward cross-platform release artifacts
- Suitable foundation for a **future tooling platform** (shared engine crate, plugin-style validators)

**Tradeoffs**

- Steeper learning curve for contributors unfamiliar with ownership and lifetimes
- Smaller contributor pool than JavaScript or Python
- Slower initial implementation velocity while patterns and workspace layout settle

**Assessment:** Best match for correctness, explicit modeling, and portable CLI distribution. Tradeoffs accepted.

---

### Go

**Advantages**

- Simple language with fast compilation
- Approachable for many backend engineers
- Good standard library for CLI and file I/O
- Single-binary deployment story

**Tradeoffs**

- Less expressive type system for modeling complex validation domains and diagnostic structures
- Error handling patterns less structured than Rust for rich, rule-ID-tagged diagnostics
- Weaker compile-time enforcement as validator count and registry schemas grow

**Assessment:** Viable for a thin CLI; weaker fit for a multi-milestone validation engine with composable rule types.

---

### TypeScript

**Advantages**

- Very large contributor ecosystem
- Rich Markdown and YAML libraries
- Rapid iteration during early prototyping

**Tradeoffs**

- Requires Node.js (or bundled runtime) at execution time—CI and contributor environment coupling
- Package management and dependency tree complexity at scale
- Weaker deployment story for a **production-quality, version-pinned CLI** expected in spec merge gates
- Tooling behavior may vary with Node version unless carefully pinned

**Assessment:** Strong for docs websites and scripts; weaker as the canonical validator runtime for institutional CI.

---

### Python

**Advantages**

- Enormous ecosystem and parsing libraries
- Rapid development for one-off scripts and experiments

**Tradeoffs**

- Packaging and dependency management friction for reproducible CI
- Slower execution (usually acceptable, but compounds in large corpora)
- Distribution complexity—harder to ship one portable, pinned CLI artifact
- Virtual environment and interpreter version drift across contributors

**Assessment:** Excellent for ad hoc automation; poor fit as the long-lived core of the Specification Platform validators.

---

## Rationale

Registry validation is the **foundation** of Phase II tooling. Errors caught here prevent false confidence in cross-reference checks, Edition manifests, and downstream reference/conformance work.

Rust provides:

1. **Compile-time structure** for diagnostics (severity, rule ID, location)—reducing silent format drift between CLI and CI JSON output
2. **A single artifact** maintainers can pin in GitHub Actions and document for grant auditors
3. **Explicit error paths** when YAML is malformed, enums are invalid, or references break—matching the "what / where / why / how to fix" diagnostic bar
4. **Workspace layout** (`cargo` workspace) that mirrors validator independence in [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md)

Go, TypeScript, and Python remain reasonable choices for **adjacent** repositories (documentation sites, one-off migration scripts). They are not rejected globally—only for **this** repository's core mission.

---

## Implementation guidance

The following are **recommendations**, not architectural requirements. Deviations are acceptable with documented rationale in a follow-up ADR or PR description.

| Area | Likely choice |
|------|----------------|
| Workspace | `cargo` workspace (engine crate + CLI binary + validator crates) |
| CLI | `clap` |
| Serialization | `serde`, `serde_yaml` |
| Markdown parsing | `pulldown-cmark` or equivalent |
| Error types | `thiserror` (library errors), `anyhow` (CLI boundary) |
| Observability | `tracing` |

Implementation should follow [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) and [REGISTRY_VALIDATION.md](../REGISTRY_VALIDATION.md)—not invent parallel validation architecture.

First code milestone: **Milestone B — registry validation** only.

---

## Consequences

### Positive

- **Robust tooling** — type-checked validators and diagnostics reduce entire classes of runtime surprises
- **Maintainable architecture** — explicit modules per validator; engine utilities shared without coupling rule logic
- **Reliable CI** — pinned binary or reproducible build; no interpreter version matrix
- **Portable binary** — contributors and forks run the same artifact locally and in Actions
- **Strong future ecosystem** — foundation for cross-reference, Edition, and documentation validators in one workspace

### Negative

- **Rust learning curve** — some contributors will need onboarding before first merged validator PR
- **Slower initial velocity** — workspace bootstrap, ADR patterns, and idiomatic error modeling take time upfront

**Why these tradeoffs are acceptable**

`veritypay-tooling` is **institutional infrastructure**, not a weekend script. A slower start that yields a stable validation engine is preferable to fast iteration that produces flaky CI and diagnostic drift. Contributor onboarding cost is paid once; specification integrity benefit compounds on every spec PR.

Documentation ([CONTRIBUTING.md](../../CONTRIBUTING.md), examples, good-first issues) will lower the Rust barrier for tooling contributors who do not need to author protocol RFCs.

---

## Future reconsideration

This decision should **only** be revisited if:

- Rust becomes a **blocker to ecosystem sustainability** (e.g. no maintainers able to steward the codebase over a documented period)
- A **future requirement** cannot reasonably be met in Rust without disproportionate cost
- An **accepted ADR supersedes** this one with new evidence

Changing the implementation language **requires a new ADR** (e.g. ADR-0002). Partial rewrites without ADR are not acceptable for the core engine.

Supersession does not retroactively invalidate Milestone B work—it documents a forward direction for new code.

---

## Related decisions

| Document | Relationship |
|----------|--------------|
| [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) | Validator lifecycle implementation target |
| [REGISTRY_VALIDATION.md](../REGISTRY_VALIDATION.md) | First validator scope (Milestone B) |
| [ROADMAP.md](../../ROADMAP.md) | Milestone B proceeds after this ADR |
| [CLI_PHILOSOPHY.md](../../CLI_PHILOSOPHY.md) | CLI behavior; Rust enables static `vp` binary |
| [veritypay-spec — ADR Guide](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/ADR_GUIDE.md) | ADR process for this repository |

---

## Follow-up

- [ ] Bootstrap `cargo` workspace (Milestone B implementation—separate PR)
- [ ] Document local build and CI invoke path in README when code lands
- [ ] Add Rust onboarding notes to CONTRIBUTING.md when workspace exists

---

## Conclusion

The Verity Specification Platform values **correctness over convenience**. Validators exist so specification defects fail loudly before they become protocol confusion.

Rust best supports that commitment: explicit models, reliable diagnostics, and portable tooling that can still be read and maintained when today's contributors have moved on.

---

*Accepted ADRs are historical records. Supersede with a new ADR; do not silently rewrite this decision.*
