# ctx — Context Engine

> **Descrição**: CLI em Rust que gera mapas de repositório para LLMs. Dado um diretório, descobre arquivos, extrai assinaturas de código, ranqueia por relevância e gera saída compacta dentro de um orçamento de tokens.

## Objetivo

Reduzir o contexto enviado para LLMs de forma inteligente: em vez de enviar arquivos completos, extrai apenas assinaturas (funções, classes, referências) e ranqueia os arquivos mais relevantes para uma query ou ticket.

## Tecnologias

- **Rust** — performance e segurança de memória
- **Tree-sitter** — parsing de código por linguagem
- **SQLite** — cache persistente de assinaturas
- **rayon** — paralelismo no scan e extração
- **BM25 + PageRank** — ranqueamento híbrido

## Estrutura do Projeto

```text
context-engine/
├── src/
│   ├── main.rs          # Entry point, CLI args (clap)
│   ├── lib.rs           # Orquestração do pipeline
│   ├── scanner.rs       # Descoberta de arquivos (.gitignore)
│   ├── cache.rs         # Cache SQLite por SHA256
│   ├── tokenizer.rs     # Tokenização para BM25
│   ├── output.rs        # Formatação text/JSON
│   ├── extractors/      # Extratores por linguagem (Tree-sitter)
│   └── ranking/         # BM25 + Personalized PageRank
├── grammars/            # Gramáticas customizadas (Groovy)
├── docs/                # Documentação técnica
├── tests/               # Testes de integração
└── build.rs             # Compilação das gramáticas
```

## Instalação e Uso

### Pré-requisitos

- Rust toolchain (`rustup`)

### Build

```bash
cargo build             # build de desenvolvimento
cargo build --release   # build otimizado (LTO, opt-level=3)
```

### Uso

```bash
# Gerar mapa de contexto para um diretório
ctx <dir> [query]

# Com orçamento de tokens
ctx <dir> --max-tokens 4000 "autenticação JWT"

# Com seeds para PageRank personalizado
ctx <dir> --seeds src/auth.rs "login flow"

# Saída em JSON
ctx <dir> --format json

# Ver ajuda
cargo run -- --help
```

## Pipeline

```
Scanner → Extractor (Tree-sitter + Cache) → Ranker (BM25 + PageRank) → Output
```

1. **Scanner**: varre diretórios respeitando `.gitignore`
2. **Extractor**: extrai assinaturas via Tree-sitter, cacheia em SQLite
3. **Ranker**: BM25 ranqueia por query; PageRank personalizado com seeds
4. **Output**: formata dentro do orçamento de tokens (`--max-tokens`)

## Documentação

Consulte a pasta `docs/` para documentação completa:

- [`docs/INDEX.md`](docs/INDEX.md) — índice central
- [`docs/arquitetura.md`](docs/arquitetura.md) — referência técnica detalhada
- [`docs/roadmap.md`](docs/roadmap.md) — próximas features planejadas

## Desenvolvimento

```bash
cargo test      # roda testes de integração
cargo clippy    # lint
```
