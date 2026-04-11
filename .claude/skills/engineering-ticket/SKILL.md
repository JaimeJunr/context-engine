---
name: engineering-ticket
description: >-
  Use when the user asks for an engineering ticket, backlog item, Jira description,
  dev handoff from BRD or product spec, technical requirements for implementation,
  critérios de aceitação in ticket form, escopo funcional/não funcional, or any
  product specification that must paste cleanly into trackers without markdown.
  Triggers also include: "escrever ticket", "formatar requisitos", "história de usuário",
  "especificação para o time", "descrição da issue".
metadata:
  short-description: Plain-text engineering tickets (Jira-ready)
---

# Engineering Ticket (plain text)

## Core rule

Deliver the ticket as **plain text only**: no markdown headings, fences, HTML, or rich-text markers—unless the user **explicitly** asks for another format. Trackers and legacy fields often break with `#`, `**`, or code blocks.

## Quick reference

| Topic | Rule |
| ----- | ---- |
| Format | Plain text, UPPERCASE section titles, bullet lines with `•` or `-` consistently |
| Language | Match the user (default PT-BR for section labels in template); body may follow product language |
| User story | `Como … / Eu quero … / Para que …` when the ticket is user-facing |
| Acceptance | Checkbox lines `[ ]` with objective, testable statements |
| Unknowns | Mark `TBD` with what is missing—do not invent facts |

## Workflow

1. **Confirm intent** — Ticket for implementation handoff (not a design doc essay). If inputs are vague, list assumptions or `TBD` in CONTEXTO.
2. **Load scaffold** — Reuse section order and labels from `references/ticket-template.txt`. Omit optional CHECKLIST only if the user says it is not needed.
3. **Fill sections** — Prefer concrete behaviors, data rules, and edge cases over slogans. Tie acceptance criteria to observable outcomes.
4. **Self-check** — Run the checklist in “Output validation” before responding.

## Output validation (before send)

- [ ] Response is plain text (no `#`, no ```, no `**bold**` unless user opted out).
- [ ] All non-optional sections from the template are present or explicitly marked N/A with one line why.
- [ ] CRITÉRIOS DE ACEITAÇÃO are testable (who can verify, what proves done).
- [ ] DEPENDÊNCIAS and IMPACTO NA BASE ATUAL mention regressions/migrations when relevant.

## Red flags — stop and fix

- Emitting markdown “because it looks nicer.”
- Skipping PONTOS DE ATENÇÃO TÉCNICOS or IMPACTO NA BASE ATUAL on backend/data tickets.
- Acceptance criteria that restate the title without measurable conditions.
- Mixing English section titles with a PT-BR ticket without user request—stay consistent.

## Common mistakes

| Mistake | Fix |
| ------- | --- |
| Wall of prose under REQUISITOS FUNCIONAIS | Break into bullets: default behavior, rules, branches, UI/data |
| “Melhorar UX” as acceptance | Replace with screens/flows and expected states |
| Ignoring non-functional | Add performance, security, accessibility, compatibility when applicable |
| Orphan dependencies | Every external team, flag, or ticket referenced—list under DEPENDÊNCIAS |

## Gathering inputs (when the user did not specify)

Answer implicitly or mark `TBD`: problem vs opportunity, primary actor, success metric, systems touched, rollout/migration, linked issues or docs.

## Template source

Full scaffold (copy shape, replace bracketed hints): `references/ticket-template.txt`
