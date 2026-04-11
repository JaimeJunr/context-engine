---
name: ship
description: Commit, push e abre PR. Opcionalmente cria branch e/ou faz review antes de enviar. Use para commit, push, abrir PR ou shipar código.
disable-model-invocation: true
---

# Ship

Fluxo unificado para enviar código: commit → push → PR.

## Etapas opcionais (detectar pelo contexto ou pedido do usuário)

### Branch (quando não está em branch feature)

- Se estiver em `main`/`master`/`develop`, criar branch descritiva antes de commitar
- `git checkout -b <tipo>/<descricao-curta>` a partir do branch base atualizado
- Manter escopo focado em um conjunto de mudanças

### Review (quando o usuário pedir review antes de enviar)

1. `git diff origin/main...HEAD` para identificar riscos comportamentais
2. Rodar ou atualizar testes para comportamento alterado
3. Corrigir problemas críticos antes de prosseguir
4. Priorizar correção, segurança e regressões — ignorar estilo

## Fluxo principal

### 1. Preparação

- `git status` para identificar arquivos modificados e novos
- Excluir arquivos sensíveis ou temporários do staging
- Se houver mudanças que devem ir em commits separados, perguntar ao usuário

### 2. Commit

- Conventional Commits: `<tipo>(<escopo>): <descrição>`
- Mensagem descritiva e clara
- Manter commits focados — sem mudanças não relacionadas

### 3. Push

- `git push -u origin <branch>`
- Se pre-commit hooks falharem, corrigir o problema (nunca usar `--no-verify`)

### 4. PR

- Criar PR via `gh pr create` com summary e test notes, ou retornar link para PR existente
- Formato do link conforme remote (GitHub ou Bitbucket)

## Output

- Branch utilizada
- Resumo do commit
- PR URL

## Argumentos

$ARGUMENTS pode ser:
- `review` - Faz review antes de enviar
- `branch` - Força criação de branch nova
- `no-pr` - Apenas commit + push, sem abrir PR
