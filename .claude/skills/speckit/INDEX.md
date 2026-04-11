## Índice da documentação do Speckit

### 🚀 Quick Start

- **[Guia rápido (README)](README.md)**  
  Como rodar os comandos principais em 5 minutos, fluxo mínimo para testar e fluxo recomendado completo.

---

### 📋 Skills Speckit

- **[`/speckit.specify`](speckit.specify/SKILL.md)**  
  Cria branch, pasta de feature e `spec.md` a partir de uma descrição em linguagem natural.

- **[`/speckit.clarify`](speckit.clarify/SKILL.md)**  
  Faz até 5 perguntas críticas sobre a spec e grava as respostas nela.

- **[`/speckit.plan`](speckit.plan/SKILL.md)**  
  Gera o plano técnico (`plan.md`) e artefatos de design (`research.md`, `data-model.md`, `contracts/`, `quickstart.md`).

- **[`/speckit.checklist`](speckit.checklist/SKILL.md)**  
  Gera checklists de qualidade de requisitos (unit tests de inglês) em `checklists/*.md`.

- **[`/speckit.tasks`](speckit.tasks/SKILL.md)**  
  Constrói `tasks.md` organizado por fases e user stories.

- **[`/speckit.analyze`](speckit.analyze/SKILL.md)**  
  Analisa consistência entre `spec.md`, `plan.md`, `tasks.md` e constituição (read‑only).

- **[`/speckit.implement`](speckit.implement/SKILL.md)**  
  Executa o plano de implementação seguindo `tasks.md`, marcando tarefas concluídas.

- **[`/speckit.taskstoissues`](speckit.taskstoissues/SKILL.md)**  
  Converte tasks em issues no GitHub, respeitando o remote atual.

---

### 🧰 Scripts Bash

- **[`scripts/bash/create-new-feature.sh`](scripts/bash/create-new-feature.sh)**  
  Criado por `/speckit.specify` para abrir nova feature (branch + diretórios + spec).

- **[`scripts/bash/setup-plan.sh`](scripts/bash/setup-plan.sh)**  
  Usado por `/speckit.plan` para resolver paths e preparar `plan.md`.

- **[`scripts/bash/update-agent-context.sh`](scripts/bash/update-agent-context.sh)**  
  Atualiza contexto dos agentes com novas tecnologias/decisões da feature.

- **[`scripts/bash/check-prerequisites.sh`](scripts/bash/check-prerequisites.sh)**  
  Checa pré‑requisitos e descobre `FEATURE_DIR`, `spec.md`, `plan.md`, `tasks.md` etc.

---

### 📑 Templates

- **[`templates/spec-template.md`](templates/spec-template.md)** – Estrutura padrão de `spec.md`.  
- **[`templates/plan-template.md`](templates/plan-template.md)** – Estrutura de `plan.md`.  
- **[`templates/tasks-template.md`](templates/tasks-template.md)** – Estrutura de `tasks.md`.  
- **[`templates/checklist-template.md`](templates/checklist-template.md)** – Estrutura de checklists.  
- **[`templates/constitution-template.md`](templates/constitution-template.md)** – Base para constituição de projeto.  
- **[`templates/agent-file-template.md`](templates/agent-file-template.md)** – Modelo de arquivos de agente.

---

### 🎯 Por perfil

#### 👨‍💻 Dev novo na IVT

1. [Guia rápido (README)](README.md)  
2. [Skills Speckit (esta página)](#-skills-speckit)  
3. Ler rapidamente `speckit.specify/SKILL.md` e `speckit.plan/SKILL.md`  
4. Criar uma feature de teste com `/speckit.specify` + `/speckit.plan` + `/speckit.tasks`

#### 👨‍💻 Dev experiente na IVT

1. [Guia rápido (README)](README.md)  
2. Olhar a tabela de Skills acima para lembrar o nome de cada comando  
3. Focar em `/speckit.tasks`, `/speckit.analyze` e `/speckit.implement` para acelerar execução  
4. Usar `/speckit.checklist` em features de risco (segurança, performance, integrações críticas)

---

### 🔗 Links úteis

- **Constituição do projeto**: `.claude/memory/constitution.md`  
- **Referência de paths da feature**: [`references/paths.md`](references/paths.md)
