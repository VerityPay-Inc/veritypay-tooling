//! Shared contracts between the validation engine and validators.

pub mod context;
pub mod spec_repository;
pub mod spec_root;
pub mod validator;
pub mod yaml;

pub use context::ValidationContext;
pub use spec_repository::{ReadError, SpecRepository};
pub use spec_root::canonicalize_spec_root;
pub use validator::Validator;
pub use yaml::parse_yaml;
