# veritypay-tooling

**Specification hygiene for the VerityPay protocol ecosystem.**

This repository is part of the **Verity Specification Platform**. It maintains, validates, and publishes support for the specification—it does **not** implement protocol behavior.

**Repository maturity:** **Scaffold** — architecture and roadmap documented; validators not yet implemented.

---

## What is veritypay-tooling?

`veritypay-tooling` is the **engineering infrastructure** that keeps the VerityPay specification **internally consistent, machine-checkable, and Edition-ready**.

It provides CLI and library tooling run in CI against [`veritypay-spec`](https://github.com/veritypay/veritypay-spec): registry validation, cross-reference checks, front matter linting, Edition manifest validation, and (eventually) documentation generation.

The tooling **follows** the specification. It never **defines** it.

---

## Why does it exist?

Phase I produced a rich specification corpus: constitutional layer, Architecture Alpha, governance process, VP-TERM and VP-RFC registries, and conformance scenarios in prose. That work answers *what VerityPay is*.

Phase II requires the specification to become a **platform**—validated before it is executed, traceable before it is cited, and reproducible when published as an Edition.

Manual review does not scale. Broken cross-links, orphan registry IDs, and invalid RFC metadata are **specification defects** that should fail CI—not become interpreter bugs or grant-review surprises.

`veritypay-tooling` exists so that:

- Contributors merge against a **verified corpus**
- Maintainers can trust **registry and link integrity**
- Genesis Edition publication has **automated readiness checks**
- Downstream repos (`veritypay-reference`, `veritypay-conformance`) build on **validated** upstream text

See [Phase II Platform Plan](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/PHASE_II_PLATFORM_PLAN.md) in `veritypay-spec`.

---

## Relationship to veritypay-spec

```
veritypay-spec          ← source of truth (normative text, registries, RFCs)
       ↓ consumed by
veritypay-tooling         ← validates and supports publication (this repo)
       ↓ enables
veritypay-reference       ← executable semantics (future)
veritypay-conformance     ← VP-CS runners (future)
```

| Responsibility | `veritypay-spec` | `veritypay-tooling` |
|----------------|------------------|---------------------|
| Protocol meaning | Yes | No |
| Registry YAML and document text | Yes | Reads; does not author normatively |
| Validation rules | Documented or implied | Implemented here |
| Edition Manifest (when published) | Issued as artifact | Validates structure |
| CI on spec PRs | May invoke this repo | Provides checks |

When tooling and specification disagree on **what is valid**, the specification wins. Tooling is updated—not the protocol.

---

## Problems this repository will solve

| Problem | Tooling response |
|---------|------------------|
| Registry drift (glossary vs `registry.yaml`) | Registry validation |
| Unknown VP-TERM / VP-RFC references in docs | Cross-reference validation |
| Inconsistent RFC front matter | Front matter validation |
| Broken internal links | Documentation validation |
| Edition bundle not reproducible | Edition builder / validator |
| Manual manifest assembly error-prone | Edition validation and (future) generation |
| Contributors unsure if a spec PR is "green" | Unified `vp validate` in CI |

---

## What this repository intentionally does NOT do

| Out of scope | Where it belongs |
|--------------|------------------|
| Protocol semantics or claim evaluation | `veritypay-reference` |
| Conformance pass/fail for implementations | `veritypay-conformance` |
| Normative specification edits | `veritypay-spec` via RFC |
| SDKs or integrator APIs | Future `veritypay-sdk-*` |
| Production applications | Product repositories |
| Defining VP-CS scenario meaning | `veritypay-spec` ([CONFORMANCE_MODEL](https://github.com/veritypay/veritypay-spec/blob/main/docs/03-development/CONFORMANCE_MODEL.md)) |

If a change alters **what the protocol means**, it belongs in an RFC—not in this repository.

---

## Planned capabilities

Capabilities are delivered **capability-based** per [ROADMAP.md](ROADMAP.md)—not on a fixed calendar.

| Capability | Description | Milestone |
|------------|-------------|-----------|
| Registry validation | VP-TERM, VP-RFC schema and consistency | B |
| Cross-reference validation | IDs, links, dependency graph | C |
| Front matter validation | Document and RFC metadata rules | B–C |
| Edition validation | Edition Manifest structure and pins | D |
| Documentation validation | Internal links, required sections | C |
| CLI (`vp`) | Discoverable commands for local and CI use | E |
| Documentation generation | Derived views from registries (future) | F |
| Public automation | Reusable CI workflows, org integration | G |

Command philosophy: [CLI_PHILOSOPHY.md](CLI_PHILOSOPHY.md).  
Long-term structure: [ARCHITECTURE.md](ARCHITECTURE.md).

---

## Repository layout (planned)

This repository is at **Scaffold** maturity. Layout will evolve as milestones land.

```
veritypay-tooling/
├── README.md              ← You are here
├── ARCHITECTURE.md        ← Component boundaries (conceptual)
├── ROADMAP.md             ← Capability milestones A–G
├── CLI_PHILOSOPHY.md      ← Future `vp` CLI design principles
├── CONTRIBUTING.md        ← How to contribute to tooling
└── (implementation)       ← Future: src/, packages/, workflows/
```

No validators are implemented yet. Milestone A is **architectural clarity**, not functionality.

---

## Links to veritypay-spec

| Resource | Location |
|----------|----------|
| Specification home | [veritypay-spec](https://github.com/veritypay/veritypay-spec) |
| Specification status | [SPECIFICATION_STATUS.md](https://github.com/veritypay/veritypay-spec/blob/main/SPECIFICATION_STATUS.md) |
| Phase II platform plan | [PHASE_II_PLATFORM_PLAN.md](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/PHASE_II_PLATFORM_PLAN.md) |
| Release process | [SPECIFICATION_RELEASE_PROCESS.md](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_RELEASE_PROCESS.md) |
| Versioning policy | [SPECIFICATION_VERSIONING.md](https://github.com/veritypay/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_VERSIONING.md) |
| VP-TERM registry | [spec/terminology/registry.yaml](https://github.com/veritypay/veritypay-spec/blob/main/spec/terminology/registry.yaml) |
| VP-RFC registry | [spec/rfcs/registry.yaml](https://github.com/veritypay/veritypay-spec/blob/main/spec/rfcs/registry.yaml) |
| RFC process | [VP-RFC-0000](https://github.com/veritypay/veritypay-spec/blob/main/rfcs/0000-rfc-process.md) |
| Contributing (spec) | [CONTRIBUTING.md](https://github.com/veritypay/veritypay-spec/blob/main/CONTRIBUTING.md) |

---

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md). You are building **engineering infrastructure**, not protocol behavior.

---

## License

See [LICENSE](LICENSE).
