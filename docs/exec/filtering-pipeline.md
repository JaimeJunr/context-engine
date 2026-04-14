# Filtering Pipeline — 8 Estágios

**Last Updated:** 2026-04-13

O pipeline de filtragem é o coração do sistema. Cada filtro (nativo ou declarativo) define uma sequência de 8 transformações que são aplicadas **sequencialmente e determinísticamente** sobre a saída de um comando.

## Estágio 1: Remoção de Códigos de Escape

Remove códigos ANSI de cor, negrito, resetar:
- Entrada: `\x1b[31mERROR\x1b[0m`
- Saída: `ERROR`

**Resultado:** Texto puro, sem formatação visual.

---

## Estágio 2: Substituições por Padrão

Aplica substituições textuais baseadas em regex. Exemplo:

```toml
[[exec.filters.substitutions]]
pattern = "^Build cache invalidated at .+$"
replacement = "[Cache invalidation suppressed]"
```

**Resultado:** Texto normalizável (timestamps removidos, etc).

---

## Estágio 3: Curto-Circuito (Short-Circuit)

Se a saída casou com um padrão global específico, **emite mensagem substituta** e para o pipeline.

Exemplo:

```toml
[exec.filters.short_circuit]
pattern = "FATAL.*out of memory"
message = "[Process killed: out of memory]"
```

**Resultado:** Saída breve para falhas catastróficas.

---

## Estágio 4: Seleção de Linhas

Filtra linhas por inclusão ou exclusão. Apenas um dos dois:

**Inclusão (whitelist):** Se `include_pattern` definido, retém apenas linhas que batem.

```toml
include_pattern = "^(ERROR|WARN|PASS|FAIL)"
```

**Exclusão (blacklist):** Se `exclude_pattern` definido, descarta linhas que batem.

```toml
exclude_pattern = "^\\s*\\|.*\\|\\s*$"  # Descarta linhas de tabela visual
```

**Resultado:** Apenas linhas relevantes permanecem.

---

## Estágio 5: Truncamento por Linha

Limita a largura máxima de cada linha:

```toml
max_line_width = 120
```

Linhas mais longas são cortadas com `[...]` no final.

**Resultado:** Saída formatável em terminais com largura fixa.

---

## Estágio 6: Retenção de Primeiras/Últimas Linhas

Preserva as primeiras N e/ou últimas M linhas:

```toml
keep_first_lines = 10
keep_last_lines = 20
```

Útil para tests (header + summary no fim).

**Resultado:** Linhas do meio são descartadas; contexto crítico preservado.

---

## Estágio 7: Limite Absoluto de Linhas

Força máximo de linhas totais:

```toml
max_lines = 100
```

Se resultado ainda tiver >100 linhas após etapa 6, descarta até ficar ≤100.

**Resultado:** Output nunca explode em tamanho.

---

## Estágio 8: Mensagem Padrão para Vazio

Se o resultado for vazio, substitui por mensagem padrão:

```toml
empty_message = "[No relevant output after filtering]"
```

**Resultado:** Agente LLM sabe que comando foi executado mesmo com output vazio.

---

## Exemplo Completo: Filtro de Teste

```toml
[[exec.native_filters]]
name = "pytest"
pattern = "^pytest.*"  # Identifica comando
ignore_case = true

[exec.native_filters.pipeline]
# 1. Remove codes ANSI (automático)
# 2. Substitui timestamps
substitutions = [
  {pattern = "\\d{2}:\\d{2}:\\d{2}", replacement = "[time]"}
]
# 3. Sem short-circuit
# 4. Inclui apenas linhas com PASS, FAIL, ERROR, etc
include_pattern = "^(PASSED|FAILED|ERROR|WARNINGS|=====)"
# 5. Max 120 chars por linha
max_line_width = 120
# 6. Guarda primeiro test header + últimas linhas de summary
keep_first_lines = 5
keep_last_lines = 10
# 7. Max 200 linhas totais
max_lines = 200
# 8. Se vazio, aviso
empty_message = "[Tests executed but no matches in output]"
```

**Input (1000 linhas):**
```
===== test session starts =====
collected 250 items
test/auth.py::test_login PASSED                                   [0%]
test/auth.py::test_logout PASSED                                  [1%]
...
test/profile.py::test_update_avatar FAILED                        [98%]
===== FAILURES =====
...
===== short test summary info =====
FAILED test/profile.py::test_update_avatar - AssertionError
===== 1 failed, 249 passed in 12.34s =====
```

**Output (50 linhas):**
```
===== test session starts =====
test/auth.py::test_login PASSED
test/auth.py::test_logout PASSED
...
===== FAILURES =====
...
FAILED test/profile.py::test_update_avatar
===== short test summary info =====
FAILED test/profile.py::test_update_avatar - AssertionError
===== 1 failed, 249 passed in [time]s =====
```

**Redução:** 95% (1000 → 50 linhas) ≈ 80% em tokens.

---

## Ordem é Crítica

Os estágios são **sequenciais e determinísticos**. Para mesma entrada + config:
- Resultado é sempre idêntico
- Reversão de ordem = resultado diferente (não faça!)
- Cada estágio recebe saída do anterior

---

## Configuração

Filtros podem ser:

1. **Nativos (hardcoded):** compilados no binário (git, pytest, cargo, etc)
2. **Declarativos:** carregados de arquivo `~/.config/ctx/filters.toml` ou `./.ctx/filters.toml` (projeto)

Prioridade:
1. Projeto (se confiado)
2. Usuário global
3. Nativo
4. Passagem íntegra

---

Veja:
- **[Configuration](configuration.md)** — Definir filtros declarativos
- **[Overview](overview.md)** — Visão geral do ctx exec
- **[Metrics](metrics.md)** — Entender economia dos filtros
