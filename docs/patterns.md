# ctx — Patterns de Engenharia

> Padrões de design e arquitetura do projeto. Leitura obrigatória antes de implementar qualquer nova capacidade. De cima para baixo: da filosofia até os detalhes.

---

## Filosofia

O `ctx` existe para resolver um problema de escala: quanto maior a codebase, mais caro fica para um agente de IA descobrir o que é relevante. Nossa resposta é **inteligência local-first** — algoritmos que rodam na máquina do usuário, sem dependência de APIs externas, com estado persistido localmente.

O mercado caminha para economia de tokens. Nossa vantagem é fazer isso com mais inteligência: não apenas limpar ruído, mas entender o que é relevante e entregar só isso.

---

## As Duas Invariantes

Toda capacidade que o `ctx` implementar deve respeitar estas duas regras. Sem exceção.

### 1. Token Budget

Toda saída do `ctx` deve respeitar um orçamento de tokens. O agente nunca deve receber mais do que precisa.

- Todo output passa por `budget.rs` antes de ser entregue
- `--max-tokens` é o contrato com o chamador
- Sem budget definido, use o default — nunca retorne ilimitado

### 2. Memória SQLite

Toda informação computada que possa ser reutilizada deve ser persistida em SQLite. Recomputar o que já foi calculado é desperdício.

- Assinaturas extraídas → cache por sha256 do conteúdo
- Futuro: resultados de busca, estado de sessão, embeddings
- Cache mora em `~/.cache/context_engine/`
- Invalidação por conteúdo (sha256), nunca por tempo

---

## Arquitetura Modular

O pipeline atual é `Scanner → Extractor → Cache → Ranker → Output`. Mas os módulos não devem ser acoplados a esse fluxo linear — eles devem ser **quase independentes**, para poder ser compostos de formas diferentes conforme o `ctx` cresce.

### Por quê isso importa

Quando o `ctx` ganhar novos comandos (ex: `ctx grep`, `ctx ls`, `ctx diff`), cada um vai precisar de combinações diferentes dos módulos existentes:

- `ctx grep` usa Scanner + Cache + Output (sem Ranker)
- `ctx ls` usa Scanner + Output (sem Extractor)
- `ctx map` usa todos os módulos (fluxo atual)

Se os módulos estiverem acoplados, cada novo comando vira um fork do pipeline. Se forem independentes, cada comando é uma composição.

### Regra

> Nenhum módulo deve importar de outro módulo do pipeline diretamente. A orquestração fica em `lib.rs`.

---

## Inteligência Acumulada

Quando adicionamos um novo comando, **não perdemos** a inteligência existente — nós a reutilizamos.

Exemplo: se amanhã implementarmos `ctx grep`, ele não é um `grep` simples filtrado. É um `grep` que:
1. Usa o cache SQLite para saber quais arquivos já foram indexados
2. Aplica BM25 para priorizar resultados mais relevantes para a query
3. Respeita o token budget antes de retornar

Cada camada de inteligência que construímos é disponível para todos os comandos futuros.

### Estratégias de filtragem (inspirado no RTK, com mais inteligência)

Todo output do `ctx` deve aplicar ao menos uma dessas estratégias:

| Estratégia | Descrição |
|---|---|
| **Smart Filtering** | Remove ruído: comentários, whitespace, boilerplate |
| **Grouping** | Agrupa itens similares (arquivos por dir, erros por tipo) |
| **Truncation** | Mantém contexto relevante, corta redundância |
| **Deduplication** | Colapsa linhas repetidas com contadores |
| **Signature Extraction** | Substitui corpo de funções por interfaces (~50→8 tokens) |
| **Relevance Ranking** | Ordena por BM25/PPR antes de entregar |

---

## Como Adicionar um Novo Comando

1. **Defina a invariante de saída**: qual é o budget de tokens? Qual formato?
2. **Identifique quais módulos você precisa**: Scanner? Extractor? Cache? Ranker?
3. **Não crie nova lógica se um módulo já resolve**: reutilize.
4. **Persista o que puder em SQLite**: se o resultado pode ser reutilizado, cacheia.
5. **Aplique ao menos uma estratégia de filtragem** da tabela acima.
6. **Orquestre em `lib.rs`**: o comando não faz orquestração, só declara o que quer.

---

## Como Adicionar Suporte a uma Nova Linguagem

1. Crie `src/extractors/<linguagem>.rs`
2. Implemente o trait `Extractor`:
   - `extract_signatures(source: &str) -> Vec<Signature>`
   - `extract_refs(source: &str) -> Vec<String>`
3. Registre a extensão em `src/extractors/mod.rs`
4. Se precisar de grammar customizada (como Groovy): adicione em `grammars/` e registre em `build.rs`
5. Escreva pelo menos um teste de integração em `tests/integration.rs`

**Regra:** o extractor não acessa cache, não faz I/O de arquivo, não conhece o pipeline. Recebe string, retorna signatures. Simples.

---

## Padrões de Código

### Imutabilidade

Criar novos objetos, nunca mutar os existentes. Ver `coding-style.md`.

### Paralelismo

Use `rayon` para operações sobre coleções de arquivos (scan, parse). Não use threads manuais.

### Erros

Propague com `?`. Nunca silenciar erros. Mensagens de erro devem ser acionáveis.

### Tamanho de arquivo

Máximo 800 linhas. Se um extractor está ficando grande, considere extrair helpers privados.

---

## Referências

- `docs/arquitetura.md` — detalhes técnicos do pipeline atual
- `docs/produto.md` — visão de produto e os 3 horizontes
- `docs/pesquisa/` — decisões técnicas e estado da arte
- `.claude/rules/common/coding-style.md` — regras de estilo aplicadas ao projeto
