//! Edition Manifest validation (Milestone D).

mod edition;
mod registry_source;
mod validator;

pub use edition::validate;
pub use validator::EditionValidator;
