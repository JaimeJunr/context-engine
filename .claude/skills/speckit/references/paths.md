# Speckit path conventions

Paths are relative to **repository root** unless stated otherwise. Speckit works alongside Cursor/IDE rules: constitution and extensions live under `.claude/`; scripts and templates live in the skill pack.

## Project-level (repo)

| Purpose | Path |
|--------|------|
| Constitution | `.claude/memory/constitution.md` |
| Extension hooks | `.claude/speckit-extensions.yml` |
| Template overrides | `.claude/speckit-overrides/templates/<name>.md` |
| Feature specs | `specs/NNN-branch-name/spec.md`, `plan.md`, `tasks.md`, etc. |

**Memory is project-local.** The skill pack must not ship `.claude/memory/`; each project creates it when running `/speckit.constitution` (or the first command that needs the constitution). The skill creates `.claude/memory/` and copies the constitution template when the file is missing.

## Skill pack (this pack)

| Purpose | Path |
|--------|------|
| Scripts | `.claude/skills/speckit/scripts/bash/*.sh` |
| Templates | `.claude/skills/speckit/templates/*.md` |

## Rules integration

- `update-agent-context.sh` (after `/speckit.plan`) updates `.cursor/rules/specify-rules.mdc` (and other agent files) from `plan.md`. Use Cursor rules so the IDE loads project context from the spec.
- Constitution (`.claude/memory/constitution.md`) is the single source of principles; skills and rules should align with it.

## Invoking scripts from repo root

## Migration from .specify

If the repo previously used `.specify/`:

1. Copy `.specify/memory/constitution.md` to `.claude/memory/constitution.md` (create `.claude/memory/` if needed).
2. If you used `.specify/extensions.yml`, copy or rename it to `.claude/speckit-extensions.yml`.
3. You can remove `.specify/` once speckit runs from `.claude/skills/speckit/`. Scripts still resolve templates from `.specify/templates/` as fallback if the skill pack has no templates.

## Invoking scripts from repo root

Always run scripts from the repository root, for example:

```bash
.claude/skills/speckit/scripts/bash/check-prerequisites.sh --json
.claude/skills/speckit/scripts/bash/create-new-feature.sh "Description" --json --short-name "short-name"
.claude/skills/speckit/scripts/bash/setup-plan.sh --json
.claude/skills/speckit/scripts/bash/update-agent-context.sh claude
```
