# Configuration — Customizing ctx exec

**Last Updated:** 2026-04-13

## Arquivo de Configuração

Localização padrão: `~/.config/ctx/config.toml`

Localização de projeto: `./.ctx/config.toml` (requer `ctx exec trust-project`)

### Seção `[exec]` — Comportamento Global

```toml
[exec]
# Como lidar com falhas de comando
# Options: "failures" (salva saída bruta), "always", "never"
fallback_mode = "failures"

# Quantos dias manter métricas de execução
metrics_retention_days = 90

# Tentar autodetectar filtro por extensão/tipo?
auto_detect_filters = true

# Audit mode: registra todos os comandos reescritos por hooks
audit_mode = false

# Aceitar filtros de projeto sem confirmação?
trust_project_filters = false
```

---

## Filtros Declarativos

### Localização

- **Global:** `~/.config/ctx/filters.toml`
- **Projeto:** `./.ctx/filters.toml` (requer confiança)

### Estrutura

```toml
[[exec.declarative_filters]]
name = "my-custom-ls"
description = "My ls customization"

# Padrão para identificar este filtro (regex)
# Exemplo: "^ls\\s" identifica "ls ..." mas não "sl"
invocation_pattern = "^ls\\s"
ignore_case = true

# Pipeline de transformação (vide Filtering Pipeline)
[exec.declarative_filters.pipeline]
substitutions = [
  {pattern = "\\x1b\\[[0-9;]*m", replacement = ""}  # ANSI codes
]
include_pattern = "^[^.]"  # Sem hidden files
max_line_width = 100
keep_first_lines = 10
max_lines = 50
empty_message = "[No files]"
```

### Exemplo: Filtro para `npm test`

```toml
[[exec.declarative_filters]]
name = "npm-test"
invocation_pattern = "^npm\\s+(run\\s+)?test"

[exec.declarative_filters.pipeline]
# Remove timestamps
substitutions = [
  {pattern = "\\d{1,2}:\\d{2}:\\d{2}", replacement = "[time]"},
  {pattern = "in \\d+ms", replacement = "in [duration]"}
]

# Inclui apenas linhas importantes
include_pattern = "^(PASS|FAIL|●|✓|✗|Test|Error|at )"

# Limita output
max_line_width = 120
keep_last_lines = 20
max_lines = 150
empty_message = "[Tests completed]"
```

### Exemplo: Filtro para `docker ps`

```toml
[[exec.declarative_filters]]
name = "docker-ps"
invocation_pattern = "^docker\\s+ps"

[exec.declarative_filters.pipeline]
# Mantém apenas coluna essencial (container ID + status + names)
include_pattern = "CONTAINER|STATUS|NAMES|Up|Exited"
max_line_width = 100
empty_message = "[No containers]"
```

---

## Exclusões de Reescrita

Comandos que **nunca** devem ser filtrados pelo hook:

```toml
[exec.hook_exclusions]
# Lista de padrões regex
patterns = [
  "^cat\\s+secret",
  "^openssl\\s+",
  "^grep\\s+-P"  # Grep with perl regex pode falhar com filtragem
]
```

Quando um agente tenta executar um comando que bate nesses padrões, o hook passa-o **sem alteração**.

---

## Configuração de Hook

```toml
[exec.hooks]
# Agentes suportados: "claude", "anthropic", etc
enabled_for = ["claude"]

# Modo debug: mostra comando original e reescrito
debug = false

# Se hook falhar, continuar com comando original?
graceful_fallback = true
```

---

## Persistência de Métricas

```toml
[exec.metrics]
# Onde armazenar histórico
# $HOME é expandido automaticamente
storage_dir = "$HOME/.cache/ctx/exec-metrics"

# Quantum de tempo entre cleanup de registros antigos
prune_interval_days = 1

# Nível de detalhe a registrar
# Options: "summary" (tokens + tempo), "full" (+ comando, projeto)
level = "summary"
```

---

## Telemetria (Disabled by Default)

```toml
[exec.telemetry]
enabled = false

# Se enabled, qual endpoint?
endpoint = "https://telemetry.example.com/ctx"

# Incluir SO/versão?
include_system_info = true

# Incluir caminho do projeto?
include_project_path = false
```

**Nota:** Telemetria é **opt-in** explícito. Desabilitar globalmente:

```bash
ctx exec telemetry disable
```

---

## Exemplo Completo

```toml
# ~/.config/ctx/config.toml

[exec]
fallback_mode = "failures"
metrics_retention_days = 90
auto_detect_filters = true
audit_mode = false
trust_project_filters = false

[exec.hook_exclusions]
patterns = [
  "^cat\\s+\\.env",
  "^openssl\\s+"
]

[exec.hooks]
enabled_for = ["claude"]
debug = false
graceful_fallback = true

[exec.metrics]
storage_dir = "$HOME/.cache/ctx/exec-metrics"
prune_interval_days = 1
level = "summary"

[exec.telemetry]
enabled = false

# Filtros customizados
[[exec.declarative_filters]]
name = "test-output"
invocation_pattern = "^(pytest|cargo test|npm test|go test)"
[exec.declarative_filters.pipeline]
include_pattern = "^(PASSED|FAILED|ERROR|test_)"
max_lines = 100
empty_message = "[Tests ran]"
```

---

## Validação de Configuração

Verificar se config está válida:

```bash
ctx exec validate-config
```

Mostra erros e avisos.

---

Veja:
- **[Filtering Pipeline](filtering-pipeline.md)** — Entender estágios do pipeline
- **[Overview](overview.md)** — Como usar ctx exec
- **[Metrics](metrics.md)** — Consultar histórico de economia
