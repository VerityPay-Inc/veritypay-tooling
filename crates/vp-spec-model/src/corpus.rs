//! Markdown corpus discovery under a specification root.

use std::path::{Component, Path, PathBuf};

use vp_core::SpecRepository;

pub const ROOT_MARKDOWN_FILES: &[&str] =
    &["README.md", "CONTRIBUTING.md", "SPECIFICATION_STATUS.md"];

pub const CORPUS_DIRECTORIES: &[&str] = &["docs", "rfcs"];

pub const SKIP_DIRECTORY_NAMES: &[&str] =
    &["target", "crates", "generated", "node_modules", ".git"];

/// Non-live template/snippet trees excluded from document corpus scans.
pub const EXCLUDED_CORPUS_PREFIXES: &[&str] =
    &["docs/templates", "docs/snippets", "rfcs/templates"];

/// Collect Markdown files in the document corpus discovery scope.
pub fn collect_markdown_paths(repo: &SpecRepository) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let spec_root = repo.spec_root();

    for dir in CORPUS_DIRECTORIES {
        let root = repo.canonical_path(dir);
        if root.is_dir() {
            collect_markdown_recursive(&root, spec_root, &mut files);
        }
    }

    for name in ROOT_MARKDOWN_FILES {
        let rel = PathBuf::from(name);
        if repo.is_file(&rel) && !is_excluded_corpus_path(&rel) {
            files.push(rel);
        }
    }

    files.sort();
    files.dedup();
    files
}

fn collect_markdown_recursive(dir: &Path, spec_root: &Path, out: &mut Vec<PathBuf>) {
    let rel_dir = relative_from_spec_root(dir, spec_root);
    if rel_dir.as_ref().is_some_and(|rel| is_excluded_corpus_path(rel)) {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if should_skip_directory(&path) {
                continue;
            }
            collect_markdown_recursive(&path, spec_root, out);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            if let Some(rel) = relative_from_spec_root(&path, spec_root) {
                if !is_excluded_corpus_path(&rel) {
                    out.push(rel);
                }
            }
        }
    }
}

fn relative_from_spec_root(path: &Path, spec_root: &Path) -> Option<PathBuf> {
    let absolute = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    absolute.strip_prefix(spec_root).ok().map(Path::to_path_buf)
}

fn is_excluded_corpus_path(rel: &Path) -> bool {
    EXCLUDED_CORPUS_PREFIXES.iter().any(|prefix| {
        let excluded = Path::new(prefix);
        rel == excluded || rel.starts_with(excluded.join(""))
    })
}

fn should_skip_directory(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            Component::Normal(name) if SKIP_DIRECTORY_NAMES.iter().any(|skip| name == *skip)
        )
    })
}
