---
name: brand-guidelines-ivt
description: Applies IVT design tokens (colors and typography) from ivt-lib to artifacts that should match the IVT look-and-feel. Use when brand colors, style guidelines, visual formatting, or IVT design standards apply. Canonical source: ivt-lib/src/globals.css.
license: Complete terms in LICENSE.txt (if present)
---

# IVT Brand Styling (ivt-lib)

## Overview

This skill applies the **IVT design system** defined in **ivt-lib** (`ivt-lib/src/globals.css`) to any artifact that should match IVT’s visual identity.

**Keywords**: branding, IVT, ivt-lib, design tokens, visual identity, brand colors, typography, corporate identity, visual formatting, visual design

**Canonical source**: `ivt-lib/src/globals.css` — CSS variables and `@theme` there are the single source of truth. When in doubt, read that file.

---

## Brand Guidelines

### Colors (light theme — `:root`)

**Base:**

| Token             | Hex       | Use                                  |
| ----------------- | --------- | ------------------------------------ |
| background        | `#FFFFFF` | Page/card backgrounds                |
| foreground        | `#18181B` | Primary text                         |
| accent            | `#ECFEFF` | Accent backgrounds (e.g. highlights) |
| accent-foreground | `#18181B` | Text on accent                       |

**Primary (brand):**

| Token              | Hex       | Use                                 |
| ------------------ | --------- | ----------------------------------- |
| primary            | `#3B8091` | Main brand (buttons, links, key UI) |
| primary-foreground | `#FAFAFA` | Text on primary                     |

**Neutrals / UI:**

| Token                | Hex       | Use                          |
| -------------------- | --------- | ---------------------------- |
| secondary            | `#F4F4F5` | Secondary surfaces           |
| secondary-foreground | `#18181B` | Text on secondary            |
| muted                | `#F4F4F5` | Muted backgrounds            |
| muted-foreground     | `#71717A` | Secondary text, placeholders |
| border               | `#E4E4E7` | Borders                      |
| ring                 | `#3F3F46` | Focus rings                  |
| input                | `#E4E4E7` | Input borders/backgrounds    |

**Semantic:**

| Token                  | Hex       | Use                    |
| ---------------------- | --------- | ---------------------- |
| destructive            | `#DC2626` | Errors, danger, remove |
| destructive-foreground | `#FEF2F2` | Text on destructive    |
| positive               | `#16A34A` | Success, confirm       |
| positive-foreground    | `#F0FDF4` | Text on positive       |
| warning                | `#EA580C` | Warnings               |
| warning-foreground     | `#FFF7ED` | Text on warning        |
| info                   | `#2563EB` | Informational          |
| info-foreground        | `#EFF6FF` | Text on info           |

**Content hierarchy:**

| Token          | Hex       | Use                            |
| -------------- | --------- | ------------------------------ |
| content-high   | `#0F172A` | Headings, emphasis             |
| content-medium | `#334155` | Subheadings, secondary content |
| body           | `#f1f5f9` | Page background (optional)     |
| soft           | `#FAFAFA` | Soft surfaces                  |

**Charts (light):**

- chart-1: `#2a9d90`
- chart-2: `#E76E50`
- chart-3: `#274754`
- chart-4: `#e8c468`
- chart-5: `#f4a462`

**Dark theme** is defined under `:root.dark` in `ivt-lib/src/globals.css`. Use those variables when targeting dark mode.

---

### Typography

- **Body / UI**: **Lato**, sans-serif (`--font-lato: "Lato", sans-serif` in ivt-lib).
- Prefer system sans-serif fallback (e.g. Arial, system-ui) when Lato is unavailable.
- Use **content-high** for headings and **content-medium** or **muted-foreground** for secondary text to keep hierarchy consistent with ivt-lib.

---

## Features

### Smart font application

- Use Lato for body and UI text (and headings when appropriate).
- Fallback to system sans-serif if Lato is not loaded.
- Preserve hierarchy with content-high / content-medium / muted-foreground.

### Color application

- Prefer CSS variables from ivt-lib (`var(--primary)`, `var(--foreground)`, etc.) in HTML/CSS/React.
- For other contexts (e.g. design tools, RGB), use the hex values above; they match `ivt-lib/src/globals.css` comments.
- Primary brand color for key actions and identity: **#3B8091** (primary).

### Semantic and charts

- Use **primary** for main CTAs and brand moments.
- Use **destructive**, **positive**, **warning**, **info** for feedback and states.
- Use **chart-1** … **chart-5** for data viz so charts stay on-brand.

---

## Technical details

### Where tokens live

- **ivt-lib**: `src/globals.css` — `:root` and `:root.dark` CSS custom properties, plus `@theme inline` for Tailwind.
- Components in `ivt-lib/src` use these tokens (e.g. `text-foreground`, `bg-primary`, `font-lato`). Align any new artifact with the same tokens and naming.

### Using in artifacts

- **Web/React**: Import or mirror the variable names from `globals.css`; use Tailwind theme colors where the app uses Tailwind.
- **Other media**: Map the hex values from this skill (or from `globals.css` comments) into the target format (e.g. RGB for PDFs or design tools).

---

## Summary

| Role           | Token / value          |
| -------------- | ---------------------- |
| Brand primary  | `#3B8091` (primary)    |
| Main text      | `#18181B` (foreground) |
| Background     | `#FFFFFF` (background) |
| Accent (light) | `#ECFEFF` (accent)     |
| Body font      | Lato, sans-serif       |

Always prefer **ivt-lib** tokens and **ivt-lib/src/globals.css** as the single source of truth for IVT brand styling.
