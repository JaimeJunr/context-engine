# Estendendo o Projeto

## Adicionar Suporte a Nova Linguagem

1. Criar `src/extractors/<lang>.rs` implementando trait `Extractor`
2. Registrar dispatch em `src/extractors/mod.rs`
3. Testar em `tests/integration.rs`

Ver `guides/design-patterns.md` para padrões de código.

## Estender Catalog

1. Novos comandos: editar `src/main.rs` (clap) + `src/catalog/mod.rs`
2. Novos filtros de exec: editar `src/exec/pipeline.rs`
3. Estratégias de re-ranking: `src/catalog/reranker.rs`
