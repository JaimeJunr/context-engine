# Quick Start

## Instalação

```bash
cargo build --release
cp target/release/ctx ~/.local/bin/  # ou /usr/local/bin
```

## Exemplos Rápidos

### 1. Gerar Repo Map

```bash
ctx map --title "Entender autenticação" --dirs src/auth,src/models --max-tokens 3000
```

### 2. Busca Semântica em Docs

```bash
# Registrar
ctx add meu-projeto --source ./docs --include "**/*.md"

# Indexar
ctx index meu-projeto --with-embed

# Buscar
ctx search meu-projeto "Como usar a API de pagamento?"
```

### 3. Compressão de Output

```bash
ctx exec cargo test
```
