use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GlobalConfig {
    #[serde(default)]
    pub llm: LlmConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LlmConfig {
    pub endpoint: Option<String>,
    pub embedder: Option<String>,
    pub reranker: Option<String>,
}

/// Retorna o caminho do arquivo de configuração
pub fn config_path() -> PathBuf {
    if let Ok(custom) = std::env::var("CTX_CONFIG_DIR") {
        PathBuf::from(&custom).join("config.toml")
    } else {
        dirs::home_dir()
            .expect("home dir not found")
            .join(".ctx")
            .join("config.toml")
    }
}

/// Carrega configuração global; retorna Default se arquivo não existe
pub fn load() -> Result<GlobalConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(GlobalConfig::default());
    }
    let content = fs::read_to_string(&path)?;
    let cfg = toml::from_str(&content)?;
    Ok(cfg)
}

/// Salva configuração global no arquivo
pub fn save(cfg: &GlobalConfig) -> Result<()> {
    let path = config_path();
    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("config path has no parent"))?;
    fs::create_dir_all(dir)?;
    let content = toml::to_string_pretty(cfg)?;
    fs::write(&path, content)?;
    Ok(())
}

/// Define um valor na configuração usando notação com ponto (ex: "llm.endpoint")
pub fn set_key(cfg: &mut GlobalConfig, key: &str, value: &str) -> Result<()> {
    match key {
        "llm.endpoint" => cfg.llm.endpoint = Some(value.to_string()),
        "llm.embedder" => cfg.llm.embedder = Some(value.to_string()),
        "llm.reranker" => cfg.llm.reranker = Some(value.to_string()),
        _ => return Err(anyhow!("chave desconhecida: {}", key)),
    }
    Ok(())
}

/// Obtém um valor da configuração usando notação com ponto
pub fn get_key(cfg: &GlobalConfig, key: &str) -> Result<Option<String>> {
    match key {
        "llm.endpoint" => Ok(cfg.llm.endpoint.clone()),
        "llm.embedder" => Ok(cfg.llm.embedder.clone()),
        "llm.reranker" => Ok(cfg.llm.reranker.clone()),
        _ => Err(anyhow!("chave desconhecida: {}", key)),
    }
}

/// Prompt interativo com valor padrão
fn prompt(question: &str, default: Option<&str>) -> io::Result<String> {
    print!("{} ", question);
    if let Some(d) = default {
        print!("[{}]: ", d);
    } else {
        print!(": ");
    }
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();

    if trimmed.is_empty() {
        Ok(default.unwrap_or("").to_string())
    } else {
        Ok(trimmed)
    }
}

/// Estrutura para resposta /v1/models
#[derive(Debug, serde::Deserialize)]
struct ModelsResponse {
    data: Vec<ModelInfo>,
}

#[derive(Debug, serde::Deserialize)]
struct ModelInfo {
    id: String,
}

/// Busca modelos disponíveis no endpoint
fn fetch_models(endpoint: &str) -> Result<Vec<String>> {
    let url = format!("{}/v1/models", endpoint.trim_end_matches('/'));
    let client = reqwest::blocking::Client::new();
    let resp = client.get(&url).send()?;
    let data: ModelsResponse = resp.json()?;
    Ok(data.data.into_iter().map(|m| m.id).collect())
}

/// Exibe lista numerada de opções e retorna a escolha do usuário
fn choose_from_list(items: &[String], default_idx: usize) -> io::Result<String> {
    println!("\nModelos disponíveis:");
    for (i, item) in items.iter().enumerate() {
        println!("  {} — {}", i + 1, item);
    }
    let default_item = items.get(default_idx).map(|s| s.as_str());
    let choice = prompt("Escolha um (número ou nome)", default_item)?;

    // Se é número, converte para índice
    if let Ok(idx) = choice.parse::<usize>() {
        items
            .get(idx.saturating_sub(1))
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "índice inválido"))
    } else {
        // Se é string, procura nos items
        items
            .iter()
            .find(|item| **item == choice)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "modelo não encontrado"))
    }
}

/// Wizard interativo para configurar endpoint e modelos
pub fn run_init_wizard() -> Result<()> {
    let mut cfg = load()?;

    // 1. Pergunta endpoint
    let endpoint_default = cfg
        .llm
        .endpoint
        .as_deref()
        .unwrap_or("http://localhost:8080");
    let endpoint = prompt("Endpoint LLM", Some(endpoint_default))?;
    cfg.llm.endpoint = Some(endpoint.clone());

    // 2. Tenta conectar e buscar modelos
    println!("\nConectando em {}...", endpoint);
    let models = match fetch_models(&endpoint) {
        Ok(m) => {
            println!("✓ {} modelos encontrados", m.len());
            m
        }
        Err(e) => {
            println!("⚠ Não foi possível conectar: {}", e);
            println!("Continuando com configuração manual...");
            vec![]
        }
    };

    // 3. Escolhe embedder
    let embedder = if models.is_empty() {
        // Se falhou, pede string simples
        prompt(
            "Modelo de embedding",
            cfg.llm.embedder.as_deref().or(Some("qwen3-embedding")),
        )?
    } else {
        // Se temos models, deixa usuário escolher
        choose_from_list(&models, 0)?
    };
    cfg.llm.embedder = Some(embedder);

    // 4. Escolhe reranker
    let reranker = if models.is_empty() {
        prompt(
            "Modelo de reranking",
            cfg.llm.reranker.as_deref().or(Some("gemma")),
        )?
    } else {
        choose_from_list(&models, 0)?
    };
    cfg.llm.reranker = Some(reranker);

    // 5. Salva
    save(&cfg)?;
    let path = config_path();
    println!("\n✓ Configuração salva em {}", path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    struct ConfigDirGuard {
        dir: PathBuf,
    }

    impl ConfigDirGuard {
        fn new(dir: PathBuf) -> Self {
            std::env::set_var("CTX_CONFIG_DIR", &dir);
            ConfigDirGuard { dir }
        }
    }

    impl Drop for ConfigDirGuard {
        fn drop(&mut self) {
            std::env::remove_var("CTX_CONFIG_DIR");
            let _ = fs::remove_file(self.dir.join("config.toml"));
        }
    }

    fn test_config_dir() -> PathBuf {
        let dir = std::env::temp_dir().join("ctx_test_config");
        let _ = fs::create_dir_all(&dir);
        // Remove config.toml anterior se existir
        let _ = fs::remove_file(dir.join("config.toml"));
        dir
    }

    #[test]
    fn test_load_missing_file() {
        let dir = test_config_dir();
        let _guard = ConfigDirGuard::new(dir);
        let cfg = load().expect("should load default");
        assert!(cfg.llm.endpoint.is_none());
        assert!(cfg.llm.embedder.is_none());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = test_config_dir();
        let _guard = ConfigDirGuard::new(dir);

        let mut cfg = GlobalConfig::default();
        cfg.llm.endpoint = Some("http://localhost:8080".to_string());
        cfg.llm.embedder = Some("test-embed".to_string());
        save(&cfg).expect("save failed");

        let loaded = load().expect("load failed");
        assert_eq!(
            loaded.llm.endpoint,
            Some("http://localhost:8080".to_string())
        );
        assert_eq!(loaded.llm.embedder, Some("test-embed".to_string()));
    }

    #[test]
    fn test_set_key_valid() {
        let mut cfg = GlobalConfig::default();
        set_key(&mut cfg, "llm.endpoint", "http://test").expect("set failed");
        assert_eq!(cfg.llm.endpoint, Some("http://test".to_string()));

        set_key(&mut cfg, "llm.embedder", "embed-model").expect("set failed");
        assert_eq!(cfg.llm.embedder, Some("embed-model".to_string()));
    }

    #[test]
    fn test_set_key_invalid() {
        let mut cfg = GlobalConfig::default();
        let result = set_key(&mut cfg, "invalid.key", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_key() {
        let mut cfg = GlobalConfig::default();
        cfg.llm.endpoint = Some("http://test".to_string());

        let val = get_key(&cfg, "llm.endpoint").expect("get failed");
        assert_eq!(val, Some("http://test".to_string()));

        let val = get_key(&cfg, "llm.embedder").expect("get failed");
        assert_eq!(val, None);
    }
}
