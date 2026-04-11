---
name: update-jira-tokens-estimado-realizado
description: Use when the user asks to set the Jira field labeled Estimado/Realizado for a ticket (by key like CAP-123 or by Jira URL). The user may optionally provide the token count (e.g. 12345 tokens, 12345 tokens gastos, or tokens=12345). If omitted, the skill will fetch the token count by running `ccusage` locally for the latest session; Jira is only the storage target.
---

# Update Jira Tokens - Estimado/Realizado

Atualizar o campo Jira `Estimado/Realizado` com a quantidade de `tokens gastos` da tarefa.

## 0. Extração de parâmetros

Extrair:

- `issueIdOrKey`: key da issue (`CAP-123`) ou ID (quando fornecido) ou pegar da URL `/browse/<KEY>` (obrigatório).
- `tokensSpent`: número inteiro (sem separadores) (opcional).
  Aceitar formas como:
  - `12345 tokens gastos`
  - `tokens=12345`
  - `12,345 tokens`

Se `tokensSpent` não for fornecido na mensagem:

1. Obter `tokensSpent` via `ccusage` (última sessão) executando:
   - Primeiro (se disponível):
     `bunx ccusage@latest session --order desc --jq '.sessions[0] | {sessionId, inputTokens, outputTokens, totalTokens, totalCost, lastActivity}'`
   - Fallback (se o `bunx` não existir/der erro):
     `npx ccusage@latest session --order desc --jq '.sessions[0] | {sessionId, inputTokens, outputTokens, totalTokens, totalCost, lastActivity}'`
2. Usar `totalTokens` como `tokensSpent` (arredondar/converter para inteiro se vier como número).
3. Se falhar para rodar `ccusage` ou se não conseguir extrair `totalTokens`, pedir ao usuário para informar `tokensSpent`.

## 1. Obter `cloudId` antes das chamadas Jira

Chamar o MCP:

```json
{
  "server": "plugin-atlassian-atlassian",
  "toolName": "getAccessibleAtlassianResources",
  "arguments": {}
}
```

Guardar o `cloudId` retornado.

## 2. Ler a issue para descobrir `project` e `issuetype`

Chamar:

```json
{
  "server": "plugin-atlassian-atlassian",
  "toolName": "getJiraIssue",
  "arguments": {
    "cloudId": "<cloudId>",
    "issueIdOrKey": "<issueIdOrKey>",
    "fields": ["project", "issuetype"]
  }
}
```

Obter:

- `projectIdOrKey` a partir de `issue.project.key` (ou o equivalente retornado pelo MCP).
- `issueTypeId` a partir de `issue.issuetype.id` (ou equivalente retornado pelo MCP).

## 3. Descobrir o campo customizado por nome exibido `Estimado/Realizado`

Chamar:

```json
{
  "server": "plugin-atlassian-atlassian",
  "toolName": "getJiraIssueTypeMetaWithFields",
  "arguments": {
    "cloudId": "<cloudId>",
    "projectIdOrKey": "<projectIdOrKey>",
    "issueTypeId": "<issueTypeId>"
  }
}
```

No retorno, localizar o field cuja descrição/rótulo/nome corresponda (case-insensitive) a `Estimado/Realizado`.
Salvar o identificador que será usado no payload de escrita:

- preferir algo como `customfield_XXXXX` (ou `fieldId`/`id` equivalente).

Se não conseguir identificar o field ID do rótulo `Estimado/Realizado`, parar e pedir ao usuário o `customfield_XXXXX` (ou o ID do campo) para evitar escrita no campo errado.

## 4. Atualizar o Jira via `editJiraIssue`

Pré-condição: ter definido `tokensSpent` como número inteiro.
Chamar:

```json
{
  "server": "plugin-atlassian-atlassian",
  "toolName": "editJiraIssue",
  "arguments": {
    "cloudId": "<cloudId>",
    "issueIdOrKey": "<issueIdOrKey>",
    "fields": {
      "<fieldIdForEstimadoRealizado>": <tokensSpent>
    }
  }
}
```

Se o Jira exigir string para campos numéricos, repetir com `"<tokensSpent>"` (string) no valor.

## 5. Validar lendo novamente

Chamar novamente:

```json
{
  "server": "plugin-atlassian-atlassian",
  "toolName": "getJiraIssue",
  "arguments": {
    "cloudId": "<cloudId>",
    "issueIdOrKey": "<issueIdOrKey>",
    "fields": ["<fieldIdForEstimadoRealizado>"]
  }
}
```

Confirmar que o valor em `issue.fields[<fieldIdForEstimadoRealizado>]` (ou equivalente no retorno) corresponde ao `tokensSpent`.
