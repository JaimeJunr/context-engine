## Speckit – guia rápido

O **Speckit** é um conjunto de skills/comandos que organizam o ciclo completo de uma feature:

- **Especificar** o que será feito (`/speckit.specify`)
- **Esclarecer** pontos ambíguos da spec (`/speckit.clarify`)
- **Planejar** a implementação (`/speckit.plan`)
- **Criar checklists** de qualidade de requisitos (`/speckit.checklist`)
- **Gerar tasks** de implementação (`/speckit.tasks`)
- **Analisar** consistência entre spec/plan/tasks (`/speckit.analyze`)
- **Implementar** seguindo o plano e as tasks (`/speckit.implement`)
- **Converter tasks em issues** de GitHub (`/speckit.taskstoissues`)

Ele usa scripts em `.claude/skills/speckit/scripts/bash` e a constituição do projeto em `.claude/memory/constitution.md` para manter tudo padronizado.

---

## Quick Start – fluxo mínimo para testar

Se você só quer **testar o Speckit** ou usar no dia a dia com o mínimo de atrito:

1. **Criar a feature**
   - No chat do Cursor:  
     `/speckit.specify Quero adicionar autenticação com login e senha`
2. **(Opcional) Clarificar a spec**
   - Se a spec ficou meio vaga:  
     `/speckit.clarify`
3. **Planejar rapidamente**
   - Criar o plano técnico básico e artefatos de design:  
     `/speckit.plan`
4. **Gerar tasks executáveis**
   - Quebrar em tarefas concretas, por user story:  
     `/speckit.tasks`
5. **Implementar seguindo as tasks**
   - Ir marcando `[X]` direto no `tasks.md`:  
     `/speckit.implement`

Quando estiver confortável, adicione os passos “premium”:

- **Checklists de requisitos**: `/speckit.checklist`
- **Análise de consistência**: `/speckit.analyze`
- **Issues no GitHub**: `/speckit.taskstoissues`

---

## Como usar na prática (detalhado)

### 1. Criar uma feature nova – `/speckit.specify`

- **Quando usar**: você tem uma ideia/feature em linguagem natural.
- **Como chamar** (no chat do Cursor):
  - `/speckit.specify Quero adicionar autenticação com login e senha…`
- **O que ele faz**:
  - Cria um **branch** de feature com nome curto.
  - Cria uma pasta de feature (ex.: `features/123-user-auth/`).
  - Gera `spec.md` usando `templates/spec-template.md`.
  - Cria um checklist de qualidade da spec em `checklists/requirements.md`.

Depois de rodar esse comando, continue sempre dentro do branch criado por ele.

### 2. Esclarecer ambiguidades da spec – `/speckit.clarify`

- **Quando usar**: após ter uma `spec.md` inicial, mas com pontos ainda vagos.
- **O que ele faz**:
  - Lê `spec.md`.
  - Gera até **5 perguntas** super focadas para reduzir ambiguidade.
  - Escreve as respostas de volta na própria `spec.md` (seção `Clarifications` + ajustes locais).
- **Regra**: idealmente rodar **antes** de `/speckit.plan`.

### 3. Planejar a implementação – `/speckit.plan`

- **Quando usar**: spec já razoavelmente clara, pronto para desenhar solução técnica.
- **O que ele faz** (via `scripts/bash/setup-plan.sh`):
  - Descobre `FEATURE_SPEC`, `IMPL_PLAN`, `SPECS_DIR`, `BRANCH`.
  - Lê `spec.md` + `.claude/memory/constitution.md`.
  - Usa `templates/plan-template.md` como base para escrever o plano (`plan.md`).
  - Gera artefatos de design: `research.md`, `data-model.md`, `contracts/`, `quickstart.md`.
  - Atualiza o contexto do agente (arquivo de contexto específico).

Resultado: você sai com um **plano técnico completo** para a feature.

### 4. Criar checklists de qualidade de requisitos – `/speckit.checklist`

- **Quando usar**: quer validar se os **requisitos** estão bem escritos (não a implementação).
- **O que ele faz**:
  - Lê `spec.md`/`plan.md`/`tasks.md` conforme necessário.
  - Cria/atualiza arquivos em `FEATURE_DIR/checklists/` (ex.: `ux.md`, `api.md`, `security.md`).
  - Cada item é um “**unit test de inglês**” para a qualidade dos requisitos (clareza, completude, cobertura, etc.).

Use esse comando para endurecer a spec antes de sair escrevendo código.

### 5. Gerar tasks de implementação – `/speckit.tasks`

- **Pré-requisitos**: `spec.md` + `plan.md` já criados.
- **O que ele faz**:
  - Roda `check-prerequisites.sh` e descobre `FEATURE_DIR` + docs disponíveis.
  - Lê `spec.md`, `plan.md` e, se existirem, `data-model.md`, `contracts/`, `research.md`, `quickstart.md`.
  - Usa `templates/tasks-template.md` para gerar `tasks.md`:
    - Fases (Setup, Foundational, US1, US2, …, Polish).
    - Tasks com formato obrigatório:  
      `- [ ] T001 [P] [US1] Descrição com caminho de arquivo`
    - Marca oportunidades de paralelismo com `[P]`.

Esse arquivo é o **plano de execução** que `/speckit.implement` vai seguir.

### 6. Analisar spec/plan/tasks – `/speckit.analyze`

- **Quando usar**: depois de gerar `tasks.md`, antes de começar a codar pesado.
- **O que ele faz** (somente leitura):
  - Compara `spec.md`, `plan.md`, `tasks.md` + constituição.
  - Procura:
    - Requisitos sem tasks.
    - Tasks sem requisito correspondente.
    - Ambiguidades, duplicações, inconsistências.
    - Violação de princípios da constituição.
  - Gera um **relatório em Markdown** no output do chat com severidade (CRITICAL/HIGH/MEDIUM/LOW) e próximos passos sugeridos.

Nada é alterado em disco; é um passo de auditoria.

### 7. Implementar seguindo o plano – `/speckit.implement`

- **Quando usar**: `plan.md` e `tasks.md` prontos, checklists (de preferência) verdes.
- **O que ele faz**:
  - Lê `tasks.md`, `plan.md` e demais artefatos.
  - Confere checklists em `checklists/` e pergunta se pode seguir mesmo com itens em aberto.
  - Verifica/gera arquivos de ignore (`.gitignore`, `.dockerignore`, etc.) conforme stack do projeto.
  - Executa as tasks **fase a fase**, respeitando dependências e paralelismo `[P]`.
  - Marca tasks concluídas como `[X]` em `tasks.md`.

Você pode usar esse comando para ir implementando de forma disciplinada, sempre baseado nas tasks.

### 8. Converter tasks em issues – `/speckit.taskstoissues`

- **Quando usar**: quer acompanhar a feature via issues no GitHub.
- **O que ele faz**:
  - Lê `tasks.md`.
  - Verifica o remote Git (`git config --get remote.origin.url`).
  - Se o remote for GitHub, usa o **GitHub MCP** para criar uma issue por task.
  - Nunca cria issue em repositório que não bata com o remote atual.

Útil para times que gerenciam trabalho em GitHub Projects ou boards de issues.

---

## Fluxo recomendado (resumão)

1. **Descrever a feature** em linguagem natural → `/speckit.specify`.
2. **Esclarecer ambiguidades críticas** da spec → `/speckit.clarify`.
3. **Desenhar o plano técnico** e artefatos de design → `/speckit.plan`.
4. (Opcional mas recomendado) **Criar checklists de qualidade de requisitos** → `/speckit.checklist`.
5. **Gerar `tasks.md`** organizado por user story → `/speckit.tasks`.
6. (Opcional) **Auditar consistência** entre spec/plan/tasks → `/speckit.analyze`.
7. **Implementar** seguindo `tasks.md` → `/speckit.implement`.
8. (Opcional) **Criar issues no GitHub** a partir das tasks → `/speckit.taskstoissues`.

Você não é obrigado a usar todos os passos sempre, mas quanto mais você segue a sequência, mais previsível e auditável fica o trabalho.
