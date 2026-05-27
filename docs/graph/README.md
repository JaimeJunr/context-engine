# Subcomando `ctx graph` — Grafo de Chamadas Semântico

O subcomando `ctx graph` oferece ferramentas para análise estática e navegação estrutural do fluxo do código. Ele constrói um grafo de símbolos, chamadas e imports a partir das árvores de sintaxe das linguagens suportadas e armazena os dados em um banco de dados SQLite local.

Diferente do `grep` tradicional, o `ctx graph` entende a semântica do código, permitindo responder a perguntas como "quem chama este método?", "o que esta função invoca?", ou "quais rotas HTTP batem neste handler?".

---

## 🚀 Como funciona

O pipeline do `ctx graph` é dividido em cinco etapas fundamentais:

```
[Código Fonte] ──> Extractor (Tree-Sitter) ──> Framework Router ──> SQLite Store ──> Query & Ranking (PageRank/BM25)
```

1. **Scanner & Extractor**: O scanner descobre os arquivos no projeto (respeitando as regras de `.gitignore`). Cada arquivo é analisado usando **Tree-Sitter** para extrair declarações de símbolos (classes, métodos, funções, etc.), chamadas (`call sites`), e `imports` de módulos.
2. **Framework Routing**: Detecta definições de rotas HTTP para frameworks conhecidos e injeta símbolos sintéticos do tipo `route::METODO /path` vinculados às suas respectivas funções handler.
3. **Persistência**: As informações extraídas são limpas de forma idempotente e salvas em um banco de dados SQLite unificado localizado em `~/.cache/context_engine/graph.db`.
4. **Ranking & BM25**: Para as buscas, as queries textuais são resolvidas usando um algoritmo de relevância híbrido baseado em **BM25** (no nome dos símbolos) combinado com métricas de conectividade do grafo (PageRank e contagem de call sites).
5. **Token Budget**: Os resultados passam por um algoritmo de busca binária para limitar e ajustar perfeitamente o tamanho do output em número de tokens de acordo com o contexto disponível do LLM.

---

## 🛠️ Subcomandos Disponíveis

### 1. `ctx graph index`
Indexa (ou re-indexa de forma incremental) os diretórios especificados.

```bash
ctx graph index --dirs src
```

- **Extensões Suportadas (8 linguagens):** `.rs`, `.go`, `.java`, `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs`, `.py`, `.rb`, `.rake`, `.groovy`, `.gradle`.

### 2. `ctx graph callers`
Encontra todos os chamadores (`callers`) diretos de um símbolo específico.

```bash
ctx graph callers handle_request
```

### 3. `ctx graph callees`
Encontra todas as chamadas (`callees`) feitas por um símbolo específico.

```bash
ctx graph callees execute_pipeline
```

### 4. `ctx graph trace`
Mostra a cadeia e fluxo completo de chamadas ligadas a um símbolo até uma determinada profundidade.

```bash
ctx graph trace handle_request --depth 3
```

### 5. `ctx graph impact`
Realiza uma análise de impacto transitiva. Retorna os chamadores indiretos e tudo o que pode ser impactado pela alteração do símbolo fornecido.

```bash
ctx graph impact migrate_db
```

### 6. `ctx graph node`
Detalha as propriedades e o local exato onde um símbolo está definido no grafo.

```bash
ctx graph node "route::GET /users/:id"
```

---

## 🌐 Framework-Aware Routing

Uma das funcionalidades mais poderosas do `ctx graph` é o suporte nativo a rotas de frameworks. Ele detecta os caminhos declarados de URLs HTTP e injeta-os no grafo conectando o endpoint sintético à action/função controladora real.

### Frameworks Cobertos:

*   **Ruby on Rails (`config/routes.rb`):**
    *   Mapeia chamadas do DSL como `resources :users`, `resource`, `get '/login', to: 'session#new'`, além de namespaces.
    *   Conecta automaticamente `route::GET /users/:id` à action `UsersController#show`.
*   **Grails (`UrlMappings.groovy`):**
    *   Mapeia blocos `"/path"(controller: 'x', action: 'y')` e recursos dinâmicos.
    *   Normaliza parâmetros de variáveis (ex: `$id` → `:id`).
*   **NestJS (Classes `*.ts`):**
    *   Analisa os decorators `@Controller('prefix')` e os decorators de verbo `@Get(':id')`, `@Post()`, `@Patch()`, etc.
    *   Injeta a rota mapeando o handler qualificado como `ClassName::method`.

---

## 💎 Diferenciais Exclusivos vs CodeGraph e RTK

1.  **Resultados Ranqueados:** O CodeGraph retorna listas cruas de callers/callees. O `ctx graph` ranqueia os resultados usando a query textual com BM25 + PageRank para garantir que os símbolos mais relevantes para a tarefa do agente apareçam primeiro.
2.  **Deduplicação Inteligente de Call Sites:** Se um método é chamado 10 vezes pelo mesmo chamador, o `ctx graph` agrupa os call sites em uma única entrada reduzida, economizando tokens preciosos.
3.  **Token Budget Integration:** As buscas respeitam a flag `--max-tokens` do seu agente através de busca binária robusta no output final do grafo.
4.  **100% Local e Rápido:** Construído em Rust e usando o poder do SQLite e Tree-Sitter, indexa projetos gigantescos em segundos com threads paralelas via `rayon`.
