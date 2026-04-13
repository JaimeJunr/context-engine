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
    pub fn embedder_model_or_default(&self) -> &str {
        self.embedder_model.as_deref().unwrap_or("nomic-embed-text")
    }

    pub fn reranker_model_or_default(&self) -> &str {
        self.reranker_model.as_deref().unwrap_or("llama3.2")
    }

    // Prioridade: CTX_LLM_ENDPOINT env var > per-collection > None (usa default do módulo)
    pub fn llm_endpoint_resolved(&self) -> Option<String> {
        std::env::var("CTX_LLM_ENDPOINT")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| self.llm_endpoint.clone())
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
#[derive(Debug, Clone)]
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
}
