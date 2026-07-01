//! Reference discovery and cross-reference validation for VerityPay specifications.

mod constants;
mod corpus;
mod discovery;
mod kind;
mod markdown;
mod reference;
mod registry_lookup;
mod resolve;
mod validate;
mod validator;

pub use discovery::ReferenceDiscovery;
pub use kind::ReferenceKind;
pub use markdown::MarkdownDiscovery;
pub use reference::Reference;
pub use validator::CrossReferenceValidator;
