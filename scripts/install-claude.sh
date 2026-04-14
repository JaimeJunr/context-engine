#!/usr/bin/env bash
# Configura Claude Code para o projeto context-engine
# - Instala regras do projeto em ~/.claude/rules/
# - Instala RTK hook para economizar tokens
# - Valida dependências (rtk, jq)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

CLAUDE_DIR="$HOME/.claude"
CLAUDE_MD="$CLAUDE_DIR/CLAUDE.md"
RULES_DIR="$CLAUDE_DIR/rules"
HOOKS_DIR="$CLAUDE_DIR/hooks"

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}✓${NC} $1"; }
log_warn() { echo -e "${YELLOW}⚠${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1"; }

# === Validação de Dependências ===
echo "=== Verificando dependências ==="
if ! command -v rtk &>/dev/null; then
  log_warn "rtk não encontrado. Token savings desabilitado."
  log_info "Instale: cargo install rtk"
fi

if ! command -v jq &>/dev/null; then
  log_warn "jq não encontrado. RTK hook não funcionará."
  log_info "Instale: apt-get install jq (ou equivalente)"
fi

# === Instalar Regras do Projeto ===
echo -e "\n=== Instalando Regras do Projeto ==="
mkdir -p "$RULES_DIR"

# Copia regras do projeto
if [ -d "$REPO_ROOT/.claude/rules" ]; then
  find "$REPO_ROOT/.claude/rules" -type f -name "*.md" | while read -r file; do
    filename=$(basename "$file")
    dest="$RULES_DIR/$filename"
    cp "$file" "$dest"
    log_info "Regra instalada: $filename"
  done
else
  log_warn "Nenhuma regra do projeto encontrada em $REPO_ROOT/.claude/rules"
fi

# === Instalar RTK Hook ===
echo -e "\n=== Instalando RTK Hook ==="
if command -v rtk &>/dev/null && command -v jq &>/dev/null; then
  mkdir -p "$HOOKS_DIR"

  # Copia o RTK hook se existir no projeto
  if [ -f "$REPO_ROOT/scripts/rtk-rewrite.sh" ]; then
    cp "$REPO_ROOT/scripts/rtk-rewrite.sh" "$HOOKS_DIR/rtk-rewrite.sh"
    chmod +x "$HOOKS_DIR/rtk-rewrite.sh"
    log_info "RTK hook instalado"
  else
    # Cria hook genérico baseado no template do usuário
    cat > "$HOOKS_DIR/rtk-rewrite.sh" << 'HOOK_EOF'
#!/usr/bin/env bash
# rtk-hook-version: 3
# RTK Claude Code hook — rewrites commands to use rtk for token savings
# Requires: rtk >= 0.23.0, jq

[ -x "$(command -v jq)" ] || exit 0
[ -x "$(command -v rtk)" ] || exit 0

RTK_VERSION=$(rtk --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
if [ -n "$RTK_VERSION" ]; then
  MAJOR=$(echo "$RTK_VERSION" | cut -d. -f1)
  MINOR=$(echo "$RTK_VERSION" | cut -d. -f2)
  if [ "$MAJOR" -eq 0 ] && [ "$MINOR" -lt 23 ]; then
    exit 0
  fi
fi

INPUT=$(cat)
CMD=$(echo "$INPUT" | jq -r '.tool_input.command // empty')
[ -z "$CMD" ] && exit 0

REWRITTEN=$(rtk rewrite "$CMD" 2>/dev/null)
EXIT_CODE=$?

case $EXIT_CODE in
  0)
    [ "$CMD" = "$REWRITTEN" ] && exit 0
    ;;
  1|2)
    exit 0
    ;;
  3)
    ;;
  *)
    exit 0
    ;;
esac

ORIGINAL_INPUT=$(echo "$INPUT" | jq -c '.tool_input')
UPDATED_INPUT=$(echo "$ORIGINAL_INPUT" | jq --arg cmd "$REWRITTEN" '.command = $cmd')

if [ "$EXIT_CODE" -eq 3 ]; then
  jq -n --argjson updated "$UPDATED_INPUT" '{
    "hookSpecificOutput": {
      "hookEventName": "PreToolUse",
      "updatedInput": $updated
    }
  }'
else
  jq -n --argjson updated "$UPDATED_INPUT" '{
    "hookSpecificOutput": {
      "hookEventName": "PreToolUse",
      "permissionDecision": "allow",
      "permissionDecisionReason": "RTK auto-rewrite",
      "updatedInput": $updated
    }
  }'
fi
HOOK_EOF
    chmod +x "$HOOKS_DIR/rtk-rewrite.sh"
    log_info "RTK hook criado (genérico)"
  fi
else
  log_warn "RTK hook não instalado (rtk ou jq faltando)"
fi

# === Configurar settings.json ===
echo -e "\n=== Configurando settings.json ==="
SETTINGS_FILE="$CLAUDE_DIR/settings.json"
mkdir -p "$CLAUDE_DIR"

if [ ! -f "$SETTINGS_FILE" ]; then
  cat > "$SETTINGS_FILE" << 'SETTINGS_EOF'
{
  "theme": "auto",
  "alwaysThinkingEnabled": true,
  "maxThinkingTokens": 31999
}
SETTINGS_EOF
  log_info "settings.json criado com padrões"
else
  # Validar se já está configurado
  if grep -q "PreToolUse.*rtk-rewrite" "$SETTINGS_FILE" 2>/dev/null || grep -q "rtk-rewrite" "$SETTINGS_FILE" 2>/dev/null; then
    log_info "settings.json já configurado"
  else
    log_warn "Verifique $SETTINGS_FILE manualmente para adicionar hooks"
  fi
fi

# === Resumo ===
echo -e "\n=== Resumo da Instalação ==="
log_info "Configuração concluída!"
echo -e "
📁 Diretórios:
  - Regras: $RULES_DIR
  - Hooks:  $HOOKS_DIR

🔧 Para habilitar o RTK hook manualmente:
  1. Abra Claude Code → Settings
  2. Vá para 'Hooks' → 'PreToolUse'
  3. Adicione: $HOOKS_DIR/rtk-rewrite.sh

📖 Para mais info:
  - Regras: https://github.com/anthropics/claude-code/wiki/Rules
  - RTK: https://github.com/rtk-ai/rtk
"
