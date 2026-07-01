# Validation output formats

This document defines the stable output contract for `vp validate`.

Human output is optimized for local development. JSON output is optimized for CI and automation. Both formats are produced from the same engine result; only presentation differs.

## CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--format human` | yes | Grouped diagnostics, validator progress, and summary |
| `--format json` | | Machine-readable JSON (see schema below) |
| `--quiet` | no | Human format only: print summary counts, omit progress and diagnostics |

Exit codes are unchanged across formats:

| Code | Meaning |
|------|---------|
| `0` | Validation passed |
| `1` | Validation failed |
| `2` | User error (invalid `--spec` path or flags) |

## JSON schema

Top-level object:

```json
{
  "summary": {
    "errors": 2,
    "warnings": 1,
    "info": 0
  },
  "diagnostics": []
}
```

### `summary`

| Field | Type | Description |
|-------|------|-------------|
| `errors` | integer | Count of `error` severity findings |
| `warnings` | integer | Count of `warning` severity findings |
| `info` | integer | Count of `info` severity findings |

### `diagnostics`

Array of diagnostic objects in deterministic order (category, file, line, rule id).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `severity` | string | yes | One of: `error`, `warning`, `info` |
| `rule_id` | string | yes | Stable rule identifier (e.g. `vp-crossref-broken-link`) |
| `title` | string | yes | Human-readable rule title (e.g. `Broken Link`) |
| `description` | string | yes | Short description of what the rule checks |
| `category` | string | yes | One of: `registry`, `cross_reference`, `metadata`, `edition`, `documentation`, `future` |
| `message` | string | yes | Instance-specific finding description |
| `file` | string | no | Relative path within the spec root |
| `location` | object | no | Source or YAML location |
| `suggestion` | string | no | Remediation hint when available |
| `help` | string | no | Extended guidance when available |
| `note` | string | no | Additional context when available |
| `related` | string | no | Related rule id or doc reference when available |

### `location`

| Field | Type | Description |
|-------|------|-------------|
| `line` | integer | 1-based line number in `file` |
| `column` | integer | 1-based column number in `file` |
| `path` | string | YAML path (e.g. `terms[0].id`) when not line-based |

Omitted optional fields are not present in JSON output.

## Stability

Field names and enum string values in this document are part of the CI contract. Additive changes (new optional fields) may appear in minor tooling releases. Breaking changes require a documented migration and semver bump in tooling releases.

## Examples

Human (default):

```text
Running validators...

✓ RFC Registry
✓ Terminology Registry
✓ Cross References

────────────────────────────

Validation Summary

Errors:   0
Warnings: 0
Info:     0

Validation passed.
```

Quiet:

```text
Validation Summary

Errors: 0
Warnings: 0
Info: 0
```

JSON:

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
