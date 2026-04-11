# Schemas, obrigatoriedade e corpo

Checklist para **qualquer** OpenAPI (agnóstico de repo). O agregado `api-ivt-docs` pode servir de **exemplo de rigor**, não como contrato do seu serviço.

## 1. `requestBody`

- Declare **`requestBody.required`** (`true` / `false`) sempre que houver corpo.
- Objetos JSON de entrada: array **`required`** listando o mínimo que o cliente envia; o restante é opcional.
- Envelopes com uma chave principal + `$ref` para DTO (ex.: `required: [security]` + `SecurityInput`) são um padrão comum no agregado — replique a ideia no seu serviço quando fizer sentido.
- Prefira **`$ref`** para `components.schemas` em DTOs reutilizados.

## 2. Array `required` no schema

- **Writes** (`*Input`, PATCH/PUT): `required` alinhado ao validador real da API.
- **Reads**: muitos specs omitem `required` no raiz da resposta (payload variável); não invente obrigatoriedade em resposta sem alinhar ao backend.

## 3. Parâmetros

- `in: path` → `required: true`.
- Query obrigatória → `required: true`; opcional → `false` ou omitir (default OAS3).
- **`schema` obrigatório e tipado:** use pelo menos `type` (`string`, `integer`, `boolean`, …) e `format` quando fizer sentido (`date`, `uuid`). Evite `schema: {}` — gera documentação inútil e some validação visual no Swagger.
- Reutilize `components.parameters` para paginação/filtros comuns quando o projeto já tiver.

## 4. `nullable`

- Só quando o JSON pode ser **`null`** naquele campo. Opcional ≠ `nullable`.

## 5. `description` e `example`

- `summary` / descrições de operação e propriedades: idioma do projeto (muito Ivt em PT-BR).
- `example` ou `examples` em respostas críticas reduz ambiguidade.

## 6. Erros

- Se o arquivo tiver `components.responses` padronizados, referencie-os (`401`, `400`, `404`, `500`).

## 7. Anti-padrões

- Corpo sem `requestBody.required` / sem `required` nos campos mínimos.
- Obrigatoriedade só no texto.
- **Tags**: um único balde para domínios diferentes — ver `SKILL.md` e `references/tags-from-codebase.md`.
