// MCP server expondo os pipelines do `ctx` como tools para agentes
// compatíveis com Model Context Protocol (Claude Desktop, Cursor, opencode…).
//
// Tools expostas:
//   - ctx_exec      executa comando shell com filtros de compressão
//   - ctx_search    busca semântica em acervo do catalog
//   - ctx_map       gera repo map curado
//   - ctx_list      lista acervos catalogados
//
// Transporte: stdio (long-running process, JSON-RPC).

pub mod server;

use anyhow::Result;

/// Sobe o MCP server em stdio até o cliente fechar a conexão.
pub async fn serve() -> Result<()> {
    server::serve().await
}

/// Lista nominal das tools expostas — útil para `ctx mcp tools` (debug)
/// e para registro em settings.json de agentes.
pub fn tool_names() -> &'static [&'static str] {
    &[
        "ctx_exec",
        "ctx_search",
        "ctx_map",
        "ctx_list",
        "ctx_graph_index",
        "ctx_callers",
        "ctx_callees",
        "ctx_trace",
        "ctx_impact",
        "ctx_node",
    ]
}
