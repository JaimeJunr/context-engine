# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## O que é este projeto

`ctx` é uma CLI em Rust que gera mapas de repositório para LLMs. Dado um diretório, ele descobre arquivos, extrai assinaturas de código (funções, classes, referências), ranqueia por relevância e gera uma saída compacta dentro de um orçamento de tokens.

## Comandos

```bash
cargo build                    # Build
cargo build --release          # Build otimizado (LTO, opt-level=3)
cargo test                     # Roda testes de integração (tests/integration.rs)
cargo run -- <dir> --help      # CLI local
cargo clippy                   # Lint
```

## Arquitetura

**Pipeline:** `Scanner → Extractor → Cache → Ranker → Output`

```
src/
├── main.rs          # Entry point, configuração clap (CLI args)
├── lib.rs           # Orquestração do pipeline
├── scanner.rs       # Descoberta de arquivos com suporte a .gitignore
├── cache.rs         # Cache SQLite invalidado por SHA256 do conteúdo
├── tokenizer.rs     # Tokenização para BM25
├── output.rs        # Formatação text/JSON
├── extractors/      # Extratores por linguagem via Tree-Sitter
│   ├── mod.rs       # Trait Extractor + dispatch por extensão
│   ├── python.rs
│   ├── ruby.rs
│   ├── typescript.rs
│   └── groovy.rs    # Usa gramática customizada compilada em build.rs
└── ranking/
    ├── mod.rs       # Coordena BM25 + PageRank
    ├── bm25.rs      # TF-IDF / BM25 por query
    └── pagerank.rs  # Personalized PageRank no grafo de dependências
```

**Padrões-chave:**
- Paralelismo via `rayon` no scan e extração
- Seleção de arquivos por orçamento de tokens (`--max-tokens`)
- Gramática Groovy customizada em `grammars/groovy/` compilada via `build.rs`
- Cache SQLite em `~/.cache/ctx/` para evitar re-parsear arquivos não alterados

## Documentação

- `docs/arquitetura.md` — referência técnica detalhada (PT-BR)
- `docs/roadmap.md` — próximas features planejadas
- `docs/pesquisa/` — notas de pesquisa sobre code-search e decisões de implementação
