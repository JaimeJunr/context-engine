use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static SKIP_DIRS: OnceLock<HashSet<&'static str>> = OnceLock::new();
static PLUGIN_RE: OnceLock<Regex> = OnceLock::new();

fn skip_dirs() -> &'static HashSet<&'static str> {
    SKIP_DIRS.get_or_init(|| {
        [
            "node_modules",
            ".git",
            "vendor",
            "build",
            "dist",
            ".gradle",
            "target",
            "tmp",
            "log",
            "coverage",
            ".worktrees",
            "test",
            "tests",
            "spec",
            "specs",
            "__tests__",
            "contrib",
            ".claude",
            ".husky",
            ".github",
            ".vscode",
            ".idea",
            ".serverless",
            ".turbo",
            ".nx",
            ".next",
            ".nuxt",
            "__pycache__",
        ]
        .iter()
        .copied()
        .collect()
    })
}

fn plugin_re() -> &'static Regex {
    PLUGIN_RE.get_or_init(|| Regex::new(r"^.+-\d+\.\d+").unwrap())
}

pub const SUPPORTED_EXTS: &[&str] = &[".rb", ".rake", ".groovy", ".gradle", ".py", ".ts", ".tsx"];

fn collect_worktree_roots(root: &Path) -> HashSet<PathBuf> {
    let mut worktrees = HashSet::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return worktrees;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join(".git").is_file() {
            if let Ok(canonical) = path.canonicalize() {
                worktrees.insert(canonical);
            }
        }
    }
    worktrees
}

fn is_plugin_dir(path: &Path, root: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(root) else {
        return false;
    };
    if let Some(first) = rel.components().next() {
        let name = first.as_os_str().to_string_lossy();
        return plugin_re().is_match(&name);
    }
    false
}

pub fn scan_files(dirs: &[String], max_depth: usize) -> Vec<PathBuf> {
    scan_files_with_exts(dirs, max_depth, SUPPORTED_EXTS)
}

/// Variante do scanner com conjunto de extensões customizado.
///
/// Útil para pipelines que cobrem linguagens diferentes do `map` (ex: `graph`
/// também inclui `.rs`, `.go`, `.java`).
pub fn scan_files_with_exts(dirs: &[String], max_depth: usize, exts: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let skip = skip_dirs();

    for d in dirs {
        let root = Path::new(d);
        if !root.is_dir() {
            continue;
        }

        let excluded_roots = collect_worktree_roots(root);

        let walker = ignore::WalkBuilder::new(root)
            .hidden(false)
            .ignore(false)
            .git_ignore(false)
            .git_global(false)
            .max_depth(Some(max_depth))
            .sort_by_file_name(|a, b| a.cmp(b))
            .build();

        for entry in walker.flatten() {
            let path = entry.path().to_path_buf();
            if !path.is_file() {
                continue;
            }

            // Verifica skip_dirs apenas nos componentes relativos ao root
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let skip_it = rel.components().any(|c| {
                let s = c.as_os_str().to_string_lossy();
                skip.contains(s.as_ref())
            });
            if skip_it {
                continue;
            }

            // Check plugin dirs
            if is_plugin_dir(&path, root) {
                continue;
            }

            // Check worktree exclusion
            if let Ok(canonical) = path.canonicalize() {
                if excluded_roots.iter().any(|ex| canonical.starts_with(ex)) {
                    continue;
                }
            }

            // Check extension
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{}", e))
                .unwrap_or_default();
            // Also handle extensionless files like Rakefile, Gemfile
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let is_ruby_named = matches!(name, "Rakefile" | "Gemfile" | "Capfile");

            if !exts.contains(&ext.as_str()) && !is_ruby_named {
                continue;
            }

            files.push(path);
        }
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn claude_dir_is_in_skip_list() {
        assert!(
            skip_dirs().contains(".claude"),
            ".claude deve estar na lista de skip"
        );
    }

    #[test]
    fn husky_dir_is_in_skip_list() {
        assert!(skip_dirs().contains(".husky"));
    }

    #[test]
    fn github_dir_is_in_skip_list() {
        assert!(skip_dirs().contains(".github"));
    }

    #[test]
    fn vscode_dir_is_in_skip_list() {
        assert!(skip_dirs().contains(".vscode"));
    }

    #[test]
    fn idea_dir_is_in_skip_list() {
        assert!(skip_dirs().contains(".idea"));
    }

    #[test]
    fn serverless_dir_is_in_skip_list() {
        assert!(skip_dirs().contains(".serverless"));
    }

    #[test]
    fn turbo_dir_is_in_skip_list() {
        assert!(skip_dirs().contains(".turbo"));
    }

    #[test]
    fn nx_dir_is_in_skip_list() {
        assert!(skip_dirs().contains(".nx"));
    }

    #[test]
    fn next_dir_is_in_skip_list() {
        assert!(
            skip_dirs().contains(".next"),
            ".next deve estar na lista de skip"
        );
    }

    #[test]
    fn nuxt_dir_is_in_skip_list() {
        assert!(
            skip_dirs().contains(".nuxt"),
            ".nuxt deve estar na lista de skip"
        );
    }

    #[test]
    fn pycache_dir_is_in_skip_list() {
        assert!(
            skip_dirs().contains("__pycache__"),
            "__pycache__ deve estar na lista de skip"
        );
    }

    #[test]
    fn max_depth_1_excludes_nested_files() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("foo.rb"), "def foo; end").unwrap();

        let dir = tmp.path().to_string_lossy().to_string();
        // max_depth=1 significa apenas o diretório raiz, sem recursão
        let found = scan_files(&[dir], 1);

        assert!(
            found.is_empty(),
            "max_depth=1 não deve encontrar arquivos em a/b/ (encontrou: {:?})",
            found
        );
    }

    #[test]
    fn max_depth_15_scans_normal_tree() {
        let tmp = tempfile::tempdir().unwrap();
        // Usa nomes que não colidem com skip_dirs
        let nested = tmp.path().join("app").join("models").join("concerns");
        fs::create_dir_all(&nested).unwrap();
        let file_path = nested.join("app.rb");
        fs::write(&file_path, "def hello; end").unwrap();

        let dir = tmp.path().to_string_lossy().to_string();
        let found = scan_files(&[dir], 15);

        assert!(
            found.contains(&file_path),
            "max_depth=15 deve encontrar '{}' em 3 níveis de profundidade, encontrado: {:?}",
            file_path.display(),
            found
        );
    }

    #[test]
    fn files_inside_claude_dir_are_not_scanned() {
        let tmp = tempfile::tempdir().unwrap();
        let claude_skills = tmp.path().join(".claude").join("skills");
        fs::create_dir_all(&claude_skills).unwrap();
        let py_file = claude_skills.join("foo.py");
        fs::write(&py_file, "def foo(): pass").unwrap();

        let dir = tmp.path().to_string_lossy().to_string();
        let found = scan_files(&[dir], 15);

        assert!(
            !found.contains(&py_file),
            "arquivos em .claude/ não devem ser incluídos no scan"
        );
    }
}
