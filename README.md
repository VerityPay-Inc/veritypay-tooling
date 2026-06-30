# veritypay-tooling

**Specification hygiene for the VerityPay protocol ecosystem.**

This repository is part of the **Verity Specification Platform**. It maintains, validates, and publishes support for the specification—it does **not** implement protocol behavior.

**Repository maturity:** **Scaffold** (Milestone A) — architecture and roadmap documented; validators not yet implemented.

---

## Documentation

| Document | Description |
|----------|-------------|
| [README.md](README.md) | Purpose, boundaries, and links to `veritypay-spec` *(this file)* |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Long-term component model—conceptual, not implementation |
| [ROADMAP.md](ROADMAP.md) | Capability milestones A–G with success criteria |
| [CLI_PHILOSOPHY.md](CLI_PHILOSOPHY.md) | Future `vp` CLI principles *(illustrative commands only)* |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute to tooling infrastructure |
| [LICENSE](LICENSE) | License terms for this repository |
| [docs/VALIDATION_ENGINE.md](docs/VALIDATION_ENGINE.md) | Shared validator lifecycle and composition (Milestone B design) |
| [docs/REGISTRY_VALIDATION.md](docs/REGISTRY_VALIDATION.md) | Registry validator architecture (VP-TERM, VP-RFC) |
| [docs/adrs/README.md](docs/adrs/README.md) | Architecture Decision Records |
| [docs/adrs/0001-tooling-implementation-language.md](docs/adrs/0001-tooling-implementation-language.md) | ADR-0001 — Rust implementation language |

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
├── ARCHITECTURE.md        ← [Component boundaries](ARCHITECTURE.md) (conceptual)
├── ROADMAP.md             ← [Capability milestones A–G](ROADMAP.md)
├── CLI_PHILOSOPHY.md      ← [Future `vp` CLI](CLI_PHILOSOPHY.md) design principles
├── CONTRIBUTING.md        ← [How to contribute](CONTRIBUTING.md)
├── LICENSE                ← [License terms](LICENSE)
├── docs/
│   ├── VALIDATION_ENGINE.md    ← [Validation framework](docs/VALIDATION_ENGINE.md)
│   ├── REGISTRY_VALIDATION.md  ← [Registry validator](docs/REGISTRY_VALIDATION.md)
│   └── adrs/                   ← [Architecture Decision Records](docs/adrs/README.md)
└── (implementation)       ← Future: src/, packages/, workflows/ — not in Milestone A
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
