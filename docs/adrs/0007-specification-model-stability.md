---
id: ADR-0007
title: Specification Model Stability
status: accepted
version: 1.0.0
authors:
  - VerityPay Core Team
reviewers: []
related_docs:
  - docs/adrs/0005-specification-model.md
  - docs/adrs/0006-spec-model-migration-complete.md
  - docs/SPECIFICATION_MODEL.md
  - ROADMAP.md
decision_date: 2026-06-30
superseded_by: null
---

# ADR-0007 — Specification Model Stability

**Status:** Accepted · **Version:** 1.0.0 · **Date:** 2026-06-30

**Related:** [ADR-0005](0005-specification-model.md) · [ADR-0006](0006-spec-model-migration-complete.md) · [SPECIFICATION_MODEL.md](../SPECIFICATION_MODEL.md) · [ROADMAP.md](../../ROADMAP.md)

---

## Purpose

Record that **`vp-spec-model`** is considered the **stable internal representation layer** for `veritypay-tooling` v1 readiness.

---

## Context

[ADR-0005](0005-specification-model.md) introduced the specification model. [ADR-0006](0006-spec-model-migration-complete.md) recorded that existing validators consume it as shared input.

The model now provides:

| Structure | Role |
|-----------|------|
| **`RegistrySet`** | Typed VP-TERM and VP-RFC registries |
| **`DocumentCorpus`** | Markdown documents with front matter and section anchors |
| **`ReferenceGraph`** | Symbolic references as typed nodes and edges |

All current validators (`vp-registry`, `vp-crossref`, `vp-edition`) load through **`SpecificationBuilder`**. The tooling readiness gate passes against `veritypay-spec`. Downstream work (`veritypay-reference`, docs generation) can build on these types instead of re-parsing the corpus.

---

## Decision

Treat **`vp-spec-model`** as **feature-complete for current tooling needs** and **stable for v1**.

| Rule | Detail |
|------|--------|
| **Feature-complete for v1** | `RegistrySet`, `DocumentCorpus`, and `ReferenceGraph` satisfy current validator requirements |
| **New validators consume the model first** | Load via `SpecificationBuilder`; do not duplicate registry or document parsing inline |
| **New structures require justification** | Add model types only with evidence of multiple consumers or strong future need |
| **Model stays validation-neutral** | No rule IDs, severity policy, or diagnostic aggregation in `vp-spec-model` |
| **Validators own rule decisions and diagnostics** | Rule IDs, messages, and severity remain in validator crates |
| **Specification remains source of truth** | The model reflects `veritypay-spec` artifacts; it does not define protocol meaning |

Guardrail (unchanged):

> **The model must not become a new source of protocol truth.**

---

## Consequences

### Positive

- **Stable base for `veritypay-reference`** — downstream repos can depend on typed structures instead of re-parsing
- **Less duplicate parsing** — registries, documents, and references loaded once per session
- **Clearer validator boundaries** — representation in the model; coherence checks in validators
- **Readiness to move to the next repo** — tooling v1 gate passed; model API is the agreed integration surface

### Negative

- **Model API changes now require more care** — breaking changes affect multiple validators and future consumers
- **Future additions should be justified** — scope creep in the model is resisted unless multiple consumers need it
- **Hybrid raw paths remain** — malformed input may still require raw YAML or text access for actionable diagnostics

**Acceptable** because stability enables downstream work while preserving validator independence.

---

## Future

| Topic | Direction |
|-------|-----------|
| **`EditionManifest`** | Typed manifest model may be added later; Edition validation keeps rule ownership in `vp-edition` |
| **Docs generation** | May consume `Specification` snapshots without adding validation behavior to the model |
| **Repository extraction** | Extract `vp-spec-model` to its own repository **only when** multiple repos need it as a versioned dependency |

Revisit public API semver and extraction when sibling repos depend on the same types outside this workspace.

---

## Related decisions

| Document | Relationship |
|----------|--------------|
| [ADR-0005](0005-specification-model.md) | Introduced `vp-spec-model` |
| [ADR-0006](0006-spec-model-migration-complete.md) | Validator migration to shared model input |
| [SPECIFICATION_MODEL.md](../SPECIFICATION_MODEL.md) | Architecture and migration status |
| [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) | Validators own rules; model is read-only input |

---

## Conclusion

**The specification model is stable for tooling v1.**

`vp-spec-model` holds structure—`RegistrySet`, `DocumentCorpus`, and `ReferenceGraph`—for all current validators. Validators judge coherence. The specification defines meaning.

---

*Accepted ADRs are historical records. Supersede with a new ADR; do not silently rewrite this decision.*
