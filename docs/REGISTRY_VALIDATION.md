# Registry Validation

**Architecture for the first validator: machine-readable registry integrity.**

This document defines what **registry validation** will do when implemented in Milestone B. It does not specify code, algorithms, parsing libraries, or performance characteristics.

For the shared validator lifecycle and diagnostic model, see [VALIDATION_ENGINE.md](VALIDATION_ENGINE.md).

---

## Purpose

Registry validation **protects machine-readable registries from structural drift**.

Registries in `veritypay-spec` are authoritative indexes—VP-TERM for terminology, VP-RFC for accepted change proposals. Authors edit YAML alongside prose. Without automated checks, drift appears as:

- Duplicate or orphan IDs
- Invalid status values
- Broken cross-references between registry entries
- Fields that contradict VP-RFC-0000 or documented schema

Registry validation fails CI on **structural defects** before they propagate to cross-reference checks, Edition manifests, or implementers citing invalid IDs.

**Boundary:** Registry validation checks **registry file structure and internal consistency**. It does **not** define term meanings, accept RFCs, or validate that prose correctly uses a term.

---

## Position in the validation engine

```
Input (spec root)
       ↓
Discovery → spec/terminology/registry.yaml
           → spec/rfcs/registry.yaml
           → (future registry paths)
       ↓
Registry Validator
       ↓
Diagnostics (category: Registry)
       ↓
Report Aggregator
```

The registry validator is **independent**: it runs without cross-reference or edition validators. Other validators may **consume** registry contents as lookup tables later—that is read-only reuse, not coupling.

---

## Supported registries

### Initial (Milestone B)

| Registry | File (in `veritypay-spec`) | ID scheme | Human-readable companion |
|----------|----------------------------|-----------|--------------------------|
| **VP-TERM** | `spec/terminology/registry.yaml` | `VP-TERM-NNNN` | `docs/00-overview/GLOSSARY.md` |
| **VP-RFC** | `spec/rfcs/registry.yaml` | `VP-RFC-NNNN` | `rfcs/` directory |

### Future (not Milestone B)

| Registry | Purpose | Gate |
|----------|---------|------|
| **VP-CS** | Conformance scenarios | Spec governance + Milestone C/D |
| **VP-ADR** | Engineering decision records | ADR guide + registry scaffold in spec |
| **VP-EDITION** | Published Edition manifests | Edition publication process |

Future registries **extend** this validator with documented schemas—they do not require a separate engine. Each addition updates this document and [ROADMAP.md](../ROADMAP.md).

---

## Validation responsibilities

Rules below are **architectural obligations** for Milestone B. Exact rule IDs and enum values follow schemas documented in `veritypay-spec` and tooling ADRs at implementation time.

### Registry file-level

| Check | Description |
|-------|-------------|
| **Parse validity** | YAML is well-formed and loadable |
| **Required top-level fields** | Registry header fields present (e.g. `spec`, `title`, `version`, `status`, collection key `terms` / `rfcs`) |
| **Collection non-empty** | At least one entry where policy requires (may warn during early draft phases—policy documented) |

### Entry identity

| Check | Description |
|-------|-------------|
| **Duplicate IDs** | No two entries share the same `VP-TERM-*` or `VP-RFC-*` id |
| **Duplicate titles** | No ambiguous duplicate titles within a registry (warning or error per policy) |
| **Duplicate anchors** | Slug/anchor uniqueness where `anchor` field exists |
| **ID format** | IDs match documented pattern (`VP-TERM-NNNN`, `VP-RFC-NNNN`) |

### Required fields (per entry)

| Check | Description |
|-------|-------------|
| **Missing required fields** | Each entry type declares required keys; absence is an error |
| **Unknown fields** | Unexpected keys flagged (warning or error—policy in ADR) |
| **Empty required strings** | Required text fields non-empty |

Fields required for VP-TERM and VP-RFC entries derive from current registry shape in `veritypay-spec` (e.g. `id`, `title`, `stability` / `status`, paths, version fields)—not invented by tooling alone.

### Status and lifecycle

| Check | Description |
|-------|-------------|
| **Unknown status** | Status values must be from documented enums (e.g. RFC: `draft`, `accepted`, `rejected`, `superseded`) |
| **Invalid status transitions** | Registry-only checks (e.g. `superseded` without `superseded_by`) where schema defines rules |
| **Version fields** | Document/version fields present where required |

### Semantic version

| Check | Description |
|-------|-------------|
| **Invalid semantic version** | `version` fields conform to agreed semver or documented spec version pattern |

Tooling validates **format**, not whether a version bump was **governance-correct**.

### Internal references (within registry)

| Check | Description |
|-------|-------------|
| **Broken dependency references** | `depends_on` entries reference existing IDs in same or declared registry |
| **Missing referenced terminology** | VP-RFC `related_terms` (or equivalent) reference existing VP-TERM IDs when present |
| **Missing referenced RFC** | `supersedes`, `superseded_by`, `depends_on` RFC IDs exist in VP-RFC registry |
| **Self-reference sanity** | Entry does not depend on itself; supersession chains acyclic where checkable at registry layer |

### Path and document pointers

| Check | Description |
|-------|-------------|
| **Missing path targets** | `path` fields point to files that exist under spec root |
| **RFC number consistency** | RFC numeric field aligns with `VP-RFC-NNNN` id and filename convention where documented |

### Section IDs (VP-TERM)

| Check | Description |
|-------|-------------|
| **Invalid section IDs** | `section_id` values match documented architecture ID patterns (e.g. `DM-4.8`, `VP-TERM-009`) where present |
| **Normative definition pointers** | Required `normative_definition` structure present for terms that declare it |

### Cross-registry consistency (registry layer only)

| Check | Description |
|-------|-------------|
| **VP-RFC process reference** | Registry header references valid process RFC where required (`process_rfc: VP-RFC-0000`) |
| **Human-readable companion path** | `human_readable` path exists |

Full **prose cross-reference** validation (IDs mentioned in Markdown but absent from registry) belongs to the **Cross Reference** category and Milestone C—not registry validation alone.

---

## Diagnostic philosophy

Registry diagnostics must help a contributor **fix the registry in one pass** without reading validator source.

Every diagnostic should explain:

| Question | Diagnostic field |
|----------|------------------|
| **What failed?** | Message + Rule ID |
| **Where?** | File + Location (YAML key path, entry `id`) |
| **Why?** | Message context (e.g. "status `active` is not in allowed set") |
| **How to fix?** | Suggestion |

### Example (illustrative shape)

```text
error[vp-term-duplicate-id]: duplicate terminology ID VP-TERM-012
  registry: spec/terminology/registry.yaml
  location: terms[34].id
  context: first defined at terms[12].id
  help: merge entries or assign a new unused VP-TERM-NNNN id
```

### Quality bar

| Good | Bad |
|------|-----|
| Points to entry ID and YAML path | "Invalid YAML" |
| Names allowed enum values | "Bad status" |
| Distinguishes registry file vs entry | Generic stack trace |
| Stable rule IDs for CI filtering | Changing message text every release |

Warnings vs errors follow [VALIDATION_ENGINE.md](VALIDATION_ENGINE.md) severity policy. Structural integrity defaults to **error**.

---

## Discovery scope

The registry validator discovers:

| Artifact | Action |
|----------|--------|
| `spec/terminology/registry.yaml` | Validate VP-TERM |
| `spec/rfcs/registry.yaml` | Validate VP-RFC |
| Referenced paths inside entries | Existence check relative to spec root |

It does **not** scan the full Markdown corpus in Milestone B.

---

## Outputs

| Output | Consumer |
|--------|----------|
| Diagnostic list | Report aggregator, CLI, CI JSON |
| Validator summary | `registry: 2 errors, 1 warning` |
| Non-zero exit (via engine) | CI merge gate |

No files written to `veritypay-spec`. No auto-commit of registry fixes.

---

## Success criteria (Milestone B)

Milestone B is **complete** when registry validation can **detect structural defects** in supported registries (VP-TERM, VP-RFC):

| # | Criterion |
|---|-----------|
| 1 | Intentionally broken fixture registries produce expected rule IDs and locations |
| 2 | Current `veritypay-spec` main registries pass with zero errors |
| 3 | Duplicate IDs, missing required fields, unknown status, and broken `depends_on` / path references are detected |
| 4 | Invalid semver-format version fields are detected per policy |
| 5 | Validator runs via engine entrypoint locally and in documented CI path |
| 6 | Diagnostics include what / where / why / how to fix for each rule class |

**Not included in Milestone B:** cross-reference scanning of Markdown, Edition manifests, CLI polish beyond minimal invoke, documentation generation.

See [ROADMAP.md](../ROADMAP.md) for milestone boundaries.

---

## Rule ID namespace (illustrative)

Stable rule IDs use prefixes for filtering and documentation:

| Prefix | Domain |
|--------|--------|
| `vp-term-*` | VP-TERM registry |
| `vp-rfc-*` | VP-RFC registry |
| `vp-registry-*` | Shared registry file checks |

Final rule catalog lives in implementation docs and ADR when code lands.

---

## Governance alignment

| Topic | Authority |
|-------|-----------|
| What fields are **required** | Documented in `veritypay-spec` registry README / glossary / VP-RFC-0000 |
| What values are **allowed** for status | Spec governance; tooling implements |
| Adding a **new registry** | Spec change + update this document |
| Changing **normative meaning** of a term | RFC in `veritypay-spec`—not validator PR |

When a registry rule should become normative policy, propose the change in **`veritypay-spec` first**, then implement here.

---

## Related documents

| Document | Relationship |
|----------|--------------|
| [VALIDATION_ENGINE.md](VALIDATION_ENGINE.md) | Shared lifecycle and diagnostics |
| [ARCHITECTURE.md](../ARCHITECTURE.md) | Registry Validation component |
| [ROADMAP.md](../ROADMAP.md) | Milestone B delivery |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | Specification boundary |
| [veritypay-spec — terminology registry](https://github.com/VerityPay-Inc/veritypay-spec/blob/main/spec/terminology/registry.yaml) | VP-TERM source |
| [veritypay-spec — RFC registry](https://github.com/VerityPay-Inc/veritypay-spec/blob/main/spec/rfcs/registry.yaml) | VP-RFC source |

---

*Registries are indexes. Broken indexes break everything downstream.*
