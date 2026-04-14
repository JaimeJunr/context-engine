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
| `ctx exec run <cmd>` | Executa comando com filtragem automática |
| `ctx exec report` | Relatório de economia acumulada |
| `ctx exec rewrite <cmd>` | Retorna forma filtrada do comando |
| `ctx exec discover` | Analisa histórico para oportunidades |

## Domínios suportados

- Navegação de arquivos (ls, find, tree)
- Controle de versão (git status, log, diff)
- Testes (pytest, cargo test, jest)
- Build/lint (cargo, npm, eslint)
- Cloud/container (docker, kubectl, aws cli)
- Rede/dados (curl, jq, sqlite)
- Outros (via filtros declarativos)

## Integração com LLMs

Dois modos:

1. **Explícito:** Usuário prefixas comando com `ctx exec run`
2. **Transparente:** Hook de pré-execução intercepta automaticamente (instalado via `ctx exec install-hook`)

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
