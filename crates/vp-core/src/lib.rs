//! Shared contracts between the validation engine and validators.

pub mod context;
pub mod validator;

pub use context::ValidationContext;
pub use validator::Validator;
