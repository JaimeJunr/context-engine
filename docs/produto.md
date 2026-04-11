# ctx — Produto

> Este documento explica **por que o ctx existe**, qual problema ele resolve e para onde está indo. Leitura obrigatória antes de contribuir ou tomar decisões de roadmap.

---

## O Problema

Agentes de IA que trabalham em codebases grandes chegam "cegos". Sem contexto, um agente de planejamento precisa descobrir a estrutura do código por conta própria — gastando turns em `ls`, `find`, `grep`, lendo arquivos inteiros para entender o que é relevante.

O resultado: tokens desperdiçados, planos superficiais, implementações que erram o alvo.

A abordagem ingênua de mandar o repositório inteiro para o LLM não escala. Um monorepo com milhares de arquivos simplesmente não cabe no contexto — e mesmo que coubesse, seria ruído, não sinal.

---

## O que é o ctx

`ctx` é uma CLI (e futuramente MCP server) que substitui comandos ingênuos como `ls`, `tree` e `grep` por **contexto inteligente e comprimido** para agentes de IA.

Em vez de despejar dados brutos, o `ctx`:

1. **Descobre** arquivos relevantes para uma query ou ticket
2. **Extrai** apenas as assinaturas (funções, classes, referências) — não o corpo completo
3. **Ranqueia** por relevância via BM25 + Personalized PageRank
4. **Entrega** dentro de um orçamento de tokens configurável

O agente recebe o que precisa, não o que existe.

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

## Onde Estamos Hoje

O `ctx` resolve bem o caso central: **gerar contexto de código para um agente de planejamento**, dado um ticket ou query.

Usado em produção pelo `claude-auto-coder` (`/home/jaime/me/claudejob/claude-auto-coder`) no pipeline:

```
Jira ticket → Triage → Plan (usa ctx para repo_map) → Implementation → Push
```

O `ctx` é invocado em `plan.sh` para gerar o `repo_map` que o Opus usa ao planejar a implementação. Sem ele, o Opus começa às cegas.

---

## Visão de Futuro — Framework dos 3 Horizontes

### H1 — Core: Limpar a Casa

O `ctx` funciona, mas foi construído como solução tática. Para sustentar o que vem a seguir, precisa ser refatorado para ser uma base sólida: módulos mais independentes, interfaces limpas, engenharia de verdade.

**Objetivo:** transformar um "trabalho escolar que funciona" em um produto que possa crescer.

### H2 — Emergente: Substituir o Context Mode

[Context Mode](https://github.com/...) resolve o outro lado do problema de contexto: continuidade de sessão e sandbox de execução. Hoje é um MCP server externo — guarda estado em SQLite, usa BM25 para recuperar contexto relevante após compactação.

Já temos as peças: SQLite, BM25, local-first. A ideia é trazer essa lógica para dentro do `ctx`, de forma mais poderosa e integrada.

**Objetivo:** `ctx` como engine de contexto completo — código *e* sessão.

### H3 — Experimental: Substituir o RTK e incorporar QMD

**RTK** filtra e comprime outputs de comandos antes de chegarem ao LLM (ex: `git status` de 2.000 tokens → 200 tokens). A abordagem é certa, mas o `ctx` pode ir além: não só filtrar, mas filtrar *com inteligência* — usando o que já sabe sobre o repositório para comprimir melhor.

**QMD** combina BM25 + busca semântica vetorial + LLM re-ranking, tudo local via GGUF. Já temos a base de BM25 e SQLite. A busca semântica vetorial é o próximo passo natural para cobrir casos onde BM25 falha (ex: queries em português puro sem termos técnicos).

**Objetivo:** `ctx` como engine unificado que cobre os três lados do problema de contexto: código, sessão e outputs de comandos.

---

## Pontos Críticos Hoje

- A interface é limitada: uma CLI que roda uma vez e sai. Não há estado entre invocações (além do cache de assinaturas).
- Suporte a linguagens é manual: cada linguagem nova exige um extractor Tree-sitter.
- Sem observabilidade: difícil saber quando o ranking acertou ou errou.
- Sem modo servidor: para ser um MCP server, precisa de uma interface de longa duração.
