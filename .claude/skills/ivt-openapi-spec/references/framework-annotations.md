# Anotações por stack (agnóstico de projeto)

Objetivo: gerar OpenAPI **correto** em qualquer repo: **tag = módulo/domínio**, **security = guards reais**, **required** nos DTOs, textos no idioma do projeto.

## Tags

- Alinhar `@Tags` / `tags:` / `APIRouter(..., tags=)` ao **controller ou prefixo**, não a um YAML agregado externo.
- Vários schemes (`ApiKey`, `BearerToken`, …) são comuns; documente por rota com `@ApiSecurity` / equivalente.

## NestJS (ex.: API interna + integrações)

Separar tags **por feature**, coerente com pastas `modules/*` e `integrations/*`:

```typescript
// adm-broker.controller.ts
@ApiTags('Administradoras')
@ApiBearerAuth('ApiKey')
@Controller('internal/v1/adm-broker')
export class AdmBrokerController { /* ... */ }

// mellon-integration.controller.ts
@ApiTags('Integrações Mellon')
@ApiBearerAuth('ApiKey')
@Controller('internal/v1/integrations/mellon')
export class MellonIntegrationController { /* ... */ }

// players.controller.ts
@ApiTags('Players')
@Controller('internal/v1/players')
export class PlayersController { /* ... */ }

// funds.controller.ts
@ApiTags('Fundos')
@Controller('internal/v1/funds')
export class FundsController { /* ... */ }
```

No `DocumentBuilder`, registre **todos** os schemes que o serviço expõe (ex.: `addBearerAuth(..., 'ApiKey')` e `addBearerAuth(..., 'BearerToken')`) e use `@ApiBearerAuth('ApiKey')` só nas rotas que o guard exige — rotas sem guard não devem declarar segurança falsa.

`@ApiProperty` / `@ApiPropertyOptional` controlam `required` no schema gerado.

## Go — `swag` (swaggo)

```go
// ListarItens godoc
// @Summary      Listar itens
// @Description  Lista paginada (filtros opcionais).
// @Tags         MeuModulo
// @Security     BearerToken
// @Produce      json
// @Param        q query string false "Filtro"
// @Success      200 {array} dto.Item
// @Failure      401 {object} dto.ErrorBody
// @Router       /v1/items [get]
```

Use `@Tags` alinhado ao pacote ou recurso. POST: DTO com `binding:"required"` ou body documentado para preencher `required` no OpenAPI.

## Go — go-swagger

Manter YAML/spec com `requestBody.required` e arrays `required` nos objetos de entrada como no validador HTTP.

## Python — FastAPI

```python
router = APIRouter(prefix="/v1/items", tags=["MeuModulo"])

@router.post("", summary="Criar item")
async def create_item(body: ItemCreate):
    ...
```

`Optional[...]` e defaults → opcionais no schema; campos sem default e tipo obrigatório → `required`.

## Python — Flask-Smorest / apispec

`doc={"summary": "...", "tags": ["MeuModulo"], "security": [{"BearerToken": []}]}` + schemas com campos required corretos.

## JavaScript — swagger-jsdoc

```yaml
tags: [MeuModulo]
summary: ...
security: [{ BearerToken: [] }]
requestBody:
  required: true
  content:
    application/json:
      schema:
        type: object
        required: [id]
        properties:
          id: { type: string }
```

## Erros e componentes

Se o projeto padronizar corpos de erro (`message`, `code`), referencie `components.responses` reutilizáveis. Alinhar ao `api-ivt-docs` **só** se o time decidir homogeneizar documentação entre serviços — não é obrigatório para APIs internas.
