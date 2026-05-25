// Interfaces externas: agentes de codificação, MCP server, sessões.
//
// Esta camada consome `pipelines/` como API. Não pode ser importada por
// `pipelines/` (proíbe ciclos).

pub mod agents;
pub mod mcp;
