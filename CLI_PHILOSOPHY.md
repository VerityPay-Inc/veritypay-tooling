# CLI Philosophy

**Design principles for the future `vp` command-line interface.**

This document describes **how** the VerityPay tooling CLI should behave—not exact flag syntax, parser choice, or implementation layout.

**Important:** All command names, subcommands, and flags shown below are **illustrative examples** of intent. They are **not** a stable public API, OpenAPI contract, or final CLI specification. Final spelling, grouping, and flags may change through ADRs and Milestone E without a major protocol release.

**Binary name (working):** `vp` — short for **VerityPay platform tooling**, not "VerityPay protocol."

---

## Purpose

The CLI is the **primary interface** between humans, CI, and specification validators. It must be:

- **Discoverable** — new contributors find the right command without reading source
- **Composable** — run one check or the full suite; pipe output to CI
- **Readable** — errors teach remediation; success is quiet unless verbose

The CLI orchestrates components described in [ARCHITECTURE.md](ARCHITECTURE.md). It does not embed protocol semantics.

---

## Core principles

| Principle | Meaning |
|-----------|---------|
| **Specification is the argument** | Default input is a `veritypay-spec` tree (path or pin); not implicit cwd magic |
| **One front door** | `vp validate` is the CI default; specialized commands exist for debugging |
| **Fail clearly** | Non-zero exit; first errors visible; no silent partial success |
| **Stable CI contract** | Machine-readable output format versioned separately from human text |
| **No hidden normative power** | CLI flags cannot "accept" RFCs or change protocol meaning |
| **Progressive disclosure** | Common path is short; advanced options under `--help` per subcommand |

---

## Command groups (illustrative — not final API)

Commands are grouped by **user intent**, not internal module names. Names and flags below are **examples only**.

### `vp validate`

**Intent:** Run the **recommended validation suite** for a spec checkout or Edition pin.

Illustrative usage:

```text
vp validate --spec ../veritypay-spec
vp validate --spec ../veritypay-spec --edition genesis-draft.yaml
```

**Behavior (conceptual):**

- Aggregates registry, cross-reference, front matter, and documentation checks
- Optional edition manifest when provided
- Exit non-zero if any check fails
- Primary entry for spec PR CI

---

### `vp lint`

**Intent:** **Style and hygiene** checks that are objective but not reference integrity (subset or overlap with validate—implementation may alias or split).

Illustrative usage:

```text
vp lint --spec ../veritypay-spec
vp lint --spec ../veritypay-spec --only front-matter
```

**Behavior (conceptual):**

- Front matter enums, markdown policy, trailing whitespace if policy exists
- Faster subset for local pre-commit optional hooks
- Never substitutes for `validate` on merge gates

---

### `vp registry`

**Intent:** **Registry-focused** validation and inspection.

Illustrative usage:

```text
vp registry check --spec ../veritypay-spec
vp registry check --spec ../veritypay-spec --registry terminology
vp registry diff --spec ../veritypay-spec
```

**Behavior (conceptual):**

- VP-TERM and VP-RFC YAML validation
- Optional diff against glossary or prior snapshot (informative)
- Useful when editing `spec/terminology/` or `spec/rfcs/`

---

### `vp edition`

**Intent:** **Edition Manifest** validation and draft assistance.

Illustrative usage:

```text
vp edition validate genesis-edition.yaml --spec ../veritypay-spec
vp edition draft --spec ../veritypay-spec --name genesis
```

**Behavior (conceptual):**

- Validates manifest structure and pins
- Draft mode emits **non-normative** starter manifest for maintainer review
- Does not publish Editions

---

### `vp docs`

**Intent:** **Generate** non-normative documentation from registries (Milestone F).

Illustrative usage:

```text
vp docs generate --spec ../veritypay-spec --output ./generated
vp docs generate --spec ../veritypay-spec --only rfcs
```

**Behavior (conceptual):**

- Output clearly marked generated
- Reproducible from inputs
- Never overwrites canonical spec files in place without explicit flag and warning

---

### `vp release`

**Intent:** **Release-readiness** checks aligned with [SPECIFICATION_RELEASE_PROCESS](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_RELEASE_PROCESS.md).

Illustrative usage:

```text
vp release check --spec ../veritypay-spec --edition genesis-edition.yaml
```

**Behavior (conceptual):**

- Runs edition validation plus release checklist gates documented in spec
- Informative report for maintainers; governance still decides publication
- May include tooling self-version and spec pin reporting for audit trail

---

## Discoverability

| Requirement | Rationale |
|-------------|-----------|
| `vp --help` lists top-level commands with one-line descriptions | First run experience |
| `vp <command> --help` documents flags and examples | No wiki required |
| Related commands cross-linked in help text | e.g. `lint` vs `validate` |
| `--version` prints tooling version and supported spec layout version | CI debugging |

Avoid deep nesting beyond **two levels** (`vp registry check`, not `vp spec registry term check`) unless usage proves otherwise.

---

## Composability

| Pattern | Support |
|---------|---------|
| Run single validator | `--only registry` or dedicated subcommand |
| Run full suite | `vp validate` default |
| CI JSON output | `--format json` — see [Developer Experience](#developer-experience) |
| Human pretty output | default; optional `--quiet` for summary-only output |
| Config file | optional `.vp.toml` for paths and edition pin—local convenience only |

Exit codes:

- `0` — all requested checks passed
- `1` — validation failures
- `2` — user error (bad paths, bad flags)
- `3+` — reserved for internal errors (documented)

See [Developer Experience](#developer-experience) for output format details.

---

## Developer Experience

Validation output is a **developer tool**, not a log dump. The default experience optimizes for **fixing problems**, not merely reporting them.

### Principles

| Principle | Meaning |
|-----------|---------|
| **Fix-oriented by default** | Output teaches remediation; contributors should know what to change after one read |
| **Human-readable is the default** | `vp validate` prints grouped, annotated diagnostics without extra flags |
| **Machine-readable is explicit** | CI and automation opt in with `--format json`; JSON is never mixed into human output |
| **Every error answers four questions** | What happened? Where? Why? How do I fix it? |

Each diagnostic is structured to answer:

1. **What happened?** — rule id, human title, and instance message
2. **Where?** — file path and line, column, or YAML path when available
3. **Why?** — short rule description explaining the check
4. **How do I fix it?** — `Suggestion:` (and optional `Help:`, `Note:`, `Related:` when present)

The JSON schema mirrors the same fields for programmatic consumers. See [docs/VALIDATION_OUTPUT.md](docs/VALIDATION_OUTPUT.md) for the stable contract.

### Output flags

| Flag | Default | Use |
|------|---------|-----|
| `--format human` | yes | Local development; validator progress, grouped diagnostics, summary |
| `--format json` | | CI, scripts, and tooling integration |
| `--quiet` | no | Summary counts only; omits progress and diagnostic detail |

`--quiet` applies to human output only. It does not change exit codes or validation behavior.

Illustrative usage:

```text
vp validate --spec ../veritypay-spec
vp validate --spec ../veritypay-spec --format json
vp validate --spec ../veritypay-spec --quiet
```

### `--format human` (default)

Human output shows validator progress, diagnostics grouped by category, and a validation summary.

Illustrative example:

```text
Running validators...

✓ RFC Registry
✓ Terminology Registry
✗ Cross References

Cross References

error[vp-crossref-broken-anchor]
Broken Anchor
A markdown link fragment does not match a heading or HTML anchor.

  --> docs/page.md:1:9

broken anchor `#missing-section` in link `target.md#missing-section`

Suggestion:
add a matching heading to docs/target.md or fix the link fragment

────────────────────────────

Validation Summary

Errors:   1
Warnings: 0
Info:     0

Validation failed.
```

When a location is unavailable, the `  --> ` line is omitted. Optional annotations (`Suggestion:`, `Help:`, `Note:`, `Related:`) appear only when the validator provides them.

### `--format json`

JSON output is for machines. It serializes the same validation result as human output—summary counts plus a deterministic diagnostic list—without progress lines or category headers.

Illustrative example:

```json
{
  "summary": {
    "errors": 1,
    "warnings": 0,
    "info": 0
  },
  "diagnostics": [
    {
      "severity": "error",
      "rule_id": "vp-crossref-broken-anchor",
      "title": "Broken Anchor",
      "description": "A markdown link fragment does not match a heading or HTML anchor.",
      "category": "cross_reference",
      "message": "broken anchor `#missing-section` in link `target.md#missing-section`",
      "file": "docs/page.md",
      "location": {
        "line": 1,
        "column": 9
      },
      "suggestion": "add a matching heading to docs/target.md or fix the link fragment"
    }
  ]
}
```

Field names, enum values, and ordering are documented in [docs/VALIDATION_OUTPUT.md](docs/VALIDATION_OUTPUT.md).

### `--quiet`

Quiet mode prints only the validation summary counts. Use it when CI logs need a pass/fail line without diagnostic detail, or when a wrapper script handles failures separately.

Illustrative example:

```text
Validation Summary

Errors: 1
Warnings: 0
Info: 0
```

Quiet output omits validator progress, diagnostic bodies, and the pass/fail sentence. Exit codes remain the same: non-zero when errors are present.

---

## Readability

Diagnostic design follows [Developer Experience](#developer-experience). In summary: every error should be actionable without reading source.

Avoid dumping internal stack traces on validation failures.

---

## What the CLI must never do

| Forbidden | Reason |
|-----------|--------|
| Modify normative spec files without explicit `--write` and maintainer intent | Tooling follows spec; spec defines |
| Accept or reject RFCs | Governance in `veritypay-spec` |
| Evaluate payment claims | `veritypay-reference` |
| Run conformance against implementations | `veritypay-conformance` |
| Phone home or require network for core validation | Offline-first against local spec tree |

---

## Relationship to CI

CI should prefer:

```text
vp validate --spec . --format json
```

over invoking internal modules directly—so local and CI paths stay identical.

Reusable workflows (Milestone G) wrap this invocation; they do not fork validation logic.

---

## Evolution

Command names and flags stabilize at **Milestone E**. Before that, breaking CLI changes are expected in pre-1.0 tooling releases.

Changes that affect **what counts as valid spec** require spec governance alignment first, then tooling implementation.

See [ROADMAP.md](ROADMAP.md) for delivery milestones.

---

*The CLI should feel like a standards-body instrument—not a startup script.*
