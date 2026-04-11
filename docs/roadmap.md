# Roadmap: Context Engine — Próximos Passos

Planejamento futuro do context-engine: próximos passos priorizados, recomendações com código de referência, e métricas para avaliar progresso.

> **Nota:** Exemplos de código em Python são referências conceituais da versão original. Implementação real será em Rust.

---

## Framework dos 3 Horizontes

O roadmap segue o Framework dos 3 Horizontes para organizar prioridades:

| Horizonte | Foco | Objetivo |
|---|---|---|
| **H1 — Core** | Limpar e solidificar o que existe | Base sólida para crescer; módulos independentes; engenharia de produto |
| **H2 — Emergente** | Substituir o Context Mode | Continuidade de sessão, sandbox de execução, BM25 indexado de eventos |
| **H3 — Experimental** | Substituir RTK + incorporar QMD | Filtragem inteligente de outputs, busca semântica vetorial local (GGUF) |

Ver `docs/produto.md` para o contexto completo de cada horizonte.

---

## Próximos Passos

### Curto prazo (1-2 sprints) — Impacto alto, esforço baixo

1. **Fallback com embeddings para português puro**
   - Adicionar modelo leve (`all-MiniLM-L6-v2`, ~50MB, CPU)
   - Ativar quando: `bm25_top_score < 0.2 AND camel_case_tokens < 2`
   - Overhead: +2-3s latência, +10-15% cobertura
   - Implementar como feature flag no Rust

2. **Modo verbose/debug**
   - `--debug` flag que mostra:
     - Corpus size (total files, arquivos com sigs)
     - Top-5 BM25 scores brutos
     - Seeds vs PPR ranking diffs
   - Útil para diagnosticar why LLM ainda explora demais

3. **Integração com ripgrep para exploração emergency**
   - Se context_engine falha (timeout, erro), fallback para `rg pattern`
   - Resgate gracioso: melhor ter ripgrep output que nada

### Médio prazo (3-6 meses) — Scaling

1. **Ngram Indexing incremental** (quando corpus > 50k files)
   - Usar `git log --name-only` para detectar files changed
   - Reindex apenas delta desde último scan
   - Mantém performance O(1) em monorepos gigantes

2. **B — Mapa arquivo-nível (codebase-mapper)**
   - Em vez de mapear dirs, mapear `{dir}/{arquivo}` com frequência
   - Combina bem com Ngram Indexing (melhor targeting)
   - Depende de outro ciclo de geração do CODEBASE_MAP

3. **Métricas de cobertura**
   - Ao finalizar: relatar
     - % de arquivos com assinaturas extraídas
     - Distribuição de corpus por linguagem
     - Cache hit rate
   - Detectar linguagens não suportadas

### Longo prazo (6-12 meses) — Nice-to-have

1. **Cross-language symbol resolution**
   - Rastrear imports entre linguagens (Ruby <-> TypeScript em Rails+frontend)
   - Grafo unificado: Ruby -> TypeScript imports
   - Valor: quando fix em Rails requer mudança em frontend

2. **Incremental PPR updates**
   - Em vez de recompor grafo todo ticket, manter grafo em cache
   - Atualizar apenas edges/nodes afetados por changed files
   - Economiza 2-3 segundos em repos estáveis

3. **Análise de "dead code" via PPR**
   - Arquivos com PageRank < threshold = possível dead code
   - Report gerado mensalmente: "esses arquivos não são referenciados"

---

## Métricas de Sucesso Propostas

| Métrica | Baseline | Target | Como medir |
|---|---|---|---|
| **Turns gastos em exploração** | 6-8 | 1-2 | Audit logs (count tool calls em turn 1-3) |
| **Latência ctx** | <100ms | <50ms | Timer na invocação |
| **Cache hit rate** | N/A | >80% | Contar queries que usam SQLite vs parse |
| **Cobertura PPR vs BM25** | N/A | +15% better ranking | Comparar PageRank scores vs BM25 scores em top-5 |
| **Tickets português puro** | 10% fail | 5% fail | Rastrear tickets com score < 0.2 pré-embedding |

---

## Recomendações Específicas Baseadas em Pesquisa (2024-2025)

### 1. Implementar LLM Reranking (Phase 4.5)

Baseado em [InsertRank (ARXIV 2506.14086)](https://arxiv.org/html/2506.14086v1):

```python
# Referência conceitual (implementar em Rust ou como chamada externa)
def llm_rerank(top_10, query):
    """Reranqueia top-10 usando LLM (Haiku) + BM25 scores."""
    context = "\n".join([
        f"{i+1}. {path.name} (BM25={score:.2f})\n"
        f"   {extract_signatures(path)[:2]}"
        for i, (path, score) in enumerate(top_10)
    ])
    prompt = f"Query: {query}\nRank these files by relevance:\n{context}"
    response = run_claude_json("haiku", prompt)
    return reorder_by_llm_ranking(top_10, response)
```

**Ganho esperado:** +5-10% melhor ranking
**Custo:** +2 segundos latência

---

### 2. Considerar Harmonic Centrality para Repos Gigantes

Quando monorepo > 50k files, PageRank fica O(n^2):

```python
# Alternativa mais rápida (10x)
def harmonic_centrality(G):
    from networkx import harmonic_centrality
    return harmonic_centrality(G)
```

**Aplicar quando:** Se notar que ctx demora >5s

---

### 3. Replicar Cursor Semantic Search para Português Puro (2025-2026)

```python
# Referência conceitual — fallback para tickets em português puro (5-10% casos)

# Fase 1: BM25 normal
bm25_results = bm25_rank(query, corpus)  # 95% casos funcionam

# Fase 2: Se BM25 falha, ativa embedding
if bm25_results[0][1] < 0.2 and num_camel_case_tokens(query) < 2:
    from sentence_transformers import SentenceTransformer
    model = SentenceTransformer('all-MiniLM-L6-v2')  # 22MB, CPU rápido
    # ... cosine similarity + RRF
```

**Ganho esperado:** +15% cobertura para português puro
**Custo:** +2-3 segundos, ~50MB model file

---

### 4. Caching Inteligente de Embeddings (se implementar #3)

Reutilizar embeddings entre tickets via cache SQLite (similar ao cache de assinaturas existente).

**Ganho:** Queries 2+ ganham 80% speedup no embedding

---

## Visão Geral do Roadmap

```
Curto Prazo (1-2 sprints)
+-- LLM Reranking Phase 4.5
+-- Embeddings fallback para português puro
+-- Modo verbose/debug
+-- Ripgrep emergency fallback

Médio Prazo (3-6 meses)
+-- Ngram Indexing (quando corpus > 50k)
+-- Codebase-mapper arquivo-nível
+-- Métricas de cobertura

Longo Prazo (6-12 meses)
+-- Cross-language symbol resolution
+-- Incremental PPR updates
+-- Dead code analysis via PPR
```

Cada item é independente — pode ser implementado isoladamente quando necessidade surgir.
