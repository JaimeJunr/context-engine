use serde::{Deserialize, Serialize};

// Configuração de um acervo documental (RD-17, seção 4.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub sources: Vec<String>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub path_contexts: Vec<PathContext>,
    pub pre_index_cmd: Option<String>,
    pub embedder_model: Option<String>,
    pub reranker_model: Option<String>,
    #[serde(default)]
    pub llm_endpoint: Option<String>,
}

impl Collection {
    // Prioridade: CTX_EMBEDDER_MODEL > per-collection > config global > built-in default
    pub fn embedder_model_or_default(&self) -> String {
        // 1. env var
        if let Ok(v) = std::env::var("CTX_EMBEDDER_MODEL") {
            if !v.is_empty() {
                return v;
            }
        }
        // 2. per-collection
        if let Some(ref m) = self.embedder_model {
            return m.clone();
        }
        // 3. config global
        if let Ok(cfg) = crate::shared::config::load() {
            if let Some(m) = cfg.llm.embedder {
                return m;
            }
        }
        // 4. built-in default
        "nomic-embed-text".to_string()
    }

    // Prioridade: CTX_RERANKER_MODEL > per-collection > config global > built-in default
    pub fn reranker_model_or_default(&self) -> String {
        // 1. env var
        if let Ok(v) = std::env::var("CTX_RERANKER_MODEL") {
            if !v.is_empty() {
                return v;
            }
        }
        // 2. per-collection
        if let Some(ref m) = self.reranker_model {
            return m.clone();
        }
        // 3. config global
        if let Ok(cfg) = crate::shared::config::load() {
            if let Some(m) = cfg.llm.reranker {
                return m;
            }
        }
        // 4. built-in default
        "llama3.2".to_string()
    }

    // Prioridade: CTX_LLM_ENDPOINT > per-collection > config global > None
    pub fn llm_endpoint_resolved(&self) -> Option<String> {
        // 1. env var
        if let Ok(v) = std::env::var("CTX_LLM_ENDPOINT") {
            if !v.is_empty() {
                return Some(v);
            }
        }
        // 2. per-collection
        if let Some(ref e) = self.llm_endpoint {
            return Some(e.clone());
        }
        // 3. config global
        if let Ok(cfg) = crate::shared::config::load() {
            if let Some(e) = cfg.llm.endpoint {
                return Some(e);
            }
        }
        // 4. None
        None
    }
}

// Metadado descritivo associado a um caminho do acervo (seção 4.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathContext {
    pub path_prefix: String,
    pub description: String,
    pub priority: i64,
}

// Documento catalogado (seção 4.2)
#[derive(Debug, Clone)]
pub struct Document {
    pub id: i64,
    pub collection: String,
    pub path: String,
    pub content_hash: String,
    pub indexed_at: String,
}

// Fragmento de conteúdo com assinatura semântica (seção 4.3)
#[derive(Debug, Clone)]
pub struct Chunk {
    pub id: i64,
    pub doc_id: i64,
    pub content: String,
    pub start_offset: usize,
    pub embedding: Option<Vec<f32>>,
    pub status: ChunkStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChunkStatus {
    Pending,
    Done,
}

impl ChunkStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ChunkStatus::Pending => "pending",
            ChunkStatus::Done => "done",
        }
    }

    pub fn parse_status(s: &str) -> Self {
        match s {
            "done" => ChunkStatus::Done,
            _ => ChunkStatus::Pending,
        }
    }
}

// Resultado de recuperação (seção 4.6)
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub doc_path: String,
    pub chunk_id: i64,
    pub chunk_text: String,
    pub chunk_offset: usize,
    pub score: f64,
    pub context: Option<String>,
}

// Relatório de saúde do acervo (seção 4.7)
#[derive(Debug, Clone)]
pub struct CollectionHealth {
    pub name: String,
    pub total_documents: usize,
    pub pending_embeddings: usize,
    pub last_indexed: Option<String>,
    pub consistent: bool,
}

// Modalidade de busca (RD-09-A)
#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    Auto,
    Exact,
    Conceptual,
    Expanded,
}

impl SearchMode {
    // Detecta qualificador de modalidade no início da query
    pub fn parse_from_query(query: &str) -> (SearchMode, String) {
        let lower = query.trim().to_lowercase();
        if let Some(rest) = lower.strip_prefix("exact:") {
            return (SearchMode::Exact, rest.trim().to_string());
        }
        if let Some(rest) = lower.strip_prefix("conceptual:") {
            return (SearchMode::Conceptual, rest.trim().to_string());
        }
        if let Some(rest) = lower.strip_prefix("expanded:") {
            return (SearchMode::Expanded, rest.trim().to_string());
        }
        (SearchMode::Auto, query.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search_mode_exact() {
        let (mode, q) = SearchMode::parse_from_query("exact:como funciona o chunking");
        assert_eq!(mode, SearchMode::Exact);
        assert_eq!(q, "como funciona o chunking");
    }

    #[test]
    fn test_parse_search_mode_auto() {
        let (mode, q) = SearchMode::parse_from_query("como funciona o chunking");
        assert_eq!(mode, SearchMode::Auto);
        assert_eq!(q, "como funciona o chunking");
    }

    struct EnvVarGuard(&'static str);
    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            std::env::remove_var(self.0);
        }
    }

    #[test]
    fn test_embedder_default_when_no_config() {
        // Sem env var, sem per-collection → usa built-in default
        let _guard = EnvVarGuard("CTX_EMBEDDER_MODEL");
        std::env::remove_var("CTX_EMBEDDER_MODEL");
        let col = Collection {
            name: "test".to_string(),
            sources: vec![],
            include_patterns: vec![],
            exclude_patterns: vec![],
            path_contexts: vec![],
            pre_index_cmd: None,
            embedder_model: None,
            reranker_model: None,
            llm_endpoint: None,
        };
        // Sem per-collection e sem env, pode pegar do config global
        // (esse teste depende de estado externo, então apenas verifica que retorna algo)
        let model = col.embedder_model_or_default();
        assert!(!model.is_empty(), "model não deve estar vazio");
    }

    #[test]
    fn test_per_collection_overrides_default() {
        let _guard = EnvVarGuard("CTX_EMBEDDER_MODEL");
        std::env::remove_var("CTX_EMBEDDER_MODEL");
        let col = Collection {
            name: "test".to_string(),
            sources: vec![],
            include_patterns: vec![],
            exclude_patterns: vec![],
            path_contexts: vec![],
            pre_index_cmd: None,
            embedder_model: Some("per-collection-model".to_string()),
            reranker_model: None,
            llm_endpoint: None,
        };
        let model = col.embedder_model_or_default();
        // per-collection sempre sobrescreve tudo (exceto env var)
        assert_eq!(model, "per-collection-model");
    }

    #[test]
    fn test_env_var_overrides_all() {
        let _guard = EnvVarGuard("CTX_EMBEDDER_MODEL");
        std::env::set_var("CTX_EMBEDDER_MODEL", "env-model");
        let col = Collection {
            name: "test".to_string(),
            sources: vec![],
            include_patterns: vec![],
            exclude_patterns: vec![],
            path_contexts: vec![],
            pre_index_cmd: None,
            embedder_model: Some("per-collection-model".to_string()),
            reranker_model: None,
            llm_endpoint: None,
        };
        let model = col.embedder_model_or_default();
        // env var sobrescreve tudo
        assert_eq!(model, "env-model");
    }
}
