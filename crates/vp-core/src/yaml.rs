//! Shared YAML parsing helpers for validators.

use serde_yaml::Value;

/// Parse YAML text into a [`Value`].
pub fn parse_yaml(text: &str) -> Result<Value, serde_yaml::Error> {
    serde_yaml::from_str(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_yaml_mapping() {
        let value = parse_yaml("key: value\n").expect("parse");
        assert_eq!(value.get("key").and_then(|v| v.as_str()), Some("value"));
    }

    #[test]
    fn parse_yaml_invalid_syntax() {
        assert!(parse_yaml("key: [").is_err());
    }
}
