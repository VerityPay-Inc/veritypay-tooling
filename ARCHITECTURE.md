# Architecture

**Long-term architecture for the VerityPay specification tooling platform.**

This document describes **components, responsibilities, and boundaries**. It does not specify implementation language, module layout, or algorithms. Those decisions follow milestones in [ROADMAP.md](ROADMAP.md) and ADRs in this repository when code begins.

**Audience:** maintainers, contributors, auditors, and grant reviewers who need to understand what tooling will become—not how it is coded today.

**Upstream dependency:** [`veritypay-spec`](https://github.com/VerityPay-Inc/veritypay-spec) defines paths, registry formats, Edition Manifest shape, and validation rules in prose. Tooling **implements checks** against that source of truth; it does not invent new normative requirements.

---

## Design stance

| Principle | Meaning |
|-----------|---------|
| **Specification upstream** | All validators read from `veritypay-spec`; none write normative text |
| **Fail fast on structure** | Registry and link errors block merge before semantics are implemented |
| **Composable checks** | Each validator is independently invokable and CI-composable |
| **Edition-aware** | Validation can target a working tree or a pinned Edition Manifest |
| **Human-readable output** | Errors cite file, line, rule, and remediation—not stack traces alone |
| **Library + CLI** | Core logic reusable in CI, local dev, and future repos |

---

## System context

```
┌─────────────────────────────────────────────────────────────┐
│                     veritypay-spec                          │
│  docs/ · rfcs/ · spec/terminology · spec/rfcs · manifests   │
└───────────────────────────┬─────────────────────────────────┘
                            │ read-only input
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   veritypay-tooling                         │
│  ┌─────────────┐ ┌──────────────┐ ┌─────────────────────┐  │
│  │  Validators │ │ Edition layer │ │  CLI / CI surface   │  │
│  └─────────────┘ └──────────────┘ └─────────────────────┘  │
└───────────────────────────┬─────────────────────────────────┘
                            │ reports, exit codes, artifacts
                            ▼
              spec PR CI · local dev · future reference/conformance
```

Tooling produces **reports and pass/fail signals**. It does not produce protocol behavior.

---

## Major components

### Registry Validation

**Purpose:** Ensure machine-readable registries in `veritypay-spec` are well-formed, complete, and consistent with documented rules.

**Responsibilities:**

- Validate VP-TERM registry schema and required fields
- Validate VP-RFC registry entries against accepted RFC metadata
- Detect duplicate IDs, orphan entries, and invalid status transitions
- (Future) Compare glossary-derived terms against registry where sync rules exist

**Boundaries:**

- Does **not** define term meanings—that is glossary and architecture text
- Does **not** accept or reject RFCs—that is governance in `veritypay-spec`
- May **report** drift between human-readable glossary and YAML; fixing drift is a spec PR

**Inputs:** `spec/terminology/registry.yaml`, `spec/rfcs/registry.yaml`, optional glossary cross-walk rules documented in spec.

**Outputs:** Structured diagnostics; non-zero exit on failure.

**Design:** [docs/REGISTRY_VALIDATION.md](docs/REGISTRY_VALIDATION.md) · Framework: [docs/VALIDATION_ENGINE.md](docs/VALIDATION_ENGINE.md)

---

### Cross-reference Validation

**Purpose:** Ensure identifiers and links across the specification corpus form a coherent graph.

**Responsibilities:**

- Resolve VP-TERM, VP-RFC, VP-CS, architecture section anchors, and document IDs referenced in prose
- Flag unknown or deprecated references
- Validate `depends_on` / `required_by` style metadata where present
- Detect broken relative links within the spec tree

**Boundaries:**

- Does **not** crawl external URLs as a primary gate in early milestones (optional later)
- Does **not** interpret semantic correctness of prose—only reference integrity
- Defers **Edition pin** resolution to Edition layer when validating a manifest bundle

**Inputs:** Markdown and YAML under `veritypay-spec` (path configurable).

**Outputs:** Per-file, per-reference diagnostics.

---

### Front Matter Validation

**Purpose:** Enforce consistent document metadata for specifications, governance docs, and RFCs.

**Responsibilities:**

- Validate YAML front matter against schemas agreed in spec or tooling ADR
- Check required fields: `spec`, `status`, `version`, RFC fields per VP-RFC-0000, etc.
- Enforce allowed enum values for `status` where documented
- Relate RFC front matter to VP-RFC registry entries

**Boundaries:**

- Does **not** judge document quality or normative correctness
- Schema changes that affect **protocol process** originate in `veritypay-spec` (RFC or governance doc), then tooling implements

**Inputs:** Markdown files with front matter; registry YAML.

**Outputs:** Schema violation reports with field-level detail.

---

### Edition Builder

**Purpose:** Support assembly and verification of **Edition Manifests** as defined in [SPECIFICATION_RELEASE_PROCESS](https://github.com/VerityPay-Inc/veritypay-spec/blob/main/docs/05-governance/SPECIFICATION_RELEASE_PROCESS.md).

**Responsibilities:**

- Validate Edition Manifest structure (required keys, document pins, registry snapshots)
- Verify pinned document paths exist at declared versions or commits
- (Future) Assist drafting manifests from registries and release checklist
- Produce reproducible bundle descriptors for auditors

**Boundaries:**

- Does **not** **publish** an Edition—that is a governance decision in `veritypay-spec`
- Does **not** mutate spec content—only validates or emits manifest artifacts
- Builder **generates drafts**; Maintainers **accept** publication

**Inputs:** Edition Manifest YAML; spec repository at pin.

**Outputs:** Validation report; optional draft manifest artifact.

---

### Documentation Validation

**Purpose:** General specification corpus hygiene beyond ID cross-refs.

**Responsibilities:**

- Internal link integrity (relative paths, anchors)
- Required README / pyramid structure checks where policy exists
- Optional markdown lint rules aligned with institutional style (not subjective editing)
- Detect stale references to withdrawn documents where registry marks them superseded

**Boundaries:**

- Does **not** replace human editorial review
- Does **not** enforce marketing or brand rules (Canon / brand docs are separate)
- Style rules must be **objective** and documented to avoid bikeshedding

**Inputs:** Documentation tree paths.

**Outputs:** Link and policy violation reports.

---

### Future Documentation Generator

**Purpose:** Derive **non-normative** views from machine-readable sources to reduce manual duplication.

**Responsibilities:**

- Generate registry indexes, dependency graphs, or status tables for docs sites
- Emit contributor-facing inventories (RFC list, term index) from YAML
- Support future public website and SPECIFICATION_STATUS automation (vision only)

**Boundaries:**

- Generated output is **informative** unless explicitly promoted through spec governance
- Generator must never become the **authoritative** source of definitions
- Human-readable glossary and RFCs remain canonical; generated pages are derivatives

**Inputs:** Registries, manifests, optional templates.

**Outputs:** Markdown or static site fragments; clearly marked as generated.

---

## CLI and CI surface (conceptual)

The **CLI** is the user-facing orchestration layer. It does not duplicate validator logic—it **routes** to components above.

| Concern | Role |
|---------|------|
| **Discoverability** | `vp --help`, grouped subcommands |
| **Composition** | Run subset of checks; aggregate exit code |
| **CI integration** | Stable JSON/text formats; versioned output schema |
| **Local ergonomics** | Point at local `veritypay-spec` clone or remote pin |

Detailed principles: [CLI_PHILOSOPHY.md](CLI_PHILOSOPHY.md).

---

## Boundaries with sibling repositories

| Repository | Interaction |
|------------|-------------|
| **veritypay-spec** | Primary input; validation targets spec PRs |
| **veritypay-reference** | May reuse report parsers or shared validation contracts documented by ADR; does not own validation |
| **veritypay-conformance** | Consumes VP-CS definitions from spec; may invoke tooling in CI for spec pin integrity |
| **Organization `.github`** | May host reusable workflows that call `vp validate` (Milestone G) |

---

## Non-goals (architectural)

- **Protocol interpreter** — belongs in `veritypay-reference`
- **Conformance oracle** — belongs in `veritypay-conformance`
- **Normative schema authority** — lives in spec + accepted RFCs; tooling implements snapshots
- **Single monolithic "spec linter"** — prefer composable validators with clear ownership

---

## Evolution

Architecture will be refined through **ADRs in this repository** once implementation starts. Structural changes that affect how spec validity is defined require alignment with `veritypay-spec` governance—not silent tooling releases.

**Current maturity:** Scaffold — components described; no implementation yet.

See [ROADMAP.md](ROADMAP.md) for delivery order.
