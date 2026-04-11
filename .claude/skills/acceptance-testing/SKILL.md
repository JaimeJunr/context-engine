---
name: acceptance-testing
description: Realiza testes de aceitação sistemáticos com critérios de aceite, uso de browser (nativo ou MCP) e relatório detalhado. Use quando o usuário pedir teste de aceitação, validação de funcionalidade ou verificação de critérios de aceite.
---

# Acceptance Testing

## Parâmetros

- **--feature**: Nome da funcionalidade (obrigatório)
- **--criteria**: Arquivo ou caminho para critérios de aceite (opcional)
- **--browser**: `native` (padrão, @Browser Cursor) ou `mcp` (Browser MCP)

## Fluxo

### 1. Preparação

- Identificar funcionalidade e consultar critérios de aceite documentados
- Verificar pré-requisitos, ambiente e estado do sistema
- Documentar versão e configurações relevantes

### 2. Ferramentas de teste

**Frontend**: Conforme --browser:

- **native**: Usar @Browser nativo do Cursor (navegar, snapshots, screenshots, interações)
- **mcp**: Usar ferramentas MCP (browser_navigate, browser_snapshot, browser_click, browser_type, browser_console_messages, browser_network_requests)

**Backend**: Chamadas via console (fetch/curl), validar status, headers, body; testar sucesso, erro, validação, autenticação.

**Integração**: Validar fluxo frontend → API; tratamento de erros em ambas as camadas.

### 3. Execução

- Seguir passos sequenciais; executar e documentar cada passo
- Verificar comportamento esperado; capturar evidências
- Testar casos de sucesso e erro quando aplicável

### 4. Critérios de aceite

- Verificar cada critério individualmente
- Validar em diferentes cenários; integrar com outras funcionalidades quando relevante

### 5. Relatório

Incluir sempre:

1. **Resumo executivo**: Aprovado/Reprovado e visão geral
2. **Passos executados**: Lista detalhada
3. **Partes acessadas**: Áreas do sistema testadas
4. **Ferramentas utilizadas**: Browser (nativo ou MCP) e console
5. **Critérios validados**: Status por critério
6. **Problemas encontrados**: Falhas ou inconsistências
7. **Itens não testados**: O que não foi possível validar e motivos
8. **Recomendações**: Reteste ou correções necessárias

## Checklist

- [ ] Critérios de aceite consultados e validados
- [ ] Ferramentas (browser + console) documentadas
- [ ] Resultado final (Aprovado/Reprovado) e recomendações definidos
