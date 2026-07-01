//! Registry validator — VP-TERM and VP-RFC (Milestone B).

mod registry_source;
mod registry_validator;
mod rfc_registry;
mod term_registry;
mod term_registry_validator;

pub use registry_validator::{RegistryValidator, RfcRegistryValidator};
pub use term_registry_validator::TermRegistryValidator;
