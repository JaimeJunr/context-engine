---
name: review
description: Review de código focado no git diff. Analisa mudanças por correção, segurança, performance e testes. Classifica issues como blocker/warning/suggestion.
---

# Review

Review focado exclusivamente nas mudanças do git diff — não no arquivo inteiro.

## Fluxo

### 1. Obter diff

```bash
git diff --stat origin/main...HEAD
git diff origin/main...HEAD
```

Se o usuário informar outra branch base ou um range específico (BASE_SHA..HEAD_SHA), usar esse.

### 2. Analisar mudanças

Focar nas linhas alteradas. Código existente serve apenas como contexto para entender impacto.

**Critérios (por ordem de prioridade):**

1. **Correção** — lógica de negócio, edge cases, tratamento de erros
2. **Segurança** — OWASP top 10, dados sensíveis expostos, injeção
3. **Performance** — N+1 queries, loops desnecessários, operações bloqueantes
4. **Testes** — cobertura das mudanças, testes validam comportamento real (não mocks)
5. **Design** — responsabilidade única, acoplamento, padrões do projeto
6. **Escopo** — mudanças fora do escopo do ticket devem ser sinalizadas

### 3. Classificar issues

- **Blocker** — impede aprovação: bugs, falhas de segurança, testes quebrando, escopo expandido sem justificativa
- **Warning** — deve ser corrigido: complexidade alta, testes insuficientes, nomes confusos
- **Suggestion** — melhoria opcional: refatorações menores, clareza

### 4. Rodar validações

Executar linter/testes do projeto apenas nos arquivos alterados, se disponíveis.

## Output

```
## Decisão: APROVAR | REVISAR | REJEITAR

## Mudanças: X arquivos, +N/-N linhas

## Blockers (N)
- [arquivo:linha] Descrição — como corrigir

## Warnings (N)
- [arquivo:linha] Descrição — como corrigir

## Suggestions (N)
- [arquivo:linha] Descrição

## Pontos positivos
- O que está bem feito
```

Adaptar o formato conforme a complexidade. PRs pequenas não precisam de todas as seções.
