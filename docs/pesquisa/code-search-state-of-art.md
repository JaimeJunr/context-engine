# Pesquisa: Code Search State of Art (2024-2026)

Este documento captura a pesquisa de mercado e estado da arte em busca de código para agentes IA. É tanto um registro do que foi investigado quanto uma referência para decisões futuras.

## O Problema

O `claude-auto-coder` tem um orçamento fixo de **15 turns** para o Opus gerar o plano de implementação. O `claude-codebase-mapper` fornece os **diretórios corretos** ("o bairro"), mas o Opus ainda desperdiça turns executando comandos exploratórios para encontrar os arquivos exatos ("o endereço").

### Fluxo atual (desperdício de turns)

```
Opus recebe: map -> app/dto/, app/controllers/
  -> turn 1: ls app/dto/
  -> turn 2: cat app/dto/FundA.groovy
  -> turn 3: cat app/dto/FundJsonDTO.groovy
  -> turn 4: ls app/controllers/
  -> turn 5: cat app/controllers/FundController.groovy
  -> turn 6+: finalmente entende o padrão e começa a planejar
```

Com 15 turns, o Opus esgota o orçamento explorando e não escreve o plano.

---

## Estratégias Avaliadas

### C — Injeção de Árvore de Arquivos (File Tree)

Antes de chamar o Opus, rodar `find <dir> -type f -name "*.groovy" | sort` nos diretórios mapeados e injetar a árvore no prompt.

**Vantagem:** simples, zero dependências, implementável imediatamente.
**Limitação:** pode ser verboso em diretórios grandes; não informa o conteúdo dos arquivos.

```
Opus recebe: map + file tree
  app/dto/FundJsonDTO.groovy
  app/dto/FundSummaryDTO.groovy
  app/controllers/FundController.groovy
  -> turn 1: cat app/dto/FundJsonDTO.groovy  <- vai direto ao arquivo certo
  -> turn 2: entende o padrão, planeja
```

### A — Grep Automático por Identificadores Técnicos

Extrair termos camelCase/PascalCase do título do ticket (ex: `highShares`, `holidayMarkets`) e fazer grep nos repos antes do prompt, retornando os arquivos que contêm esses identificadores.

**Vantagem:** entrega os arquivos exatos que contêm os símbolos do ticket.
**Limitação:** depende de o título ter identificadores de código; falha em tickets puramente descritivos em português.

**Lógica de extração (A2 — Smart Grep):**
- Regex para capturar camelCase (`[a-z][a-zA-Z0-9]+`) e PascalCase (`[A-Z][a-zA-Z0-9]+`)
- Filtrar stopwords e palavras genéricas curtas
- Fazer grep nos dirs matched do codebase map

**Combinação ideal:** A + C juntos — o grep encontra os arquivos com os símbolos do ticket, o file tree dá visão geral dos diretórios restantes.

### B — Mapa mais granular (arquivo, não diretório)

Atualizar o `claude-codebase-mapper` para indexar a nível de arquivo em vez de diretório.

**Status: pausado** — depende de outro ciclo de geração do codebase-mapper.

---

## Comparativo: Grep vs context_engine

| Aspecto | grep tradicional | context_engine |
|---|---|---|
| **Estratégia** | Busca textual linear | BM25 + Tree-sitter AST |
| **Resultado** | Lista de linhas (código completo) | Assinaturas (sem corpo) |
| **Compressão** | 0% (verboso demais) | ~84% (50 tokens -> 8 tokens) |
| **Indexação** | 0 (relê cada arquivo toda vez) | SQLite com cache/hash |
| **Ranking** | 0 (ordem alfabética) | TF-IDF inteligente + PPR |
| **Dependências** | 0 | tree-sitter |
| **Turns economizados** | 0 (Opus explora cegamente) | ~6-8 turns |

**Exemplo real:**
```
grep "class FundJsonDTO" performit-rails/
  ->
performit-rails/app/dto/FundJsonDTO.groovy: (50 linhas completas, 50+ tokens)

vs.

ctx --title "fundo alto shares"
  ->
app/dto/FundJsonDTO.groovy:
  class FundJsonDTO
    String highShares
    def mapHighShares()
  (8 tokens, contexto suficiente)
```

---

## Por que RAG vetorial clássico não resolve aqui

- **Falta de precisão:** busca semântica retorna conceitos similares, não o identificador exato (`FundJsonDTO`)
- **Destruição estrutural:** chunking arbitrário fragmenta assinaturas de função
- **Custo de sincronização:** reindexação contínua a cada commit é inviável como serviço background

---

## Alternativas vectorless (sem banco vetorial)

**BM25 local** — algoritmo TF-IDF que ignora palavras comuns (`class`, `def`) e dá peso máximo a identificadores raros (`FundJsonDTO`). Roda em memória. Retorna top-N arquivos mais relevantes em milissegundos.

**Tree-sitter (AST)** — parser em C com bindings Rust/Python/Node que extrai apenas assinaturas sem ler o corpo dos métodos. Exemplo de output:

```
app/dto/FundJsonDTO.groovy:
  class FundJsonDTO
    String highShares
    def mapHighShares()
    def getMarket()
```

**SQLite-VSS / ChromaDB local** — busca semântica com modelo de embedding leve (`all-MiniLM-L6-v2`, roda em CPU). Útil quando o ticket usa linguagem natural sem identificadores de código (ex: "fundo de feriado" -> `holidayMarket`). Overkill no momento — context_engine (BM25 + PPR) já resolve 95% dos cases.

---

## Pesquisa Aplicada: O que está acontecendo agora (2024-2026)

### 1. Ranking Algorithms Modernos

**Hybrid Search + LLM Reranking (SOTA 2025)**

A tendência atual não é usar UM algoritmo, mas combinar vários em pipeline:

```
Sparse Retrieval (BM25)  -+
                          +- Reciprocal Rank Fusion (RRF) -+
Dense Embeddings        -+                                 +- LLM Reranking
                                                            |
                                                           (ListWise)
                                                            v
                                                        Final Ranking
```

**Resultados reais (NDCG benchmark):**
- BM25 puro: 0.585
- Dense embeddings puro: 0.611
- **Hybrid (RRF): 0.628** — 7% melhor que ambos sozinhos

**Novo em 2025:** LLM-powered listwise ranking
> "Injetar BM25 scores no prompt do LLM com zero finetuning resulta em ganhos consistentes em Gemini, GPT-4, Deepseek" — [InsertRank paper](https://arxiv.org/html/2506.14086v1)

**Implicação:** BM25 + PPR é excelente, mas poderia adicionar:
- **Fase 4.5:** Reranking com LLM leve (Haiku) dos top-10 BM25+PPR
  - Input: BM25 scores + assinaturas
  - Output: score melhorado
  - Custo: +1-2 segundos, +0.5% token

---

### 2. Como Cursor e VS Code Fazem Busca (2024)

#### Cursor Fast Search (Semantic + Server-side)

```
User types "add fund to portfolio"
         v
Query embedding (client-side)
         v
Send embedding to Turbopuffer (vector DB)
         v
Vector similarity search (nearest neighbor)
         v
Return top-20 chunks + snippets
```

**Características:**
- Splits files into chunks (funções, classes)
- Computa embeddings no servidor (Turbopuffer)
- Merkle tree fingerprint para change detection
- Context window: até 272k tokens
- **Accuracy: 12.5% melhor que BM25 puro**

**Tradeoff:** Requer enviar código para servidor (não é open-source)

#### VS Code Symbol Search (LSP-based)

```
User presses Cmd+Shift+O
         v
Language Server Parse (árvore de símbolos)
         v
Filter por current file + dependencies
         v
Return classes, functions, variables
```

**Características:**
- Entende linguagem (tipos, imports)
- Apenas open files + immediate deps
- Context: 64k-128k tokens
- Rápido (local, não HTTP)
- **Limitação:** não entende semântica (não sabe que `getPrice()` é relacionado a `calculateTotal()`)

**Nosso case:** Tree-sitter é híbrido ideal
- Entende estrutura (como LSP)
- Batch processing (melhor que LSP)
- Open-source + zero network
- Mais lento que Cursor (mas aceitável <500ms)

---

### 3. Centrality Measures para Grafos de Dependências

O context-engine usa **Personalized PageRank**, mas há alternativas:

| Measure | Definição | Vantagem | Desvantagem | Para Code |
|---|---|---|---|---|
| **PageRank** | Probabilidade aleatória de "cair" nó | Considera importância + popularidade | Penaliza nós com muitos outlinks | Excelente |
| **Harmonic Centrality** | Inverso da média de distâncias | 10x mais rápido que PageRank | Baseado em distância (não importância) | Bom |
| **Betweenness** | Vezes que nó é "ponte" entre outros | Encontra hubs críticos (pontos de falha) | O(N^3) complexidade | Regular |
| **Eigenvector** | Importância de vizinhos | Similar a PageRank | Não converge em grafos com ciclos | Bom |

**PPR vence em:**
- Detecta "hubs" importantes (arquivos chamados por muitos)
- Personalização por seeds (dirs do codebase_map)
- Trade-off excelente (200 LOC vs 10x valor)

**Quando trocar:**
- Se ranking está "plano" (todos scores similares) -> tente Harmonic (10x mais rápido)
- Se precisar encontrar "código crítico" -> tente Betweenness

---

### 4. Tree-sitter Alternatives e Code Compression (2024-2025)

#### Tree-sitter Status Quo

**Por que Tree-sitter venceu:**
- Suporta 100+ linguagens
- WASM-enabled (funciona em browsers)
- Incremental (re-parse apenas deltas)
- Mantém whitespace + tokens (sem perda)

**Alternativas pesquisadas:**
- Rust `tree-sitter` crate — mesmo projeto, bindings nativos
- Python `ast` nativo — só Python, sem suporte a Ruby/Groovy
- **Não há alternativa melhor em 2025**

#### Code Compression (Emerging)

Não é Tree-sitter problem, é LLM context problem:

**Pesquisa 2024:** Comprimir código sem perder semântica
- Remove comentários, whitespace, literais
- Mantém AST (estrutura)
- Resultado: **50-70% compressão** sem perder significado

**Nosso tradeoff:**
```
Assinatura de classe (nosso approach):
  class FundJsonDTO
    String highShares
    def mapHighShares()
  = 8 tokens

Full class + comments:
  class FundJsonDTO {
    // Representa um fundo com shares altos
    private String highShares;
    ...
  }
  = 50 tokens

Compressão: 84% — já fazemos isso via Tree-sitter!
```

**Conclusão:** Extrair apenas assinaturas JÁ é code compression. Não há ganho em ir mais além.

---

## Referências de Mercado (2025-2026)

| Ferramenta | Estratégia | Contribuição |
|---|---|---|
| **Aider** | Tree-sitter + PageRank sobre AST | Extrai apenas assinaturas (classe, método, campo) — comprime 50 tokens de função para 8 tokens de interface |
| **Cline** | "Descoberta, não recuperação" | Contexto restrito à tarefa; evita RAG genérico |
| **Mentat** | AST-aware chunking + grafos de conhecimento | Fragmenta código respeitando escopos de classe |
| **Claude Code** | Sub-agentes + compactação JIT | Padrão para delegação de exploração densa a sub-agentes descartáveis |

---

## Fontes & Leitura Complementar

### Ranking Algorithms & RAG Patterns

- [Building a Modern Search Ranking Stack: From Embeddings to LLM-Powered Relevance (2026)](https://slavadubrov.github.io/blog/2026/02/08/building-a-modern-search-ranking-stack-from-embeddings-to-llm-powered-relevance/)
- [InsertRank: LLMs can reason over BM25 scores to Improve Listwise Reranking (ARXIV 2506.14086)](https://arxiv.org/html/2506.14086v1) — Directly applicable: inject BM25 scores no prompt
- [Advanced RAG: From Naive Retrieval to Hybrid Search and Re-ranking (DEV Community)](https://dev.to/kuldeep_paul/advanced-rag-from-naive-retrieval-to-hybrid-search-and-re-ranking-4km3)
- [Production RAG: Hybrid Search + Re-Ranking (ColBERT, SPLADE, e5/BGE) (Medium)](https://machine-mind-ml.medium.com/production-rag-that-works-hybrid-search-re-ranking-colbert-splade-e5-bge-624e9703fa2b)

### IDE Search Internals

- [How Cursor Indexes Codebases Fast (Engineer's Codex)](https://read.engineerscodex.com/p/how-cursor-indexes-codebases-fast) — semantic embeddings + Turbopuffer
- [Fast Regex Search: Indexing Text for Agent Tools (Cursor Blog)](https://cursor.com/blog/fast-regex-search)
- [Searching Codebase at the Speed of Thought in Cursor/Cline (Medium)](https://tomoima525.medium.com/searching-codebase-at-the-speed-of-thought-in-cursor-cline-vscode-60e98c35da1a) — Comparação Cursor vs VS Code
- [Semantic Code Search (Medium)](https://medium.com/@wangxj03/semantic-code-search-010c22e7d267)

### Centrality Measures & Network Analysis

- [Can Harmonic Centrality Be the New PageRank? (Search Engine Journal)](https://www.searchenginejournal.com/harmonic-centrality-pagerank/283985/)
- [Centrality Measures Tutorial (NetworkKit)](https://networkit.github.io/dev-docs/notebooks/Centrality.html) — Python library para calcular centrality em grafos
- [PageRank Centrality & EigenCentrality (Cambridge Intelligence)](https://cambridge-intelligence.com/eigencentrality-pagerank/)
- [Betweenness Centrality and Other Centrality Measures (Memgraph Blog)](https://memgraph.com/blog/betweenness-centrality-and-other-centrality-measures-network-analysis)

### Tree-sitter & Parsing

- [Tree-sitter: Revolutionizing Parsing with an Incremental Parsing Library (Deus in Machina)](https://www.deusinmachina.net/p/tree-sitter-revolutionizing-parsing)
- [Semantic Code Indexing with AST and Tree-sitter for AI Agents (Medium, 3 parts)](https://medium.com/@email2dineshkuppan/semantic-code-indexing-with-ast-and-tree-sitter-for-ai-agents-part-1-of-3-eb5237ba687a)
- [Tree-sitter GitHub (Official Repository & Parser List)](https://github.com/tree-sitter/tree-sitter)
- [Tree-sitter Hacker News Discussion (2021, Still Relevant)](https://news.ycombinator.com/item?id=26225298)
