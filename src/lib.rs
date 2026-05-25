// Estrutura modular:
//
// - `pipelines/` — lógica de domínio (map, catalog, exec)
// - `integrations/` — interfaces externas (agents/hooks, futuro mcp, session)
// - `shared/` — utilitários cross-cutting (cache, config, tokenizer, workspace)
//
// Regras de dependência:
//   shared       ← independente
//   pipelines/*  ← pode importar shared
//   integrations ← pode importar pipelines + shared
//   pipelines    ← NÃO importa integrations (proíbe ciclos)

pub mod integrations;
pub mod pipelines;
pub mod shared;

// Entry points mais usados — atalhos de conveniência.
// Consumidores que precisem de algo mais específico devem importar
// pelo caminho canônico (ex: `context_engine::pipelines::catalog`).
pub use pipelines::exec::run_proxy;
pub use pipelines::map::run;
