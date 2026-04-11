---
name: learn
description: Extrai padrões reutilizáveis da sessão atual e salva como skills aprendidas. Use ao resolver problemas não-triviais para capturar conhecimento do time.
disable-model-invocation: true
---

# Learn - Extração de Padrões

Analisa a sessão atual e extrai padrões que valem ser salvos como skills reutilizáveis.

## Quando Usar

- Ao resolver um problema não-trivial
- Quando o usuário pedir `/learn`
- Após descobrir um workaround importante
- Quando um padrão de debugging se revelar útil

## O Que Extrair

### 1. Padrões de Resolução de Erros
- Qual erro ocorreu?
- Qual foi a causa raiz?
- O que corrigiu?
- É reutilizável para erros similares?

### 2. Técnicas de Debugging
- Passos de debugging não-óbvios
- Combinações de ferramentas que funcionaram
- Padrões de diagnóstico

### 3. Workarounds
- Quirks de bibliotecas
- Limitações de APIs
- Fixes específicos de versão

### 4. Padrões Específicos do Projeto
- Convenções do codebase descobertas
- Decisões de arquitetura tomadas
- Padrões de integração

## Formato de Output

Criar skill em `~/.claude/skills/learned/[nome-do-padrao].md`:

```markdown
# [Nome Descritivo do Padrão]

**Extraído:** [Data]
**Contexto:** [Descrição breve de quando se aplica]

## Problema
[O que este padrão resolve - ser específico]

## Solução
[O padrão/técnica/workaround]

## Exemplo
[Exemplo de código se aplicável]

## Quando Usar
[Condições de trigger - o que deve ativar esta skill]
```

## Processo

1. Revisar a sessão para padrões extraíveis
2. Identificar o insight mais valioso/reutilizável
3. Rascunhar o arquivo de skill
4. Pedir confirmação do usuário antes de salvar
5. Salvar em `~/.claude/skills/learned/`

## O Que NÃO Extrair

- Fixes triviais (typos, erros de sintaxe simples)
- Issues one-time (outages de API específicas, etc.)
- Padrões que já existem como skills
- Manter uma skill por padrão - focado e reutilizável
