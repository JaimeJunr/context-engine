---
name: update-changelog
description: Generates changelog for a new release version. Use when preparing a release, updating CHANGELOG.md, or when the user asks to add a new version section or generate release notes.
allowed-tools: Edit, Bash(git add:*), Bash(git commit:*)
---

# Update Changelog for Release

You are updating the changelog for a new release. Add a new version section to CHANGELOG.md and commit the change.

## Scope

- **Allowed**: Edit CHANGELOG.md; `git add` and `git commit` (no push).
- **Do not**: Push, create tags, or modify other files unless necessary to create CHANGELOG.md.

## 1. Determine the new version

Use the version the user provided (e.g. `v1.2.0`, `2.0.0`). If not given, infer from context or ask.

## 2. Gather changes since last release

- Read CHANGELOG.md to find the **last release heading** (e.g. `## [1.1.0]` or `## v1.1.0 - 2025-01-15`).
- Review **recent commits** since that release: `git log --oneline` (or since the tag/date of last release).
- If available, consider **merged pull requests** (titles and descriptions) for user-facing wording.

Use conventional commit types and PR titles to categorize: `feat`/feature, `fix`/bugfix, `docs`, `chore`/internal, breaking changes.

## 3. CHANGELOG format

Insert the new section **at the top of the file, directly after the `# Changelog` heading**. Keep existing content below.

Use this structure (include only sections that have items):

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Breaking Changes

- User-focused description of breaking change.

### New Features

- User-focused description of new feature.

### Bug Fixes

- User-focused description of fix.

### Documentation

- User-focused description of doc change.

### Internal / Other

- User-focused description of internal or other change.
```

- Use `## [X.Y.Z]` or `## vX.Y.Z` to match the existing style in the file.
- Add a date line (` - YYYY-MM-DD`) if the rest of the file uses it.
- Write **clear, user-focused** bullet points (what changed and why it matters), not raw commit messages.
- **Omit** any section that has no entries.

If CHANGELOG.md does not exist, create it with a `# Changelog` heading and then the new version section.

## 4. Commit

After updating CHANGELOG.md:

```bash
git add CHANGELOG.md
git commit -m "docs: update changelog for vX.Y.Z"
```

Use the actual new version in the commit message (e.g. `v1.2.0`).

## Checklist

- [ ] New version section is directly under `# Changelog`.
- [ ] Only sections with changes are included.
- [ ] Entries are user-focused and clear.
- [ ] Commit message is `docs: update changelog for v{new_version}`.
