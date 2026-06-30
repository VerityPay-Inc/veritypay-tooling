# CLI Philosophy

**Design principles for the future `vp` command-line interface.**

This document describes **how** the VerityPay tooling CLI should behave—not exact flag syntax, parser choice, or implementation layout. Illustrative commands show intent; final spelling may evolve through ADRs and Milestone E.

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

## Command groups (illustrative)

Commands are grouped by **user intent**, not internal module names.

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
| CI JSON output | `--format json` (contract versioned) |
| Human pretty output | default TTY; optional `--quiet` for logs-only failures |
| Config file | optional `.vp.toml` for paths and edition pin—local convenience only |

Exit codes:

- `0` — all requested checks passed
- `1` — validation failures
- `2` — user error (bad paths, bad flags)
- `3+` — reserved for internal errors (documented)

---

## Readability

Errors should answer:

1. **What** failed (rule id or check name)
2. **Where** (file, line, column if applicable)
3. **Why** (one sentence)
4. **How to fix** (when non-obvious)

Good:

```text
error[vp-term-unknown]: unknown concept ID VP-TERM-9999
  --> docs/01-architecture/DOMAIN_MODEL.md:142:18
  help: add term to spec/terminology/registry.yaml or fix typo
```

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
