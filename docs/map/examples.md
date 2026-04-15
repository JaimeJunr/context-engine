# Exemplos: `ctx map`

## Caso 1: Entender Fluxo de Autenticação

```bash
ctx map \
  --title "Entender fluxo de login" \
  --dirs src/controllers,src/services,src/models \
  --seeds src/auth \
  --max-tokens 4000
```

Resultado: Top arquivos relacionados a autenticação, priorizando src/auth

## Caso 2: Refatoração de Um Módulo

```bash
ctx map \
  --title "Refatorar sistema de cache" \
  --dirs src/cache \
  --max-tokens 3000
```

Resultado: Estrutura completa do módulo cache

## Caso 3: Adicionar Feature Rápido

```bash
ctx map \
  --title "CAP-123: Adicionar export CSV" \
  --dirs src \
  --seeds src/models,src/export \
  --max-tokens 2000 \
  --format json
```

Resultado: Retorna JSON (melhor para processamento programático)

## Caso 4: Top 10 Arquivos

```bash
ctx map \
  --title "Visão geral do projeto" \
  --dirs . \
  --top 10
```

Resultado: Exatamente 10 arquivos mais relevantes (ignora `--max-tokens`)
