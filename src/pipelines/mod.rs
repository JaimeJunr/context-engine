// Pipelines de domínio do `ctx`.
//
// Cada pipeline é auto-contido e expõe uma API estável para `integrations/`
// e para o binário consumirem. Regras:
//
// - Pode importar de `shared/`
// - NÃO pode importar de `integrations/`
// - NÃO deve depender de outro pipeline (exceto via tipos comuns)

pub mod catalog;
pub mod exec;
pub mod graph;
pub mod map;
