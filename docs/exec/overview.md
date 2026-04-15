# ctx exec — Command Output Compression

**Last Updated:** 2026-04-13

## O que é

`ctx exec` é um proxy de linha de comando que comprime a saída de comandos de desenvolvimento para consumo eficiente por agentes LLM. Reduz tipicamente 60-90% do volume textual sem perder informações críticas.

## Por que existe

Agentes LLM executam comandos cuja saída bruta satura a janela de contexto: testes com 1000 linhas de output, logs verbose, relatórios com tabelas gigantes. O sistema intercepta, filtra e preserva apenas o essencial: estado, erros, avisos. Descarta ruído: códigos de cor, headers repetidos, progresso.

## Como funciona (visão geral)

```
Comando do usuário
       |
       v
┌─────────────────────────────────────┐
│  ctx exec <subcomando> <cmd> <args> │
└────────────────┬────────────────────┘
                 |
         ┌───────┴────────┐
         |                |
         v                v
   Nativo?          Declarativo?
   (hardcoded)      (arquivo .toml)
         |                |
         └────┬───────────┘
              v
      ┌─────────────────┐
      │ Pipeline filtro │  8 etapas sequenciais
      │  (8 estágios)   │
      └────────┬────────┘
               v
      Saída comprimida + métricas
```

## Subcomandos principais

| Subcomando | Função |
|---|---|
| `ctx exec <cmd> [args...]` | Executa comando com filtragem automática (passthrough se sem filtro) |
| `ctx exec report` | Relatório de economia acumulada |

## Domínios suportados

- Navegação de arquivos: `ls`, `find`, `tree`, `grep`, `rg`
- Controle de versão: `git status`, `git log`, `git diff`, `git show`, `git branch`
- Build/teste Rust: `cargo test`, `cargo build`, `cargo clippy`, `cargo fmt`, `cargo run`
- Build/teste Node: `npm`, `yarn`, `pnpm` (install, test, build), `jest`, `vitest`
- Testes Python: `pytest`, `python`
- GitHub CLI: `gh pr`, `gh issue`, `gh run`
- Cloud/container: `docker ps`, `docker images`, `docker logs`, `kubectl get`, `kubectl logs`
- AWS CLI: `aws` (genérico), `aws logs` (otimizado)
- Rede/dados: `curl`, `wget`, `jq`, `sqlite3`
- Qualquer outro comando: **passthrough transparente** (executa normalmente, sem erro)

## Integração com LLMs

Dois modos:

1. **Explícito:** Prefixar comando com `ctx exec` — ex: `ctx exec cargo test`
2. **Transparente:** Hook de pré-execução intercepta automaticamente (configurar via Claude Code Settings → Hooks → PreToolUse)

## Métricas

Cada execução registra:
- Tokens antes/depois (aproximação: 1 token ≈ 4 caracteres)
- Tempo decorrido
- Projeto (diretório de trabalho)
- Comando executado

Histórico retido por 90 dias (configurável).

## Configuração mínima

Arquivo: `~/.config/ctx/config.toml`

```toml
[exec]
fallback_mode = "failures"
metrics_retention_days = 90
```

Sem arquivo: funciona com defaults sensatos.

---

Veja:
- **[Filtering Pipeline](filtering-pipeline.md)** — Como os 8 estágios de filtro funcionam
- **[Configuration](configuration.md)** — Customização de filtros e comportamento
- **[Metrics](metrics.md)** — Entender e consultar métricas de economia
