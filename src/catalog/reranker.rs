// Reranking qualitativo via API OpenAI-compatible (RD-13, RD-14)
// Geração de variantes de query (RD-09) e hipótese (RD-09-A Expanded)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::cache_ops::{self, OP_RERANK_JUDGMENT};
use super::store;

const DEFAULT_BASE_URL: &str = "http://localhost:11434";
const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(5 * 60);

struct LazyClient {
    client: reqwest::blocking::Client,
    last_used: Instant,
}

static CLIENT_STATE: Mutex<Option<LazyClient>> = Mutex::new(None);

fn get_client() -> reqwest::blocking::Client {
    let mut guard = CLIENT_STATE.lock().unwrap();
    let now = Instant::now();

    if let Some(ref state) = *guard {
        if now.duration_since(state.last_used) > INACTIVITY_TIMEOUT {
            *guard = None;
        }
    }

    if guard.is_none() {
        *guard = Some(LazyClient {
            client: reqwest::blocking::Client::new(),
            last_used: now,
        });
    }

    let state = guard.as_mut().unwrap();
    state.last_used = now;
    state.client.clone()
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    stream: bool,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResp,
}

#[derive(Deserialize)]
struct ChatMessageResp {
    content: String,
}

// Geração via API OpenAI-compatible (/v1/chat/completions)
fn llm_generate(model: &str, prompt: &str, base_url: Option<&str>) -> Result<String> {
    let url = format!(
        "{}/v1/chat/completions",
        base_url.unwrap_or(DEFAULT_BASE_URL).trim_end_matches('/')
    );
    let client = get_client();
    let body = ChatRequest {
        model,
        messages: vec![ChatMessage {
            role: "user",
            content: prompt,
        }],
        stream: false,
    };
    let http_resp = client.post(&url).json(&body).send()?;
    let status = http_resp.status();
    if !status.is_success() {
        let body = http_resp.text().unwrap_or_default();
        anyhow::bail!("LLM API retornou {}: {}", status, body);
    }
    let resp: ChatResponse = http_resp.json()?;
    resp.choices
        .into_iter()
        .next()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| anyhow::anyhow!("resposta de chat vazia"))
}

// RD-09: gera 2-3 variações semânticas da query original
pub fn generate_query_variants(
    model: &str,
    query: &str,
    count: usize,
    base_url: Option<&str>,
) -> Result<Vec<String>> {
    let prompt = format!(
        "Gere exatamente {} reformulações semânticas da seguinte pergunta, uma por linha, sem numeração, sem explicação:\n{}",
        count, query
    );
    let response = llm_generate(model, &prompt, base_url)?;
    let variants: Vec<String> = response
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && l != query)
        .take(count)
        .collect();
    Ok(variants)
}

// RD-09-A Expanded: gera hipótese de resposta ideal para orientar busca
pub fn generate_hypothesis(model: &str, query: &str, base_url: Option<&str>) -> Result<String> {
    let prompt = format!(
        "Escreva um parágrafo que seria a resposta ideal para a seguinte pergunta. Seja conciso:\n{}",
        query
    );
    llm_generate(model, &prompt, base_url)
}

// RD-13/RD-14: reranking com julgamento binário pertinente/não pertinente
pub fn rerank(
    model: &str,
    query: &str,
    candidates: &[(i64, f64)],
    collection: &str,
    base_url: Option<&str>,
) -> Result<Vec<(i64, f64)>> {
    let mut results: Vec<(i64, f64)> = Vec::new();

    // Busca todos os chunks uma vez para evitar N queries
    let all_chunks = store::get_all_chunks_with_embeddings(collection)?;
    let chunk_map: std::collections::HashMap<i64, &str> = all_chunks
        .iter()
        .map(|c| (c.id, c.content.as_str()))
        .collect();

    for (chunk_id, base_score) in candidates {
        let content = match chunk_map.get(chunk_id) {
            Some(c) => *c,
            None => continue,
        };

        let is_relevant = judge_relevance(model, query, *chunk_id, content, base_url)?;

        if is_relevant {
            // RD-14: incrementa score com multiplicador de relevância qualitativa
            let boosted = base_score * 1.5;
            results.push((*chunk_id, boosted));
        }
        // Candidatos não pertinentes são removidos (RD-14)
    }

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    Ok(results)
}

// Julgamento binário de pertinência (com cache RD-22)
fn judge_relevance(
    model: &str,
    query: &str,
    chunk_id: i64,
    content: &str,
    base_url: Option<&str>,
) -> Result<bool> {
    let cache_key = format!("{}:{}:{}", model, query, chunk_id);

    // RD-22: verifica cache
    if let Some(cached) = cache_ops::get_cached::<bool>(&cache_key, OP_RERANK_JUDGMENT) {
        return Ok(cached);
    }

    let prompt = format!(
        "Você é um avaliador de relevância. Responda SOMENTE com 'SIM' ou 'NÃO'.\n\
         Pergunta: {}\n\
         Fragmento: {}\n\
         Este fragmento responde ou é relevante para a pergunta?",
        query,
        &content[..content.len().min(500)]
    );

    let response = llm_generate(model, &prompt, base_url)?;
    let upper = response.trim().to_uppercase();
    let relevant = upper == "SIM" || upper.starts_with("SIM\n") || upper.starts_with("SIM ");

    let _ = cache_ops::set_cached(&cache_key, OP_RERANK_JUDGMENT, &relevant);

    Ok(relevant)
}
