# Configuration Architecture

**Architecture for Milestone C.4: centralized validation options via `.vp.toml`.**

This document defines the **configuration subsystem**—how **`vp`** loads, merges, and exposes validation options before Milestone D (Edition validation) and validation profiles add more surface area. It does not specify parser libraries, config file discovery algorithms, or user-facing setup guides.

For CLI principles, see [CLI_PHILOSOPHY.md](../CLI_PHILOSOPHY.md). For the validation engine lifecycle, see [VALIDATION_ENGINE.md](VALIDATION_ENGINE.md). For Edition manifest checks (after C.4), see [EDITION_VALIDATION.md](EDITION_VALIDATION.md).

---

## Purpose

Today, every validation option is a **CLI flag**. That works for early milestones but does not scale:

- CI workflows repeat the same flag bundles on every invocation
- Edition manifest paths, profiles, and output format sprawl across repositories
- Future options (ignore rules, strict mode, local overrides) have nowhere stable to live

Configuration **centralizes durable choices** so `vp validate` can run with minimal arguments while remaining explicit and overrideable.

**Boundary:** Configuration selects **how and against what** validation runs. It does **not** change normative spec meaning, publish Editions, or replace governance authorization.

> **Institutional principle:** Tooling supports release readiness; maintainers authorize publication.
>
> Config files convenience local and CI runs—they do not authorize publication or alter what counts as valid spec.

---

## Configuration identity

A **Validation Configuration** represents the complete set of inputs that determine how a validation session executes.

It contains:

| Component | Role |
|-----------|------|
| **Specification root** | Path to the `veritypay-spec` checkout under validation |
| **Validation profile** | Named intent for validation depth (`fast`, `ci`, `release`, …) |
| **Output configuration** | Human vs JSON format and related presentation options |
| **Edition selection** | Path to Edition Manifest when Edition validation applies |
| **Strictness policy** | Whether warnings fail the run (future CI strict mode) |

A Validation Configuration is **immutable during execution**. Validators **observe** configuration; they **never modify** it.

**Configuration is not runtime state.** It is resolved once before validators run and frozen on `ValidationContext`. Do not conflate it with:

| Concept | Role |
|---------|------|
| **Configuration** | Inputs that select *how* validation runs |
| **Execution context** | Immutable snapshot (spec root, config, repositories) for one run |
| **Diagnostics** | Findings emitted by validators |
| **Reports** | Aggregated diagnostics and outcomes from the engine |

---

## Position in the platform

```
.vp.toml (optional)
       ↓
Config loader (vp-core or vp-cli)
       ↓
ValidationConfig merged with CLI flags
       ↓
ValidationContext { spec_root, config, … }
       ↓
Validators (read config; emit diagnostics)
       ↓
Engine → CLI
```

Configuration sits **between** CLI parsing and the validation engine—alongside `SpecRepository`, not inside individual validators' ad hoc flag parsing.

| Layer | Responsibility |
|-------|----------------|
| **`.vp.toml`** | Repository-local defaults for spec root, profile, output, edition path, strict mode |
| **CLI flags** | Override config for this invocation |
| **Built-in defaults** | Apply when neither config nor flag supplies a value |
| **Validators** | Read frozen `ValidationContext`; never parse CLI or TOML directly |

This keeps the platform growing **horizontally**—shared infrastructure before the next validator crate.

---

## Merge precedence

```
CLI flags  >  .vp.toml  >  built-in defaults
```

| Rule | Meaning |
|------|---------|
| **CLI overrides config** | `--spec ./other` wins over `spec_root` in `.vp.toml` |
| **Config overrides defaults** | `[validation] profile = "ci"` applies when `--profile` omitted |
| **Defaults override nothing** | Missing keys mean "use documented default"—not error |

Config is **optional**. `vp validate --spec ../veritypay-spec` must continue to work with zero config file.

Future versions may introduce additional configuration sources (environment variables, organization defaults, workspace settings), provided they preserve a **deterministic precedence model**. The merge order above is the minimum contract; new layers document their rank explicitly in a Milestone ADR.

---

## Supported keys (initial Milestone C.4)

Illustrative `.vp.toml` shape—**not** a frozen public API until Milestone C.4 ADR is accepted:

```toml
[validation]
spec_root = "../veritypay-spec"
profile = "ci"
output = "human"      # human | json
edition = "editions/genesis-edition.yaml"
strict = false
```

| Key | Type | Purpose |
|-----|------|---------|
| **`spec_root`** | path | Default `veritypay-spec` checkout (maps to `--spec`) |
| **`profile`** | string | Validation profile intent: `fast`, `ci`, `release` (future; stored now, applied when profiles ship) |
| **`output`** | string | Default output format: `human` or `json` (maps to `--format`) |
| **`edition`** | path | Edition Manifest path relative to spec root (future Edition validator input) |
| **`strict`** | bool | When true, warnings fail validation (future CI strict mode) |

Keys not present are omitted from merged config—not interpreted as empty strings or false unless documented.

### Illustrative usage

**Today (flags only):**

```text
vp validate \
  --spec ../veritypay-spec \
  --edition editions/genesis-edition.yaml \
  --profile release \
  --format json
```

**Tomorrow (config + minimal CLI):**

```toml
# .vp.toml at repo root
[validation]
spec_root = "."
profile = "release"
output = "json"
edition = "editions/genesis-edition.yaml"
```

```text
vp validate
```

**Override for one run:**

```text
vp validate --profile ci
```

---

## Discovery scope

### In scope (initial Milestone C.4)

| Concern | Direction |
|---------|-----------|
| Load `.vp.toml` from spec root or cwd (policy in C.4 ADR) | One file; no config search path maze |
| Parse `[validation]` table | Keys above |
| Merge with CLI | Precedence rules |
| Pass merged config into `ValidationContext` | Validators read `ctx.config` |
| Unknown keys | Warn or ignore per ADR—never silent misconfiguration |

### Out of scope (initial Milestone C.4)

| Concern | Reason |
|---------|--------|
| **Validation profiles implementation** | Documented in [CLI_PHILOSOPHY.md](../CLI_PHILOSOPHY.md); config stores profile name only until profiles ship |
| **Edition validator** | Milestone D; config provides `edition` path only |
| **Global user config** (`~/.config/vp/…`) | Future; repo-local first |
| **Environment variable layer** | Future; keep C.4 minimal |
| **Secrets or credentials** | Never in `.vp.toml` |
| **Normative spec content** | Config is tooling convenience only |

---

## Relationship to other milestones

| Milestone | How config helps |
|-----------|------------------|
| **C.3 CLI UX** | `--format`, `--quiet` remain; config supplies defaults |
| **C.4 Configuration** | This document |
| **Profiles (future)** | `profile = "ci"` selects validator bundle—no `--skip-*` sprawl |
| **D Edition validation** | `edition = "…"` → `ctx.config.edition` instead of new CLI flags |
| **E CLI polish** | Single `vp validate` entrypoint for CI |

Validation profiles and configuration are related but distinct. A profile selects **which validators run**; configuration selects **session inputs** (spec root, output, edition path, strictness). **Profiles may define default configuration values in future releases**—for example, `profile = "release"` could imply `strict = true`—without merging the two concepts in Milestone C.4.

**Recommended sequence:**

```
Validation Engine → Registries → Cross References → CLI UX
       ↓
Configuration (.vp.toml)     ← Milestone C.4
       ↓
Edition Validation         ← Milestone D
       ↓
Reference Interpreter      ← later
```

Edition validation should receive **`ctx.config.edition`**, not accumulate edition-specific flags on the CLI.

---

## Success criteria (Milestone C.4)

| # | Criterion |
|---|-----------|
| 1 | `.vp.toml` with `[validation]` loads without error when present |
| 2 | Missing config file does not change current CLI behavior |
| 3 | CLI flags override config values for the same key |
| 4 | `spec_root`, `output`, `edition`, `profile`, `strict` available on `ValidationContext` (or nested `ValidationConfig`) |
| 5 | `vp validate --spec X` still works with no config—no regression |
| 6 | Invalid TOML or unknown section fails with exit code 2 and clear message |
| 7 | Unit tests cover merge precedence (CLI > file > default) |

**Not included in Milestone C.4:** profile engine, Edition validator, global config, env vars, config schema versioning beyond documented keys.

---

## Governance alignment

| Topic | Authority |
|-------|-----------|
| **Config file name and location** | Tooling ADR (C.4) |
| **What config may NOT do** | Cannot accept RFCs, publish Editions, or waive normative rules |
| **Strict mode semantics** | Tooling + spec maintainer policy |
| **Edition path meaning** | [EDITION_VALIDATION.md](EDITION_VALIDATION.md) + spec governance |

---

## Related documents

| Document | Relationship |
|----------|--------------|
| [CLI_PHILOSOPHY.md](../CLI_PHILOSOPHY.md) | Validation profiles, composability, config mention |
| [VALIDATION_ENGINE.md](VALIDATION_ENGINE.md) | `ValidationContext` extension |
| [EDITION_VALIDATION.md](EDITION_VALIDATION.md) | Consumes `config.edition` in Milestone D |
| [VALIDATION_OUTPUT.md](VALIDATION_OUTPUT.md) | `output` key maps to format contract |
| [ROADMAP.md](../ROADMAP.md) | Milestone C.4 delivery |
| [ADR-0003](adrs/0003-validator-execution-model.md) | Validators receive immutable context |

---

> **Design principle:** Flags are for overrides. Config is for defaults. Defaults are for sensible out-of-the-box behavior.
