---
name: using-git-worktrees
description: Use when feature work must stay isolated from the current checkout, parallel branches are needed without git switch or checkout, an implementation plan should run in a separate working tree, the working tree is dirty and blocks branch changes, or the user asks for a git worktree, isolated workspace, bare repository, or second checkout of the same repo.
---

# Using Git Worktrees

## Overview

Git worktrees allow multiple working directories to share the same Git object database, so different branches can be edited simultaneously without switching.

This skill supports two patterns, detected automatically per project:

| Pattern | Detecção | Worktrees |
|---------|----------|-----------|
| **IVT Container** | `.git` é arquivo + `.bare/` existe | Dentro do container (`./CAP-XXXX`) |
| **Convencional** | `.git` é diretório | Como irmãos fora (`../CAP-XXXX`) |

**Announce at start:** State that the using-git-worktrees skill is being used and which pattern was detected for each project involved.

**Two-phase flow:** Every use of this skill follows Etapa 1 (setup do repo) → Etapa 2 (gerenciamento de worktrees).

---

## Etapa 1 — Detecção e Setup do Repo

### Detecção de padrão

```bash
# IVT Container Pattern?
[ -f .git ] && grep -q "gitdir: ./.bare" .git && [ -d .bare ]

# Convencional?
[ -d .git ]
```

**Se IVT Container detectado:** skip para Etapa 2.

**Se convencional:** escolha um dos sub-cenários abaixo.

---

### Sub-cenário A — Novo projeto (clone bare direto da URL)

```bash
git clone --bare <url> .bare
echo "gitdir: ./.bare" > .git

# Configurar refspec para que git fetch popule refs/remotes/origin/*
# (bare clones não fazem isso por padrão)
cd .bare
git config remote.origin.fetch "+refs/heads/*:refs/remotes/origin/*"
git fetch origin
cd ..

git worktree add master master
git worktree add development -b development origin/development
git worktree add release -b release origin/release
```

---

### Sub-cenário B — Migrar repo convencional in-place (arquivos fonte ficam na raiz do container)

Informar o usuário:

```
Este repo usa .git/ convencional.
Migrar para IVT Container Pattern: renomeia .git/ → .bare/ e cria arquivo .git pointer.
Os arquivos fonte permanecem na raiz do container (checkout do branch atual).
Operação segura mas irreversível sem passos manuais.

Comandos para executar:
  cd /path/do/projeto
  mv .git .bare
  echo "gitdir: ./.bare" > .git
  git config --file .bare/config core.bare false
  git worktree add master master
  git worktree add development -b development origin/development
  git worktree add release -b release origin/release
```

> Passar os comandos para o usuário executar — não executar diretamente por serem destrutivos.

---

### Sub-cenário C — Migrar repo convencional para container limpo via clone local (padrão IVT)

**Preferir este sub-cenário** quando o objetivo é ter o container root limpo (sem arquivos fonte), apenas `.bare/`, `.git` e worktrees nomeadas — exatamente o padrão usado em `performit` e `performit-rails`.

Verificar antes:
1. Repo está limpo (`git status --short` vazio)
2. Estar no branch principal (`git checkout main` ou `git checkout master`)
3. Quais branches remotos existem (`git branch -r | grep -E "origin/(master|main|development|release)$"`)

Passar ao usuário para executar (substituir `<projeto>`, `<remote-url>` e os branches reais):

```bash
cd /home/jaime/ivt && \
mv <projeto> <projeto>-old && \
mkdir <projeto> && \
git clone --bare <projeto>-old/.git <projeto>/.bare && \
echo "gitdir: ./.bare" > <projeto>/.git && \
git config --file <projeto>/.bare/config core.bare false && \
git -C <projeto> remote set-url origin <remote-url> && \
git config --file <projeto>/.bare/config remote.origin.fetch "+refs/heads/*:refs/remotes/origin/*" && \
git -C <projeto> fetch origin --quiet && \
git -C <projeto> worktree add main main && \
rm -rf <projeto>-old && \
git -C <projeto> worktree list
```

Adicionar worktrees para os branches que existirem:

```bash
# Se development existir no remote:
git -C <projeto> worktree add development -b development origin/development

# Se release existir no remote:
git -C <projeto> worktree add release -b release origin/release
```

> Passar os comandos para o usuário executar — mv e rm -rf são destrutivos.

---

## Etapa 2 — Gerenciamento de Worktrees

### IVT Container Pattern

Worktrees ficam **dentro** do container:

```bash
# Criar worktree para uma feature/tarefa
cd /home/jaime/ivt/<projeto>
git worktree add <CAP-XXXX> -b <CAP-XXXX>-<descricao>

# Para branch já existente no remote
git worktree add <CAP-XXXX> origin/<branch-name>
```

#### Setup de projeto — Rails (`Gemfile` detectado)

```bash
cd <CAP-XXXX>
bundle install
# Copiar .env do worktree de referência (master ou development)
cp ../master/.env .env 2>/dev/null || cp ../development/.env .env 2>/dev/null || true
```

#### Setup de projeto — Grails (`grails-app/` detectado)

```bash
cd <CAP-XXXX>
# Grails compila na primeira execução; verificar application.properties se necessário
cp ../master/application.properties application.properties 2>/dev/null || true
```

#### Setup de projeto — outros (auto-detect)

```bash
if [ -f package.json ]; then npm install; fi
if [ -f Cargo.toml ]; then cargo build; fi
if [ -f requirements.txt ]; then pip install -r requirements.txt; fi
if [ -f pyproject.toml ]; then poetry install; fi
if [ -f go.mod ]; then go mod download; fi
```

#### Verificar baseline

Executar o comando de testes do projeto:

- Rails: `bundle exec rspec` (ou subconjunto relevante)
- Grails: `./grails-build.sh test` (ou comando documentado)
- Node: `npm test`

**Se testes falharem:** reportar output, distinguir regressão do baseline vs. problema de ambiente, perguntar se continua ou corrige primeiro.

#### Reportar ao usuário

```
Worktree pronta em <caminho-absoluto>
Branch: <branch-name>
Setup: <o que foi executado>
Testes: <comando> — <resumo>
```

---

### Padrão Convencional

Worktrees ficam como **irmãos fora** do diretório do projeto:

```bash
git worktree add ../<CAP-XXXX> -b <CAP-XXXX>-<descricao>
cd ../<CAP-XXXX>
```

Setup de projeto e verificação de baseline: mesmas regras da seção IVT Container acima.

---

## CAP Workspace — Multi-Projeto (IVT)

Quando uma tarefa envolve múltiplos projetos, criar um **CAP workspace** em `/home/jaime/ivt/CAP-XXXX/` com symlinks para cada worktree.

### Criação

```bash
CAP=CAP-XXXX
mkdir -p /home/jaime/ivt/$CAP

# Para cada projeto envolvido na tarefa:

# performit (IVT Container) → worktree dentro do container
git -C /home/jaime/ivt/performit worktree add $CAP -b ${CAP}-grails-<descricao>
ln -s ../performit/$CAP /home/jaime/ivt/$CAP/grails

# performit-rails (IVT Container)
git -C /home/jaime/ivt/performit-rails worktree add $CAP -b ${CAP}-rails-<descricao>
ln -s ../performit-rails/$CAP /home/jaime/ivt/$CAP/rails

# grand-bazaar (Convencional) → worktree como irmão
git -C /home/jaime/ivt/grand-bazaar worktree add /home/jaime/ivt/grand-bazaar-$CAP -b ${CAP}-gb-<descricao>
ln -s ../grand-bazaar-$CAP /home/jaime/ivt/$CAP/grand-bazaar

# outros projetos convencionais seguem o mesmo padrão de grand-bazaar
```

> Incluir apenas os projetos realmente envolvidos na tarefa. Omitir os demais.

### Estrutura resultante

```
/ivt/CAP-XXXX/
  grails        -> ../performit/CAP-XXXX
  rails         -> ../performit-rails/CAP-XXXX
  grand-bazaar  -> ../grand-bazaar-CAP-XXXX   (convencional)
```

### Remoção

```bash
CAP=CAP-XXXX

# IVT Container projects
git -C /home/jaime/ivt/performit worktree remove $CAP
git -C /home/jaime/ivt/performit-rails worktree remove $CAP

# Projetos convencionais
git -C /home/jaime/ivt/grand-bazaar worktree remove /home/jaime/ivt/grand-bazaar-$CAP
rm -rf /home/jaime/ivt/grand-bazaar-$CAP

# Workspace
rm -rf /home/jaime/ivt/$CAP
```

---

## Listar worktrees ativas

```bash
# IVT Container
git -C /home/jaime/ivt/performit worktree list
git -C /home/jaime/ivt/performit-rails worktree list

# Convencional
git worktree list   # dentro do diretório do projeto
```

---

## Navegação rápida (zoxide + fzf)

```bash
# zoxide: aprende diretórios frequentes
z CAP-8519       # salta para o workspace da tarefa
zi               # seleção interativa via zoxide

# fzf: seleção entre worktrees irmãos
cd $(ls -d /home/jaime/ivt/*/ | fzf)
```

---

## Erros comuns

| Erro | Por que prejudica | Correção |
|------|------------------|----------|
| Criar worktree dentro do `.bare/` | Polui o banco Git | Usar sempre caminhos fora de `.bare/` |
| Esquecer `core.bare false` após migração | `git fetch/push` falha | `git config --file .bare/config core.bare false` |
| Compartilhar `.env` entre worktrees | Conflito de portas, vars erradas | Copiar e ajustar por worktree |
| Continuar com testes falhando no baseline | Confunde regressão nova com velha | Parar e confirmar com o usuário |
| Colocar worktrees de projeto convencional dentro do container de outro | Mistura os git objects | Manter worktrees como irmãos para repos convencionais |
| Criar symlink antes da worktree existir | Link quebrado | Criar worktree primeiro, symlink depois |

---

## Integração

**Frequentemente encadeado de:**

- **brainstorming** — quando o design é aprovado e a implementação segue
- **subagent-driven-development** — antes de tarefas que precisam de árvore limpa
- **executing-plans** — antes de executar etapas do plano em isolamento

**Par natural com:**

- **finishing-a-development-branch** — para merge/PR/limpeza após a branch da worktree estar pronta
