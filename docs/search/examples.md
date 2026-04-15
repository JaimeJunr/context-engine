# Exemplos: `ctx search`

## Setup Inicial

```bash
# Registrar documentação do projeto
ctx add meu-projeto \
  --source ./docs ./guides \
  --include "**/*.md" \
  --exclude "**/draft/**"

# Indexar
ctx index meu-projeto --with-embed

# Status
ctx status meu-projeto
```

## Caso 1: Busca Simples

```bash
ctx search meu-projeto "como fazer autenticação OAuth?"
```

Resultado: Top 10 chunks mais relevantes com scores.

## Caso 2: Busca Exata

```bash
ctx search meu-projeto "exact:JWT configuration"
```

Resultado: Apenas resultados que contêm exatamente "JWT configuration".

## Caso 3: Busca por Conceito

```bash
ctx search meu-projeto "conceptual:deploy"
```

Resultado: Documentação sobre deploy, deployment, release, publish, etc.

## Caso 4: Top K Customizado

```bash
ctx search meu-projeto "como debugar" --top-k 20
```

Resultado: Top 20 resultados em vez dos default 10.

## Caso 5: Ver Conteúdo Completo

```bash
ctx search meu-projeto "API keys" --full
```

Resultado: Mostra fragmento inteiro de cada resultado, não apenas preview.

## Caso 6: Múltiplas Coleções

```bash
# Criar 2 coleções
ctx add docs-api --source ./api-docs
ctx add guides --source ./user-guides

# Buscar em docs-api
ctx search docs-api "REST endpoints"

# Buscar em guides
ctx search guides "getting started"
```
