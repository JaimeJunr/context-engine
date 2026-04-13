# ctx — Produto

> Este documento explica **por que o ctx existe**, qual problema ele resolve e para onde está indo. Leitura obrigatória antes de contribuir ou tomar decisões de roadmap.

---

## O Problema

Agentes de IA que trabalham em codebases grandes chegam "cegos". Sem contexto, um agente de planejamento precisa descobrir a estrutura do código por conta própria — gastando turns em `ls`, `find`, `grep`, lendo arquivos inteiros para entender o que é relevante.

O resultado: tokens desperdiçados, planos superficiais, implementações que erram o alvo.

A abordagem ingênua de mandar o repositório inteiro para o LLM não escala. Um monorepo com milhares de arquivos simplesmente não cabe no contexto — e mesmo que coubesse, seria ruído, não sinal.

---

## O que é o context-engine

`context-engine` é um framework Rust com **dois binários** que resolvem diferentes facetas do problema de contexto:

### `ctx` — Contexto de Código

Substitui `ls`, `tree`, `grep` por contexto inteligente para agentes.

1. **Descobre** arquivos relevantes (Scanner respeitando .gitignore)
2. **Extrai** assinaturas via Tree-sitter (funções, classes, tipos)
3. **Ranqueia** por relevância (BM25 + Personalized PageRank)
4. **Comprime** dentro de orçamento de tokens (cache SQLite, reutilização)

Redução típica: ~50 tokens (função inteira) → ~8 tokens (assinatura) + contexto mantido.

### `ctx-search` — Contexto de Documentação (RAG local)

Busca semântica **totalmente local** em acervos documentais.

1. **Indexa** documentação/specs (markdown, PDFs)
2. **Busca** por intenção (BM25 + embeddings semânticos via Ollama)
3. **Re-ranking** qualitativo via LLM (valida top-30)
4. **Offline:** sem APIs externas, sem vazamento de dados

Resultado: usuários/agentes encontram informações relevantes sem ler documentação inteira.

---

## Como Chegamos Aqui

### Fase 1 — Zero contexto

O `claude-auto-coder` (sistema de automação de tickets Jira que usa o `ctx`) começou sem nenhum contexto de código. O agente de planejamento recebia apenas o ticket e precisava descobrir tudo do zero.

O problema era evidente: planos genéricos, exploração excessiva, resultados ruins.

### Fase 2 — Repo-map ingênuo

Primeira tentativa: um script simples que gerava um mapa do repositório e indicava possíveis caminhos no código. Funcionava, mas era fraco — sem inteligência, sem priorização, sem economia real de tokens.

### Fase 3 — Pesquisa e estrutura

Estudo das abordagens de produtos consolidados: `aider`, `cursor`, `Claude Code`. Investigação de RAG, BM25, embeddings, local-first SQLite. Conclusão: a abordagem local-first com cache SQLite é mais adequada — adotada por diversas implementações sérias do mercado.

### Fase 4 — Pipeline estruturado (hoje)

Reescrita em Rust com pipeline bem definido: Scanner → Extractor (Tree-sitter) → Cache (SQLite) → Ranker (BM25 + PageRank) → Output. Reduz ~50 tokens de função para ~8 tokens de interface. Cache hit rate >80% em repos estáveis.

---

## Onde Estamos Hoje — Uso em Produção

### `ctx` — claude-auto-coder

Automatização de tickets Jira que usa `ctx` para contexto de código:

```
Jira ticket → Triage → Plan (ctx --title "ticket" --dirs "src" → repo_map)
  ↓
Opus recebe: ticket + repo_map (curado) → plano detalhado
  ↓
Implement → Push
```

Invocado em `plan.sh` para gerar repo_map. Sem ele, Opus começa às cegas explorando `ls`, `find`, `grep`.

**Resultado:** Planos mais precisos, menos exploração, economia de tokens.

### `ctx-search` — Fase Beta

Usado internamente para busca em documentação. Pipeline ainda em refinamento:

1. Indexar docs: `ctx-search add docs --source ./docs --include "**/*.md"`
2. Reindexar com embeddings: `ctx-search index docs --with-embed`
3. Usar em prompts: "Segundo a documentação, como funciona X?" + `ctx-search search docs "X"`

**Feedback:** Busca semântica funciona bem para contexto português. Próxima: integrar com prompts de LLM automaticamente.

---

## Visão de Futuro — Framework dos 3 Horizontes

### H1 — Core: Engenharia Sólida ✅ Em Progresso

Base está implementada (`ctx` e `ctx-search`), mas arquitetura precisa de refinamento:

- ✅ **Dois pipelines:** código + documentação
- ✅ **Modular:** Scanner, Extractor, Ranker, Chunker, Embedder independentes
- ✅ **SQLite:** cache persistente, reutilização entre invocações
- 🔄 **Próximos:** melhorar observabilidade, testes de edge-cases, otimizar performance

**Objetivo:** base sólida e confiável para tudo que vem a seguir.

### H2 — Emergente: RAG Multimodal Local ✅ Implementado

`ctx-search` (módulo `catalog/`) é a prova de conceito:

- ✅ **Busca local:** BM25 + embeddings semânticos (Ollama)
- ✅ **RRF fusion:** combina termo-a-termo + semântica
- ✅ **Re-ranking:** LLM valida top-30 (qualidade)
- ✅ **Offline:** sem APIs, sem vazar dados

Próximos passos: estender a múltiplos tipos de documentos (PDFs, arquivos binários via OCR), suportar múltiplas línguas, benchmarks públicos.

**Objetivo:** RAG local como commodity — reutilizável em qualquer projeto.

### H3 — Experimental: MCP Server + Compressão Inteligente

**MCP Server:** converter `ctx` e `ctx-search` em MCP servers para integração com Claude, outras IDEs.

- Interface de longa duração (não CLI one-shot)
- Persistência de contexto entre turns
- Streaming de resultados

**Compressão de Outputs:** estender `ctx-search` para comprimir outputs de comandos:

- `git log` retorna 5.000 linhas? Resumir semanticamente.
- `ls -R` retorna árvore gigante? Priorizar diretórios relevantes.
- `test output` falha em 100 testes? Agrupar por tipo de erro, retornar top-20.

Usar a inteligência já desenvolvida (BM25, RRF, LLM re-ranking) para comprimir outputs com **inteligência**, não só com filtros ingênuos.

**Objetivo:** `context-engine` como **plataforma unificada** para reduzir contexto irrelevante em code, docs, session history e command outputs.

---

## Status Atual — Mapa de Features

| Feature | Status | Notas |
|---------|--------|-------|
| **ctx** — Repo map curado | ✅ Prod | Scanner + Extractor + BM25 + PageRank |
| TypeScript/TSX extractor | ✅ Prod | Tree-sitter, completo |
| Python extractor | ✅ Prod | Tree-sitter, completo |
| Ruby extractor | ✅ Prod | Tree-sitter, completo |
| Groovy extractor | ✅ Prod | Gramática custom, compilada via build.rs |
| BM25 ranking | ✅ Prod | TF-IDF sobre assinaturas |
| Personalized PageRank | ✅ Prod | Com seeds, grafo de dependências |
| Token budget fitting | ✅ Prod | --max-tokens respeitado |
| SQLite cache (signatures) | ✅ Prod | SHA256 invalidation, >80% hit rate |
| **ctx-search** — RAG local | ✅ Prod | Chunker + Indexer + Embedder + Searcher |
| Markdown chunking | ✅ Prod | Heading-aware, 15% overlap |
| BM25 search | ✅ Prod | Term-based |
| Semantic search | ✅ Prod | Ollama embeddings (nomic-embed-text) |
| RRF fusion | ✅ Prod | Combina BM25 + vetorial |
| LLM re-ranking | ✅ Prod | llama3.2, top-30 validation |
| SQLite collections | ✅ Prod | Multiple named acervos |
| JSON output (ctx) | ✅ Prod | {path, score, signatures} |
| MCP server | 🔄 Roadmap | H3, estado persistente entre turns |
| Output compression | 🔄 Roadmap | H3, inteligente (não só filtros) |
| Múltiplas linguagens | 🔄 Roadmap | Java, C#, Go (extenders Tree-sitter) |
| PDF suporte | 🔄 Roadmap | OCR + chunking |

---

## Pontos Críticos Hoje

### Resolvidos ✅

- **RAG local:** implementado em `ctx-search` (BM25 + embeddings + re-ranking)
- **SQLite persistência:** cache de assinaturas + chunks + embeddings
- **Modularidade:** pipelines independentes, fácil estender

### A Resolver 🔄

1. **Observabilidade:** sem métricas de ranking quality (acertou top-5?)
   - Solução: adicionar modo debug com scores, benchmarks
   
2. **Suporte a linguagens:** manual (cada nova requer extractor Tree-sitter)
   - Mitigação: focar nas top 5 linguagens (TypeScript, Python, Ruby, Groovy, Java)
   - Futuro: suporte genérico via Tree-sitter base
   
3. **Performance:** embeddings Ollama podem ser lentos (primeira indexação)
   - Mitigação: lazy loading, cache de embeddings, batching
   
4. **Interface:** CLI one-shot, sem estado entre turns
   - Solução H3: MCP server com contexto persistente

5. **Teste em produção:** `claude-auto-coder` é usuário único
   - Próximo: feedback de usuários reais, cases públicos
