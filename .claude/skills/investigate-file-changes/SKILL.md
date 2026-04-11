---
name: investigate-file-changes
description: Investiga quem alterou arquivos específicos e o que mudou, para correlacionar regressões ou quebras com commits e autores.
disable-model-invocation: true
---

# Investigate File Changes

Investiga quem alterou arquivos específicos e o que mudou, para correlacionar regressões ou quebras com commits e autores.

## Escopo e segurança

- **Repo**: Comandos assumem um repo git; execute da raiz do repo.
- **Paths**: Sempre passe caminhos após `--` para git tratar como paths (`git log -- <path>`).
- **Output limits**: Use `-N` e `head -N` para evitar output enorme.
- **Read-only**: Apenas `log`, `show`, `blame` (sem checkout, reset, ou branch changes).

## Workflow

### 0. Blame: quem alterou uma linha específica

```bash
cd <REPO_ROOT> && git blame -L <START>,<END> -- <FILE_PATH>
```

Linha única: `-L 15,15` ou `-L 15,+1`. Use o hash retornado com `git show <HASH>`.

### 1. Listar commits que tocaram o arquivo

```bash
cd <REPO_ROOT> && git log --oneline -30 -- <FILE_PATH>
```

### 2. Ver o que um commit específico alterou

**Resumo:**
```bash
cd <REPO_ROOT> && git show <COMMIT_HASH> --stat
```

**Diff de um arquivo (preferido):**
```bash
cd <REPO_ROOT> && git show <COMMIT_HASH> -- <FILE_PATH>
```

### 3. Histórico de patches de um arquivo (com follow para renames)

```bash
cd <REPO_ROOT> && git log -p --follow -5 -- <FILE_PATH> 2>/dev/null | head -400
```

### 4. Ancestrais de um commit

```bash
cd <REPO_ROOT> && git log --oneline -10 <COMMIT_HASH>
```

### 5. Regressão entre versões

```bash
cd <REPO_ROOT> && git log -p <OLD_TAG_OR_HASH>..<NEW_TAG_OR_HASH> -- <FILE_PATH>
```

## Apresentando resultados

1. **Quem / quando**: `git blame -L <line>,<line> -- <FILE>` → `git log -1 --format="%h %an %ad (%ar) — %s" --date=short <HASH>`
2. **O que**: `git show <HASH> --stat`
3. **Como**: `git show <HASH> -- <FILE_PATH>`

Para regressões entre versões:
- Listar commits no range: `git log --oneline <OLD>..<NEW> -- <FILE_PATH>`
- Inspecionar cada suspeito: `git show <HASH> -- <FILE_PATH>`
- Sugerir o commit mais provável e a mudança exata que pode ter causado a quebra.
