//! Typed specification model for `veritypay-spec`.

mod builder;
mod error;
mod registry_set;
mod rfc;
mod specification;
mod terminology;

pub use builder::SpecificationBuilder;
pub use error::BuildError;
pub use registry_set::RegistrySet;
pub use rfc::{RfcEntry, RfcRegistry, REGISTRY_PATH as RFC_REGISTRY_PATH};
pub use specification::Specification;
pub use terminology::{
    NormativeDefinition, TerminologyEntry, TerminologyRegistry,
    REGISTRY_PATH as TERMINOLOGY_REGISTRY_PATH,
};
