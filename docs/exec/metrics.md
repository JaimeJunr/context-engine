# Metrics — Entender Economia de Tokens

**Last Updated:** 2026-04-13

## O que é rastreado

Cada execução de `ctx exec run <cmd>` registra:

| Campo | Descrição |
|-------|-----------|
| `command` | Comando executado (ex: `git status`) |
| `timestamp` | Quando (ISO 8601 UTC) |
| `project_path` | Diretório de trabalho (ex: `/home/user/repo`) |
| `exit_code` | Código de saída (0 = sucesso) |
| `tokens_before` | Tokens na saída bruta (~1 token = 4 chars) |
| `tokens_after` | Tokens na saída filtrada |
| `reduction_percent` | Redução: `(before - after) / before * 100` |
| `elapsed_ms` | Tempo de execução em ms |
| `filter_applied` | Nome do filtro usado (ex: `git-status`) |

---

## Visualizar Métricas

### Relatório agregado

```bash
ctx exec report
```

Output:
```
┌─────────────────────────────────────────────────┐
│ ctx exec — Relatório de Economia                │
├─────────────────────────────────────────────────┤
│ Período: últimos 7 dias                          │
│ Execuções: 1,247                                 │
│ Tokens economizados: 892,641 (71% médio)        │
│ Tempo total: 14h 23m                            │
└─────────────────────────────────────────────────┘

Top 10 Comandos
────────────────────────────────────────────────────
git status      | 342 exec | 89% redução médio
pytest          | 289 exec | 78% redução médio
ls -la          | 156 exec | 65% redução médio
cargo test      | 98 exec  | 84% redução médio
npm test        | 87 exec  | 91% redução médio
```

### Relatório por período

```bash
ctx exec report --days 30      # Últimos 30 dias
ctx exec report --since 2026-04-01  # Desde data específica
ctx exec report --project /path/to/repo  # Apenas este projeto
```

### Formato JSON

```bash
ctx exec report --format json
```

Output:
```json
{
  "summary": {
    "period_days": 7,
    "executions": 1247,
    "tokens_before": 12543210,
    "tokens_after": 3651409,
    "total_reduction_percent": 71,
    "elapsed_total_ms": 51780000
  },
  "by_command": [
    {
      "command": "git status",
      "executions": 342,
      "tokens_before": 3421000,
      "tokens_after": 376310,
      "reduction_percent": 89
    }
  ]
}
```

---

## Consultas Customizadas

### Histórico completo em CSV

```bash
ctx exec query --format csv --output metrics.csv
```

Útil para análise em planilha.

### Filtrar por comando

```bash
ctx exec report --filter "^git"
```

Apenas comando que batem regex `^git`.

### Filtrar por projeto

```bash
ctx exec report --filter-project /home/user/context-engine
```

Apenas execuções dentro daquele diretório.

---

## Interpretar Redução

### Percentual de Redução

- **60-70%:** Bom (remover headers, linhas verbosas)
- **70-85%:** Excelente (tabelas, repetição, timestamps)
- **85-95%:** Ótimo (logs comprimidos, testes, git diffs)
- **>95%:** Excepcional (output gigante reduzido drasticamente)

### Exemplo: `pytest`

**Bruto:**
```
===== test session starts =====
platform linux -- Python 3.10.0, pytest-7.0.0, py-1.11.0
rootdir: /home/user/project
collected 250 items

test/auth.py::test_login PASSED                           [0%]
test/auth.py::test_logout PASSED                          [1%]
... (1000 linhas)
===== FAILURES =====
...
===== short test summary =====
FAILED test/profile.py::test_avatar
===== 1 failed, 249 passed in 12.34s =====
```

**Tokens bruto:** ~4,000

**Filtrado:**
```
===== test session starts =====
test/auth.py::test_login PASSED
... (50 linhas)
===== short test summary =====
FAILED test/profile.py::test_avatar
===== 1 failed, 249 passed =====
```

**Tokens filtrado:** ~250

**Redução:** (4000-250)/4000 = 93.75%

---

## Gestão de Dados

### Limpeza automática

Registros mais antigos que `metrics_retention_days` (padrão: 90) são removidos automaticamente durante leitura.

```toml
[exec.metrics]
metrics_retention_days = 90
```

### Limpeza manual

```bash
# Remover registros >30 dias
ctx exec prune --older-than 30

# Remover registros específicos
ctx exec prune --project /home/user/old-repo
```

### Backup

```bash
# Exportar antes de limpar
ctx exec query --format json > metrics-backup.json
```

---

## Insights Úteis

### Qual comando consome mais tokens?

```bash
ctx exec report --order tokens_before --limit 5
```

### Qual filtro é mais eficiente?

```bash
ctx exec report --group-by filter --order reduction_percent
```

### Histórico de um projeto

```bash
ctx exec report --project . --days 30
```

Output:
```
┌────────────────────────────────────────┐
│ context-engine — últimos 30 dias       │
├────────────────────────────────────────┤
│ Execuções: 234                         │
│ Economia total: 156,789 tokens         │
│ Redução média: 76%                     │
│ Top comando: git status (184 exec)     │
└────────────────────────────────────────┘
```

---

## Dica de Performance

Menos execuções = menos overhead de contexto:

- Agrupe comandos quando possível
- Reutilize `ctx exec report` em vez de re-rodar
- Use `--filter` para buscar métricas específicas em vez de gerar relatório completo

---

Veja:
- **[Overview](overview.md)** — Começar com ctx exec
- **[Configuration](configuration.md)** — Customizar retenção de métricas
- **[Filtering Pipeline](filtering-pipeline.md)** — Como redução acontece
