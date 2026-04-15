# Referência de API (Rust Crate)

Documentação da biblioteca Rust `context_engine` para usar em código Rust.

Consulte:
- `CLAUDE.md` — Estrutura completa de código
- `src/lib.rs` — Orquestração principal
- Testes em `tests/` — Exemplos de uso

## Módulos Públicos

- `run()` — Função principal do pipeline `map`
- `catalog::*` — API pública do catalog (add_collection, index, search, etc)
- `cache::*` — Operações de cache
- `tokenizer::*` — Tokenização para BM25
