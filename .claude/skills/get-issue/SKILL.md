---
name: get-issue
description: Fetches Jira issues by key or JQL using the Atlassian MCP. Use when the user asks to get a ticket, look up a Jira issue, search issues in Jira, or when an issue key (e.g. CAP-123) or JQL is mentioned.
---

# Get Issue – Busca de tickets no Jira

Busca issues no Jira via MCP Atlassian (server `plugin-atlassian-atlassian`). Usar a ferramenta **call_mcp_tool** com `server`, `toolName` e `arguments`. Sempre obter `cloudId` antes de chamar getJiraIssue ou searchJiraIssuesUsingJql.

## Pré-requisito: obter cloudId

Todas as ferramentas Jira do MCP exigem `cloudId`. Obter primeiro:

- Chamar **getAccessibleAtlassianResources** (sem argumentos) e usar o `cloudId` retornado nas chamadas abaixo.

## 1. Buscar issue por key

Para issue key conhecido (ex.: CAP-123), usar a ferramenta **getJiraIssue**.

**Parâmetros:**

| Parâmetro    | Obrigatório | Tipo   | Descrição                                                           |
| ------------ | ----------- | ------ | ------------------------------------------------------------------- |
| cloudId      | Sim         | string | UUID ou URL do site (do passo acima)                                |
| issueIdOrKey | Sim         | string | Key da issue (ex.: CAP-123)                                         |
| fields       | Não         | array  | Nomes dos campos (ex.: summary, status, assignee, priority, labels) |
| expand       | Não         | string | Ex.: changelog                                                      |

**Exemplo – busca básica:**

```json
{
  "server": "plugin-atlassian-atlassian",
  "toolName": "getJiraIssue",
  "arguments": {
    "cloudId": "<cloudId do getAccessibleAtlassianResources>",
    "issueIdOrKey": "CAP-123"
  }
}
```

**Exemplo – com campos e expand:**

```json
{
  "server": "plugin-atlassian-atlassian",
  "toolName": "getJiraIssue",
  "arguments": {
    "cloudId": "<cloudId>",
    "issueIdOrKey": "CAP-123",
    "fields": ["summary", "status", "assignee", "priority", "labels"],
    "expand": "changelog"
  }
}
```

## 2. Buscar issues por JQL

Para listar issues por critérios, usar **searchJiraIssuesUsingJql**.

**Parâmetros:**

| Parâmetro     | Obrigatório | Tipo   | Descrição                                 |
| ------------- | ----------- | ------ | ----------------------------------------- |
| cloudId       | Sim         | string | Mesmo cloudId obtido acima                |
| jql           | Sim         | string | Query JQL                                 |
| maxResults    | Não         | number | Máximo de resultados (10–100; default 10) |
| fields        | Não         | array  | Campos a retornar                         |
| nextPageToken | Não         | string | Token para próxima página                 |

**Exemplos de JQL (projeto CAP):**

- Por status: `project = CAP AND status = '2. Doing'`
- Por assignee: `project = CAP AND assignee = currentUser()`
- Por prioridade: `project = CAP AND priority = Highest`
- Por label: `project = CAP AND labels = frontend`
- Recentes: `project = CAP AND updated >= -7d ORDER BY updated DESC`
- Em sprint (quando aplicável): `project = CAP AND sprint in openSprints() ORDER BY updated DESC`

**Exemplo de chamada:**

```json
{
  "server": "plugin-atlassian-atlassian",
  "toolName": "searchJiraIssuesUsingJql",
  "arguments": {
    "cloudId": "<cloudId>",
    "jql": "project = CAP AND status = '2. Doing'",
    "maxResults": 20,
    "fields": ["summary", "status", "assignee", "priority", "labels", "created"]
  }
}
```

## 3. Fluxo recomendado

1. Chamar **getAccessibleAtlassianResources** e guardar `cloudId`.
2. Se o usuário passou uma **issue key** → usar **getJiraIssue** com `issueIdOrKey` e opcionalmente `fields`/`expand`.
3. Se o usuário pediu **lista/critérios** → usar **searchJiraIssuesUsingJql** com `jql` (e `maxResults`/`fields` conforme necessidade).
4. Tratar erros (401/403/404) e informar mensagem clara (autenticação, permissão, issue não encontrada).

## 4. Contexto do projeto CAP (InvestTools)

- **Projeto:** CAP (InvestTools).
- **Board:** “Board DM - Controladoria” (ID 46); issues do board/sprint via JQL (ex.: `sprint in openSprints()`), pois o MCP atual não expõe endpoint de board.
- **Tipos:** História, Tarefa, Bug, Análise.
- **Status:** 1. TO DO, 2. Doing, 5. N1 TEST, Waiting deploy, Concluído.
- **Prioridades:** Highest, Medium.
- **Labels:** frontend, backend, bug, feature, urgent.

## 5. Tratamento de erros

- **Issue não encontrada:** Confirmar que a key está correta e que o usuário tem acesso ao projeto.
- **401/403:** Verificar autenticação e permissões do usuário no Jira.
- **Campos ausentes:** Usar **getJiraIssueTypeMetaWithFields** (cloudId + projectIdOrKey + issueTypeId) para listar campos disponíveis do projeto, se necessário.

Para JQL avançado, criação de issues, transições e Confluence, usar o skill **atlassian-mcp-rule**.
