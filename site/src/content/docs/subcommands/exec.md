---
title: ctx exec
description: Universal command proxy with smart compression. 17 command families covered, 60-90% token reduction.
---

`ctx exec <command>` runs any shell command and applies a context-aware filter to compress the output before returning it. Commands without a registered filter pass through unchanged.

## Quick examples

```bash
ctx exec git status              # only modified files, no boilerplate
ctx exec cargo test              # only failures + summary preserved
ctx exec kubectl logs my-pod     # deduplicates repeated lines with (×N) grouping
ctx exec terraform plan          # strips refresh/lock noise, keeps diff + summary
ctx exec git push origin main    # reduces to "ok abc1234..def5678 main -> main"
```

## Token savings report

```bash
ctx exec report
ctx exec report --days 7         # last week
ctx exec report --project /path  # filter by project
ctx exec report --json           # machine-readable
```

## Supported command families (17)

| Category | Commands |
|---|---|
| **VCS** | `git status/log/diff/show/branch/tag/stash/blame/push/pull/add/commit/fetch` |
| **Rust** | `cargo test/build/check/clippy/fmt/run/install` |
| **JS/TS** | `npm/yarn/pnpm` (install/test/build/run), `jest`, `vitest` |
| **Linters JS/TS** | `tsc`, `eslint`, `prettier`, `biome` |
| **Python** | `ruff`, `mypy`, `pytest` |
| **Go** | `go test`, `go build`, `go vet`, `golangci-lint` |
| **Ruby** | `rubocop`, `rspec`, `rake` |
| **JVM** | `gradle/gradlew/mvn/mvnw` (test/build/package/install/verify), `grails` |
| **Containers** | `docker ps/images/logs/compose`, `kubectl get/logs/describe` |
| **AWS** | `aws logs/sts/s3/ec2/lambda/iam/dynamodb/cloudformation` |
| **IaC** | `terraform plan/apply/init/validate`, `tofu plan/apply/init/validate` |
| **GitHub** | `gh pr/issue/run` |
| **Filesystem** | `ls`, `find`, `tree`, `grep/rg/ag` |
| **Data** | `curl/wget`, `jq`, `sqlite3` |

## Smart filters

- **Log deduplication** — `kubectl logs`, `docker logs`, `aws logs` normalize timestamps/UUIDs/hex/paths and group identical messages with `(×N)` suffix
- **Git ok-style** — `git push/pull/commit/add` reduce to `ok abc1234 main: feat: msg` (-92% as in RTK)
- **AWS service-specific** — `dynamodb` unwraps `{"S":"value"}` → `"value"`; `iam` strips inline `PolicyDocument` blocks
- **Terraform** — `plan` keeps diff + final `Plan: N to add` summary, strips refresh/install noise
- **Linters formatted** — preserves `file:line:col: error/warning code msg` removing tables and footer noise

## Filter not registered?

The command passes through unchanged. No surprise behavior. To request a new filter, open an issue at [github.com/JaimeJunr/context-engine](https://github.com/JaimeJunr/context-engine).

## Via MCP

`ctx exec` is also available as the `ctx_exec` MCP tool — agents can call it directly with `{"command": ["git", "status"]}` JSON arguments.
