# ctx — Context Engine

> **Para LLMs, agentes e desenvolvedores:** gerador de mapas de repositório inteligente que entrega apenas o contexto relevante.

## O Problema

Ao trabalhar em codebases grandes, LLMs e agentes chegam "cegos": precisam descobrir por conta própria quais arquivos são relevantes. `ctx` resolve isso extraindo automaticamente assinaturas de código (funções, classes, tipos) e ranqueando por relevância — economizando tokens e turns.

## O que você pode fazer

**`ctx` — Gera um mapa curado da estrutura de código** (repo_map):
- Descobre todos os arquivos + extrai assinaturas (não código inteiro)
- Ranqueia por relevância com BM25 + PageRank
- Respeita `.gitignore` + filtra testes
- Saída em texto ou JSON, respeitando orçamento de tokens

```bash
# "Preciso de contexto sobre autenticação"
ctx map /path/to/repo --title "Adicionar suporte a 2FA" --max-tokens 4000

# Saída: lista curada de arquivos + assinaturas, pronto para colar em prompt de LLM
```

**`ctx catalog` — Busca semântica em documentação** (RAG local):
- Registra, indexa e busca em documentação, especificações e guias
- Busca por intenção (não apenas palavras-chave)
- Totalmente local (roda offline, sem APIs externas)

```bash
# Registrar acervo documental
ctx add meu-projeto --source ./docs --include "**/*.md"

# Indexar documentos + gerar embeddings
ctx index meu-projeto --with-embed

# Buscar
ctx search meu-projeto "Como configurar autenticação OAuth?"
```

**`ctx exec` — Compressão inteligente de output**:
- Intercepta comandos shell verbosos (testes, build, logs)
- Filtra, comprime e entrega apenas essência do output
- Economiza 60-90% de tokens

```bash
# Executar teste comprimindo output
ctx exec cargo test

# Ver relatório de economia acumulada
ctx exec report
```

## Início Rápido

### Pré-requisitos

- **Rust 1.70+** ([instalar](https://rustup.rs/))
- **Para `ctx map`:** nenhuma dependência adicional
- **Para `ctx catalog` com embeddings:** Ollama rodando ([instalar](https://ollama.ai/))
  ```bash
  # Terminal 1:
  ollama serve
  
  # Terminal 2:
  ollama pull nomic-embed-text  # embedder (padrão)
  ollama pull llama3.2          # reranker (padrão)
  ```
  Sem Ollama, você ainda pode usar `ctx search` com busca léxica (sem embeddings semânticos).

### Build & Install

```bash
# Build otimizado
cargo build --release

# Binário ficará em: target/release/ctx
# (Opcionalmente, copie para ~/.local/bin ou /usr/local/bin)
cp target/release/ctx ~/.local/bin/
```

### Exemplo 1: `ctx map` — Repo Map

```bash
# Gerar mapa de contexto para um repositório
ctx map \
  --title "CAP-123: Adicionar validação de CPF" \
  --dirs "src/models,src/validators" \
  --max-tokens 4000

# Resultado: Lista de arquivos + assinaturas, pronto para colar em prompt
```

**Opções úteis:**
- `--max-tokens N` — limitar tamanho da saída (default: 4096)
- `--format json` — saída em JSON (default: texto)
- `--seeds dir1,dir2` — ativar Personalized PageRank (prioriza arquivos seed)
- `--top N` — retornar top N arquivos (se omitido, usa token budget)

### Exemplo 2: `ctx catalog` — Busca Semântica

```bash
# Registrar acervo documental
ctx add meu-projeto \
  --source ./docs \
  --include "**/*.md" \
  --exclude "**/node_modules/**"

# Indexar documentos + gerar embeddings (requer Ollama)
ctx index meu-projeto --with-embed

# Buscar por intenção
ctx search meu-projeto "como funciona o pipeline de dados?"

# Ver status do acervo
ctx status meu-projeto

# Listar todos os acervos
ctx list

# Otimizar armazenamento
ctx compact meu-projeto
```

**Subcomandos de Catalog:**
- `add` — registrar novo acervo documental
- `index` — indexar documentos (detecta novos/modificados)
- `embed` — gerar embeddings para chunks pendentes
- `search` — busca semântica no acervo
- `list` — listar acervos registrados
- `status` — exibir stats do acervo
- `compact` — otimizar storage removendo dados obsoletos
- `init` — configurar endpoint LLM interativamente
- `config` — gerenciar configuração global

### Exemplo 3: `ctx exec` — Compressão de Output

```bash
# Executar testes com compressão automática
ctx exec run cargo test

# Ver logs do build comprimidos
ctx exec run cargo build

# Relatório de economia de tokens
ctx exec report
```

## Linguagens Suportadas

| Linguagem | Suporte |
|-----------|---------|
| TypeScript / TSX | ✅ Completo |
| Python | ✅ Completo |
| Ruby | ✅ Completo |
| Groovy | ✅ Completo |
| Outras | 🚧 Roadmap |

## Documentação Completa

Para desenvolvedores e contribuidores:

- **[docs/INDEX.md](docs/INDEX.md)** — mapa de documentação
- **[docs/arquitetura.md](docs/arquitetura.md)** — como funciona internamente
- **[docs/patterns.md](docs/patterns.md)** — padrões de engenharia
- **[docs/produto.md](docs/produto.md)** — visão e roadmap
- **[docs/especificacao-rag.md](docs/especificacao-rag.md)** — especificação do módulo catalog

## Desenvolvimento

```bash
cargo test                             # Rodar testes
cargo clippy --all-targets --all-features -- -D warnings   # Lint
cargo fmt -- --check                   # Verificar formatação
cargo build --release                  # Build otimizado
cargo run -- map --help                # Ver opções do subcomando map
cargo run -- catalog --help            # Ver opções do subcomando catalog
cargo run -- exec --help               # Ver opções do subcomando exec
```

**Git Hooks (via Lefthook):**
- `pre-commit`: `cargo fmt --check` + `cargo clippy` — formata e detecta erros
- `pre-push`: `cargo test --locked --all-features` — executa suite de testes

## Performance

- **Parsing:** paralelizado com `rayon` (multi-thread)
- **Cache:** SQLite persistente (`~/.cache/context_engine/`) — reutiliza entre execuções
- **Ranking:** BM25 + PageRank híbrido (personalizável com seeds)
- **Output:** comprimido para caber em token budget
- **Command Compression:** `ctx exec` filtra logs verbosos, economizando 60-90% de tokens em saída de comandos
- **Embeddings:** processados em lotes, com cache em SQLite

## Licença

MIT (veja LICENSE)
