#!/bin/bash
# read-xml4-anbima.sh - Wrapper shell para read-xml4-anbima.rb (skill read-xml4-anbima)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUBY_SCRIPT="${SCRIPT_DIR}/read-xml4-anbima.rb"

if ! command -v ruby &> /dev/null; then
  echo "❌ Erro: Ruby não está instalado"
  exit 1
fi

if ! ruby -e "require 'nokogiri'" 2>/dev/null; then
  echo "❌ Erro: Nokogiri não está instalado. Instale com: gem install nokogiri"
  exit 1
fi

exec ruby "$RUBY_SCRIPT" "$@"
