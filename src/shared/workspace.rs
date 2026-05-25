// Detecção de stack e geração de .ctx/config.toml para ctx init

use anyhow::Result;
use std::path::Path;

#[derive(Debug, PartialEq, Clone)]
pub enum StackKind {
    Rust,
    Node,
    React,
    Rails,
    Jvm,
    Python,
}

pub struct WorkspaceProfile {
    pub stacks: Vec<StackKind>,
    pub is_monorepo: bool,
    pub monorepo_subprojects: Vec<String>,
    pub has_docs: bool,
    pub has_readme: bool,
    pub project_name: String,
}

pub fn detect_stack(path: &Path) -> WorkspaceProfile {
    let mut stacks = Vec::new();

    if path.join("Cargo.toml").exists() {
        stacks.push(StackKind::Rust);
    }

    if path.join("package.json").exists() {
        stacks.push(StackKind::Node);
        // Verificar se tem react nas dependências
        if let Ok(contents) = std::fs::read_to_string(path.join("package.json")) {
            if contents.contains("\"react\"") {
                stacks.push(StackKind::React);
            }
        }
    }

    if path.join("Gemfile").exists() {
        stacks.push(StackKind::Rails);
    }

    if path.join("build.gradle").exists()
        || path.join("build.gradle.kts").exists()
        || path.join("pom.xml").exists()
    {
        stacks.push(StackKind::Jvm);
    }

    if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
        stacks.push(StackKind::Python);
    }

    let has_docs = path.join("docs").is_dir();
    let has_readme = path.join("README.md").exists() || path.join("README.rst").exists();

    // Detecção de monorepo
    let is_monorepo = path.join("turbo.json").exists()
        || path.join("nx.json").exists()
        || workspaces_in_package_json(path);

    let mut monorepo_subprojects = Vec::new();
    if is_monorepo {
        for subdir in ["apps", "packages"] {
            let subdir_path = path.join(subdir);
            if subdir_path.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&subdir_path) {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            monorepo_subprojects.push(format!(
                                "{}/{}",
                                subdir,
                                entry.file_name().to_string_lossy()
                            ));
                        }
                    }
                }
            }
        }
    }

    let project_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "projeto".to_string());

    WorkspaceProfile {
        stacks,
        is_monorepo,
        monorepo_subprojects,
        has_docs,
        has_readme,
        project_name,
    }
}

fn workspaces_in_package_json(path: &Path) -> bool {
    if let Ok(contents) = std::fs::read_to_string(path.join("package.json")) {
        return contents.contains("\"workspaces\"");
    }
    false
}

pub fn generate_config(profile: &WorkspaceProfile) -> String {
    let now = chrono::Local::now().format("%Y-%m-%d").to_string();
    let stack_names: Vec<&str> = profile
        .stacks
        .iter()
        .map(|s| match s {
            StackKind::Rust => "Rust",
            StackKind::Node => "Node",
            StackKind::React => "React",
            StackKind::Rails => "Rails",
            StackKind::Jvm => "JVM",
            StackKind::Python => "Python",
        })
        .collect();
    let stacks_str = if stack_names.is_empty() {
        "(nenhuma detectada)".to_string()
    } else {
        stack_names.join(", ")
    };

    let dirs_value = if profile.is_monorepo && !profile.monorepo_subprojects.is_empty() {
        let entries: Vec<String> = profile
            .monorepo_subprojects
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect();
        format!("[{}]", entries.join(", "))
    } else {
        "[\".\"]\n".to_string()
    };

    format!(
        "# Gerado por ctx init em {}\n# Stack detectada: {}\n\n[map]\ndirs = {}\nmax_depth = 15\nignore_extra = []\n# Adicione padrões para ignorar diretórios adicionais:\n# ignore_extra = [\"**/fixtures/**\", \"**/snapshots/**\"]\n",
        now, stacks_str, dirs_value
    )
}

pub fn run_workspace_init(path: &str, force: bool) -> Result<String> {
    let base = std::path::Path::new(path)
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(path));

    let ctx_dir = base.join(".ctx");
    let config_path = ctx_dir.join("config.toml");

    if config_path.exists() && !force {
        anyhow::bail!(
            ".ctx/config.toml já existe em '{}'. Use --force para sobrescrever.",
            base.display()
        );
    }

    std::fs::create_dir_all(&ctx_dir)?;

    let profile = detect_stack(&base);
    let config = generate_config(&profile);
    std::fs::write(&config_path, &config)?;

    let stack_names: Vec<&str> = profile
        .stacks
        .iter()
        .map(|s| match s {
            StackKind::Rust => "Rust",
            StackKind::Node => "Node",
            StackKind::React => "React",
            StackKind::Rails => "Rails",
            StackKind::Jvm => "JVM",
            StackKind::Python => "Python",
        })
        .collect();

    let mut report = format!("Workspace inicializado em '{}'.\n", base.display());

    if stack_names.is_empty() {
        report.push_str("Stack detectada: (nenhuma)\n");
    } else {
        report.push_str(&format!("Stack detectada: {}\n", stack_names.join(", ")));
    }

    if profile.is_monorepo {
        report.push_str(&format!(
            "Monorepo: sim ({} subprojetos)\n",
            profile.monorepo_subprojects.len()
        ));
    }

    report.push_str(&format!(
        "Configuração criada em: {}\n",
        config_path.display()
    ));

    if profile.has_docs || profile.has_readme {
        report.push_str(&format!(
            "\nDica: para indexar documentação, execute:\n  ctx catalog bootstrap --path . --name {}\n",
            profile.project_name
        ));
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn test_detect_rust_stack() {
        let dir = make_dir();
        fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        let profile = detect_stack(dir.path());
        assert!(profile.stacks.contains(&StackKind::Rust));
    }

    #[test]
    fn test_detect_node_stack() {
        let dir = make_dir();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let profile = detect_stack(dir.path());
        assert!(profile.stacks.contains(&StackKind::Node));
        assert!(!profile.stacks.contains(&StackKind::React));
    }

    #[test]
    fn test_detect_react_stack() {
        let dir = make_dir();
        fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies":{"react":"^18.0"}}"#,
        )
        .unwrap();
        let profile = detect_stack(dir.path());
        assert!(profile.stacks.contains(&StackKind::Node));
        assert!(profile.stacks.contains(&StackKind::React));
    }

    #[test]
    fn test_detect_rails_stack() {
        let dir = make_dir();
        fs::write(dir.path().join("Gemfile"), "source 'https://rubygems.org'").unwrap();
        let profile = detect_stack(dir.path());
        assert!(profile.stacks.contains(&StackKind::Rails));
    }

    #[test]
    fn test_detect_jvm_gradle() {
        let dir = make_dir();
        fs::write(dir.path().join("build.gradle"), "plugins {}").unwrap();
        let profile = detect_stack(dir.path());
        assert!(profile.stacks.contains(&StackKind::Jvm));
    }

    #[test]
    fn test_detect_python_stack() {
        let dir = make_dir();
        fs::write(dir.path().join("pyproject.toml"), "[build-system]").unwrap();
        let profile = detect_stack(dir.path());
        assert!(profile.stacks.contains(&StackKind::Python));
    }

    #[test]
    fn test_detect_empty_dir() {
        let dir = make_dir();
        let profile = detect_stack(dir.path());
        assert!(profile.stacks.is_empty());
    }

    #[test]
    fn test_detect_monorepo_turbo() {
        let dir = make_dir();
        fs::write(dir.path().join("turbo.json"), "{}").unwrap();
        let profile = detect_stack(dir.path());
        assert!(profile.is_monorepo);
    }

    #[test]
    fn test_detect_has_docs() {
        let dir = make_dir();
        fs::create_dir(dir.path().join("docs")).unwrap();
        let profile = detect_stack(dir.path());
        assert!(profile.has_docs);
    }

    #[test]
    fn test_generate_config_has_map_section() {
        let profile = WorkspaceProfile {
            stacks: vec![StackKind::Rust],
            is_monorepo: false,
            monorepo_subprojects: vec![],
            has_docs: false,
            has_readme: false,
            project_name: "meu-projeto".to_string(),
        };
        let config = generate_config(&profile);
        assert!(config.contains("[map]"));
    }

    #[test]
    fn test_generate_config_has_dirs() {
        let profile = WorkspaceProfile {
            stacks: vec![],
            is_monorepo: false,
            monorepo_subprojects: vec![],
            has_docs: false,
            has_readme: false,
            project_name: "test".to_string(),
        };
        let config = generate_config(&profile);
        assert!(config.contains("dirs ="));
    }

    #[test]
    fn test_run_init_creates_file() {
        let dir = make_dir();
        let result = run_workspace_init(dir.path().to_str().unwrap(), false);
        assert!(result.is_ok());
        assert!(dir.path().join(".ctx/config.toml").exists());
    }

    #[test]
    fn test_run_init_no_force_fails_if_exists() {
        let dir = make_dir();
        run_workspace_init(dir.path().to_str().unwrap(), false).unwrap();
        let result = run_workspace_init(dir.path().to_str().unwrap(), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_init_force_overwrites() {
        let dir = make_dir();
        run_workspace_init(dir.path().to_str().unwrap(), false).unwrap();
        let result = run_workspace_init(dir.path().to_str().unwrap(), true);
        assert!(result.is_ok());
    }
}
