//! Edition Manifest validation (Milestone D).

mod edition;
mod validator;

pub use edition::validate;
pub use validator::EditionValidator;
