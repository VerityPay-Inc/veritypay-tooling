# Contributing to veritypay-tooling

**Handbook for contributors building specification engineering infrastructure.**

You are not implementing the VerityPay protocol here. You are building the **tools that keep the specification honest**—registries validated, references resolved, Editions reproducible, and CI trustworthy.

Read this before opening a pull request.

---

## Welcome

Contributing to `veritypay-tooling` means strengthening **public infrastructure** around [`veritypay-spec`](https://github.com/veritypay/veritypay-spec).

This repository exists because:

- **Specification defects** (broken links, orphan IDs, invalid metadata) should fail fast in CI—not become interpreter bugs
- **Genesis Edition** publication requires automated readiness checks
- **Auditors and grant reviewers** need evidence that the corpus is machine-verifiable
- **Downstream repos** (`veritypay-reference`, `veritypay-conformance`) deserve a validated upstream

We welcome engineers, technical writers, and CI specialists. You do not need permission to read, propose issues, or draft ADRs. You **do** need to respect the boundary: **tooling follows the specification; it never defines it.**

---

## Before you start

| Order | Document | Why |
|-------|----------|-----|
| 1 | [README.md](README.md) | Purpose, boundaries, maturity |
| 2 | [ARCHITECTURE.md](ARCHITECTURE.md) | Component model |
| 3 | [ROADMAP.md](ROADMAP.md) | Current milestone and success criteria |
| 4 | [CLI_PHILOSOPHY.md](CLI_PHILOSOPHY.md) | Future CLI behavior |
| 5 | [veritypay-spec — SPECIFICATION_STATUS](https://github.com/veritypay/veritypay-spec/blob/main/SPECIFICATION_STATUS.md) | Ecosystem maturity |
| 6 | [veritypay-spec — Phase II Platform Plan](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/PHASE_II_PLATFORM_PLAN.md) | Where tooling sits in the platform |

For protocol and governance context, read [veritypay-spec — CONTRIBUTING](https://github.com/veritypay/veritypay-spec/blob/main/CONTRIBUTING.md) when your work touches validation **rules** defined there.

---

## The golden rule

> **Changes to protocol semantics belong in `veritypay-spec` through RFCs.**  
> **Tooling implements checks against accepted specification—it never invents the protocol.**

| If you want to… | Do this |
|-----------------|---------|
| Change what VP-TERM-012 **means** | RFC or glossary change in `veritypay-spec` |
| Add a field to the VP-RFC registry schema **as normative policy** | Governance or RFC in `veritypay-spec`, then implement validator here |
| Detect invalid registry YAML | Pull request in **this** repository |
| Add a new `vp` subcommand | ADR + pull request here |
| Change Architecture Alpha | RFC in `veritypay-spec` ([GOVERNANCE](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/GOVERNANCE.md)) |

When unsure, **default to spec governance first.**

---

## What belongs in this repository

| In scope | Examples |
|----------|----------|
| Validators | Registry, cross-ref, front matter, edition, docs |
| CLI | `vp` commands per [CLI_PHILOSOPHY.md](CLI_PHILOSOPHY.md) |
| CI integration | Workflows, reusable actions, exit codes |
| ADRs | Language choice, output format, module boundaries |
| Tests | Fixture spec trees, golden error messages |
| Docs | README, architecture, roadmap updates |

## What does not belong here

| Out of scope | Belongs in |
|--------------|------------|
| Normative protocol text | `veritypay-spec` |
| RFC proposals | `veritypay-spec/rfcs/` |
| Reference interpreter | `veritypay-reference` |
| VP-CS execution | `veritypay-conformance` |
| SDKs | Future `veritypay-sdk-*` |
| Product applications | Product repos |

---

## Contribution workflow

### 1. Find or file an issue

Issues should reference a **roadmap milestone** (A–G) when applicable. Good first issues appear after Milestone B begins.

Include:

- Which validator or CLI surface is affected
- Whether spec governance must change first
- Expected failure messages (for validation features)

### 2. Architectural decisions

Non-trivial choices (language, crate layout, JSON schema version) require an **ADR** in this repository before large implementation merges.

Follow the spirit of [veritypay-spec — ADR Guide](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/ADR_GUIDE.md). ADRs here are **tooling decisions**, not protocol law.

### 3. Pull requests

| Requirement | Detail |
|-------------|--------|
| **Scope** | One milestone-sized concern per PR when possible |
| **Tests** | Required once code exists; fixture-based against spec snippets |
| **Docs** | Update README/ARCHITECTURE/ROADMAP if behavior or boundaries change |
| **No spec edits** | Do not bundle normative spec changes unless coordinated and labeled |
| **CI** | Must pass; new checks documented |

### 4. Review expectations

Reviewers ask:

- Does this PR define protocol behavior? (If yes, reject or split.)
- Are errors actionable per CLI philosophy?
- Is validation composable and Edition-aware where needed?
- Does spec main (or documented fixture) pass?

---

## Contributor levels (tooling)

| Level | Activity |
|-------|----------|
| **0 — Reader** | Run docs locally; comment on issues |
| **1 — Hygiene** | Typos, docs, issue triage, fixture spec trees |
| **2 — Validator** | Implement checks for Milestones B–D |
| **3 — CLI / CI** | Milestones E–G; reusable automation |
| **4 — Maintainer** | Roadmap, releases, ADR acceptance, spec coordination |

Start at the level matching your first merged contribution. Protocol RFC authorship is **not** required to contribute here.

---

## Phase II contributor policy

During Phase II, **protocol architecture remains frozen** unless changed through RFC in `veritypay-spec`.

**Encouraged here:**

- Tooling, tests, CI, examples of invalid spec trees
- Documentation and ADRs
- Performance and ergonomics of validators

**Not encouraged without RFC:**

- Workarounds that encode undeclared protocol rules in validators
- "Fixing" spec text from tooling PRs instead of spec PRs

---

## Code of conduct

Participate respectfully. Specification and tooling communities share contributors— uphold the same institutional standards described in `veritypay-spec` governance when adopted.

Security issues in tooling (path traversal on spec trees, credential leaks in CI) should be reported privately per organization policy when available; otherwise open a confidential security issue.

---

## License

Contributions are accepted under the same license as this repository. See [LICENSE](LICENSE).

By contributing, you agree your contributions may be used under that license.

---

## Questions

| Question | Where |
|----------|-------|
| What milestone are we on? | [ROADMAP.md](ROADMAP.md) |
| What does a component do? | [ARCHITECTURE.md](ARCHITECTURE.md) |
| How should `vp` behave? | [CLI_PHILOSOPHY.md](CLI_PHILOSOPHY.md) |
| What is the protocol? | [veritypay-spec](https://github.com/veritypay/veritypay-spec) |
| How do I change the protocol? | [VP-RFC-0000](https://github.com/veritypay/veritypay-spec/blob/main/rfcs/0000-rfc-process.md) |

---

*Build instruments that make the specification trustworthy—not shortcuts that replace it.*
