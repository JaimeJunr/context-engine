// Utilitários cross-cutting usados por múltiplos pipelines e integrações.
//
// Regra de dependência: este módulo NÃO importa de `pipelines/` nem `integrations/`.

pub mod cache;
pub mod config;
pub mod tokenizer;
pub mod workspace;
