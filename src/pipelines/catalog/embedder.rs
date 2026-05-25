// Geração de embeddings via API OpenAI-compatible /v1/embeddings (RD-06, RD-07, RD-08, RD-20, RD-21)
//
// Lazy loading: o client só é criado na primeira chamada (RD-20).
// Inactividade: após 5 min sem uso, o client é descartado (RD-21).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(5 * 60);
const DEFAULT_BASE_URL: &str = "http://localhost:11434";

struct LazyClient {
    client: reqwest::blocking::Client,
    last_used: Instant,
}

static CLIENT_STATE: Mutex<Option<LazyClient>> = Mutex::new(None);

fn get_client() -> reqwest::blocking::Client {
    let mut guard = CLIENT_STATE.lock().unwrap();
    let now = Instant::now();

    // RD-21: descarta client se inativo por mais de 5 min
    if let Some(ref state) = *guard {
        if now.duration_since(state.last_used) > INACTIVITY_TIMEOUT {
            *guard = None;
        }
    }

    // RD-20: cria client apenas quando necessário
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
struct EmbedRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Deserialize)]
struct EmbedResponse {
    data: Vec<EmbedData>,
}

#[derive(Deserialize)]
struct EmbedData {
    embedding: Vec<f32>,
}

// Gera embedding para um único texto via API OpenAI-compatible (/v1/embeddings)
pub fn embed_text(model: &str, text: &str, base_url: Option<&str>) -> Result<Vec<f32>> {
    let url = format!(
        "{}/v1/embeddings",
        base_url.unwrap_or(DEFAULT_BASE_URL).trim_end_matches('/')
    );
    let client = get_client();
    let body = EmbedRequest { model, input: text };
    let http_resp = client.post(&url).json(&body).send()?;
    let status = http_resp.status();
    if !status.is_success() {
        let body = http_resp.text().unwrap_or_default();
        anyhow::bail!("embedding API retornou {}: {}", status, body);
    }
    let resp: EmbedResponse = http_resp.json()?;
    resp.data
        .into_iter()
        .next()
        .map(|d| d.embedding)
        .ok_or_else(|| anyhow::anyhow!("resposta de embedding vazia"))
}

// Gera embeddings em lote, retornando na mesma ordem
pub fn embed_batch(model: &str, texts: &[String], base_url: Option<&str>) -> Result<Vec<Vec<f32>>> {
    texts
        .iter()
        .map(|t| embed_text(model, t, base_url))
        .collect()
}

// Cosine similarity entre dois vetores (para busca vetorial)
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| *x as f64 * *y as f64)
        .sum();
    let norm_a: f64 = a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

// Formata texto de entrada para embedding (RD-07: chunk + título como âncora)
pub fn format_embed_input(doc_title: &str, chunk_content: &str) -> String {
    if doc_title.is_empty() {
        chunk_content.to_string()
    } else {
        format!("{}\n\n{}", doc_title, chunk_content)
    }
}

// Extrai título do documento a partir do caminho do arquivo
pub fn doc_title_from_path(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .replace(['-', '_'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0f32, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0f32, 0.0];
        let b = vec![0.0f32, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn test_format_embed_input_with_title() {
        let result = format_embed_input("meu documento", "conteúdo aqui");
        assert!(result.starts_with("meu documento"));
        assert!(result.contains("conteúdo aqui"));
    }

    #[test]
    fn test_doc_title_from_path() {
        let title = doc_title_from_path("/home/user/docs/meu-arquivo.md");
        assert_eq!(title, "meu arquivo");
    }
}
