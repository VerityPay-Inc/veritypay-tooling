---
id: ADR-0006
title: Specification Model as Shared Validator Input
status: accepted
version: 1.0.0
authors:
  - VerityPay Core Team
reviewers: []
related_docs:
  - docs/adrs/0003-validator-execution-model.md
  - docs/adrs/0005-specification-model.md
  - docs/SPECIFICATION_MODEL.md
  - ROADMAP.md
decision_date: 2026-06-30
superseded_by: null
---

# ADR-0006 — Specification Model as Shared Validator Input

**Status:** Accepted · **Version:** 1.0.0 · **Date:** 2026-06-30

**Related:** [ADR-0003](0003-validator-execution-model.md) · [ADR-0005](0005-specification-model.md) · [SPECIFICATION_MODEL.md](../SPECIFICATION_MODEL.md) · [ROADMAP.md](../../ROADMAP.md)

---

## Purpose

Record that existing validators now consume **`vp-spec-model`** as their shared typed input layer.

---

## Context

[ADR-0005](0005-specification-model.md) introduced **`vp-spec-model`** and described an incremental migration path. That migration is **complete** for the current validator set:

| Validator | Model consumption |
|-----------|-------------------|
| **`vp-registry`** | Typed `RegistrySet` loading (with hybrid raw fallback for malformed registries) |
| **`vp-edition`** | `RegistrySet` for accepted RFC and registry snapshot lookup |
| **`vp-crossref`** | `RegistrySet` and `DocumentCorpus` for registry resolution, document scan, and anchor lookup |

The model now loads:

- VP-TERM and VP-RFC **registries** (`RegistrySet`)
- Markdown **document corpus** (`DocumentCorpus`, `SpecificationDocument`, `DocumentSection`, `DocumentFrontMatter`)

Validators **still own diagnostics and rule decisions**. The model does not emit `vp-*` rule IDs, assign severity, or define protocol semantics ([ADR-0003](0003-validator-execution-model.md)).

---

## Decision

Treat **`vp-spec-model`** as the **shared representation layer** for validators going forward.

New validators and new validator features that need specification corpus data should load through the model first—not by re-implementing registry or document parsing inline.

---

## Rules

| Rule | Detail |
|------|--------|
| **New validators consume the model** | When a validator needs registry or document corpus data, use `SpecificationBuilder` and typed model structures |
| **Hybrid raw access is allowed** | Validators may still use raw `SpecRepository` access when malformed input requires actionable diagnostics the typed loader cannot preserve |
| **Model stays validation-neutral** | No rule IDs, severity policy, or diagnostic aggregation in `vp-spec-model` |
| **Diagnostics stay in validators** | Rule IDs, severity, and message text remain in validator crates (`vp-registry`, `vp-crossref`, `vp-edition`, …) |
| **Model represents; spec defines** | The model reflects `veritypay-spec` artifacts; it does not invent normative schema or protocol meaning |

Guardrail (unchanged from [ADR-0005](0005-specification-model.md)):

> **The model must not become a new source of protocol truth.**

---

## Consequences

### Positive

- **Less duplicate parsing** — registries and documents loaded once per session where the model is used
- **Consistent loading** — shared discovery scope, exclusions, and typed structures across validators
- **Easier docs generation** — future tooling can render from the same `Specification` snapshot
- **Easier reference interpreter bootstrap** — future `veritypay-reference` can consume stable types instead of re-parsing

### Negative

- **Model API stability now matters** — breaking changes to `vp-spec-model` affect multiple validators
- **Hybrid paths remain** — malformed fixtures and edge cases may still require raw YAML or text access for rich diagnostics
- **Contributor boundary** — contributors must understand **model vs validator** responsibilities ([SPECIFICATION_MODEL.md](../SPECIFICATION_MODEL.md))

**Acceptable** because shared representation reduces drift while preserving validator independence and diagnostic ownership.

---

## Future

| Topic | Direction |
|-------|-----------|
| **`ReferenceGraph`** | Add to the model in a later milestone; cross-reference validation may consume it without rebuilding discovery inline |
| **`EditionManifest`** | Typed manifest model may be added later; Edition validation may consume it while keeping manifest rule ownership in `vp-edition` |
| **Repository extraction** | Extract `vp-spec-model` to its own repository **only when** multiple repos need it as a stable, versioned dependency—not preemptively |

Revisit extraction and public API semver when `veritypay-reference`, `veritypay-conformance`, or sibling repos depend on the same types outside this workspace.

---

## Related decisions

| Document | Relationship |
|----------|--------------|
| [ADR-0003](0003-validator-execution-model.md) | Validators own rules; model is read-only input |
| [ADR-0005](0005-specification-model.md) | Introduced `vp-spec-model`; this ADR records migration completion |
| [SPECIFICATION_MODEL.md](../SPECIFICATION_MODEL.md) | Architecture and migration status |
| [REGISTRY_VALIDATION.md](../REGISTRY_VALIDATION.md) | Registry validator rules |
| [CROSS_REFERENCE_VALIDATION.md](../CROSS_REFERENCE_VALIDATION.md) | Cross-reference validator rules |
| [EDITION_VALIDATION.md](../EDITION_VALIDATION.md) | Edition validator rules |

---

## Conclusion

**Validators judge coherence. The Specification Model holds structure.**

Existing validators now share **`vp-spec-model`** for registry and document corpus representation. Rule ownership, diagnostics, and CLI contracts remain in validator crates. Future model growth (`ReferenceGraph`, `EditionManifest`) extends representation—not validation policy.

---

*Accepted ADRs are historical records. Supersede with a new ADR; do not silently rewrite this decision.*
