//! Strongly typed validation rule identifiers.

/// Registry or document family that scopes a rule's external identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RuleScope {
    RfcRegistry,
    TermRegistry,
    CrossReference,
    Edition,
}

/// Shared rule semantics reused across validators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RuleKind {
    RegistryMissing,
    RegistryYamlInvalid,
    TopLevelMissingField,
    EmptyList,
    EntryMissingField,
    InvalidId,
    DuplicateId,
    DuplicateTitle,
    IdNumberMismatch,
    UnknownStatus,
    UnknownStability,
    InvalidVersion,
    MissingPath,
    UnknownReference,
    InvalidNormativeDefinition,
    InvalidReferencedBy,
    InvalidSectionId,
    UnknownTerm,
    UnknownRfc,
    BrokenLink,
    BrokenAnchor,
    InvalidReferenceFormat,
    ManifestMissing,
    ManifestYamlInvalid,
    MissingField,
    InvalidEditionId,
    InvalidEditionStatus,
    DocumentMissing,
    PinnedVersionMismatch,
    UnknownAcceptedRfc,
    RegistrySnapshotMissing,
    InvalidConformanceId,
}

/// Internal rule identifier; render with [`RuleId::external_id`] for CLI output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuleId {
    pub scope: RuleScope,
    pub kind: RuleKind,
}

impl RuleId {
    pub const fn rfc(kind: RuleKind) -> Self {
        Self {
            scope: RuleScope::RfcRegistry,
            kind,
        }
    }

    pub const fn term(kind: RuleKind) -> Self {
        Self {
            scope: RuleScope::TermRegistry,
            kind,
        }
    }

    pub const fn crossref(kind: RuleKind) -> Self {
        Self {
            scope: RuleScope::CrossReference,
            kind,
        }
    }

    pub const fn edition(kind: RuleKind) -> Self {
        Self {
            scope: RuleScope::Edition,
            kind,
        }
    }

    /// Stable external rule id (e.g. `vp-rfc-duplicate-id`).
    pub fn external_id(self) -> &'static str {
        match (self.scope, self.kind) {
            (RuleScope::RfcRegistry, RuleKind::RegistryMissing) => "vp-rfc-registry-missing",
            (RuleScope::RfcRegistry, RuleKind::RegistryYamlInvalid) => {
                "vp-rfc-registry-yaml-invalid"
            }
            (RuleScope::RfcRegistry, RuleKind::TopLevelMissingField) => {
                "vp-rfc-top-level-missing-field"
            }
            (RuleScope::RfcRegistry, RuleKind::EmptyList) => "vp-rfc-empty-list",
            (RuleScope::RfcRegistry, RuleKind::EntryMissingField) => "vp-rfc-entry-missing-field",
            (RuleScope::RfcRegistry, RuleKind::InvalidId) => "vp-rfc-invalid-id",
            (RuleScope::RfcRegistry, RuleKind::DuplicateId) => "vp-rfc-duplicate-id",
            (RuleScope::RfcRegistry, RuleKind::DuplicateTitle) => "vp-rfc-duplicate-title",
            (RuleScope::RfcRegistry, RuleKind::IdNumberMismatch) => "vp-rfc-id-number-mismatch",
            (RuleScope::RfcRegistry, RuleKind::UnknownStatus) => "vp-rfc-unknown-status",
            (RuleScope::RfcRegistry, RuleKind::UnknownStability) => "vp-rfc-unknown-stability",
            (RuleScope::RfcRegistry, RuleKind::InvalidVersion) => "vp-rfc-invalid-version",
            (RuleScope::RfcRegistry, RuleKind::MissingPath) => "vp-rfc-path-missing",
            (RuleScope::RfcRegistry, RuleKind::UnknownReference) => "vp-rfc-unknown-reference",
            (RuleScope::RfcRegistry, RuleKind::InvalidNormativeDefinition) => {
                "vp-rfc-invalid-normative-definition"
            }
            (RuleScope::RfcRegistry, RuleKind::InvalidReferencedBy) => {
                "vp-rfc-invalid-referenced-by"
            }
            (RuleScope::RfcRegistry, RuleKind::InvalidSectionId) => "vp-rfc-invalid-section-id",
            (RuleScope::RfcRegistry, _) => "vp-rfc-unmapped",

            (RuleScope::TermRegistry, RuleKind::RegistryMissing) => "vp-term-registry-missing",
            (RuleScope::TermRegistry, RuleKind::RegistryYamlInvalid) => {
                "vp-term-registry-yaml-invalid"
            }
            (RuleScope::TermRegistry, RuleKind::TopLevelMissingField) => {
                "vp-term-top-level-missing-field"
            }
            (RuleScope::TermRegistry, RuleKind::EmptyList) => "vp-term-empty-list",
            (RuleScope::TermRegistry, RuleKind::EntryMissingField) => "vp-term-entry-missing-field",
            (RuleScope::TermRegistry, RuleKind::InvalidId) => "vp-term-invalid-id",
            (RuleScope::TermRegistry, RuleKind::DuplicateId) => "vp-term-duplicate-id",
            (RuleScope::TermRegistry, RuleKind::DuplicateTitle) => "vp-term-duplicate-title",
            (RuleScope::TermRegistry, RuleKind::IdNumberMismatch) => "vp-term-id-number-mismatch",
            (RuleScope::TermRegistry, RuleKind::UnknownStatus) => "vp-term-unknown-status",
            (RuleScope::TermRegistry, RuleKind::UnknownStability) => "vp-term-unknown-stability",
            (RuleScope::TermRegistry, RuleKind::InvalidVersion) => "vp-term-invalid-version",
            (RuleScope::TermRegistry, RuleKind::MissingPath) => "vp-term-path-missing",
            (RuleScope::TermRegistry, RuleKind::UnknownReference) => "vp-term-unknown-reference",
            (RuleScope::TermRegistry, RuleKind::InvalidNormativeDefinition) => {
                "vp-term-invalid-normative-definition"
            }
            (RuleScope::TermRegistry, RuleKind::InvalidReferencedBy) => {
                "vp-term-invalid-referenced-by"
            }
            (RuleScope::TermRegistry, RuleKind::InvalidSectionId) => "vp-term-invalid-section-id",
            (RuleScope::TermRegistry, _) => "vp-term-unmapped",

            (RuleScope::CrossReference, RuleKind::UnknownTerm) => "vp-crossref-unknown-term",
            (RuleScope::CrossReference, RuleKind::UnknownRfc) => "vp-crossref-unknown-rfc",
            (RuleScope::CrossReference, RuleKind::BrokenLink) => "vp-crossref-broken-link",
            (RuleScope::CrossReference, RuleKind::BrokenAnchor) => "vp-crossref-broken-anchor",
            (RuleScope::CrossReference, RuleKind::InvalidReferenceFormat) => {
                "vp-crossref-invalid-reference-format"
            }
            (RuleScope::CrossReference, _) => "vp-crossref-unmapped",

            (RuleScope::Edition, RuleKind::ManifestMissing) => "vp-edition-manifest-missing",
            (RuleScope::Edition, RuleKind::ManifestYamlInvalid) => "vp-edition-yaml-invalid",
            (RuleScope::Edition, RuleKind::MissingField) => "vp-edition-missing-field",
            (RuleScope::Edition, RuleKind::InvalidEditionId) => "vp-edition-invalid-id",
            (RuleScope::Edition, RuleKind::InvalidEditionStatus) => "vp-edition-invalid-status",
            (RuleScope::Edition, RuleKind::DocumentMissing) => "vp-edition-document-missing",
            (RuleScope::Edition, RuleKind::PinnedVersionMismatch) => "vp-edition-version-mismatch",
            (RuleScope::Edition, RuleKind::UnknownAcceptedRfc) => "vp-edition-unknown-rfc",
            (RuleScope::Edition, RuleKind::RegistrySnapshotMissing) => {
                "vp-edition-registry-missing"
            }
            (RuleScope::Edition, RuleKind::InvalidConformanceId) => {
                "vp-edition-invalid-conformance-id"
            }
            (RuleScope::Edition, _) => "vp-edition-unmapped",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_ids_are_stable() {
        assert_eq!(
            RuleId::rfc(RuleKind::DuplicateId).external_id(),
            "vp-rfc-duplicate-id"
        );
        assert_eq!(
            RuleId::crossref(RuleKind::UnknownTerm).external_id(),
            "vp-crossref-unknown-term"
        );
    }
}
