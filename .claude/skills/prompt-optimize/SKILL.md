---
name: prompt-optimize
description: Analisa um prompt draft e gera versão otimizada. NÃO executa a tarefa — apenas análise e otimização do prompt.
disable-model-invocation: true
---

# /prompt-optimize

Analisa e otimiza o prompt fornecido para máximo aproveitamento dos recursos disponíveis.

## Tarefa

Aplicar pipeline de análise em 6 fases ao input do usuário:

0. **Detecção de Projeto** — Ler CLAUDE.md, detectar stack do projeto (package.json, Gemfile, go.mod, pyproject.toml, etc.)
1. **Detecção de Intenção** — Classificar tipo de tarefa (nova feature, bug fix, refactor, research, testing, review, documentação, infra, design)
2. **Avaliação de Escopo** — Avaliar complexidade (TRIVIAL / LOW / MEDIUM / HIGH / EPIC), usando tamanho do codebase como sinal
3. **Mapeamento de Componentes** — Mapear para skills, commands, agents e tier de modelo específicos disponíveis
4. **Detecção de Contexto Faltante** — Identificar gaps. Se 3+ itens críticos faltando, pedir esclarecimento antes de gerar
5. **Workflow & Modelo** — Determinar posição no ciclo de vida, recomendar tier de modelo, dividir em múltiplos prompts se HIGH/EPIC

## Formato de Output

### Diagnóstico
```
Intenção: [tipo]
Escopo: [nível]
Stack: [detectado]
Componentes recomendados: [skills/agents/commands]
Modelo sugerido: [Haiku/Sonnet/Opus]
```

### Versão Completa
Prompt otimizado detalhado, pronto para copiar e colar em nova sessão.

### Versão Rápida
Prompt compacto para a mesma tarefa.

## CRÍTICO

NÃO executar a tarefa do usuário. Output APENAS análise e prompt otimizado.
Se o usuário pedir execução direta, explicar que `/prompt-optimize` só produz análise e orientar a iniciar uma tarefa normal.

Responder no mesmo idioma do input do usuário.

## Input do Usuário

$ARGUMENTS
