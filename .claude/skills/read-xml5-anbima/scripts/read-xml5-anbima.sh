#!/bin/bash
# read-xml5-anbima.sh - Wrapper shell para o script read-xml5-anbima.rb (skill read-xml5-anbima)
# Facilita a execução do script Ruby com validação de dependências

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUBY_SCRIPT="${SCRIPT_DIR}/read-xml5-anbima.rb"

# Verificar se Ruby está instalado
if ! command -v ruby &> /dev/null; then
  echo "❌ Erro: Ruby não está instalado"
  echo "Instale Ruby para usar este script"
  exit 1
fi

# Verificar se Nokogiri está instalado
if ! ruby -e "require 'nokogiri'" 2>/dev/null; then
  echo "❌ Erro: Nokogiri não está instalado"
  echo "Instale com: gem install nokogiri"
  exit 1
fi

# Executar script Ruby com todos os argumentos passados
exec ruby "$RUBY_SCRIPT" "$@"
