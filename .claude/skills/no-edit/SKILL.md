---
name: no-edit
description: Modo somente leitura: nunca modifica código ou banco diretamente; apenas propõe alterações. Use quando o usuário pedir "no edit", "apenas sugestões" ou "não alterar código".
---

# No Edit

## Restrições

- **NUNCA** modificar código diretamente
- **NUNCA** modificar banco de dados diretamente
- **NUNCA** executar comandos que alterem arquivos sem autorização explícita
- **SEMPRE** apresentar propostas e sugestões antes de qualquer alteração

## Estrutura de propostas

Quando houver necessidade de alteração:

### Introdução

- Apresentar introdução clara sobre a alteração proposta
- Explicar contexto e motivo
- Descrever o que será modificado e por quê

### Código

- Apresentar código em blocos markdown com syntax highlighting
- Código completo e funcional
- Identificar claramente arquivos afetados

### Comentários

- Adicionar comentários apenas para: lógicas complexas, nomes não autoexplicativos, decisões arquiteturais importantes
- Manter comentários concisos; evitar óbvios ou redundantes

## Apresentação

- Aguardar aprovação explícita antes de implementar
- Fornecer contexto suficiente para decisão
- Destacar impactos e consequências
- Estar disponível para esclarecimentos

## Checklist

- [ ] Nenhuma modificação direta feita
- [ ] Propostas com introdução e código completo
- [ ] Comentários apenas quando necessário
