---
name: harness-audit
description: Auditoria de qualidade do setup Claude Code com scorecard em 7 categorias. Identifica gaps e sugere melhorias prioritárias.
disable-model-invocation: true
---

# Harness Audit

Audita a configuração do Claude Code no repositório atual e retorna um scorecard priorizado.

## Usage

`/harness-audit [scope]`

- `scope` (opcional): `repo` (padrão), `hooks`, `skills`, `commands`, `agents`

## Categorias de Avaliação (0-10 cada)

1. **Tool Coverage** — Skills, agents e commands cobrem as necessidades do projeto?
2. **Context Efficiency** — CLAUDE.md, skills e rules estão otimizados em tokens?
3. **Quality Gates** — Hooks de verificação, linting, type-check existem?
4. **Memory Persistence** — Sessões persistem? Padrões são capturados?
5. **Eval Coverage** — Testes automatizados cobrem scripts e hooks?
6. **Security Guardrails** — Proteções de segurança (secrets, input validation) ativas?
7. **Cost Efficiency** — Modelo correto pra cada tarefa? MCP servers otimizados?

## Processo

### 1. Inventário
- Contar agents, skills, commands, rules, MCP servers
- Verificar existência de CLAUDE.md, settings.json, hooks
- Detectar stack do projeto (package.json, Gemfile, etc)

### 2. Verificações
Para cada categoria, checkar:
- Arquivos existem nos caminhos esperados
- Configurações estão completas
- Não há redundância excessiva

### 3. Scorecard

```
Harness Audit (repo): XX/70

- Tool Coverage:       X/10
- Context Efficiency:  X/10
- Quality Gates:       X/10
- Memory Persistence:  X/10
- Eval Coverage:       X/10
- Security Guardrails: X/10
- Cost Efficiency:     X/10

Top 3 Ações:
1) [Categoria] Ação específica (caminho do arquivo)
2) [Categoria] Ação específica (caminho do arquivo)
3) [Categoria] Ação específica (caminho do arquivo)
```

### 4. Recomendações
- Listar checks que falharam com caminhos exatos
- Sugerir skills/agents que resolveriam os gaps
- Priorizar por impacto

## Arguments

$ARGUMENTS:
- `repo|hooks|skills|commands|agents` (escopo opcional)
