---
name: skill-extract
description: Analyze local git history to extract coding patterns and generate SKILL.md files. Local version of the Skill Creator GitHub App.
disable-model-invocation: true
---

# /skill-extract - Extract Patterns from Git History

Analyze your repository's git history to extract coding patterns and generate SKILL.md files that teach Claude your team's practices.

## Usage

```bash
/skill-create                    # Analyze current repo
/skill-create --commits 100      # Analyze last 100 commits
/skill-create --output ./skills  # Custom output directory
```

## What It Does

1. **Parses Git History** - Analyzes commits, file changes, and patterns
2. **Detects Patterns** - Identifies recurring workflows and conventions
3. **Generates SKILL.md** - Creates valid Claude Code skill files

## Analysis Steps

### Step 1: Gather Git Data

```bash
# Get recent commits with file changes
git log --oneline -n ${COMMITS:-200} --name-only --pretty=format:"%H|%s|%ad" --date=short

# Get commit frequency by file
git log --oneline -n 200 --name-only | grep -v "^$" | grep -v "^[a-f0-9]" | sort | uniq -c | sort -rn | head -20

# Get commit message patterns
git log --oneline -n 200 | cut -d' ' -f2- | head -50
```

### Step 2: Detect Patterns

Look for these pattern types:

| Pattern | Detection Method |
|---------|-----------------|
| **Commit conventions** | Regex on commit messages (feat:, fix:, chore:) |
| **File co-changes** | Files that always change together |
| **Workflow sequences** | Repeated file change patterns |
| **Architecture** | Folder structure and naming conventions |
| **Testing patterns** | Test file locations, naming, coverage |

### Step 3: Generate SKILL.md

Output format:

```markdown
---
name: {repo-name}-patterns
description: Coding patterns extracted from {repo-name}
version: 1.0.0
source: local-git-analysis
analyzed_commits: {count}
---

# {Repo Name} Patterns

## Commit Conventions
{detected commit message patterns}

## Code Architecture
{detected folder structure and organization}

## Workflows
{detected repeating file change patterns}

## Testing Patterns
{detected test conventions}
```

## Example Output

Running `/skill-create` on a TypeScript project might produce:

```markdown
---
name: my-app-patterns
description: Coding patterns from my-app repository
version: 1.0.0
source: local-git-analysis
analyzed_commits: 150
---

# My App Patterns

## Commit Conventions

This project uses **conventional commits**:
- `feat:` - New features
- `fix:` - Bug fixes
- `chore:` - Maintenance tasks
- `docs:` - Documentation updates

## Code Architecture

```
src/
├── components/     # React components (PascalCase.tsx)
├── hooks/          # Custom hooks (use*.ts)
├── utils/          # Utility functions
├── types/          # TypeScript type definitions
└── services/       # API and external services
```

## Workflows

### Adding a New Component
1. Create `src/components/ComponentName.tsx`
2. Add tests in `src/components/__tests__/ComponentName.test.tsx`
3. Export from `src/components/index.ts`

## Testing Patterns

- Test files: `__tests__/` directories or `.test.ts` suffix
- Coverage target: 80%+
```

## Arguments

$ARGUMENTS:
- `--commits N` (optional, default 200)
- `--output <dir>` (optional, default current skills dir)
