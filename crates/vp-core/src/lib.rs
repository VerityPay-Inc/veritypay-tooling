//! Shared contracts between the validation engine and validators.

pub mod context;
pub mod spec_repository;
pub mod validator;
pub mod yaml;

pub use context::ValidationContext;
pub use spec_repository::{ReadError, SpecRepository};
pub use validator::Validator;
pub use yaml::parse_yaml;
