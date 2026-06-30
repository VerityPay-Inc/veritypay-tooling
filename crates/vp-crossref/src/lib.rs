//! Reference discovery for VerityPay specification documents (Milestone C.1).
//!
//! This crate finds references in source documents. It does not validate them.

mod discovery;
mod kind;
mod markdown;
mod reference;

pub use discovery::ReferenceDiscovery;
pub use kind::ReferenceKind;
pub use markdown::MarkdownDiscovery;
pub use reference::Reference;
