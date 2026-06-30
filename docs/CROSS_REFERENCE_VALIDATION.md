# Cross-Reference Validation

**Architecture for Milestone C: reference integrity across the specification corpus.**

This document defines what **cross-reference validation** will do when implemented in Milestone C. It does not specify code, parsing libraries, scan algorithms, or performance characteristics.

For the shared validator lifecycle and diagnostic model, see [VALIDATION_ENGINE.md](VALIDATION_ENGINE.md). For registry structural checks (Milestone B), see [REGISTRY_VALIDATION.md](REGISTRY_VALIDATION.md).

---

## Purpose

Cross-reference validation **protects the specification corpus from broken traceability**.

Authors cite VP-TERM and VP-RFC identifiers, link between Markdown documents, and reference architecture section IDs in prose and YAML. Without automated checks, drift appears as:

- VP-TERM or VP-RFC IDs mentioned in documentation that do not exist in registries
- Relative links that point at removed or renamed files
- Anchor links that target headings that no longer exist
- Malformed identifier strings that look like references but violate documented patterns

Cross-reference validation fails CI on **reference integrity defects**—unknown IDs, broken internal links, and invalid reference formats—before they propagate to implementers, Edition bundles, or conformance work.

**Boundary:** Cross-reference validation checks that **references resolve** against known registries, files, and anchors. It does **not** judge whether prose correctly explains a term, whether an RFC should have been accepted, or whether external URLs remain live.

---

## Position in the validation engine

```
Input (spec root)
       ↓
Discovery → Markdown corpus (configured globs)
           → YAML prose fields (where in scope)
           → Registry lookup tables (read directly)
       ↓
Cross-Reference Validator
       ↓
Diagnostics (category: CrossReference)
       ↓
Report Aggregator
```

The cross-reference validator is **independent**: it runs without requiring registry validators to execute first. It may **read** the same registry YAML files that registry validation checks, building in-memory lookup tables for ID resolution. That is read-only reuse of spec artifacts—not coupling to another validator's execution or diagnostic output.

Per [ADR-0003](adrs/0003-validator-execution-model.md):

- Validators emit diagnostics only; the engine aggregates; the CLI presents.
- Engine invocation order must **not** affect correctness.
- Shared file access belongs in **`SpecRepository`** (`vp-core`); shared diagnostic types belong in **`vp-diagnostics`**.

---

## Supported reference types

Cross-reference validation recognizes reference shapes documented in `veritypay-spec` and tooling architecture. The table below lists types by delivery phase.

| Reference type | Pattern (illustrative) | Resolution target | Initial Milestone C |
|----------------|------------------------|-------------------|---------------------|
| **VP-TERM** | `VP-TERM-NNN` / `VP-TERM-NNNN` | `spec/terminology/registry.yaml` entry `id` | Yes |
| **VP-RFC** | `VP-RFC-NNNN` | `spec/rfcs/registry.yaml` entry `id` | Yes |
| **Architecture section ID** | `DM-1.1`, `IM-2.3`, `BM-4.2`, … | Document heading anchors or declared section map (policy TBD) | Partial — validate format where cited; full anchor map deferred where impractical |
| **Internal Markdown link** | `[text](relative/path.md)` | File under spec root | Yes |
| **Internal anchor link** | `[text](path.md#anchor)` | Heading slug in target file | Yes, when practical |
| **VP-CS** | `VP-CS-NNNN` | Future conformance scenario registry | Planned — not initial C |
| **Registry YAML cross-refs** | `depends_on`, `related_terms`, etc. | Same-registry or declared registry IDs | No — owned by [Registry Validation](REGISTRY_VALIDATION.md) |

Architecture section ID prefixes follow terminology registry conventions: `DM`, `IM`, `BM`, `DAT`, `SM`, `CM`, `GV`, `VI`, `GL` (and extensions documented in spec before tooling adopts them).

---

## Discovery scope

The cross-reference validator discovers artifacts **within its documented scope**. Discovery is explicit and deterministic from spec root.

### In scope (initial Milestone C)

| Artifact | Action |
|----------|--------|
| Markdown under `docs/` | Scan prose and link targets for VP-TERM, VP-RFC, relative links, anchors |
| Markdown under `rfcs/` | Same as above |
| Other Markdown paths agreed in milestone implementation ADR | Extend globs with documented exclusions |
| `spec/terminology/registry.yaml` | Load as VP-TERM lookup table (read-only) |
| `spec/rfcs/registry.yaml` | Load as VP-RFC lookup table (read-only) |
| Linked Markdown files | Resolve relative paths from link source location |

Exact glob patterns, exclusions (e.g. `_drafts/`, generated artifacts), and whether root-level Markdown is included are **implementation decisions** recorded in a Milestone C ADR—not invented silently at code time.

### Out of scope (initial Milestone C)

| Artifact | Reason |
|----------|--------|
| External URLs (`https://…`) | Not required merge gate; optional advisory later |
| Edition Manifest YAML | Milestone D |
| VP-CS registry and scenario files | Future registry + Milestone F conformance runner |
| Generated documentation output | Milestone F; not part of normative spec tree |
| Full YAML corpus beyond prose fields | Registry-internal refs stay in registry validator |
| Semantic review of definitions | Human governance, not tooling |

---

## Validation responsibilities

Rules below are **architectural obligations** for initial Milestone C. Exact matching rules (code spans, backtick boundaries, table cells) follow spec conventions and implementation ADRs at code time.

### Identifier resolution (Markdown prose)

| Check | Description |
|-------|-------------|
| **Unknown VP-TERM** | Every `VP-TERM-*` token in scanned Markdown resolves to an entry in the terminology registry |
| **Unknown VP-RFC** | Every `VP-RFC-*` token in scanned Markdown resolves to an entry in the RFC registry |
| **Invalid reference format** | Tokens resembling VP-TERM / VP-RFC but violating documented ID patterns are flagged |
| **Deprecated / reserved advisory** | Optional warning when prose cites `deprecated` or `reserved` terms (policy documented; may be info severity) |

Registry lookup uses **current registry contents at spec root**. The cross-reference validator does **not** require registry validation to have run first; it loads and parses registry YAML independently via `SpecRepository`. If registry YAML is missing or unparseable, the validator emits diagnostics for lookup failure—it does not delegate to or invoke the registry validator.

### Internal Markdown links

| Check | Description |
|-------|-------------|
| **Broken relative link** | Relative link targets resolve to an existing file under spec root |
| **Broken anchor** | Fragment (`#heading-slug`) matches a heading in the target file when anchor extraction is practical |
| **Ambiguous or escaping edge cases** | Documented policy for `<`, autolinks, and reference-style links |

Absolute paths outside spec root and `mailto:` / `http(s):` links are **ignored** in initial Milestone C (not errors).

### Architecture section IDs (initial pass)

| Check | Description |
|-------|-------------|
| **Invalid section ID format** | Cited section IDs use recognized architecture prefixes where machine-detectable |
| **Unknown section anchor** | When a section ID maps to a known document + anchor convention, verify target exists |

Full semantic validation of section-to-prose alignment remains out of scope.

### What cross-reference validation does not do

| Excluded | Owner |
|----------|-------|
| Duplicate IDs inside registries | Registry validator |
| Invalid registry entry fields | Registry validator |
| Front matter schema | Front matter validator (Milestone C expansion per [ROADMAP.md](../ROADMAP.md)) |
| Edition pin existence | Edition validator (Milestone D) |
| VP-CS scenario registry integrity | Future registry + conformance milestones |

---

## Relationship to registry validation

Registry validation and cross-reference validation are **complementary layers** with a clear split:

```
┌─────────────────────────────┐     ┌──────────────────────────────┐
│   Registry Validation (B)   │     │ Cross-Reference Validation(C)│
│   YAML structure & internal │     │ Prose & links → registries   │
│   registry consistency      │     │ files & anchors              │
└─────────────────────────────┘     └──────────────────────────────┘
         │                                       │
         └─────────── same spec root ────────────┘
                    read-only registries
```

| Aspect | Registry validation | Cross-reference validation |
|--------|---------------------|----------------------------|
| **Primary files** | `spec/*/registry.yaml` | Markdown corpus (+ registry YAML as lookup) |
| **Category** | `Registry` | `CrossReference` |
| **Rule ID prefix** | `vp-term-*`, `vp-rfc-*` | `vp-crossref-*` |
| **Depends on other validator running?** | No | No |
| **Uses registry content?** | Validates registry shape | Uses registry IDs as lookup tables |

A broken registry may cause **both** validators to report errors independently. Cross-reference validation must **not** assume registries passed registry validation—only that it can load or fail with its own diagnostics.

Recommended contributor workflow: fix registry structural errors first, then cross-reference errors—but CI may run the full suite in any order without changing pass/fail semantics.

---

## Diagnostic philosophy

Cross-reference diagnostics must help a contributor **fix the reference in one pass** without reading validator source.

Every diagnostic should explain:

| Question | Diagnostic field |
|----------|------------------|
| **What failed?** | Message + Rule ID |
| **Where?** | File + line/column (or span) in source Markdown |
| **Why?** | Context (e.g. "VP-TERM-999 not in terminology registry") |
| **How to fix?** | Suggestion (add registry entry, fix typo, update link path) |

### Example (illustrative shape)

```text
error[vp-crossref-unknown-term]: unknown VP-TERM reference VP-TERM-999
  file: docs/01-architecture/DOMAIN_MODEL.md
  location: line 142, column 18
  context: terminology registry contains VP-TERM-001 through VP-TERM-040
  help: add VP-TERM-999 to spec/terminology/registry.yaml or correct the citation
```

### Quality bar

| Good | Bad |
|------|-----|
| Cites file, line, and referenced token | "Broken reference" |
| Names the registry checked | "Invalid ID" |
| Distinguishes unknown ID vs broken link vs bad format | One generic cross-ref error |
| Stable rule IDs for CI filtering | Message-only identifiers |

Warnings vs errors follow [VALIDATION_ENGINE.md](VALIDATION_ENGINE.md) severity policy. Unknown VP-TERM, unknown VP-RFC, and broken internal file links default to **error** in initial Milestone C.

---

## Rule ID namespace

Cross-reference rules use the **`vp-crossref-*`** prefix. Internal storage uses strongly typed `RuleId` values in `vp-diagnostics`; external CLI and CI output render stable strings via `RuleId::external_id()` (see Milestone B.3 shared infrastructure).

### Planned rule IDs (initial Milestone C)

| Rule ID | Trigger |
|---------|---------|
| `vp-crossref-unknown-term` | VP-TERM ID in Markdown not found in terminology registry |
| `vp-crossref-unknown-rfc` | VP-RFC ID in Markdown not found in RFC registry |
| `vp-crossref-broken-link` | Relative Markdown link target file does not exist |
| `vp-crossref-broken-anchor` | Link fragment does not match a heading in target file |
| `vp-crossref-invalid-reference-format` | Token matches reference shape but violates ID pattern |

### Future rule IDs (documented, not initial C)

| Rule ID | Trigger |
|---------|---------|
| `vp-crossref-unknown-section-id` | Architecture section ID format or target invalid |
| `vp-crossref-unknown-conformance-scenario` | VP-CS ID not in registry (when VP-CS registry exists) |
| `vp-crossref-deprecated-reference` | Prose cites deprecated term or superseded RFC (advisory) |

Final rule catalog is frozen when Milestone C implementation ADR is accepted.

---

## Success criteria (Milestone C)

Milestone C is **complete** when cross-reference validation can **detect reference integrity defects** in the initial scope:

| # | Criterion |
|---|-----------|
| 1 | Reference to non-existent `VP-TERM-*` in fixture Markdown fails with `vp-crossref-unknown-term`, file, and line |
| 2 | Reference to non-existent `VP-RFC-*` fails with `vp-crossref-unknown-rfc` |
| 3 | Broken relative internal links fail with `vp-crossref-broken-link` |
| 4 | Broken anchor links fail with `vp-crossref-broken-anchor` when anchor checking is enabled for fixture |
| 5 | Malformed ID tokens fail with `vp-crossref-invalid-reference-format` per documented patterns |
| 6 | Current `veritypay-spec` main passes with zero errors—or known exceptions tracked in spec with explicit waiver policy |
| 7 | Validator runs via engine entrypoint; composable **independently** of registry-only validation (either alone or in full suite) |
| 8 | Diagnostics include what / where / why / how to fix for each rule class |
| 9 | Registry YAML loaded as lookup tables without invoking or depending on registry validator execution order |

**Not included in Milestone C:** external URL crawling, Edition manifests, VP-CS registry, generated docs, semantic prose review, auto-fix of citations.

See [ROADMAP.md](../ROADMAP.md) for milestone boundaries.

---

## Governance alignment

| Topic | Authority |
|-------|-----------|
| Which ID patterns are **valid** | Documented in `veritypay-spec` (glossary, VP-RFC-0000, registry README) |
| Which paths are **in corpus scope** | Spec maintainers + Milestone C ADR in tooling |
| Whether deprecated citations are **warning or error** | Spec governance; tooling implements |
| Adding **VP-CS** cross-refs | Spec registry scaffold + governance; then tooling extends this document |
| Changing **normative meaning** of a term | RFC in `veritypay-spec`—not validator PR |

When a cross-reference rule should become normative policy, propose the change in **`veritypay-spec` first**, then implement here.

---

## Implementation notes (non-normative)

These constraints guide Milestone C code but are not themselves normative spec policy:

| Concern | Direction |
|---------|-----------|
| **File access** | All reads via `SpecRepository`; no direct `std::fs` in validator logic |
| **Registry load** | Parse registry YAML inside cross-reference validator; tolerate duplicate diagnostics if registry is structurally broken |
| **Markdown parsing** | Dedicated crate or module (e.g. `vp-crossref` per [ADR-0002](adrs/0002-workspace-architecture.md)); no Markdown parsing in Milestone B crates |
| **Performance** | Full corpus scan acceptable for initial C; incremental scans are future optimization |
| **Rule IDs** | Extend shared `RuleKind` / `RuleScope` in `vp-diagnostics`; external prefix `vp-crossref-*` |

---

## Related documents

| Document | Relationship |
|----------|--------------|
| [VALIDATION_ENGINE.md](VALIDATION_ENGINE.md) | Shared lifecycle, categories, composition |
| [REGISTRY_VALIDATION.md](REGISTRY_VALIDATION.md) | Registry structural checks; complementary layer |
| [ARCHITECTURE.md](../ARCHITECTURE.md) | Cross-reference Validation component |
| [ROADMAP.md](../ROADMAP.md) | Milestone C delivery |
| [ADR-0003](adrs/0003-validator-execution-model.md) | Validator independence and pipeline |
| [ADR-0002](adrs/0002-workspace-architecture.md) | Planned `vp-crossref` crate |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | Specification boundary |
| [veritypay-spec — terminology registry](https://github.com/veritypay/veritypay-spec/blob/main/spec/terminology/registry.yaml) | VP-TERM lookup source |
| [veritypay-spec — RFC registry](https://github.com/veritypay/veritypay-spec/blob/main/spec/rfcs/registry.yaml) | VP-RFC lookup source |

---

*Registries define what exists. Cross-reference validation ensures the corpus actually points at it.*
