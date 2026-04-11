---
name: ivt-openapi-spec
description: Use when writing or fixing OpenAPI 3.x in any repo (agnostic)—tags from module/path/domain, explicit required/requestBody/schemas, security schemes that match real guards, PT-BR or project language. Optional style baseline from api-ivt-docs (not tag names). NestJS, Go, Python, etc.
---

# OpenAPI — padrões (agnóstico de projeto)

## Escopo

- **`api-ivt-docs/v1/swagger.yaml`** é **referência opcional de estilo** na família Ivt: blocos de texto sobre OAuth2/Cognito, ideias de `info`, formato de erros comuns, `openapi: 3.0.3`. **Não** é fonte de nomes de tags nem de estrutura de pastas do seu projeto.
- Cada API define **suas próprias tags**, **seus paths** (`/internal/v1/...`, `/api/...`, etc.) e **seus** `securitySchemes` (ApiKey interno, Bearer M2M, ambos, …).

## Tags: derivar do código ou do contrato, não do agregado

1. **Regra:** `tags` devem refletir **módulo, prefixo de rota ou bounded context** — o que o leitor do Swagger usa para achar o grupo certo.
2. **Heurística:** um **controller / router / blueprint** (ou prefixo estável) → **pelo menos uma tag**; não misturar, por exemplo, **ADM broker**, **integração Mellon**, **players** e **fundos** na mesma tag só para “simplificar”.
3. **Anti-padrão:** uma única tag (ex.: “Fundos”) em **todas** as operações quando o código está separado em `modules/adm-broker`, `integrations/mellon`, `modules/players`, `modules/funds`.
4. Exemplo ilustrativo (Nest / módulos internos): ver `references/tags-from-codebase.md`.

**Anexo** com tags do YAML agregado Ivt — **somente** se você estiver editando o merge em `api-ivt-docs`: `references/tag-catalog.md`.

## Segurança

- Documente **o que o serviço realmente exige** por rota: `security` por operação ou padrão global coerente com guards/middleware.
- É normal ter **vários** `components.securitySchemes` (ex.: `ApiKey` para frontend interno, `BearerToken` para M2M) e **nem toda rota** usar o mesmo scheme — isso deve aparecer no OpenAPI.
- Texto de ajuda no `info.description` pode citar o ecossistema Ivt (Cognito, URLs) **quando fizer sentido para integradores**; não copie obrigatoriamente o texto do agregado se a API for só interna.

## Estrutura do documento (qualquer produto)

| Elemento           | Orientação                                                                        |
| ------------------ | --------------------------------------------------------------------------------- |
| `openapi`          | `3.0.3` é um bom padrão comum na Ivt; siga o que o CI do repo exigir              |
| `info` / `servers` | Refletem **este** deploy (local, homolog, prod); paths reais                      |
| `tags`             | Vocabulário **deste** projeto; descrições por tag ajudam o Swagger UI             |
| Operações          | `summary` (e textos) no **idioma acordado no repo** (muito PT-BR na Ivt)          |
| `security`         | Alinhado aos guards; omitir ou marcar público só se for verdade                   |
| Respostas          | `content` + `schema` / `$ref` quando houver corpo; códigos coerentes com o código |
| Parâmetros         | `schema` com **`type` (e `format` se preciso)** — evitar `schema: {}` vazio       |

Obrigatoriedade de corpos e propriedades: `references/schema-and-required.md`.

## Fluxo de trabalho

1. Mapear **módulo ou prefixo** → **tag(s)** (ver referência de exemplo se útil).
2. Para cada operação: `summary`, parâmetros com tipo explícito, `requestBody`/`required`, respostas com schema quando aplicável, `security` correto.
3. **Opcional:** comparar com `api-ivt-docs` só para inspirar redação ou formato de erro **se** o time quiser alinhar documentação externa — sem renomear tags do seu serviço para coincidir com o agregado.

## Referências

- Tags a partir da árvore de código (exemplo): `references/tags-from-codebase.md`
- `required`, nullable, parâmetros: `references/schema-and-required.md`
- Anexo tags **só** `api-ivt-docs`: `references/tag-catalog.md`
- Anotações por stack: `references/framework-annotations.md`

## Erros comuns

- Tratar `api-ivt-docs` como **lista de tags** do seu microserviço.
- **Uma tag para API inteira** apesar de módulos/prefixos distintos.
- `parameters[].schema` vazio — perde tipos no Swagger e validação de contrato.
- Descrever auth no texto mas **não** declarar `security` / `securitySchemes` nas rotas que exigem token.
- “Obrigatório” só em `description` sem `required` no schema JSON.
