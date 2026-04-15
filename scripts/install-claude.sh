#!/usr/bin/env bash
# Configura Claude Code para o projeto context-engine
# - Compila e instala ctx (versão mais recente)
# - Instala regras do projeto em ~/.claude/rules/
# - Instala ctx-rewrite hook para comprimir output de comandos
# - Valida dependências (ctx, jq)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

CLAUDE_DIR="$HOME/.claude"
CLAUDE_MD="$CLAUDE_DIR/CLAUDE.md"
RULES_DIR="$CLAUDE_DIR/rules"
HOOKS_DIR="$CLAUDE_DIR/hooks"
LOCAL_BIN_DIR="$HOME/.local/bin"

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}✓${NC} $1"; }
log_warn() { echo -e "${YELLOW}⚠${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1"; }

# === Compilar e Instalar ctx ===
echo "=== Compilando e instalando ctx ==="
mkdir -p "$LOCAL_BIN_DIR"

# Build release
cd "$REPO_ROOT"
cargo build --release 2>&1 | tail -5
if [ -f "target/release/ctx" ]; then
  cp "target/release/ctx" "$LOCAL_BIN_DIR/ctx"
  chmod +x "$LOCAL_BIN_DIR/ctx"
  log_info "ctx compilado e instalado em $LOCAL_BIN_DIR/ctx"
else
  log_error "Falha ao compilar ctx"
  exit 1
fi

# === Validação de Dependências ===
echo -e "\n=== Verificando dependências ==="
if command -v ctx &>/dev/null; then
  log_info "ctx encontrado (versão: $(ctx --version))"
else
  log_error "ctx não foi instalado corretamente"
  exit 1
fi

if ! command -v jq &>/dev/null; then
  log_warn "jq não encontrado. ctx-rewrite hook não funcionará."
  log_info "Instale: apt-get install jq (ou equivalente)"
fi

# === Instalar Regras do Projeto ===
echo -e "\n=== Instalando Regras do Projeto ==="
mkdir -p "$RULES_DIR"

# Copia regras do projeto (.claude/rules/)
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

# Copia rule de ctx (llms/rules/)
if [ -d "$REPO_ROOT/llms/rules" ]; then
  find "$REPO_ROOT/llms/rules" -type f -name "*.md" | while read -r file; do
    filename=$(basename "$file")
    dest="$RULES_DIR/$filename"
    cp "$file" "$dest"
    log_info "Regra instalada: $filename"
  done
else
  log_warn "Nenhuma rule de ctx encontrada em $REPO_ROOT/llms/rules"
fi

# === Instalar ctx-rewrite Hook ===
echo -e "\n=== Instalando ctx-rewrite Hook ==="
if command -v jq &>/dev/null && command -v ctx &>/dev/null; then
  mkdir -p "$HOOKS_DIR"

  # Copia o ctx-rewrite hook do projeto (agora em llms/hooks/)
  if [ -f "$REPO_ROOT/llms/hooks/ctx-rewrite.sh" ]; then
    cp "$REPO_ROOT/llms/hooks/ctx-rewrite.sh" "$HOOKS_DIR/ctx-rewrite.sh"
    chmod +x "$HOOKS_DIR/ctx-rewrite.sh"
    log_info "ctx-rewrite hook instalado"
  else
    log_error "ctx-rewrite.sh não encontrado em $REPO_ROOT/llms/hooks/"
    exit 1
  fi
else
  log_warn "ctx-rewrite hook não instalado (jq ou ctx faltando)"
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
  - ctx:    $LOCAL_BIN_DIR/ctx
  - Regras: $RULES_DIR
  - Hooks:  $HOOKS_DIR

🚀 Comandos do ctx:
  - ctx map --title \"título\" --dirs . --max-tokens 4096
  - ctx catalog search acervo \"query\"
  - ctx exec report (relatório de economia de tokens)

🔧 Para habilitar o ctx-rewrite hook manualmente:
  1. Abra Claude Code → Settings
  2. Vá para 'Hooks' → 'PreToolUse'
  3. Adicione: $HOOKS_DIR/ctx-rewrite.sh

📖 Para mais info:
  - Regras: $RULES_DIR
  - ctx guide: $REPO_ROOT/llms/rules/context-engine.md
  - CLAUDE.md: $REPO_ROOT/CLAUDE.md
"
