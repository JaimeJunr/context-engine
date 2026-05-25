# Estendendo o Projeto

A estrutura modular ([modules.md](modules.md)) dita onde cada extensão deve morar.

## Adicionar suporte a linguagem (extração de assinaturas)

1. Criar `src/pipelines/map/extractors/<lang>.rs` implementando o trait `Extractor`.
2. Registrar dispatch em `src/pipelines/map/extractors/mod.rs`.
3. Testar em `tests/integration.rs`.

## Estender Catalog (RAG)

- **Novos subcomandos**: editar `src/main.rs` (clap) + `src/pipelines/catalog/mod.rs`.
- **Nova estratégia de busca**: implementar em `src/pipelines/catalog/searcher.rs` ou novo módulo `src/pipelines/catalog/<estratégia>.rs`.
- **Re-ranking diferente**: `src/pipelines/catalog/reranker.rs`.

## Adicionar filtro de comando ao Exec

1. Criar/editar `src/pipelines/exec/filters/<família>.rs` com função que retorna `FilterConfig`.
2. Registrar match em `src/pipelines/exec/registry.rs::lookup_specific` ou `lookup_generic`.
3. Adicionar fixture em `tests/fixtures/` e teste em `src/pipelines/exec/pipeline.rs` ou `tests/filter_snapshots.rs`.

## Adicionar integração com novo agente

1. Implementar `src/integrations/agents/<agente>.rs` com impl do trait `AgentInstaller`.
2. Adicionar variante em `AgentName` enum (`src/integrations/agents/mod.rs`).
3. Roteamento em `installer_for()`.
4. Testes em `tests/agent_install.rs`.

## Adicionar nova camada de integração (MCP, session continuity, etc)

1. Criar `src/integrations/<nome>/mod.rs`.
2. Registrar em `src/integrations/mod.rs`.
3. Pode importar de `pipelines/*` e `shared/*`. **Não** pode ser importado por `pipelines/*` (proíbe ciclos).

## Adicionar utilitário cross-cutting

Se algo é usado por múltiplos pipelines (ex: tokenizer, cache, config), vai em `src/shared/`.

Critério: se só **um** pipeline usa, mantenha dentro daquele pipeline. Só promove para `shared/` quando houver segundo consumidor.

## Padrões a seguir

- Ver [design-patterns.md](design-patterns.md) para invariantes (imutabilidade, error handling, naming).
- Ver [docs/competitors/](../competitors/) para inspirações de features futuras já mapeadas.
