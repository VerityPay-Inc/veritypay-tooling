# Edition Validation

**Architecture for Milestone D: Edition Manifest structural integrity and internal coherence.**

This document defines what **Edition validation** will do when implemented in Milestone D. It does not specify code, parsing libraries, manifest generation, or publication workflows.

For the shared validator lifecycle and diagnostic model, see [VALIDATION_ENGINE.md](VALIDATION_ENGINE.md). For registry structural checks (Milestone B), see [REGISTRY_VALIDATION.md](REGISTRY_VALIDATION.md). For corpus reference integrity (Milestone C), see [CROSS_REFERENCE_VALIDATION.md](CROSS_REFERENCE_VALIDATION.md). For CLI and validation options, see [CONFIGURATION_ARCHITECTURE.md](CONFIGURATION_ARCHITECTURE.md) (Milestone C.4—**implement before Edition validation code**).

---

## Purpose

Edition validation **protects publication artifacts from structural drift before governance publication**.

An **Edition Manifest** is the machine-readable record of what a published Edition contains: declared Edition ID, Protocol Version, publication status, pinned specification documents, accepted RFCs, registry snapshots, and conformance baseline references. Authors and maintainers prepare manifest YAML alongside the release checklist. Without automated checks, drift appears as:

- Manifest files that do not parse or omit required fields
- `edition_id` or Protocol Version strings that violate documented patterns
- Pinned document paths that no longer exist under the spec root
- Pinned document versions that disagree with document front matter
- Accepted RFC IDs that are not present in the VP-RFC registry
- Registry snapshot references that point at missing files
- Malformed conformance baseline IDs cited before the VP-CS registry exists

Edition validation fails CI on **manifest structural and coherence defects**—missing files, invalid enums, unknown RFC pins, and version mismatches—before maintainers treat a manifest as release-ready.

**Boundary:** Edition validation checks that an Edition Manifest is **structurally valid and internally coherent** against the spec tree and registries at validation time. It does **not** publish an Edition, authorize publication, sign artifacts, certify implementations, or judge whether the Edition *should* be published.

> **Institutional principle:** Tooling supports release readiness; maintainers authorize publication.
>
> Validators report structural and coherence defects. They never issue, sign, or declare an Edition published. Governance records that act separately.

---

## Edition identity

Every **Edition** represents a published snapshot of the VerityPay specification—a named, reviewable bundle that implementers can cite without negotiating ad hoc document versions.

An Edition is a conceptual object with:

| Component | Role |
|-----------|------|
| **Identifier** | Stable `edition_id` (e.g. `vp-edition-genesis-1`) |
| **Protocol Version** | Implementer-facing rule label declared for the Edition |
| **Publication status** | Lifecycle state (see [Edition states](#edition-states)) |
| **Publication date** | When the Edition was published (ISO date) |
| **Manifest** | Machine-readable YAML describing the snapshot |
| **Document set** | Pinned specification documents and versions |
| **Registry snapshots** | Terminology and RFC registry revisions at publication |

An Edition is **immutable once published**. Corrections flow through governance (errata, new RFCs, a successor Edition)—not by rewriting a published manifest in place.

Future Editions **supersede** earlier Editions but **never modify** them. Auditors and integrators must always reconstruct what was published at each era.

Edition validation operates on the **manifest** as the authoritative machine-readable view of an Edition object. Field-level checks in Milestone D validate that the manifest faithfully describes a coherent snapshot—not that publication was authorized.

---

## Edition states

Editions follow an explicit lifecycle, analogous to state machines used elsewhere in the platform (Claims, Participants, Specifications). The manifest `status` field records the current state; Edition validation ensures the value is recognized and transitions are plausible for the artifact's role.

```
Draft
  ↓
Candidate
  ↓
Published
  ↓
Maintained
  ↓
Superseded
  ↓
Archived
```

| State | Meaning |
|-------|---------|
| **Draft** | Work in progress; manifest may be incomplete or inconsistent—informative only |
| **Candidate** | Release checklist in progress; tooling should report all structural defects |
| **Published** | Institutional commitment; manifest is the authoritative publication record |
| **Maintained** | Published Edition still supported; errata may apply without a new Edition |
| **Superseded** | A successor Edition is the preferred migration target; this Edition remains accessible |
| **Archived** | Retained for audit; not recommended for new implementations |

Initial Milestone D validates that `status` is a **known enum value**. It does **not** enforce transition rules (e.g. `Draft` → `Published`) or block validation of non-published manifests—maintainers use `Candidate` manifests during release prep. Transition policy belongs in spec governance; tooling implements allowed values first.

Mapping to [SPECIFICATION_RELEASE_PROCESS.md](https://github.com/VerityPay-Inc/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_RELEASE_PROCESS.md) illustrative statuses (`published`, `maintained`, `superseded`, `archived`) is documented when Milestone D ADR is accepted. `draft` and `candidate` may appear in tooling and maintainer workflows before formal spec enum extension.

---

## Position in the validation engine

```
Input (spec root + Edition Manifest path from config or CLI)
       ↓
Discovery → Edition Manifest YAML (ValidationContext / config)
           → Pinned specification document paths
           → Document front matter (for version pins)
           → VP-RFC registry (read-only lookup)
           → Registry snapshot paths
       ↓
Edition Validator
       ↓
Diagnostics (category: Edition)
       ↓
Report Aggregator
```

The manifest path should come from **`ValidationConfig.edition`** ([CONFIGURATION_ARCHITECTURE.md](CONFIGURATION_ARCHITECTURE.md)) with CLI override—not from a growing set of edition-specific flags added per milestone.

The Edition validator is **independent**: it runs without requiring registry or cross-reference validators to execute first. It may **read** registry YAML, pinned Markdown files, and front matter via `SpecRepository`—read-only reuse of spec artifacts, not coupling to another validator's execution or diagnostic output.

Per [ADR-0003](adrs/0003-validator-execution-model.md):

- Validators emit diagnostics only; the engine aggregates; the CLI presents.
- Engine invocation order must **not** affect correctness.
- Shared file access belongs in **`SpecRepository`** (`vp-core`); shared diagnostic types belong in **`vp-diagnostics`**.

Planned validator identity (illustrative):

| Field | Value |
|-------|-------|
| `id` | `edition` |
| `name` | Edition Manifest |
| `description` | Validates Edition Manifest structure, pins, and registry references. |
| `category` | `Edition` |

Edition validation is intended for **`--profile release`** once validation profiles ship ([CLI_PHILOSOPHY.md](../CLI_PHILOSOPHY.md)). Until then, it runs when the manifest path is supplied to `vp validate` or a dedicated edition subcommand.

---

## Supported manifest fields

Field names and shapes follow governance prose in **`veritypay-spec`**. The table below reflects illustrative intent from [SPECIFICATION_RELEASE_PROCESS.md](https://github.com/VerityPay-Inc/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_RELEASE_PROCESS.md)—not a normative schema invented by tooling.

| Field | Intent | Initial Milestone D |
|-------|--------|---------------------|
| `edition` | Human-readable Edition name (e.g. `Genesis`) | Required field presence |
| `edition_id` | Stable identifier (e.g. `vp-edition-genesis-1`) | Format + presence |
| `protocol_version` | Declared Protocol Version (e.g. `vp-protocol-1.0`) | Presence |
| `publication_date` | ISO publication date | Presence (format check when policy documented) |
| `status` | Publication lifecycle (`published`, `maintained`, `superseded`, `archived`, …) | Allowed enum |
| `specification_documents` | Map of document path → pinned version string | Path existence; version vs front matter where practical |
| `accepted_rfcs` | List of **VP-RFC-*** IDs through Edition cutoff | Each ID exists in VP-RFC registry |
| `registry_snapshots` | References to terminology / RFC registry revisions at publication | Referenced paths exist |
| `conformance_baseline` | VP-CS scenario IDs included in the Edition | ID format only (no VP-CS registry yet) |
| `supersedes` | Prior `edition_id`, if any | Optional; format when present |
| `integrity` | Reserved for future checksums or signatures | Out of scope initial D |

Exact required-field set, allowed `status` values, and ID patterns are **frozen when Milestone D implementation ADR is accepted**, aligned with spec governance—not silently invented at code time.

### Manifest evolution (future-proofing)

Initial Milestone D validates **fields and referenced paths**—structure and internal coherence. Later milestones may validate **artifacts** bound to those fields without changing the conceptual role of an Edition Manifest:

- Cryptographic integrity information and signatures (`integrity`)
- Reproducible publication bundles and checksum manifests
- Additional publication metadata (build provenance, tooling version pins)

Future Editions may include these capabilities **without redefining what an Edition is**. The manifest remains the single machine-readable description of a published snapshot; new keys and validators extend verification depth. Edition validation should treat unknown optional top-level keys as documented in Milestone D ADR (ignore vs warn vs error).

---

## Discovery scope

The Edition validator discovers artifacts **within its documented scope**. Discovery is explicit and deterministic from spec root plus manifest path.

### In scope (initial Milestone D)

| Artifact | Action |
|----------|--------|
| Edition Manifest YAML | Load from explicit CLI path or documented default location under spec root |
| Paths in `specification_documents` | Verify each key resolves to a file under spec root |
| Front matter of pinned documents | Read `version` (or equivalent) and compare to pinned value where practical |
| `spec/rfcs/registry.yaml` | Load as VP-RFC lookup for `accepted_rfcs` resolution |
| Paths in `registry_snapshots` | Verify snapshot file references exist (path component before optional `@rev` suffix) |
| `conformance_baseline` entries | Validate **VP-CS-NNNN** (or documented) ID format |

### Out of scope (initial Milestone D)

| Artifact / action | Reason |
|-------------------|--------|
| **Publishing** an Edition | Governance act in `veritypay-spec`; tooling validates only |
| **Cryptographic signatures** / `integrity` enforcement | Future milestone; field may be present but not verified |
| **Release announcements** | Human/process output; not validation |
| **Generating** the manifest | Optional draft builder is separate, non-normative tooling |
| **External URLs** | Not required merge gate |
| **Certifying implementations** | `veritypay-conformance` domain |
| **VP-CS registry existence checks** | VP-CS registry not yet defined; baseline IDs are format-checked only |
| **Full cross-reference corpus scan** | Owned by [Cross-Reference Validation](CROSS_REFERENCE_VALIDATION.md) |
| **Semantic review** of Edition content | Human governance |
| **Auto-fix** of manifest or pins | Tooling is read-only per [ADR-0003](adrs/0003-validator-execution-model.md) |

---

## Validation responsibilities

Rules below are **architectural obligations** for initial Milestone D. Exact matching rules follow schemas documented in `veritypay-spec` and tooling ADRs at implementation time.

### Initial Milestone D checks

| # | Check | Description |
|---|-------|-------------|
| 1 | **Manifest exists and parses** | Edition Manifest file is present and valid YAML |
| 2 | **Required top-level fields** | Documented header fields present (e.g. `edition_id`, `protocol_version`, `status`, collections) |
| 3 | **`edition_id` format** | Stable identifier matches documented pattern |
| 4 | **`protocol_version` present** | Declared Protocol Version is non-empty |
| 5 | **`status` valid** | Status value is in the allowed publication lifecycle set |
| 6 | **Pinned document paths exist** | Each key in `specification_documents` resolves under spec root |
| 7 | **Pinned versions match front matter** | Where document front matter exposes `version`, compare to pin (practical subset—policy documents exact fields in ADR) |
| 8 | **Accepted RFC IDs exist** | Each entry in `accepted_rfcs` resolves to an id in `spec/rfcs/registry.yaml` |
| 9 | **Registry snapshot paths exist** | Each referenced terminology / RFC snapshot path exists |
| 10 | **Conformance baseline ID format** | Each `conformance_baseline` entry matches documented VP-CS ID pattern; **does not** require VP-CS registry lookup yet |

### Coherence vs completeness

Edition validation distinguishes:

| Concern | Edition validator | Human release checklist |
|---------|-------------------|-------------------------|
| YAML shape and required keys | Yes | Yes |
| Pins resolve to real files | Yes | Yes |
| RFC IDs in registry | Yes | Yes |
| Every normative doc for an Edition is listed | Partial—only checks declared pins | Yes |
| Governance authorization to publish | No | Yes |
| Conformance scenarios are runnable | No (future VP-CS registry + runner) | Future |

---

## Diagnostic philosophy

Edition diagnostics use the shared model in `vp-diagnostics` and the presentation rules in [VALIDATION_OUTPUT.md](VALIDATION_OUTPUT.md).

Every diagnostic should answer:

| Question | Diagnostic field |
|----------|------------------|
| **What happened?** | Rule ID, human title, instance message |
| **Where?** | Manifest YAML path (e.g. `specification_documents.docs/foo.md`) or pinned file + line when applicable |
| **Why?** | Rule description (e.g. "accepted RFC not in VP-RFC registry") |
| **How do I fix it?** | `Suggestion:` — add pin, fix id, update registry entry, align front matter version |

### Example (illustrative shape)

```text
error[vp-edition-unknown-rfc]
Unknown Accepted RFC
An accepted_rfcs entry is not present in the VP-RFC registry.

  --> editions/genesis-edition.yaml:accepted_rfcs[2]

accepted RFC `VP-RFC-0099` is not listed in spec/rfcs/registry.yaml

Suggestion:
add VP-RFC-0099 to the RFC registry or remove it from accepted_rfcs
```

### Quality bar

| Good | Bad |
|------|-----|
| Names manifest YAML path and field | "Invalid edition" |
| Distinguishes missing file vs bad id vs version mismatch | One generic edition error |
| Cites pinned document path and expected vs actual version | "Version wrong" |
| Stable rule IDs for CI filtering | Message-only identifiers |

Errors vs warnings follow [VALIDATION_ENGINE.md](VALIDATION_ENGINE.md) severity policy. Structural defects (missing manifest, invalid YAML, missing required field, unknown RFC, missing pin) default to **error** in initial Milestone D.

---

## Rule ID namespace

Edition rules use the **`vp-edition-*`** prefix. Internal storage will extend strongly typed `RuleId` values in `vp-diagnostics` (`RuleScope::Edition` or equivalent); external CLI and CI output render stable strings via `RuleId::external_id()`.

### Planned rule IDs (initial Milestone D)

| Rule ID | Trigger |
|---------|---------|
| `vp-edition-manifest-missing` | Edition Manifest file not found at supplied path |
| `vp-edition-yaml-invalid` | Manifest YAML is not well-formed or not a mapping |
| `vp-edition-missing-field` | Required top-level or nested field absent |
| `vp-edition-invalid-id` | `edition_id` (or related id field) violates documented pattern |
| `vp-edition-invalid-status` | `status` value not in allowed set |
| `vp-edition-document-missing` | Pinned specification document path does not exist |
| `vp-edition-version-mismatch` | Pinned document version disagrees with front matter |
| `vp-edition-unknown-rfc` | `accepted_rfcs` entry not in VP-RFC registry |
| `vp-edition-registry-missing` | Registry snapshot path does not exist |
| `vp-edition-invalid-conformance-id` | Conformance baseline entry violates VP-CS ID format |

### Future rule IDs (documented, not initial D)

| Rule ID | Trigger |
|---------|---------|
| `vp-edition-unknown-conformance-scenario` | VP-CS ID not in registry (when VP-CS registry exists) |
| `vp-edition-supersedes-unknown` | `supersedes` references unknown prior edition_id |
| `vp-edition-integrity-invalid` | Checksum or signature verification failed |
| `vp-edition-protocol-version-invalid` | Protocol Version string violates documented pattern |

Final rule catalog is frozen when Milestone D implementation ADR is accepted.

---

## Success criteria (Milestone D)

Milestone D is **complete** when Edition validation can **detect manifest structural and coherence defects** in the initial scope:

| # | Criterion |
|---|-----------|
| 1 | Missing manifest fails with `vp-edition-manifest-missing` |
| 2 | Invalid YAML fails with `vp-edition-yaml-invalid` |
| 3 | Missing required field fails with `vp-edition-missing-field` and YAML path location |
| 4 | Invalid `edition_id` fails with `vp-edition-invalid-id` |
| 5 | Invalid `status` fails with `vp-edition-invalid-status` |
| 6 | Missing pinned document fails with `vp-edition-document-missing` |
| 7 | Front matter version mismatch fails with `vp-edition-version-mismatch` where check applies |
| 8 | Unknown accepted RFC fails with `vp-edition-unknown-rfc` |
| 9 | Missing registry snapshot path fails with `vp-edition-registry-missing` |
| 10 | Malformed conformance baseline ID fails with `vp-edition-invalid-conformance-id` |
| 11 | Illustrative Genesis manifest draft validates or reports concrete, actionable gaps |
| 12 | Validator runs via engine entrypoint; independent of registry/cross-ref execution order |
| 13 | Diagnostics include what / where / why / how to fix for each rule class |
| 14 | Edition validation does not mutate `veritypay-spec` |

**Not included in Milestone D:** publication authorization, signatures, manifest generation as normative output, VP-CS registry resolution, external URL checks, conformance execution, implementation certification.

See [ROADMAP.md](../ROADMAP.md) for milestone boundaries.

---

## Governance alignment

| Topic | Authority |
|-------|-----------|
| **Which manifest fields are required** | [SPECIFICATION_RELEASE_PROCESS.md](https://github.com/VerityPay-Inc/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_RELEASE_PROCESS.md) and future Meta RFC |
| **Edition vs Protocol Version semantics** | [SPECIFICATION_VERSIONING.md](https://github.com/VerityPay-Inc/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_VERSIONING.md) |
| **Allowed `status` values** | Spec governance; tooling implements |
| **ID patterns** (`edition_id`, `protocol_version`, VP-CS) | Spec governance + glossary / VP-RFC-0000 |
| **Release checklist items beyond tooling** | Maintainers; tooling supports but does not replace |
| **Publishing an Edition** | Governance record + manifest issuance—not validator PR |
| **Normative manifest schema changes** | Spec change first, then tooling |

When an Edition rule should become normative policy, propose the change in **`veritypay-spec` first**, then implement here.

---

## Relationship to spec governance documents

### [SPECIFICATION_RELEASE_PROCESS.md](https://github.com/VerityPay-Inc/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_RELEASE_PROCESS.md)

Defines **why** Editions exist, the **release checklist**, illustrative manifest fields, and **publication outputs**. Edition validation implements the **machine-checkable subset** of that checklist:

| Release process concern | Tooling (Milestone D) |
|-------------------------|------------------------|
| Edition Manifest issued | Validates structure when path supplied |
| Pinned document versions | Checks paths and version coherence |
| Accepted RFC set | Checks IDs against VP-RFC registry |
| Registry snapshots | Checks referenced paths exist |
| Conformance baseline | Format-checks VP-CS IDs |
| Maintainer authorization | Out of scope |
| Publication announcement | Out of scope |

> **Institutional principle:** Tooling supports release readiness; maintainers authorize publication.

### [SPECIFICATION_VERSIONING.md](https://github.com/VerityPay-Inc/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_VERSIONING.md)

Defines the **three version axes**—Edition, Protocol Version, document version—and what belongs in an Edition bundle. Edition validation enforces that the manifest **declares** those axes consistently:

- **`edition_id`** identifies the published Edition snapshot
- **`protocol_version`** declares the implementer-facing rule label
- **`specification_documents`** pins per-document versions within the Edition

Edition validation does **not** decide when Protocol Version must increment; it verifies that declared pins and IDs are structurally coherent with the spec tree at validation time.

---

## Implementation notes (non-normative)

These constraints guide Milestone D code but are not themselves normative spec policy:

| Concern | Direction |
|---------|-----------|
| **Crate home** | Likely `vp-edition` per [ADR-0002](adrs/0002-workspace-architecture.md); final name via Milestone D ADR |
| **File access** | All reads via `SpecRepository` |
| **Registry load** | Parse VP-RFC registry inside Edition validator for `accepted_rfcs`; tolerate duplicate diagnostics if registry is structurally broken |
| **Front matter** | Reuse or share parsing conventions with future front-matter validator; minimal YAML/header parse for `version` field |
| **Manifest path** | From `ValidationConfig.edition` ([CONFIGURATION_ARCHITECTURE.md](CONFIGURATION_ARCHITECTURE.md)); CLI overrides config |
| **Prerequisite** | Milestone C.4 Configuration landed before Edition validator implementation |
| **Rule IDs** | Extend shared `RuleKind` / `RuleScope` in `vp-diagnostics`; external prefix `vp-edition-*` |
| **Category** | `Category::Edition` in diagnostics and validator identity |

---

## Related documents

| Document | Relationship |
|----------|--------------|
| [VALIDATION_ENGINE.md](VALIDATION_ENGINE.md) | Shared lifecycle, categories, composition |
| [REGISTRY_VALIDATION.md](REGISTRY_VALIDATION.md) | VP-RFC registry structural checks; lookup source for accepted RFCs |
| [CROSS_REFERENCE_VALIDATION.md](CROSS_REFERENCE_VALIDATION.md) | Corpus link integrity; complementary layer |
| [VALIDATION_OUTPUT.md](VALIDATION_OUTPUT.md) | Human and JSON diagnostic contract |
| [CONFIGURATION_ARCHITECTURE.md](CONFIGURATION_ARCHITECTURE.md) | Milestone C.4 — config before Edition validator; `edition` path |
| [CLI_PHILOSOPHY.md](../CLI_PHILOSOPHY.md) | `--profile release`, edition subcommand intent |
| [ARCHITECTURE.md](../ARCHITECTURE.md) | Edition validation component |
| [ROADMAP.md](../ROADMAP.md) | Milestone D delivery |
| [ADR-0003](adrs/0003-validator-execution-model.md) | Validator independence and pipeline |
| [veritypay-spec — SPECIFICATION_RELEASE_PROCESS](https://github.com/VerityPay-Inc/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_RELEASE_PROCESS.md) | Manifest purpose and release checklist |
| [veritypay-spec — SPECIFICATION_VERSIONING](https://github.com/VerityPay-Inc/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_VERSIONING.md) | Edition, Protocol Version, document version axes |

---

*Registries index what exists. Cross-reference validation ensures the corpus cites it. Edition validation ensures publication pins actually describe a coherent snapshot.*
