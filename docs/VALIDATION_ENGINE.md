# Validation Engine

**Conceptual architecture for all validators in `veritypay-tooling`.**

This document describes the **validation framework** that future code will implement. It does not specify programming language, libraries, file layout, or algorithms. For repository-level components, see [ARCHITECTURE.md](../ARCHITECTURE.md). For the first validator, see [REGISTRY_VALIDATION.md](REGISTRY_VALIDATION.md).

**Principle:** Tooling **follows** the specification. Validators **check** documented rules—they never **define** protocol behavior or modify normative files.

---

## Purpose

The validation engine is the **shared runtime** for every check in this repository. Whether invoked from `vp validate` (illustrative), CI, or a future library API, all validators:

- Accept the same **input context** (spec tree path, optional Edition pin)
- Follow the same **lifecycle**
- Emit the same **diagnostic shape**
- Aggregate into a unified **report** and **exit code**

Consistency matters because contributors and auditors must trust that **local runs and CI runs are equivalent**—only the output format may differ (human text vs JSON).

---

## Shared lifecycle

Every validator participates in the same pipeline:

```
Input
  ↓
Discovery
  ↓
Validation
  ↓
Diagnostics
  ↓
Report
  ↓
Exit Code
```

### Input

**What enters the engine.**

| Concept | Description |
|---------|-------------|
| **Spec root** | Path to a `veritypay-spec` checkout (local directory or pinned revision) |
| **Scope** | Which validators to run (all, or subset—e.g. registry only) |
| **Edition pin** | Optional Edition Manifest for release-candidate checks |
| **Output mode** | Human-readable or machine-readable (CI) |
| **Strictness** | Optional policy flags documented per validator (e.g. warnings fail CI) |

The engine does **not** mutate the spec tree. Input is **read-only**.

### Discovery

**What files and artifacts the validator considers in scope.**

Each validator declares what it discovers:

- Registry YAML paths (`spec/terminology/registry.yaml`, `spec/rfcs/registry.yaml`)
- Markdown corpora (cross-reference validator—future)
- Edition Manifest files (edition validator—future)
- Front matter blocks in matched files (metadata validator—future)

Discovery is **explicit**: a validator documents its glob patterns, registry paths, and exclusions. Hidden discovery breeds false confidence.

### Validation

**Application of documented rules** against discovered artifacts.

Rules originate from:

- Accepted specification and governance text in `veritypay-spec`
- Registry schemas and field requirements documented in spec or tooling ADRs
- VP-RFC-0000 metadata conventions

Validation checks **structure and traceability**—not semantic correctness of protocol prose. "Is VP-TERM-012 defined?" is in scope. "Does this sentence correctly explain supersession?" is out of scope unless a machine-checkable rule exists.

### Diagnostics

**Structured findings** for each violation or advisory. See [Diagnostic model](#diagnostic-model).

Validators emit **zero or more** diagnostics. No silent failures: parse errors, missing files, and internal errors become diagnostics or engine-level failures with clear codes.

### Report

**Aggregation** of all diagnostics from invoked validators.

| Report property | Description |
|-----------------|-------------|
| **Summary** | Counts by severity; validators run; duration (optional) |
| **Findings** | Ordered list of diagnostics (stable sort for CI diff) |
| **Context** | Spec root, tool version, validator versions, Edition pin if any |

The report is the **audit artifact** grant reviewers and maintainers cite.

### Exit Code

**Process outcome** for shells and CI.

| Code | Meaning (conceptual) |
|------|----------------------|
| `0` | All invoked validators passed (per policy) |
| `1` | One or more validation failures |
| `2` | User error (bad paths, unknown validator name) |
| `3+` | Engine or unexpected internal error |

Individual validators do not define conflicting exit semantics—the **engine** maps aggregated severities to a single exit code per [CLI_PHILOSOPHY.md](../CLI_PHILOSOPHY.md).

---

## Validator responsibilities

Every validator **must**:

| Responsibility | Requirement |
|----------------|-------------|
| **Discover** | Locate relevant files deterministically from spec root |
| **Validate** | Apply rules documented in spec or validator-specific architecture docs |
| **Emit diagnostics** | Structured findings with rule IDs—never opaque strings alone |
| **Never modify specification files** | Read-only; no auto-fix in normative trees without explicit maintainer-only tooling (future, out of scope for Milestone B) |
| **Support CLI and CI equally** | Same core logic; output formatting differs |

Every validator **must not**:

| Forbidden | Reason |
|-----------|--------|
| Define normative protocol behavior | Belongs in `veritypay-spec` via RFC |
| Accept or reject RFCs | Governance, not tooling |
| Write into `veritypay-spec` during validation | Breaks audit trust |
| Depend on another validator's internal state | Composability requires independence |

---

## Diagnostic model

Each finding includes **conceptual fields**. Exact serialization (JSON schema, etc.) is an implementation decision documented in a future ADR.

| Field | Purpose |
|-------|---------|
| **Severity** | `error` · `warning` · `info` — whether finding fails CI by default |
| **Rule ID** | Stable identifier (e.g. `vp-term-duplicate-id`, `vp-rfc-unknown-status`) |
| **Category** | Validation category (registry, metadata, cross-reference, …) |
| **Validator** | Which validator produced the finding |
| **File** | Path relative to spec root, if applicable |
| **Location** | Line, column, YAML key path, or registry entry ID |
| **Message** | Short human-readable description of what failed |
| **Suggestion** | Actionable fix when not obvious from message alone |
| **Context** | Optional structured detail (duplicate of ID, expected enum values) |

### Severity policy (default)

| Severity | Default CI behavior |
|----------|---------------------|
| **error** | Fail exit code |
| **warning** | Pass exit code; visible in report (policy may upgrade to fail) |
| **info** | Pass; advisory only |

Maintainers may configure strict mode so warnings fail CI during Edition release candidates.

---

## Validation categories

Validators group under **categories** for discovery, CLI grouping, and roadmap milestones.

| Category | Scope | Milestone (typical) | Validator examples |
|----------|-------|---------------------|-------------------|
| **Registry** | Machine-readable YAML registries | B | VP-TERM, VP-RFC |
| **Metadata** | Front matter, document headers | B–C | RFC front matter, spec headers |
| **Cross References** | IDs and links in prose | C | VP-TERM refs, VP-RFC refs, anchors |
| **Edition** | Edition Manifest pins and bundles | D | Manifest schema, pin existence |
| **Documentation** | Link integrity, README policy | C | Internal links, pyramid checks |
| **Future** | Reserved | F–G+ | VP-CS registry, VP-EDITION, generated doc drift |

Categories are **not** mutually exclusive at the diagnostic level—a registry validator may emit `cross-reference` category findings when registry entries reference missing RFC paths. The **validator** owns execution; the **category** tags findings for filtering.

---

## Composition

Validators compose through the engine—not through deep inheritance or shared mutable state.

```
vp validate                    (illustrative CLI entry)
       ↓
Validation Engine
       ↓
  ┌────┴────┬──────────────┬─────────────┐
  ↓         ↓              ↓             ↓
Registry   Cross-Ref      Edition       Documentation
Validator  Validator      Validator     Validator
  ↓         ↓              ↓             ↓
  └────┬────┴──────────────┴─────────────┘
       ↓
Report Aggregator
       ↓
Unified Report + Exit Code
```

### Orchestration rules

| Rule | Rationale |
|------|-----------|
| Engine invokes validators **in declared order** or parallel where safe | Predictable reports |
| Each validator returns **its own diagnostic list** | Independence |
| Aggregator **merges** lists; does not reinterpret rules | Single source of truth per rule |
| Subset invocation allowed (`registry` only) | Fast local feedback |
| Full suite is default for merge gates | No partial green builds |

### Report aggregator

The aggregator:

- Collects diagnostics from all invoked validators
- De-duplicates only when same rule ID + location + message (optional policy)
- Computes severity counts
- Applies exit code policy
- Emits human or machine-readable report

The aggregator does **not** re-run validation logic.

---

## Why validators remain independent

| Reason | Explanation |
|--------|-------------|
| **Milestone delivery** | Milestone B ships registry validation without waiting for cross-ref (C) |
| **CI performance** | Run only what changed when incremental validation exists (future) |
| **Clear ownership** | Each validator has one architecture doc and one rule ID namespace |
| **Test isolation** | Fixture spec trees test one validator without mocking others |
| **Failure diagnosis** | Contributors know which subsystem failed |
| **No hidden coupling** | Registry validator must not assume cross-ref already ran |

Shared code belongs in **engine utilities** (YAML parse helpers, path resolution, diagnostic types)—not in validator-specific rule logic bleeding across boundaries.

---

## Relationship to CLI

The CLI is a **thin orchestration layer** over the engine. See [CLI_PHILOSOPHY.md](../CLI_PHILOSOPHY.md).

| CLI concern | Engine concern |
|-------------|----------------|
| Argument parsing | Spec root resolution |
| `--help` text | Validator registry and categories |
| Output formatting | Diagnostic model |
| Exit code to shell | Aggregated severity policy |

Local development and CI must call the **same engine entrypoint** with the same validator list.

---

## Evolution

| Change type | Process |
|-------------|---------|
| New diagnostic field | ADR + schema version bump |
| New validator | Architecture doc + milestone update |
| New rule ID | Document in validator architecture doc; spec governance if normative |
| Breaking CI policy | ADR + changelog |

Implementation language and package layout are **explicitly deferred** to the first implementation ADR at Milestone B.

---

## Related documents

| Document | Relationship |
|----------|--------------|
| [ARCHITECTURE.md](../ARCHITECTURE.md) | Repository component map |
| [REGISTRY_VALIDATION.md](REGISTRY_VALIDATION.md) | First validator (Milestone B) |
| [ROADMAP.md](../ROADMAP.md) | Milestone delivery order |
| [CLI_PHILOSOPHY.md](../CLI_PHILOSOPHY.md) | CLI behavior principles |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | Specification boundary for contributors |

---

*One engine. Many validators. Read-only against the spec.*
