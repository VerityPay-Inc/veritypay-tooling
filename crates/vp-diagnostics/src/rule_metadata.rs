//! Human-readable metadata for validation rules.

use crate::{RuleId, RuleKind, RuleScope, Severity};

/// Presentation metadata for a validation rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleMetadata {
    pub title: &'static str,
    pub description: &'static str,
    pub default_severity: Severity,
}

impl RuleId {
    /// Human-readable title for CLI and CI output.
    pub fn title(self) -> &'static str {
        metadata(self).title
    }

    /// Short description of what the rule checks.
    pub fn description(self) -> &'static str {
        metadata(self).description
    }

    /// Default severity when the rule fires.
    pub fn default_severity(self) -> Severity {
        metadata(self).default_severity
    }

    /// Full metadata bundle for renderers.
    pub fn metadata(self) -> RuleMetadata {
        metadata(self)
    }
}

fn metadata(rule: RuleId) -> RuleMetadata {
    match (rule.scope, rule.kind) {
        (RuleScope::RfcRegistry, RuleKind::RegistryMissing) => RuleMetadata {
            title: "RFC Registry Missing",
            description: "The RFC registry file is absent from the spec layout.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::RegistryYamlInvalid) => RuleMetadata {
            title: "Invalid RFC Registry YAML",
            description: "The RFC registry file is not valid YAML or has an unexpected shape.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::TopLevelMissingField) => RuleMetadata {
            title: "RFC Registry Missing Field",
            description: "A required top-level field is missing from the RFC registry.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::EmptyList) => RuleMetadata {
            title: "Empty RFC List",
            description: "The RFC registry contains no RFC entries.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::EntryMissingField) => RuleMetadata {
            title: "RFC Entry Missing Field",
            description: "An RFC entry is missing a required field.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::InvalidId) => RuleMetadata {
            title: "Invalid RFC ID",
            description: "An RFC id does not match the VP-RFC-NNNN pattern.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::DuplicateId) => RuleMetadata {
            title: "Duplicate RFC ID",
            description: "The same RFC id appears more than once in the registry.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::DuplicateTitle) => RuleMetadata {
            title: "Duplicate RFC Title",
            description: "The same RFC title appears more than once in the registry.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::IdNumberMismatch) => RuleMetadata {
            title: "RFC ID Number Mismatch",
            description: "An RFC id number does not match its rfc field.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::UnknownStatus) => RuleMetadata {
            title: "Unknown RFC Status",
            description: "An RFC status value is not in the allowed set.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::InvalidVersion) => RuleMetadata {
            title: "Invalid RFC Version",
            description: "An RFC version field is not valid semver.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::MissingPath) => RuleMetadata {
            title: "RFC Path Missing",
            description: "An RFC registry path does not exist under the spec root.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::UnknownReference) => RuleMetadata {
            title: "Unknown RFC Reference",
            description: "An RFC entry references another RFC id that is not in the registry.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::InvalidNormativeDefinition) => RuleMetadata {
            title: "Invalid Normative Definition",
            description: "An RFC normative definition block has an invalid shape.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::InvalidReferencedBy) => RuleMetadata {
            title: "Invalid Referenced By",
            description: "An RFC referenced_by field is not a valid sequence.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, RuleKind::InvalidSectionId) => RuleMetadata {
            title: "Invalid Section ID",
            description: "An RFC section id uses an unrecognized architecture prefix.",
            default_severity: Severity::Error,
        },
        (RuleScope::RfcRegistry, _) => unmapped(RuleScope::RfcRegistry),

        (RuleScope::TermRegistry, RuleKind::RegistryMissing) => RuleMetadata {
            title: "Terminology Registry Missing",
            description: "The terminology registry file is absent from the spec layout.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::RegistryYamlInvalid) => RuleMetadata {
            title: "Invalid Terminology Registry YAML",
            description: "The terminology registry file is not valid YAML or has an unexpected shape.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::TopLevelMissingField) => RuleMetadata {
            title: "Terminology Registry Missing Field",
            description: "A required top-level field is missing from the terminology registry.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::EmptyList) => RuleMetadata {
            title: "Empty Term List",
            description: "The terminology registry contains no term entries.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::EntryMissingField) => RuleMetadata {
            title: "Term Entry Missing Field",
            description: "A term entry is missing a required field.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::InvalidId) => RuleMetadata {
            title: "Invalid Term ID",
            description: "A term id does not match the VP-TERM-NNN pattern.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::DuplicateId) => RuleMetadata {
            title: "Duplicate Term ID",
            description: "The same term id appears more than once in the registry.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::DuplicateTitle) => RuleMetadata {
            title: "Duplicate Term Title",
            description: "The same term title appears more than once in the registry.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::IdNumberMismatch) => RuleMetadata {
            title: "Term ID Number Mismatch",
            description: "A term id number does not match its documented identifier.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::UnknownStability) => RuleMetadata {
            title: "Unknown Term Stability",
            description: "A term stability value is not in the allowed set.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::InvalidVersion) => RuleMetadata {
            title: "Invalid Term Registry Version",
            description: "The terminology registry version field is not valid semver.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::MissingPath) => RuleMetadata {
            title: "Term Path Missing",
            description: "A term registry path does not exist under the spec root.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::UnknownReference) => RuleMetadata {
            title: "Unknown Term Reference",
            description: "A term entry references another term id that is not in the registry.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::InvalidNormativeDefinition) => RuleMetadata {
            title: "Invalid Normative Definition",
            description: "A term normative definition block has an invalid shape.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::InvalidReferencedBy) => RuleMetadata {
            title: "Invalid Referenced By",
            description: "A term referenced_by field is not a valid sequence.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::InvalidSectionId) => RuleMetadata {
            title: "Invalid Section ID",
            description: "A term section id uses an unrecognized architecture prefix.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, RuleKind::UnknownStatus) => RuleMetadata {
            title: "Unknown Term Status",
            description: "A term status value is not in the allowed set.",
            default_severity: Severity::Error,
        },
        (RuleScope::TermRegistry, _) => unmapped(RuleScope::TermRegistry),

        (RuleScope::CrossReference, RuleKind::UnknownTerm) => RuleMetadata {
            title: "Unknown Term Reference",
            description: "A document cites a VP-TERM id that is not in the terminology registry.",
            default_severity: Severity::Error,
        },
        (RuleScope::CrossReference, RuleKind::UnknownRfc) => RuleMetadata {
            title: "Unknown RFC Reference",
            description: "A document cites a VP-RFC id that is not in the RFC registry.",
            default_severity: Severity::Error,
        },
        (RuleScope::CrossReference, RuleKind::BrokenLink) => RuleMetadata {
            title: "Broken Link",
            description: "A relative markdown link target does not resolve under the spec root.",
            default_severity: Severity::Error,
        },
        (RuleScope::CrossReference, RuleKind::BrokenAnchor) => RuleMetadata {
            title: "Broken Anchor",
            description: "A markdown link fragment does not match a heading or HTML anchor.",
            default_severity: Severity::Error,
        },
        (RuleScope::CrossReference, RuleKind::InvalidReferenceFormat) => RuleMetadata {
            title: "Invalid Reference Format",
            description: "A token resembles a VP-TERM or VP-RFC id but violates the documented pattern.",
            default_severity: Severity::Error,
        },
        (RuleScope::CrossReference, _) => unmapped(RuleScope::CrossReference),

        (RuleScope::Edition, RuleKind::ManifestMissing) => RuleMetadata {
            title: "Edition Manifest Missing",
            description: "The Edition Manifest file is not present at the configured path.",
            default_severity: Severity::Error,
        },
        (RuleScope::Edition, RuleKind::ManifestYamlInvalid) => RuleMetadata {
            title: "Invalid Edition Manifest YAML",
            description: "The Edition Manifest is not valid YAML or is not a mapping at the root.",
            default_severity: Severity::Error,
        },
        (RuleScope::Edition, RuleKind::MissingField) => RuleMetadata {
            title: "Edition Manifest Missing Field",
            description: "A required top-level field is missing from the Edition Manifest.",
            default_severity: Severity::Error,
        },
        (RuleScope::Edition, RuleKind::InvalidEditionId) => RuleMetadata {
            title: "Invalid Edition ID",
            description: "The edition_id field does not match the vp-edition-* pattern.",
            default_severity: Severity::Error,
        },
        (RuleScope::Edition, RuleKind::InvalidEditionStatus) => RuleMetadata {
            title: "Invalid Edition Status",
            description: "The status field is not a recognized publication lifecycle value.",
            default_severity: Severity::Error,
        },
        (RuleScope::Edition, RuleKind::DocumentMissing) => RuleMetadata {
            title: "Pinned Document Missing",
            description: "A specification_documents path does not exist under the spec root.",
            default_severity: Severity::Error,
        },
        (RuleScope::Edition, RuleKind::PinnedVersionMismatch) => RuleMetadata {
            title: "Pinned Version Mismatch",
            description: "A pinned document version disagrees with the document front matter version.",
            default_severity: Severity::Error,
        },
        (RuleScope::Edition, RuleKind::UnknownAcceptedRfc) => RuleMetadata {
            title: "Unknown Accepted RFC",
            description: "An accepted_rfcs entry is not present in the VP-RFC registry.",
            default_severity: Severity::Error,
        },
        (RuleScope::Edition, RuleKind::RegistrySnapshotMissing) => RuleMetadata {
            title: "Registry Snapshot Missing",
            description: "A registry_snapshots path does not exist under the spec root.",
            default_severity: Severity::Error,
        },
        (RuleScope::Edition, RuleKind::InvalidConformanceId) => RuleMetadata {
            title: "Invalid Conformance ID",
            description: "A conformance_baseline entry does not match the VP-CS-NNNN pattern.",
            default_severity: Severity::Error,
        },
        (RuleScope::Edition, _) => unmapped(RuleScope::Edition),
    }
}

fn unmapped(scope: RuleScope) -> RuleMetadata {
    let title = match scope {
        RuleScope::RfcRegistry => "Unmapped RFC Rule",
        RuleScope::TermRegistry => "Unmapped Term Rule",
        RuleScope::CrossReference => "Unmapped Cross-Reference Rule",
        RuleScope::Edition => "Unmapped Edition Rule",
    };
    RuleMetadata {
        title,
        description: "This rule id is reserved but has no metadata yet.",
        default_severity: Severity::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapped_rules_have_metadata() {
        let rule = RuleId::rfc(RuleKind::InvalidVersion);
        assert_eq!(rule.title(), "Invalid RFC Version");
        assert!(!rule.description().is_empty());
        assert_eq!(rule.default_severity(), Severity::Error);
        assert_eq!(rule.external_id(), "vp-rfc-invalid-version");
    }

    #[test]
    fn crossref_rules_have_titles() {
        assert_eq!(
            RuleId::crossref(RuleKind::BrokenLink).title(),
            "Broken Link"
        );
        assert_eq!(
            RuleId::crossref(RuleKind::UnknownTerm).title(),
            "Unknown Term Reference"
        );
    }

    #[test]
    fn edition_rules_have_titles() {
        assert_eq!(
            RuleId::edition(RuleKind::ManifestMissing).external_id(),
            "vp-edition-manifest-missing"
        );
        assert_eq!(
            RuleId::edition(RuleKind::UnknownAcceptedRfc).title(),
            "Unknown Accepted RFC"
        );
    }
}
