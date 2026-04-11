---
name: run-tests
description: Executa todos os testes (unitários e integração), analisa falhas e corrige de forma ordenada. Use quando o usuário pedir para rodar testes ou corrigir falhas de teste.
---

# Run Tests

## Parâmetros

- **--test-type**: `unit` | `integration` | `all` (padrão: `all`)
- **--fix-mode**: `auto` | `manual` | `suggest` (padrão: `suggest`)
- **--verbose**: Saída detalhada (padrão: false)
- **--parallel**: Executar em paralelo (padrão: true)

## Fluxo

### 1. Preparação

- Verificar ambiente e dependências de teste
- Identificar framework (Jest, RSpec, Mocha, etc.)
- Configurar variáveis de ambiente necessárias

### 2. Execução

- Executar todos os testes unitários e de integração
- Capturar saída completa e identificar todas as falhas
- Categorizar falhas por tipo (unitário, integração) e criticidade
- Documentar ambiente e configurações

### 3. Análise e correção

- Priorizar correções pelo impacto
- Identificar falhas relacionadas e dependências
- Corrigir um problema por vez; reexecutar testes relevantes
- Validar que correções não introduzem novas falhas
- Documentar mudanças e justificativas

### 4. Finalização

- Verificar que todos os testes passam
- Confirmar que funcionalidade não foi quebrada
- Documentar resumo das correções
- Limpar arquivos temporários

## Troubleshooting

- **Testes flaky**: Verificar tempo, ordem, condições de corrida; isolar testes com estado externo
- **Configuração**: Validar variáveis de ambiente, banco de teste, paths e imports
- **Performance**: Identificar testes lentos; usar mocks; configurar paralelo quando possível

## Checklist

- [ ] Todos os testes executados; falhas categorizadas
- [ ] Correções aplicadas e validadas sem novas falhas
- [ ] Resumo das correções documentado
