use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static SKIP_DIRS: OnceLock<HashSet<&'static str>> = OnceLock::new();
static PLUGIN_RE: OnceLock<Regex> = OnceLock::new();

fn skip_dirs() -> &'static HashSet<&'static str> {
    SKIP_DIRS.get_or_init(|| {
        [
            "node_modules", ".git", "vendor", "build", "dist", ".gradle",
            "target", "tmp", "log", "coverage", ".worktrees",
            "test", "tests", "spec", "specs", "__tests__",
            "contrib",
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

pub fn scan_files(dirs: &[String]) -> Vec<PathBuf> {
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
            .sort_by_file_name(|a, b| a.cmp(b))
            .build();

        for entry in walker.flatten() {
            let path = entry.path().to_path_buf();
            if !path.is_file() {
                continue;
            }

            // Check SKIP_DIRS
            let skip_it = path.components().any(|c| {
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
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            let is_ruby_named = matches!(name, "Rakefile" | "Gemfile" | "Capfile");

            if !SUPPORTED_EXTS.contains(&ext.as_str()) && !is_ruby_named {
                continue;
            }

            files.push(path);
        }
    }

    files
}
