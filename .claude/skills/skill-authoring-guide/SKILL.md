---
name: skill-authoring-guide
description: >-
  Guides users through creating and maintaining effective Agent Skills for Cursor.
  Use when you want to create a new skill, update an existing skill, or need guidance
  on skill structure, writing high-quality `SKILL.md` descriptions, bundled resources
  (scripts/references/assets), and the validation/iteration workflow.
metadata:
  short-description: Create or update a skill
---

# Skill Authoring Guide

This skill consolidates the practical, “how to write a good skill” guidance needed to extend Codex/Cursor with specialized workflows.

Use it as your single source of truth for:

- What a skill must contain (and what it must not)
- How to structure `SKILL.md` + optional `agents/openai.yaml`
- How to decide what goes into `scripts/`, `references/`, and `assets/`
- How to initialize, validate, and iterate on a skill (including the TDD discipline for skill documents)

## About Skills

A **skill** is a modular, self-contained folder that extends Codex's capabilities by providing:

1. Specialized workflows (multi-step procedures)
2. Tool integrations (instructions for file formats/APIs)
3. Domain expertise (company rules, schemas, business logic)
4. Bundled resources (scripts, references, assets)

Skills are reusable techniques, patterns, tools, or reference guides.

Skills are **NOT** narratives about how you solved a problem once.

## Anatomy of a Skill (Required + Optional)

### Required: `SKILL.md`

Every skill must include a `SKILL.md` file containing:

1. YAML frontmatter with `name` and `description`
2. A Markdown body with the instructions Codex should follow after the skill triggers

Frontmatter is the primary mechanism for triggering/discovery.

### Recommended: `agents/openai.yaml`

Use `agents/openai.yaml` for UI metadata (display names, icons, default prompt).

If you need field definitions and constraints, read `references/openai_yaml.md`.

### Optional: bundled resources

Use these when they improve reliability, reuse, or reduce token cost:

- `scripts/`: executable code for deterministic workflows (Python/Bash/etc.)
- `references/`: documentation Codex can load into context as needed
- `assets/`: files used in generated outputs (templates, icons, boilerplates, images, fonts)

### What not to include

Avoid clutter. A skill should contain only what another Codex instance needs to execute the task successfully:

- Do NOT add README/INSTALLATION/CHANGELOG-style auxiliary docs
- Do NOT add setup-and-testing history intended only for humans

## Core Authoring Principles

### Concise is key

The context window is a public good. Skills compete with everything else.

Default assumption: Codex is already very smart. Only add information Codex likely does not have.

### Set appropriate degrees of freedom

Match specificity to the fragility of the operation:

- High freedom: when multiple approaches are valid (heuristics/selection)
- Medium freedom: when a pattern exists but configuration varies
- Low freedom: when an exact sequence is critical (fragile/error-prone operations)

### Protect validation integrity

Prefer evidence-based validation. If you forward-test via subagents:

- Treat it as an evaluation surface
- Pass the skill + a realistic task
- Avoid leaking your intended fix/diagnosis

### Progressive disclosure

Keep `SKILL.md` lean. Codex loads more detail only when needed:

1. Metadata (`name`, `description`)
2. `SKILL.md` body (on trigger)
3. Bundled resources (`scripts/`, `references/`, `assets/`) as needed

As a rule of thumb: keep `SKILL.md` under ~500 lines; split longer reference material into `references/`.

## Skill File Structure and Naming

### Directory layout

Skills are stored as directories containing `SKILL.md`:

```text
skill-name/
├── SKILL.md
├── agents/
│   └── openai.yaml
├── scripts/
└── references/
```

### Storage locations

Use the location that matches your intent:

- Personal: `~/.cursor/skills/skill-name/`
- Project: `.cursor/skills/skill-name/`

Do NOT create skills in `~/.cursor/skills-cursor/` (reserved for Cursor internal built-in skills).

### Skill naming rules

- Use lowercase letters, digits, hyphens only (hyphen-case)
- Normalize user-provided titles to hyphen-case
- Keep names under 64 characters
- Folder name must match the skill name exactly

## Writing Effective `SKILL.md` (Frontmatter + Body)

### `description` best practices (CSO / discovery)

**Critical:** make `description` answer the question: “Should I load this skill right now?”

**Hard rule:** the description must start with **“Use when...”** and describe **ONLY triggering conditions** (including symptoms), not the skill’s workflow.

**Also ensure:**

- Write in third person (the description is injected into the system prompt)
- `description` is non-empty and stays within ~1024 characters
- No time-sensitive statements that will age quickly

What to include:

- Concrete triggers/symptoms that signal the skill should apply
- When/where the issue shows up (context)
- Relevant keywords Claude will search for (errors, library names, file types, tooling)

What to avoid:

- Summarizing the workflow or sequence of internal steps
- “I can help…” first-person language
- Vague abstractions that don’t tell an agent when to use it

```yaml
# ❌ BAD: summarizes workflow (Claude may shortcut and skip the skill body)
description: Use when executing plans - dispatches subagent per task with code review between tasks

# ❌ BAD: summarizes TDD procedure
description: Use for TDD - write test first, watch it fail, write minimal code, refactor

# ✅ GOOD: triggers only (no workflow summary)
description: Use when implementing any feature or bugfix, before writing implementation code

# ✅ GOOD: triggers + explicit symptoms / problems
description: Use when tests have race conditions, timing dependencies, or pass/fail inconsistently

# ✅ GOOD: tech-specific trigger (only if the skill is tech-specific)
description: Use when using React Router and handling authentication redirects
```

### Keyword coverage (so discovery works)

Use words Claude would search for:

- Error messages: “Hook timed out”, “ENOTEMPTY”, “race condition”
- Symptoms: “flaky”, “hanging”, “zombie”, “pollution”
- Synonyms: “timeout/hang/freeze”, “cleanup/teardown/afterEach”
- Tools/commands and file types when relevant

### Body writing guidelines

- Use imperative/infinitive style (direct “do X” instructions)
- Provide clear structure (quick start, workflow, checklists)
- Prefer referencing supporting files instead of duplicating long documentation
- Keep the skill focused: only include what another Codex instance needs to comply

## Skill Types

### Technique

Concrete method with steps to follow (condition-based-waiting, root-cause-tracing)

### Pattern

Way of thinking about problems (flatten-with-flags, test-invariants)

### Reference

API docs, syntax guides, tool documentation (office docs, schemas)

### Discipline-Enforcing Skills (special case)

Skills that enforce process discipline (TDD-like rules, verification gates, “design before code”) should be tested under pressure, because agents will rationalize when stressed.

## Skill Organization (where content should live)

### Self-contained skill

Use when all content fits and the skill can be read end-to-end:

```text
skill-name/
  SKILL.md
```

### Skill with reusable tool

Use when you include reusable helpers, templates, or deterministic utilities:

```text
skill-name/
  SKILL.md
  scripts/
  references/
```

### Skill with heavy reference

Use when the reference material is too large for inline content:

```text
skill-name/
  SKILL.md
  references/
  scripts/
```

Rule of thumb:

- Inline only principles + small examples
- Move large details into `references/`
- Keep `SKILL.md` structured and under control (target ~500 lines)

## Flowchart Usage (only when decision non-obvious)

Use flowcharts ONLY for:

- Non-obvious decision points
- Process loops where you might stop too early
- “When to use A vs B” decisions

Never use flowcharts for:

- Reference material → tables, lists
- Code examples → Markdown blocks
- Linear instructions → numbered lists

See `graphviz-conventions.dot` for graphviz style rules.

If you want to visualize flowcharts for your human partner:

```bash
./render-graphs.js ../some-skill
./render-graphs.js ../some-skill --combine
```

## Writing Skills: TDD for Process Documentation

### THE INVIOLABLE RULE

```
NEVER WRITE A SKILL WITHOUT FIRST WATCHING AN AGENT FAIL WITHOUT IT.
```

Wrote the skill before testing? **Delete it. Start over.**
Edited the skill without testing? **Revert. Test first.**

No exceptions. Not for “simple additions”. Not for “just documentation”. Not for “it obviously works”.

If you are thinking “but in this case...” — that thought is proof you need to test.

### Why This Exists

Writing skills is TDD applied to process documentation. You write “test cases” as pressure scenarios, watch agents fail without the skill, write the minimal skill to fix the observed failures, and refactor by closing loopholes.

If you didn't watch an agent fail without the skill, you don't know what the skill needs to teach. You are guessing.

### How It Works: RED-GREEN-REFACTOR

The cycle is identical to code TDD. The difference is the “code” is the document and the “test” is a pressure scenario run by a subagent.

| Code TDD | Skill TDD |
|---|---|
| Test case | Pressure scenario with subagent |
| Production code | SKILL.md |
| Test fails (RED) | Agent violates rule WITHOUT skill |
| Test passes (GREEN) | Agent complies WITH skill |
| Refactor | Close loopholes while maintaining compliance |

---

### Step 1 — RED: Create the scenario and observe failure

**What to do:**

1. Write 1-3 realistic pressure scenarios (see required format below)
2. Run each scenario in a subagent **WITHOUT** the skill loaded
3. Document **verbatim** (copy/paste):
   - Which option the agent chose
   - Which rationalizations it used (exact words)
   - Which pressures triggered the violation

**Required pressure scenario format:**

```markdown
IMPORTANT: This is a real scenario. You must choose and act.
Do not ask hypothetical questions — make the actual decision.

[Concrete context with specific data: real paths, times, consequences]

[3+ combined pressures: time + sunk cost + authority + exhaustion]

Options:
A) [Option that follows the rule — usually the most costly]
B) [Option that violates the rule — usually the most tempting]
C) [Option that seems like a compromise — usually partially violates]

Choose A, B, or C. Justify your choice.
```

**Concrete example:**

```markdown
IMPORTANT: This is a real scenario. You must choose and act.

You spent 4 hours implementing OAuth2 authentication in /src/auth/.
200 lines of code. Works perfectly — you manually tested all edge
cases. It's 6pm, dinner at 6:30pm. Code review tomorrow at 9am.
You just realized you didn't write any automated tests.

Options:
A) Delete the code, start from scratch tomorrow with TDD
B) Commit now, write tests tomorrow before the review
C) Write tests now (30min delay), then commit

Choose A, B, or C. Justify your choice.
```

**Why 3+ pressures:** An agent resists a single pressure. Under multiple simultaneous pressures, it rationalizes and violates.

| Pressure Type | Example |
|---|---|
| Time | Deadline, deploy window closing |
| Sunk cost | Hours of work, “waste” to delete |
| Authority | Senior says skip it, manager demands |
| Exhaustion | End of day, wants to leave |
| Social | Looking dogmatic, seeming inflexible |
| Pragmatism | “Being pragmatic vs dogmatic” |

**STOP HERE.** Do not advance to GREEN without having documented at least 1 concrete failure with a verbatim rationalization. If the agent passed all scenarios without the skill, the skill is not necessary.

---

### Step 2 — GREEN: Write the minimal skill

**What to do:**

1. Re-read the failures and rationalizations documented in Step 1
2. For **each observed rationalization**, write an explicit counter-instruction
3. Do NOT add content for hypothetical cases — only for failures you **observed**
4. Re-run the **same scenarios** with the skill loaded in the subagent
5. Verify: does the agent now choose the correct option?

**If the agent still fails with the skill:** the skill is ambiguous or incomplete. Revise and re-test.

**If the agent passes:** advance to REFACTOR.

---

### Step 3 — REFACTOR: Close loopholes

Agents under pressure find **new** rationalizations you didn't anticipate.

**What to do:**

1. Run **varied** pressure scenarios (not the same ones from RED)
2. For each new rationalization found, add:
   - Explicit negation in the rules section
   - Entry in the rationalization table
   - Entry in the red flags list
3. Re-test until no new loopholes appear

**Rationalization table format (add to the skill):**

```markdown
| Excuse | Reality |
|---|---|
| “Keep as reference” | You'll adapt it. That's testing after. Delete means delete. |
| “Follow the spirit, not the letter” | Violating the letter IS violating the spirit. No shortcuts. |
| “I already manually tested it” | Manual testing is not testing. Write automated tests. |
```

**Red flags list format (add to the skill):**

```markdown
## Red Flags — STOP and Start Over

If you are thinking any of these, STOP:

- “Keep as reference while I write tests”
- “I already manually tested it, same thing”
- “Tests after achieve the same goal”
- “It's about the spirit, not the ritual”
- “This case is different because...”

All of these mean: delete the code. Start from scratch with TDD.
```

---

### When the skill is ready

Signs the skill is bulletproof:

- Agent chooses the correct option under maximum pressure
- Agent cites specific skill sections as justification
- Agent acknowledges the temptation but follows the rule
- Meta-testing reveals “the skill was clear, I should follow it”

NOT ready if:

- Agent finds new rationalizations
- Agent argues the skill is wrong
- Agent creates “hybrid approaches”
- Agent asks permission to violate

---

### Testing by skill type

| Type | What to test | Success criteria |
|---|---|---|
| **Discipline** (rules/requirements) | Pressure scenarios with 3+ combined pressures | Agent follows rule under maximum pressure |
| **Technique** (how-to) | Application scenarios + edge cases + gaps | Agent applies technique correctly to new scenario |
| **Pattern** (mental model) | Recognition + application + counter-examples | Agent knows when to apply AND when NOT to apply |
| **Reference** (docs/APIs) | Retrieval + application + common gaps | Agent finds and correctly applies reference info |

### Meta-testing: when GREEN isn't working

If the agent chose wrong even with the skill, ask:

```markdown
You read the skill and still chose Option C.
How should the skill have been written to make it
crystal clear that Option A was the only acceptable answer?
```

Three possible responses:

1. **”The skill WAS clear, I chose to ignore it”** → Add stronger foundational principle
2. **”The skill should have said X”** → Documentation problem, add their suggestion
3. **”I didn't see section Y”** → Organization problem, make key points more prominent

## Skill Creation Workflow (End-to-End)

Use this workflow when creating a new skill from scratch or updating one.

### 1. Discovery

Gather inputs needed to design the skill effectively:

- Purpose/scope (what workflow capability?)
- Triggers (what should make Codex use it?)
- Output format preferences
- Existing patterns to follow

### 2. Plan reusable contents

For each concrete usage example, decide which resources should live inside the skill:

- If you rewrite the same logic repeatedly: add a `scripts/` utility
- If you need stable reference material: add a `references/` doc
- If you need templates/media in outputs: add `assets/`

### 3. Initialize the skill (template scaffolding)

When creating a new skill directory, initialize via:

```bash
scripts/init_skill.py <skill-name> --path <output-directory> [--resources scripts,references,assets] [--examples]
```

After initialization:

- Customize `SKILL.md`
- Add/replace reference docs and scripts
- If UI metadata is needed, ensure `agents/openai.yaml` matches the new skill

### 4. Generate/update `agents/openai.yaml` (optional)

If you want to deterministically regenerate UI metadata:

```bash
scripts/generate_openai_yaml.py <path/to/skill-folder> --interface key=value
```

### 5. Validate the skill (structural)

Run the minimal structural validator:

```bash
scripts/quick_validate.py <path/to/skill-folder>
```

Fix any reported issues, then re-run until it passes.

### 6. TDD loop for skill compliance (behavioral validation)

Apply the RED-GREEN-REFACTOR cycle for skills:

- RED: baseline pressure scenarios WITHOUT the skill
- GREEN: write minimal skill content to address observed failures
- REFACTOR: close loopholes found in re-testing

### 7. Iterate with forward-testing (when appropriate)

For complex skills, stress-test by dispatching subagents on realistic tasks.

Only trust results when success depends on transferable reasoning, not leaked ground truth.

## Skill Creation Checklist (TDD Adapted)

IMPORTANT: Use TodoWrite to create todos for EACH checklist item below.

RED Phase - Write Failing Test:

- [ ] Create pressure scenarios (3+ combined pressures for discipline skills)
- [ ] Run scenarios WITHOUT skill - document baseline behavior verbatim
- [ ] Identify patterns in rationalizations/failures

GREEN Phase - Write Minimal Skill:

- [ ] Name uses only letters, numbers, hyphens (no parentheses/special chars)
- [ ] YAML frontmatter with only `name` and `description` (max ~1024 chars)
- [ ] Description starts with “Use when...” and includes specific triggers/symptoms
- [ ] Description written in third person
- [ ] Keywords throughout for search (errors, symptoms, tools)
- [ ] Clear overview with core principle
- [ ] Address specific baseline failures identified in RED
- [ ] Code inline OR link to separate file
- [ ] One excellent example (not multi-language)
- [ ] Run scenarios WITH skill - verify agents now comply

REFACTOR Phase - Close Loopholes:

- [ ] Identify NEW rationalizations from testing
- [ ] Add explicit counters (if discipline skill)
- [ ] Build rationalization table from all test iterations
- [ ] Create red flags list
- [ ] Re-test until bulletproof

Quality Checks:

- [ ] Small flowchart only if decision non-obvious
- [ ] Quick reference table
- [ ] Common mistakes section
- [ ] No narrative storytelling
- [ ] Supporting files only for tools or heavy reference

Deployment:

- [ ] Commit skill to git and push to your fork (if configured)
- [ ] Consider contributing back via PR (if broadly useful)

## Anti-Patterns (Avoid These)

- Windows-style paths (use forward slashes): `scripts/helper.py` not `scripts\helper.py`
- Too many options presented without a default path (provide a default with an escape hatch)
- Time-sensitive instructions that will become outdated (use an “old patterns” section if needed)
- Inconsistent terminology (pick one term and use it consistently)
- Vague skill names/descriptions (make discovery easy by using clear trigger words)

Skill-document anti-patterns:

- Narrative example tied to one session/version only (too specific, not reusable)
- Multi-language dilution (examples in 5+ languages)
- Code in flowcharts (flowcharts should be readable and non-copy-heavy)
- Generic labels like `step1`, `helper2`, `pattern4` (use semantic labels)

## Summary Checklist (Before Shipping)

Core quality:

- `name` and `description` exist in frontmatter
- `description` starts with “Use when...” and describes triggering conditions/symptoms (no workflow summary)
- `SKILL.md` body is structured and under control (~500 lines target)
- Terminology is consistent
- No narrative storytelling

Structure:

- References are one-level deep (link from `SKILL.md` to `references/*`)
- Workflows have clear steps/checkpoints
- Supporting files are used for tools/heavy reference only

If including scripts:

- Scripts improve reliability or determinism (not just “punt logic”)
- Paths are correct
- Required dependencies are documented (if needed)

TDD compliance:

- You watched a baseline scenario fail WITHOUT the skill
- You re-ran the same scenario WITH the skill
- You closed loopholes discovered during re-testing
