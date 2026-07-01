//! Markdown corpus discovery under a specification root.

use std::path::{Component, Path, PathBuf};

use vp_core::SpecRepository;

use crate::constants::{
    CORPUS_DIRECTORIES, EXCLUDED_CORPUS_PREFIXES, ROOT_MARKDOWN_FILES, SKIP_DIRECTORY_NAMES,
};

/// Collect Markdown files in the cross-reference discovery scope.
pub fn collect_markdown_files(repo: &SpecRepository) -> Vec<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn collects_docs_rfcs_and_root_markdown() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("docs/sub")).expect("docs");
        fs::write(dir.path().join("docs/sub/page.md"), "# Page").expect("page");
        fs::create_dir_all(dir.path().join("rfcs")).expect("rfcs dir");
        fs::write(dir.path().join("rfcs/0000.md"), "# RFC").expect("rfc");
        fs::write(dir.path().join("README.md"), "# Readme").expect("readme");

        let repo = SpecRepository::new(dir.path());
        let files = collect_markdown_files(&repo);
        assert!(files.contains(&PathBuf::from("docs/sub/page.md")));
        assert!(files.contains(&PathBuf::from("README.md")));
    }

    #[test]
    fn skips_target_and_crates_directories() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("docs/ok")).expect("ok");
        fs::create_dir_all(dir.path().join("docs/target/hidden")).expect("target");
        fs::write(dir.path().join("docs/ok/visible.md"), "# ok").expect("visible");
        fs::write(dir.path().join("docs/target/hidden/hidden.md"), "# hidden").expect("hidden");

        let repo = SpecRepository::new(dir.path());
        let files = collect_markdown_files(&repo);
        assert!(files.contains(&PathBuf::from("docs/ok/visible.md")));
        assert!(!files.iter().any(|p| p.to_string_lossy().contains("target")));
    }

    #[test]
    fn excludes_template_and_snippet_directories() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("docs/live")).expect("live");
        fs::create_dir_all(dir.path().join("docs/templates/snippets")).expect("templates");
        fs::create_dir_all(dir.path().join("docs/snippets/draft")).expect("snippets");
        fs::create_dir_all(dir.path().join("rfcs/templates/draft")).expect("rfc templates");
        fs::write(dir.path().join("docs/live/page.md"), "# live").expect("live file");
        fs::write(
            dir.path().join("docs/templates/snippets/nav.md"),
            "# template",
        )
        .expect("template file");
        fs::write(dir.path().join("docs/snippets/draft/x.md"), "# snippet").expect("snippet");
        fs::write(
            dir.path().join("rfcs/templates/draft/x.md"),
            "# rfc template",
        )
        .expect("rfc template file");

        let repo = SpecRepository::new(dir.path());
        let files = collect_markdown_files(&repo);
        assert!(files.contains(&PathBuf::from("docs/live/page.md")));
        assert!(!files.iter().any(|p| p.components().any(|c| c.as_os_str() == "templates")));
        assert!(
            !files
                .iter()
                .any(|p| p.starts_with("docs/snippets") || p.starts_with("rfcs/templates"))
        );
    }
}
