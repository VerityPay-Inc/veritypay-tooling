//! Shared contracts between the validation engine and validators.

pub mod config;
pub mod context;
pub mod spec_repository;
pub mod spec_root;
pub mod validation_config;
pub mod validator;
pub mod validator_info;
pub mod yaml;

pub use config::{
    load_vp_toml_from_cwd, load_vp_toml_from_dir, resolve_config_with_spec_root, ConfigError,
};
pub use context::{MissingSpecRootError, ValidationContext};
pub use spec_repository::{ReadError, SpecRepository};
pub use spec_root::canonicalize_spec_root;
pub use validation_config::{
    apply_overrides, resolve_validation_config, ValidationConfig, ValidationConfigOverrides,
    ValidationOutput,
};
pub use validator::Validator;
pub use validator_info::ValidatorInfo;
pub use yaml::parse_yaml;
