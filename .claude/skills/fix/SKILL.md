---
name: fix
description: Corrige erros de terminal ou logs: identifica causa raiz, aplica correção e valida. Use quando o usuário mostrar saída de erro ou pedir para corrigir falha de execução.
---

# Fix

## Fluxo

### 1. Identificação

- Ler com atenção a saída do terminal ou logs
- Identificar erro específico e localização
- Documentar mensagens de erro completas e contexto (comando, ambiente, configurações)

### 2. Análise técnica

- **Usar MCP CONTEXT7 obrigatoriamente** para interpretar o erro
- Ler documentação criada pelo MCP CONTEXT7 no caminho retornado
- Pesquisar na web se necessário para contexto adicional
- Identificar causa raiz

### 3. Correção

- Corrigir com base na análise
- Implementar solução mínima necessária
- Garantir que a correção não introduz novos problemas

### 4. Validação

- Reexecutar o comando ou processo que falhou
- Verificar execução sem erros no terminal
- Confirmar comportamento esperado restaurado

### 5. Novos erros

- Se surgir novo erro: repetir identificar → analisar → corrigir → validar
- Documentar cada iteração

## Checklist

- [ ] Erro identificado; MCP CONTEXT7 usado na análise
- [ ] Correção aplicada e execução sem erros
- [ ] Causa raiz e correção documentadas brevemente
