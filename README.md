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

**`ctx search` — Busca semântica em documentação** (RAG local):
- Cataloga documentação, especificações, guias
- Busca por intenção (não apenas palavras-chave)
- Totalmente local (roda offline, sem APIs externas)

```bash
# Registrar documentação
ctx add meus-docs --source ./docs --include "**/*.md"

# Buscar
ctx search meus-docs "Como configurar autenticação OAuth?"
```

## Início Rápido

### Pré-requisitos

- **Rust 1.70+** ([instalar](https://rustup.rs/))
- Para `ctx search`: **Ollama rodando** ([instalar](https://ollama.ai/))
  ```bash
  ollama serve
  # Em outro terminal:
  ollama pull nomic-embed-text
  ollama pull llama3.2
  ```

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

### Exemplo 2: `ctx search` — Documentação

```bash
# Criar catálogo de documentação
ctx add meu-projeto \
  --source ./docs \
  --include "**/*.md"

# Indexar e gerar embeddings (requer Ollama)
ctx index meu-projeto --with-embed

# Buscar
ctx search meu-projeto "como funciona o pipeline de dados?"

# Ver status
ctx status meu-projeto
```

**Subcomandos:**
- `add` — registrar novo catálogo documental
- `index` — indexar + gerar embeddings (requer Ollama)
- `search` — busca por relevância semântica
- `list` — listar catálogos registrados
- `status` — stats do catálogo
- `compact` — otimizar storage

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
cargo test                    # Rodar testes
cargo clippy                  # Lint
cargo build --release         # Build otimizado
cargo run -- --help           # Ver subcomandos disponíveis
```

## Performance

- **Parsing:** paralelizado com `rayon` (multi-thread)
- **Cache:** SQLite persistente (~`~/.cache/context_engine/`) — reutiliza entre execuções
- **Ranking:** BM25 + PageRank híbrido (personalizável com seeds)
- **Output:** comprimido para caber em token budget

## Licença

MIT (veja LICENSE)
