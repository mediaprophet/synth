use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn qualia_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../qualiaDB")
}

pub fn delegated_view_files(browser: &Path) -> BTreeSet<String> {
    let browser = browser.canonicalize().unwrap_or_else(|error| {
        panic!(
            "failed to resolve browser source root {}: {error}",
            browser.display()
        )
    });
    let mut files = BTreeSet::new();
    collect_delegated_view_files(&browser, &browser, &mut files);
    files
}

fn collect_delegated_view_files(directory: &Path, browser: &Path, out: &mut BTreeSet<String>) {
    let entries = fs::read_dir(directory).unwrap_or_else(|error| {
        panic!(
            "failed to read browser source directory {}: {error}",
            directory.display()
        )
    });
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            panic!(
                "failed to read browser source entry in {}: {error}",
                directory.display()
            )
        });
        let path = entry.path();
        let file_type = entry.file_type().unwrap_or_else(|error| {
            panic!(
                "failed to inspect browser source path {}: {error}",
                path.display()
            )
        });
        if file_type.is_dir() {
            collect_delegated_view_files(&path, browser, out);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let source = fs::read_to_string(&path).unwrap_or_else(|error| {
                panic!("failed to read browser source {}: {error}", path.display())
            });
            if has_leaf_delegation(&source) {
                let source_path = path.canonicalize().unwrap_or_else(|error| {
                    panic!(
                        "failed to resolve delegated source path {}: {error}",
                        path.display()
                    )
                });
                out.insert(inventory_key(browser, &source_path));
            }
        }
    }
}

fn has_leaf_delegation(source: &str) -> bool {
    let child_modules: BTreeSet<&str> = source.lines().filter_map(local_child_module).collect();
    source.lines().any(|line| {
        let line = line.trim_start();
        let Some(path) = line.strip_prefix("pub use ") else {
            return false;
        };
        if !path.contains("build_") || !path.contains("_view") {
            return false;
        }
        let first_segment = path.split("::").next().unwrap_or_default().trim();
        !child_modules.contains(first_segment)
    })
}

fn local_child_module(line: &str) -> Option<&str> {
    let line = line.trim_start();
    for prefix in ["pub mod ", "mod "] {
        if let Some(rest) = line.strip_prefix(prefix) {
            let name = rest.split_once(';')?.0.trim();
            return (!name.is_empty()).then_some(name);
        }
    }
    None
}

fn inventory_key(browser: &Path, source: &Path) -> String {
    let relative = source.strip_prefix(browser).unwrap_or_else(|_| {
        panic!(
            "delegated source {} is outside browser root {}",
            source.display(),
            browser.display()
        )
    });
    format!(
        "crates/poet/src/browser/{}",
        relative.to_string_lossy().replace('\\', "/")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_module_facades_are_not_leaf_delegations() {
        assert!(!has_leaf_delegation(
            "pub mod inspector;\npub use inspector::build_inspector_view;"
        ));
        assert!(!has_leaf_delegation(
            "mod inspector;\npub use inspector::{build_inspector_view, Other};"
        ));
        assert!(has_leaf_delegation(
            "pub mod inspector\npub use inspector::build_inspector_view;"
        ));
        assert!(!has_leaf_delegation("pub use inspector::Other;"));
    }

    #[test]
    fn nonlocal_delegations_are_leaf_surfaces() {
        assert!(has_leaf_delegation(
            "pub mod persist;\npub use super::persist::build_inspector_view;"
        ));
        assert!(has_leaf_delegation(
            "pub use super::workspace::build_inspector_view;"
        ));
        assert!(has_leaf_delegation(
            "pub use self::persist::build_inspector_view;\npub use self::persist::build_other_view;"
        ));
        assert!(has_leaf_delegation(
            "pub mod inspector;\npub use inspector::build_inspector_view;\npub use super::persist::build_other_view;"
        ));
    }

    #[test]
    fn inventory_key_preserves_nested_historical_path() {
        let browser = Path::new("repo/crates/poet/src/browser");
        let source = browser.join("health_views/lab_results/mod.rs");
        assert_eq!(
            inventory_key(browser, &source),
            "crates/poet/src/browser/health_views/lab_results/mod.rs"
        );
    }

    #[test]
    fn qualia_root_is_the_poet_sibling() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let expected = manifest
            .parent()
            .expect("Poet root")
            .parent()
            .expect("ecosystem root")
            .join("qualiaDB");
        assert_eq!(
            qualia_root().canonicalize().expect("QualiaDB root"),
            expected.canonicalize().expect("sibling QualiaDB root")
        );
        assert!(qualia_root().join("Cargo.toml").is_file());
    }
}
