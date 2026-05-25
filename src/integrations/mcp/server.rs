// MCP server stdio expondo os pipelines do `ctx` como tools.
//
// 4 tools: ctx_exec, ctx_search, ctx_map, ctx_list.
// Cada uma é um wrapper fino sobre as APIs públicas de `pipelines/`.

use anyhow::Result;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData, ServerHandler, ServiceExt,
};

use crate::pipelines::graph::{self, QueryOptions as GraphQuery};
use crate::pipelines::{catalog, exec, map};

// =========================================================================
// Schemas de input (gerados via schemars::JsonSchema)
// =========================================================================

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExecInput {
    /// Comando + argumentos como lista. Ex: ["git", "status"]
    pub command: Vec<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchInput {
    /// Nome do acervo previamente registrado via `ctx add`
    pub collection: String,
    /// Query de busca (pode prefixar com `exact:`, `conceptual:`, `expanded:`)
    pub query: String,
    /// Número máximo de resultados (padrão: 10)
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    10
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MapInput {
    /// Título/descrição do ticket que guia o ranqueamento
    pub title: String,
    /// Diretórios alvo
    pub dirs: Vec<String>,
    /// Número fixo de arquivos (0 = usar max_tokens)
    #[serde(default)]
    pub top: usize,
    /// Budget máximo de tokens (padrão: 4096)
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
    /// Dirs seed para Personalized PageRank
    #[serde(default)]
    pub seeds: Option<Vec<String>>,
    /// Formato de saída: "text" ou "json"
    #[serde(default = "default_format")]
    pub format: String,
    /// Profundidade máxima de scan (padrão: 15)
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

fn default_max_tokens() -> usize {
    4096
}
fn default_format() -> String {
    "text".to_string()
}
fn default_max_depth() -> usize {
    15
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListInput {}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GraphIndexInput {
    /// Diretórios a indexar
    pub dirs: Vec<String>,
    /// Profundidade máxima (default: 15)
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CallersInput {
    /// Nome do símbolo (function/method/class…)
    pub name: String,
    /// Query opcional para ranquear resultados por relevância
    #[serde(default)]
    pub query: Option<String>,
    /// Budget de tokens (1 token ≈ 4 chars)
    #[serde(default)]
    pub max_tokens: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CalleesInput {
    /// Identificador qualificado: `file::name`
    pub qualified: String,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TraceInput {
    /// Nome do símbolo
    pub name: String,
    /// Profundidade máxima da cadeia (default: 3)
    #[serde(default = "default_trace_depth")]
    pub depth: usize,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<usize>,
}

fn default_trace_depth() -> usize {
    3
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ImpactInput {
    pub name: String,
    #[serde(default = "default_impact_depth")]
    pub depth: usize,
}

fn default_impact_depth() -> usize {
    2
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NodeInput {
    pub name: String,
}

// =========================================================================
// Server
// =========================================================================

#[derive(Debug, Clone)]
pub struct CtxServer {
    tool_router: ToolRouter<Self>,
}

impl Default for CtxServer {
    fn default() -> Self {
        Self::new()
    }
}

impl CtxServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl CtxServer {
    /// Executa um comando shell aplicando o pipeline de compressão do `ctx exec`.
    /// Para comandos cobertos (git, cargo, docker, kubectl, aws, etc), retorna
    /// output filtrado economizando tokens. Sem filtro, passa-through.
    #[tool(
        name = "ctx_exec",
        description = "Executa comando shell e retorna output filtrado/comprimido. Cobre git, cargo, npm, docker, kubectl, aws, gh, gradle, maven, pytest, ls, find, grep, curl e mais."
    )]
    async fn exec(
        &self,
        Parameters(ExecInput { command }): Parameters<ExecInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let result = exec::run_proxy_capture(command)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let body = format!("[exit={}]\n{}", result.exit_code, result.stdout);
        Ok(CallToolResult::success(vec![Content::text(body)]))
    }

    /// Busca semântica em um acervo do `ctx catalog` (indexação prévia necessária).
    #[tool(
        name = "ctx_search",
        description = "Busca semântica em acervo catalogado. Retorna trechos relevantes com score."
    )]
    async fn search(
        &self,
        Parameters(SearchInput {
            collection,
            query,
            top_k,
        }): Parameters<SearchInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let results = catalog::search(&collection, &query, top_k)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Gera repo map curado para LLMs — BM25 + Personalized PageRank com budget de tokens.
    #[tool(
        name = "ctx_map",
        description = "Gera mapa de repositório com assinaturas extraídas e ranqueadas por relevância à query."
    )]
    async fn map(
        &self,
        Parameters(MapInput {
            title,
            dirs,
            top,
            max_tokens,
            seeds,
            format,
            max_depth,
        }): Parameters<MapInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let result = map::run(
            &title,
            &dirs,
            top,
            max_tokens,
            seeds.as_deref(),
            &format,
            max_depth,
        );
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Lista todos os acervos catalogados (nome + timestamp da última indexação).
    #[tool(
        name = "ctx_list",
        description = "Lista acervos do ctx catalog disponíveis para busca."
    )]
    async fn list(
        &self,
        Parameters(ListInput {}): Parameters<ListInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let collections = catalog::list_collections()
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&collections)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Indexa diretórios populando o grafo (callers/callees ficam consultáveis depois).
    #[tool(
        name = "ctx_graph_index",
        description = "Indexa diretórios populando o grafo de símbolos. Necessário antes de ctx_callers/callees/trace/impact."
    )]
    async fn graph_index(
        &self,
        Parameters(GraphIndexInput { dirs, max_depth }): Parameters<GraphIndexInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let stats = graph::index(&dirs, max_depth)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&stats)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Quem chama este símbolo? Resultados ranqueados por relevância à query e respeitam budget de tokens.
    #[tool(
        name = "ctx_callers",
        description = "Lista funções/métodos que chamam o símbolo dado. Resultados ranqueados por BM25(query) e budget de tokens."
    )]
    async fn callers(
        &self,
        Parameters(CallersInput {
            name,
            query,
            max_tokens,
        }): Parameters<CallersInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let conn = graph::store::open_default()
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let result = graph::callers(
            &conn,
            &name,
            &GraphQuery {
                query,
                max_tokens,
                depth: None,
            },
        )
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// O que esta função chama? Útil para mapear dependências saindo de um símbolo.
    #[tool(
        name = "ctx_callees",
        description = "Lista símbolos chamados a partir do identificador qualificado dado."
    )]
    async fn callees(
        &self,
        Parameters(CalleesInput {
            qualified,
            query,
            max_tokens,
        }): Parameters<CalleesInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let conn = graph::store::open_default()
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let result = graph::callees(
            &conn,
            &qualified,
            &GraphQuery {
                query,
                max_tokens,
                depth: None,
            },
        )
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Cadeia de callers (BFS reverso) até `depth` níveis.
    #[tool(
        name = "ctx_trace",
        description = "Retorna a cadeia de callers até depth níveis. Útil para entender 'como chego até esta função'."
    )]
    async fn trace(
        &self,
        Parameters(TraceInput {
            name,
            depth,
            query,
            max_tokens,
        }): Parameters<TraceInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let conn = graph::store::open_default()
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let result = graph::trace(
            &conn,
            &name,
            &GraphQuery {
                query,
                max_tokens,
                depth: Some(depth),
            },
        )
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Análise de impacto: o que quebra se eu mudar este símbolo?
    #[tool(
        name = "ctx_impact",
        description = "Lista código afetado por uma mudança no símbolo (callers diretos + indiretos até depth)."
    )]
    async fn impact(
        &self,
        Parameters(ImpactInput { name, depth }): Parameters<ImpactInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let conn = graph::store::open_default()
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let result = graph::impact(
            &conn,
            &name,
            &GraphQuery {
                depth: Some(depth),
                ..Default::default()
            },
        )
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Onde está definido X? Retorna todas as definições com nome igual.
    #[tool(
        name = "ctx_node",
        description = "Localiza definições de um símbolo no grafo (pode retornar múltiplas se houver overload/escopos)."
    )]
    async fn graph_node(
        &self,
        Parameters(NodeInput { name }): Parameters<NodeInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let conn = graph::store::open_default()
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let nodes = graph::node(&conn, &name)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&nodes)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for CtxServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::default()
            .with_server_info(rmcp::model::Implementation::new(
                "ctx",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Use ctx_exec para comandos shell com compressão automática (git, cargo, docker, kubectl…); \
                 ctx_search para busca semântica em acervos indexados; \
                 ctx_map para gerar mapa curado de repositório; \
                 ctx_list para descobrir acervos disponíveis.",
            )
    }
}

// =========================================================================
// Entrypoint
// =========================================================================

/// Sobe o server em stdio e bloqueia até o cliente encerrar.
pub async fn serve() -> Result<()> {
    let server = CtxServer::new();
    let running = server.serve(stdio()).await?;
    running.waiting().await?;
    Ok(())
}
