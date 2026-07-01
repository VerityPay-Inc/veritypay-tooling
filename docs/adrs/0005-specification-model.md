---
id: ADR-0005
title: Specification Model Layer
status: accepted
version: 1.0.0
authors:
  - VerityPay Core Team
reviewers: []
related_docs:
  - docs/adrs/0002-workspace-architecture.md
  - docs/adrs/0003-validator-execution-model.md
  - docs/SPECIFICATION_MODEL.md
  - ROADMAP.md
decision_date: 2026-06-30
superseded_by: null
---

# ADR-0005 — Specification Model Layer

**Status:** Accepted · **Version:** 1.0.0 · **Date:** 2026-06-30

**Related:** [ADR-0002](0002-workspace-architecture.md) · [ADR-0003](0003-validator-execution-model.md) · [SPECIFICATION_MODEL.md](../SPECIFICATION_MODEL.md) · [ROADMAP.md](../../ROADMAP.md)

---

## Context

`veritypay-tooling` now has:

- [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) — validation lifecycle and composition
- [ADR-0003](0003-validator-execution-model.md) — engine-orchestrated, independent validators
- Registry validation (`vp-registry`) — VP-TERM and VP-RFC structural checks
- Cross-reference validation (`vp-crossref`) — corpus reference integrity
- Edition validation (`vp-edition`) — Edition Manifest structure and coherence
- Configuration ([ADR-0004](0004-configuration-model.md), [CONFIGURATION_ARCHITECTURE.md](../CONFIGURATION_ARCHITECTURE.md)) — `.vp.toml` and `ValidationConfig`
- `SpecRepository` in `vp-core` — read-only file access under a spec root

Validators currently **parse raw registries and documents directly** inside each crate. That approach was correct for early milestones: it kept validators independent, shipped rules quickly, and avoided premature abstraction.

As the platform grows, direct parsing in every validator will **duplicate logic** and make future reference interpretation, conformance tooling, and documentation generation harder. Multiple crates already load `spec/rfcs/registry.yaml` for lookup. Edition validation parses manifest YAML and document front matter inline. Cross-reference validation rebuilds registry indexes and corpus scans on its own.

[SPECIFICATION_MODEL.md](../SPECIFICATION_MODEL.md) describes the **architecture** for a shared typed layer. **This ADR records the engineering decision** to introduce that layer as workspace crate **`vp-spec-model`**—not protocol semantics, validation rules, or reference execution.

---

## Decision

Introduce a new workspace crate:

**`vp-spec-model`**

Its responsibility is to **build and represent** the `veritypay-spec` corpus as **typed structures** via a **`SpecificationBuilder`** (discover, parse, normalize, connect—not merely read files). The model answers what was built and from where; validators answer whether it is valid.

### Initial scope

| In scope (first implementation) | Deferred |
|-----------------------------------|----------|
| VP-TERM registry model (`TerminologyRegistry`) | Full document corpus load |
| VP-RFC registry model (`RfcRegistry`) | `ReferenceGraph` construction |
| Basic `DocumentFrontMatter` parsing | Edition Manifest model (optional follow-up milestone) |
| `SpecificationBuilder` reading via `SpecRepository` | Validation rules or diagnostics |
| Reference resolution (`Reference` → `ResolvedReference`) | First milestone; deferred to future milestones |
| Stable data structures without rule behavior | Protocol execution |

| Explicitly out of scope | |
|-------------------------|---|
| Protocol execution | Model does not evaluate claims or behavior |
| Validation rules | Rule IDs and severity remain in validator crates |
| Publishing Editions | Governance act; tooling validates only |
| Normative schema invention | Shapes follow `veritypay-spec` and tooling ADRs |

### Dependency expectations

| Crate / layer | Rule |
|---------------|------|
| **`vp-spec-model`** | May depend on **`vp-core`** (`SpecRepository`) and **`vp-diagnostics`** only if needed for source `Location` types |
| **Validator crates** (`vp-registry`, `vp-crossref`, `vp-edition`, …) | May depend on **`vp-spec-model`** when migrated |
| **`vp-spec-model`** | **Must not** depend on validator crates |
| **`vp-engine`** | **Must not** depend on model-specific validator internals; orchestrates validators via `vp-core` traits only |
| **`vp-cli`** | Wires validators at the binary boundary; does not embed model loading policy inside `vp-engine` |
| **Workspace** | **No dependency cycles** involving `vp-spec-model` |

Migration is **incremental**. Existing validators continue direct parsing until explicitly moved to the model. First milestone success requires **no CLI behavior change** and **all existing validator tests passing** ([SPECIFICATION_MODEL.md](../SPECIFICATION_MODEL.md)).

### Guardrail

> **The model must not become a new source of protocol truth.**
>
> The specification remains authoritative. The model **represents** the spec; it does **not** define it.

If the model and `veritypay-spec` disagree on shape, **the specification wins**—the model is updated, not the protocol.

---

## Alternatives considered

### 1. Keep direct parsing inside each validator

**Description:** Continue the current pattern: each validator reads YAML and Markdown via `SpecRepository` and parses locally.

**Why rejected as the long-term approach:** Acceptable today; insufficient as Edition tooling, docs generation, and reference interpretation arrive. Duplicated registry parsing and ad hoc front matter handling increase drift risk and maintenance cost. Validators should share one typed representation while keeping rule ownership ([ADR-0003](0003-validator-execution-model.md)).

**Assessment:** Remains the **short-term** state until migration; not the platform direction.

---

### 2. Extract model into a separate repository immediately

**Description:** Publish `vp-spec-model` as its own repo from day one for `veritypay-reference` and `veritypay-conformance` to depend on.

**Why deferred:** No second consumer repo requires a published crate yet. Premature extraction adds release coordination, versioning, and CI overhead before the API stabilizes. The model should prove itself inside `veritypay-tooling` first.

**Assessment:** Reconsider when multiple repositories need a **stable, versioned** dependency on the same types ([Future reconsideration](#future-reconsideration)).

---

### 3. Make the reference interpreter own the model

**Description:** Defer a shared model until `veritypay-reference` exists; let the interpreter define and own specification types.

**Why rejected:** `veritypay-tooling` already loads registries, manifests, and corpus artifacts for validation. Waiting for the interpreter inverts dependency: validators and CI need typed spec data **before** executable semantics ship. The interpreter should **consume** a shared model, not be the only owner of spec structure parsing.

**Assessment:** Reference interpreter is a **future consumer**, not the model's home.

---

### 4. Generate model from Markdown directly

**Description:** Treat Markdown as the sole source of truth; generate registries and manifests from prose rather than loading YAML artifacts.

**Why rejected:** `veritypay-spec` already treats machine-readable registries and Edition Manifests as authoritative indexes alongside prose ([REGISTRY_VALIDATION.md](../REGISTRY_VALIDATION.md), [EDITION_VALIDATION.md](../EDITION_VALIDATION.md)). Generation may supplement publication workflows later; it does not replace loading the artifacts validators already check.

**Assessment:** Docs **generation** may be a future milestone; **validation and tooling** load existing YAML and Markdown as-is.

---

## Consequences

### Positive

- **Less parsing duplication** — registries and front matter parsed once per session
- **Shared typed representation** — one vocabulary for validators and future tools
- **Easier Edition tooling** — `EditionManifest` and pins compose with `RegistrySet` and documents
- **Future docs generation** — derived views from the same `Specification` snapshot
- **Future reference interpreter** — can reuse types without re-parsing `veritypay-spec`
- **Validators become simpler over time** — rule logic separated from load/parse mechanics

### Negative

- **One more crate** — workspace surface and contributor mental model grow
- **Migration work** — validators move incrementally; dual paths exist during transition
- **Risk of model becoming too broad** — temptation to embed rules, execution, or normative policy in the model layer

**Acceptable** because the platform optimizes for **shared understanding of spec structure** across validation, publication support, and future execution tooling—without collapsing validation into a monolith.

---

## Future reconsideration

Revisit this ADR if:

- **Multiple repositories** (`veritypay-reference`, `veritypay-conformance`, product SDKs) need `vp-spec-model` as a **stable external dependency** → consider extracting to its own repository with semver releases
- The **model scope** expands into validation rules or protocol execution → violates this ADR's guardrail; split or supersede
- **Generated artifacts** replace raw YAML as the primary build target → new builder strategy ADR
- A **successor ADR** supersedes crate placement or dependency rules with new evidence

Extract `vp-spec-model` into its own repository **only if** multiple repos need it as a stable dependency—not preemptively.

---

## Related decisions

| Document | Relationship |
|----------|--------------|
| [ADR-0002](0002-workspace-architecture.md) | Crate placement; no validator cycles |
| [ADR-0003](0003-validator-execution-model.md) | Validators own rules; model is input only |
| [SPECIFICATION_MODEL.md](../SPECIFICATION_MODEL.md) | Architecture this ADR operationalizes |
| [REGISTRY_VALIDATION.md](../REGISTRY_VALIDATION.md) | First domain types (`RegistrySet`) |
| [EDITION_VALIDATION.md](../EDITION_VALIDATION.md) | Future `EditionManifest` consumer |
| [CONFIGURATION_ARCHITECTURE.md](../CONFIGURATION_ARCHITECTURE.md) | Session config; model does not parse `.vp.toml` |

---

## Follow-up

- [ ] Add `vp-spec-model` crate to workspace (first milestone per [SPECIFICATION_MODEL.md](../SPECIFICATION_MODEL.md))
- [ ] Implement `SpecificationBuilder` for VP-TERM and VP-RFC registries
- [ ] Implement basic `DocumentFrontMatter` parsing
- [ ] Unit tests for build-only behavior; no validation rules in crate
- [ ] Confirm existing validators and CLI unchanged before any migration PR

---

## Conclusion

**Validators judge coherence. The Specification Model holds structure.**

`vp-spec-model` builds and represents `veritypay-spec` as typed data—without rules, without execution, without replacing normative authority. A `SpecificationBuilder` assembles the immutable `Specification` snapshot; validators may adopt it incrementally; the engine stays orchestration-only; the specification stays upstream.

---

*Accepted ADRs are historical records. Supersede with a new ADR; do not silently rewrite this decision.*
