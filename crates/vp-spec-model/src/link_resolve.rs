//! Path resolution helpers for Markdown link targets.

use std::path::{Component, Path, PathBuf};

/// Split a Markdown link target into path and optional anchor fragment.
pub fn split_link_target(target: &str) -> (String, Option<String>) {
    if let Some((path, anchor)) = target.split_once('#') {
        (
            path.to_string(),
            Some(anchor.to_string()).filter(|fragment| !fragment.is_empty()),
        )
    } else {
        (target.to_string(), None)
    }
}

/// Resolve a relative link from `source_file` to a path relative to spec root.
pub fn resolve_relative_link(source_file: &Path, path_part: &str) -> PathBuf {
    if path_part.is_empty() {
        return source_file.to_path_buf();
    }

    let base = source_file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    normalize_path(&base.join(path_part))
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            Component::Normal(part) => result.push(part),
            Component::RootDir => result.push(component.as_os_str()),
            Component::Prefix(_) => result.push(component.as_os_str()),
        }
    }
    result
}

pub fn path_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn resolves_parent_relative_links() {
        let resolved = resolve_relative_link(Path::new("docs/a/page.md"), "../other.md");
        assert_eq!(resolved, PathBuf::from("docs/other.md"));
    }
}
