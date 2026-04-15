#!/usr/bin/env bash
# ctx-rewrite-hook: Envolve comandos em ctx exec para comprimir output
# Version: 1.0 (Token savings: 60-90% em comandos com saída verbosa)
#
# COMPORTAMENTO: detecta comandos dos domínios suportados e envolve em
# ctx exec <cmd> para economizar tokens via filtragem inteligente.
# Sem permissão do usuário (permissionDecision: allow).
#
# DOMÍNIOS SUPORTADOS:
# - Navegação: ls, find, tree
# - Git: status, log, diff, show
# - Build/Lint: cargo, npm, eslint
# - Testes: cargo test, pytest, jest
# - Container: docker, kubectl
# - Cloud: aws
# - Dados: curl, jq, sqlite3

[ -x "$(command -v jq)" ] || exit 0
[ -x "$(command -v ctx)" ] || exit 0

INPUT=$(cat)
CMD=$(echo "$INPUT" | jq -r '.tool_input.command // empty')
[ -z "$CMD" ] && exit 0

# ============================================================================
# FUNÇÃO: Detectar se comando deve ser envolvido em ctx exec
# ============================================================================
should_wrap_in_ctx_exec() {
  local cmd="$1"

  # Padrão: comando começa com um destes keywords
  # Navegação de arquivos
  if [[ "$cmd" =~ ^(ls|find|tree)[[:space:]] ]]; then
    return 0
  fi

  # Git
  if [[ "$cmd" =~ ^git[[:space:]]+(status|log|diff|show|branch|tag|stash) ]]; then
    return 0
  fi

  # Build/compile/lint (Rust)
  if [[ "$cmd" =~ ^cargo[[:space:]]+(build|test|check|clippy|fmt|run|install) ]]; then
    return 0
  fi

  # Build/compile/lint (Node)
  if [[ "$cmd" =~ ^(npm|yarn)[[:space:]]+(install|test|run|build|lint|start) ]]; then
    return 0
  fi

  # Build/compile/lint (Python)
  if [[ "$cmd" =~ ^(pytest|python)[[:space:]] ]]; then
    return 0
  fi

  # Build/compile/lint (JS)
  if [[ "$cmd" =~ ^jest[[:space:]] ]]; then
    return 0
  fi

  # Container
  if [[ "$cmd" =~ ^(docker|kubectl)[[:space:]] ]]; then
    return 0
  fi

  # Cloud
  if [[ "$cmd" =~ ^aws[[:space:]] ]]; then
    return 0
  fi

  # Dados/rede
  if [[ "$cmd" =~ ^(curl|jq|sqlite3)[[:space:]] ]]; then
    return 0
  fi

  return 1
}

# ============================================================================
# LÓGICA PRINCIPAL: Detectar e envolver
# ============================================================================

if should_wrap_in_ctx_exec "$CMD"; then
  WRAPPED="ctx exec $CMD"

  ORIGINAL_INPUT=$(echo "$INPUT" | jq -c '.tool_input')
  UPDATED_INPUT=$(echo "$ORIGINAL_INPUT" | jq --arg cmd "$WRAPPED" '.command = $cmd')

  # Envolver silenciosamente (allow automático)
  jq -n --argjson updated "$UPDATED_INPUT" '{
    "hookSpecificOutput": {
      "hookEventName": "PreToolUse",
      "permissionDecision": "allow",
      "permissionDecisionReason": "ctx-rewrite: compress command output with ctx exec",
      "updatedInput": $updated
    }
  }'
else
  # Comando não reconhecido, deixar passar
  exit 0
fi
