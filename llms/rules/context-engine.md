# ctx — Guia de Uso para Agentes

**Objetivo**: Usar `ctx` de forma correta para economizar tokens e entender código rapidamente.

`ctx` tem **dois propósitos distintos**:
1. **`ctx map`** — Entender arquitetura e estrutura (sem executar comandos)
2. **`ctx exec`** — Comprimir output de comandos (economia 60-90% de tokens)

---

## 🎯 Os Três Níveis

| Nível | Ferramenta | Quando Usar | Exemplo |
|-------|-----------|------------|---------|
| **1. Shell direto** | `find`, `grep`, `ls` | Você já sabe exatamente o que quer | `find src -name "*.ts"` |
| **2. Estrutura** | `ctx map` | Precisa explorar/entender arquitetura e relevância | `ctx map --title "auth module" --dirs . --seeds src/auth` |
| **3. Tokens** | `ctx exec <cmd>` | Comando tem output verboso (testes, build, logs) | `ctx exec cargo test` |

---

## 📊 Matriz: Quando Usar Cada Um

```
┌─────────────────────────────────────────────────────────────┐
│ DECISÃO: Qual ferramenta usar?                              │
└─────────────────────────────────────────────────────────────┘

Pergunta 1: Você já sabe exatamente o que procura?
  ├─ SIM → Use shell direto: find, grep, ls
  └─ NÃO → Pergunta 2

Pergunta 2: Você precisa entender estrutura ou fluxo?
  ├─ SIM → Use ctx map
  │        (explora relevância, ranking, dependências)
  └─ NÃO → Pergunta 3

Pergunta 3: O comando tem output verboso/longo?
  ├─ SIM → Use ctx exec <cmd>
  │        (comprime 60-90% de tokens)
  └─ NÃO → Use shell direto
```

---

## 🗺️ `ctx map` — Entender Arquitetura

**O que faz**: Analisa código local, extrai assinaturas (classes, funções, tipos), ranqueia por relevância e gera mapa compacto da estrutura.

**Quando usar**:
- Primeira exploração de um módulo desconhecido
- Entender fluxo controller → service → model
- Refatoração de sistema inteiro
- Encontrar arquivos relacionados (sem saber nomes exatos)

**Não use para**:
- Localizar arquivo específico (use `find` em vez disso)
- Procurar padrão de texto (use `grep` em vez disso)
- Comprimir output de comando (use `ctx exec` em vez disso)

### Exemplos

```bash
# Entender arquitetura geral
ctx map --title "arquitetura geral" --dirs . --max-tokens 4096

# Focar em autenticação
ctx map --title "auth flow" --dirs . --seeds src/auth,src/middleware

# Top 20 arquivos mais relevantes
ctx map --title "entender ranking" --dirs . --top 20
```

---

## 🔧 `ctx exec` — Comprimir Output

**O que faz**: Intercepta comando, executa, filtra output inteligentemente (remove cores, linhas desnecessárias, trunca), economiza tokens.

**Quando usar**:
- Testes: `cargo test`, `pytest`, `jest`, `npm test` (saída de 100+ linhas)
- Build: `cargo build`, `cargo clippy`, `npm install` (com warnings/info)
- Logs: `docker logs`, `git log`, `kubectl logs` (muitas linhas)
- Navegação: `ls`, `find`, `grep`, `tree` (output longo)
- Qualquer comando com output verboso

**Comportamento para comandos sem filtro**:
- `ctx exec` é um **proxy universal** — se não tiver filtro para o comando, executa normalmente (passthrough)
- Nunca falha com "comando desconhecido" — use livremente para qualquer comando

**Domínios com filtro otimizado** (60-90% redução):
- Navegação: `ls`, `find`, `tree`, `grep`, `rg`
- Git: `git status`, `git log`, `git diff`, `git show`, `git branch`
- Rust: `cargo test`, `cargo build`, `cargo clippy`, `cargo fmt`, `cargo run`
- Node: `npm`/`yarn`/`pnpm` (install/test/build), `jest`, `vitest`
- Python: `pytest`, `python`
- GitHub: `gh pr`, `gh issue`, `gh run`
- Container: `docker ps/images/logs`, `kubectl get/logs`
- AWS: `aws` (genérico), `aws logs`
- Dados: `curl`, `jq`, `sqlite3`

### Exemplos

```bash
# Executar testes com compressão
ctx exec cargo test

# Rodar build
ctx exec npm install

# Ver logs do git
ctx exec git log --oneline

# Relatório de economia acumulada
ctx exec report
```

---

## 📍 Shell Direto — Quando Você Já Sabe

**O que faz**: Executa comando normalmente, sem reescrita.

**Quando usar**:
- Você já sabe o nome exato: `find src/controllers -name "*UserController*"`
- Padrão claro: `grep -r "class AuthService"`
- Output curto esperado: `ls -la`, `git status`

**Não use quando**:
- Procurando explorar (use `ctx map`)
- Output será verboso (use `ctx exec`)
- Não tem certeza do que quer

---

## 🚀 Fluxo Prático: "Adicionar Validação de Email"

**Cenário**: Você entra no projeto, precisa adicionar validação de email no cadastro.

```
1. EXPLORAR (ctx map)
   └─ Entender arquitetura geral
      ctx map --title "entender MVC" --dirs .

2. FOCAR (ctx map com seeds)
   └─ Encontrar fluxo de cadastro
      ctx map --title "user signup flow" --dirs . --seeds src/controllers,src/services

3. LOCALIZAR (shell direto)
   └─ Achar arquivo específico
      find src/controllers -name "*User*Controller*"
      grep -n "register\|signup" src/controllers/UserController.ts

4. IMPLEMENTAR (TDD + shell)
   └─ Escrever testes
      vim src/services/UserService.test.ts
   └─ Rodar testes (com compressão)
      ctx exec cargo test

5. VALIDAR (shell direto)
   └─ Verificar estrutura antes de commitar
      find . -name "*.test.ts" | grep -i user
      git diff src/
```

---

## ⚠️ Anti-Patterns

| ❌ Não Faça | ✅ Faça |
|-----------|--------|
| `ctx map` para localizar arquivo | `find` para localizar, depois `ctx map` para entender |
| `ctx map` para procurar padrão de texto | `grep` para procurar, depois `ctx map` se não entender |
| `ls -la src/` na exploração aleatória | `ctx map --dirs src` para exploração inteligente |
| Output verboso sem comprimir | `ctx exec <cmd>` para comprimir |
| `ctx exec` em comando interativo (vim, ssh) | Shell direto para TTY interativo |

---

## 📚 Resumo por Caso de Uso

| Caso | Ferramenta | Razão |
|-----|-----------|-------|
| Entender novo módulo | `ctx map` | Ranqueia por relevância |
| Achar classe/função específica | `find` + `grep` | Você já sabe o padrão |
| Ver testes falhando | `ctx exec cargo test` | Comprime 100+ linhas |
| Explorar sem saber por onde começar | `ctx map --seeds <dir>` | Navegação inteligente |
| Ver output de build | `ctx exec cargo build` | Remove colors/warnings desnecessários |
| Verificar git history | Shell (`git log`) ou `ctx exec git log` | Curto: shell; longo: ctx exec |

---

## 🔌 Hook Automático (Opcional)

Se o hook `ctx-rewrite` estiver instalado, comandos como `cargo test`, `npm install`, `find`, `git` são automaticamente envolvidos em `ctx exec <cmd>` — silenciosamente, sem pergunta.

**Para desabilitar**: remova o hook em Claude Code Settings → Hooks → PreToolUse.

---

## 📖 Para Mais Detalhes

- **ctx map**: `ctx map --help`
- **ctx exec**: `ctx exec report --help`
