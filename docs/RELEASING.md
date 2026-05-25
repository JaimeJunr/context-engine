# Processo de Release

## Visão geral

`ctx` segue [Semantic Versioning 2.0](https://semver.org/) e usa Conventional Commits para automatizar bump de versão + geração de CHANGELOG.

| Ferramenta | Função |
|---|---|
| [Conventional Commits](https://www.conventionalcommits.org/) | padrão obrigatório de mensagem de commit |
| [`cocogitto`](https://docs.cocogitto.io) (`cog`) | linter local (lefthook commit-msg) + CI ([commitlint.yml](../.github/workflows/commitlint.yml)) |
| [`git-cliff`](https://git-cliff.org) | gera `CHANGELOG.md` a partir do histórico de commits |
| [`cargo-dist`](https://opensource.axo.dev/cargo-dist/) | build cross-platform + GitHub Release ([release.yml](../.github/workflows/release.yml)) |
| `cargo publish` | publica em [crates.io](https://crates.io) ([publish-crates.yml](../.github/workflows/publish-crates.yml)) |

## Conventional Commits

Cada commit segue:

```
<type>(<scope>): <subject>

[body opcional]

[footer opcional]
```

**Types aceitos** (impacto na versão):

| Type | Versão | Exemplo |
|---|---|---|
| `feat` | minor (0.X.0) | `feat(graph): add ctx_impact MCP tool` |
| `fix` | patch (0.0.X) | `fix(exec): handle empty argv` |
| `feat!` ou `BREAKING CHANGE:` | major (X.0.0) | `feat(api)!: rename SearchResult.chunk_text → text` |
| `perf` / `refactor` / `docs` / `test` / `build` / `ci` / `chore` / `style` | sem bump | — |

**Scopes recomendados**: `map`, `catalog`, `exec`, `graph`, `mcp`, `agents`, `cli`, `docs`, `ci`.

**Validação automática**:
- Pre-commit local: `lefthook` chama `cog verify` em cada commit.
- CI: [commitlint.yml](../.github/workflows/commitlint.yml) roda `cog check` em PRs.

Pular validação (não recomendado): `git commit --no-verify`.

## Fluxo de release

### 1. Decidir versão

Olhe os commits desde a última tag:

```bash
git log $(git describe --tags --abbrev=0)..HEAD --oneline
```

Aplique semver:
- Qualquer `feat!`/`BREAKING CHANGE` → bump **major**
- Pelo menos um `feat` → bump **minor**
- Apenas `fix`/`perf`/outros → bump **patch**

Pré-1.0 toleramos breaking em minor (ex: `0.1.0 → 0.2.0` pode quebrar API).

### 2. Atualizar arquivos

```bash
# Versão em Cargo.toml
sed -i 's/^version = ".*"/version = "0.2.0"/' Cargo.toml
cargo check  # atualiza Cargo.lock

# Regenerar CHANGELOG (move [Unreleased] → [0.2.0])
git cliff --tag v0.2.0 -o CHANGELOG.md
```

Revise o `CHANGELOG.md` — `git-cliff` agrupa por tipo, mas a versão humana fica melhor.

### 3. Commit + tag + push

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore(release): v0.2.0"
git tag v0.2.0
git push origin main v0.2.0
```

### 4. Aguardar CI

Push da tag dispara:
- [`release.yml`](../.github/workflows/release.yml): builda binários para Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64; cria GitHub Release com installers shell/powershell.
- [`publish-crates.yml`](../.github/workflows/publish-crates.yml): roda `cargo publish` (precisa do secret `CARGO_REGISTRY_TOKEN`).

Acompanhe em: `https://github.com/JaimeJunr/ctx-engine/actions`

### 5. Validar release

- GitHub Release deve aparecer em `releases/tag/vX.Y.Z` com:
  - Body = changelog do release
  - Assets = `.tar.gz` por plataforma + `install.sh`/`install.ps1`
- crates.io deve ter `ctx-engine vX.Y.Z`: `cargo search ctx-engine`

## Instalação para usuários

```bash
# Curl (Linux/macOS)
curl -fsSL https://github.com/JaimeJunr/ctx-engine/releases/latest/download/install.sh | sh

# PowerShell (Windows)
irm https://github.com/JaimeJunr/ctx-engine/releases/latest/download/install.ps1 | iex

# Cargo
cargo install ctx-engine
```

## Hotfix

Para corrigir um bug crítico numa versão já publicada:

```bash
git checkout -b hotfix/0.1.1 v0.1.0
# ... fazer o fix ...
git commit -m "fix(scope): descrição"
# bump em Cargo.toml para 0.1.1, regenerar CHANGELOG
git tag v0.1.1
git push origin hotfix/0.1.1 v0.1.1
# abrir PR para main com o hotfix
```

## Setup local (para maintainers)

```bash
cargo install --locked cocogitto    # cog (linter)
cargo install --locked git-cliff    # geração de CHANGELOG
cargo install --locked cargo-dist   # testar build local: `cargo dist build`
lefthook install                    # ativa hooks (cog verify + fmt + clippy + tests)
```

## Anti-patterns

- ❌ Tagger sem atualizar CHANGELOG primeiro
- ❌ Pular `cog verify` rotineiramente — desabilita a automação
- ❌ Bump de major em pré-1.0 sem motivo (use minor com nota no CHANGELOG)
- ❌ Comitar `Cargo.lock` desatualizado após bump (rode `cargo check` antes)
