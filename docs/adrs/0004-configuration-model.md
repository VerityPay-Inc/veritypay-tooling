---
id: ADR-0004
title: Configuration Model
status: accepted
version: 1.0.0
authors:
  - VerityPay Core Team
reviewers: []
related_docs:
  - docs/adrs/0003-validator-execution-model.md
  - docs/CONFIGURATION_ARCHITECTURE.md
  - CLI_PHILOSOPHY.md
  - docs/EDITION_VALIDATION.md
  - ROADMAP.md
decision_date: 2026-06-30
superseded_by: null
---

# ADR-0004 — Configuration Model

**Status:** Accepted · **Version:** 1.0.0 · **Date:** 2026-06-30

**Related:** [ADR-0003](0003-validator-execution-model.md) · [CONFIGURATION_ARCHITECTURE.md](../CONFIGURATION_ARCHITECTURE.md) · [CLI_PHILOSOPHY.md](../../CLI_PHILOSOPHY.md) · [EDITION_VALIDATION.md](../EDITION_VALIDATION.md) · [ROADMAP.md](../../ROADMAP.md)

---

## Context

`veritypay-tooling` now has:

- [VALIDATION_ENGINE.md](../VALIDATION_ENGINE.md) — validation lifecycle and composition
- [ADR-0003](0003-validator-execution-model.md) — engine-orchestrated validator execution
- Registry validation (Milestone B) — VP-TERM and VP-RFC structural checks
- Cross-reference validation (Milestone C) — corpus reference integrity
- Human and JSON CLI output (Milestone C.3)
- Validator identity (`ValidatorInfo`) on the `Validator` trait
- [CONFIGURATION_ARCHITECTURE.md](../CONFIGURATION_ARCHITECTURE.md) — configuration subsystem design

The next implementation milestone is **Milestone C.4**: load validation options from a repository-local **`.vp.toml`** file.

**Milestone D (Edition validation)** follows C.4. Edition validation needs a stable manifest path, profile intent, and output defaults—without accumulating edition-specific CLI flags. Configuration must exist **before** the Edition validator is implemented so it receives `ctx.config.edition` instead of growing the CLI surface.

Today every option is a CLI flag. That worked for early milestones but does not scale:

- CI workflows repeat the same flag bundles on every invocation
- Edition manifest paths and output format sprawl across repositories
- Future options (strict mode, ignore rules, profile selection) have nowhere durable to live

Configuration architecture belongs in [CONFIGURATION_ARCHITECTURE.md](../CONFIGURATION_ARCHITECTURE.md). **This ADR records the engineering decision** to implement that model in Milestone C.4—not user setup guides or Edition manifest policy.

---

## Decision

Adopt a **repository-local `.vp.toml` configuration file** and a first-class immutable **`ValidationConfig`** carried on `ValidationContext`.

### Merge precedence

Configuration sources merge in this order:

```
CLI flags  >  .vp.toml  >  built-in defaults
```

| Rule | Meaning |
|------|---------|
| **CLI overrides config** | Explicit invocation flags win for the same key |
| **Config overrides defaults** | File values apply when the CLI omits a flag |
| **Defaults override nothing** | Missing keys use documented built-in defaults—not error |

### Initial config keys (Milestone C.4)

| Key | Purpose |
|-----|---------|
| **`spec_root`** | Default `veritypay-spec` checkout path |
| **`profile`** | Validation profile name (`fast`, `ci`, `release`)—stored now; profile engine ships later |
| **`output`** | Default output format (`human` \| `json`) |
| **`edition`** | Edition Manifest path relative to spec root—for Milestone D |
| **`strict`** | When true, warnings fail validation—semantics documented when enforced |

Keys live under `[validation]` in `.vp.toml`. Their names are part of the public tooling interface unless superseded by a future ADR.

### Decision details

| Rule | Requirement |
|------|-------------|
| **`.vp.toml` is optional** | Absent file is not an error |
| **Missing config file** | Behavior identical to today—flags and defaults only |
| **Invalid TOML** | User error (exit code **2**) with clear message |
| **Load timing** | Config merged **before** validators run |
| **Validator boundary** | Validators **never parse TOML** or CLI |
| **Observation only** | Validators read **`ValidationContext`** only |
| **Ownership** | **`ValidationContext` owns `ValidationConfig`** |
| **Immutability** | **`ValidationConfig` is immutable during execution** |
| **Not runtime state** | Config is resolved once per run; distinct from diagnostics and reports |
| **No global user config** | Repo-local first (`~/.config/vp/…` deferred) |
| **No environment variables** | Deferred; precedence must stay deterministic when added |
| **No profile engine** | C.4 stores `profile`; profile bundle selection ships later |
| **No Edition validation** | C.4 provides `edition` key only; Milestone D consumes it |

**`ValidationConfig` is a value object.** It contains no behavior beyond representing the resolved validation configuration for a single execution. Equality is based on values; there is no mutable identity, lifecycle, or hidden state—consistent with `SpecRepository`, `ValidationContext`, `ValidatorInfo`, `Diagnostic`, and `Report` as clean domain objects.

> **Institutional principle:** Tooling supports release readiness; maintainers authorize publication.
>
> Configuration convenience local and CI runs—it does **not** authorize publication or alter normative spec validity.

---

## Alternatives considered

### 1. CLI flags only

**Description:** Continue requiring every option on the command line (`--spec`, `--format`, `--edition`, future `--profile`, `--strict`).

**Why rejected:** Does not scale. CI duplicates flag bundles; Edition validation would add more flags; local developers repeat long commands. Profiles and strict mode have no stable home. Violates progressive disclosure without hiding behavior—flags multiply instead of defaults centralizing.

**Assessment:** Adequate for Milestone B–C.3; insufficient before Edition validation and release-profile workflows.

---

### 2. Environment variables

**Description:** Configure via `VP_SPEC_ROOT`, `VP_PROFILE`, etc.

**Why deferred:** Useful for containerized CI where injecting env vars is natural—but **too implicit** for the first configuration layer. Precedence becomes harder to explain (`ENV` vs CLI vs file). Contributors cannot see repo defaults in version control.

**Assessment:** Adopt later with an explicit rank in the precedence chain (documented in a successor ADR or C.4 amendment). Not Milestone C.4.

---

### 3. Global user config

**Description:** Defaults in `~/.config/vp/config.toml` for all repositories.

**Why deferred:** Hides repo-specific intent; spec PR CI should read **repository** defaults so forks and audits reproduce maintainer expectations. Global config is ergonomic for personal aliases—not for institutional merge gates.

**Assessment:** Repo-local `.vp.toml` first; global config reconsidered when multi-repo contributor workflows demand it.

---

### 4. Validator-specific config

**Description:** Each validator crate parses its own config section or flags.

**Why rejected:** Fragments validation behavior; breaks [ADR-0003](0003-validator-execution-model.md) layer ownership. Validators would couple to TOML shapes and CLI parsing; CI could not merge options centrally. Edition path would live on cross-ref or registry crates ad hoc.

**Assessment:** Configuration is a **platform concern** resolved once on `ValidationContext`—not per-validator ad hoc parsing.

---

## Consequences

### Positive

- **Repeatable CI** — repository commits defaults; workflows invoke `vp validate` with minimal flags
- **Simpler local commands** — common paths and formats configured once
- **Cleaner Edition validation later** — Milestone D reads `ctx.config.edition` without new CLI flags
- **Fewer CLI flags** — durable options move to file; flags remain overrides
- **Validators remain isolated** — still observe immutable context only ([ADR-0003](0003-validator-execution-model.md))
- **Future profiles can build on config** — `profile` key reserved; profile engine maps names to validator sets and optional config defaults later

### Negative

- **One more file type** — contributors must discover `.vp.toml` (documented in CLI help and CONFIGURATION_ARCHITECTURE)
- **Precedence must be documented and tested** — CLI vs file vs defaults requires unit tests and clear errors on invalid TOML
- **Misaligned expectations** — users may expect config to authorize behavior it cannot (publish Editions, waive normative rules, accept RFCs)

**Acceptable** because specification tooling optimizes for **reproducible validation sessions** and **horizontal platform growth** before the next validator crate.

---

## Future reconsideration

Revisit this ADR if:

- **Global config** becomes necessary for multi-repo contributor ergonomics
- **Environment variables** are required for CI injection patterns
- **Profiles** require richer structure than a single string key (nested tables, validator sets, config overrides per profile)
- **Edition validation** needs manifest-specific config beyond a path (e.g. draft vs published validation modes)
- A **successor ADR** supersedes this decision with new evidence

Adding configuration sources requires documenting their **deterministic precedence rank**—not undocumented loader shortcuts.

---

## Related decisions

| Document | Relationship |
|----------|--------------|
| [ADR-0003](0003-validator-execution-model.md) | Validators receive immutable context; never parse CLI or TOML |
| [CONFIGURATION_ARCHITECTURE.md](../CONFIGURATION_ARCHITECTURE.md) | Subsystem architecture this ADR operationalizes |
| [EDITION_VALIDATION.md](../EDITION_VALIDATION.md) | Prerequisite C.4; consumes `config.edition` in Milestone D |
| [CLI_PHILOSOPHY.md](../../CLI_PHILOSOPHY.md) | Profiles, composability, design principles |
| [VALIDATION_OUTPUT.md](../VALIDATION_OUTPUT.md) | `output` key maps to format contract |

---

## Follow-up

- [ ] Implement config loader and `ValidationConfig` on `ValidationContext` (Milestone C.4)
- [ ] Unit tests for merge precedence (CLI > file > defaults)
- [ ] Document `.vp.toml` in `vp validate --help` when loader ships
- [ ] Milestone D Edition validator reads `ctx.config.edition` only

---

## Conclusion

**Flags are for overrides. Config is for defaults. Defaults are for sensible out-of-the-box behavior.**

Configuration resolves once before validators run, freezes on `ValidationContext`, and stays separate from runtime findings. Validators keep implementing rules only; the platform owns how each session is parameterized. That sequence—**Configuration (C.4) before Edition validation (D)**—keeps the architecture coherent as the validation platform grows.

---

*Accepted ADRs are historical records. Supersede with a new ADR; do not silently rewrite this decision.*
