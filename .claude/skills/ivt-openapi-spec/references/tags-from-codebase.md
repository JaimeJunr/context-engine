# Tags a partir da estrutura do projeto (exemplo agnóstico)

Use quando o repositório já separa domínios em pastas ou controllers. A tag no OpenAPI deve **acompanhar essa separação**, não um documento externo.

## Exemplo (NestJS — control server / integrações)

Árvore simplificada:

- `modules/adm-broker/` → rotas tipo `/internal/v1/adm-broker/...` (config administradora, rotinas, scheduler)
- `integrations/mellon/` → `/internal/v1/integrations/mellon/...` (config Mellon, fundos vistos pela integração, teste de conexão)
- `modules/players/` → `/internal/v1/players/...` (lifecycle, credenciais, test-connection por player)
- `modules/funds/` → `/internal/v1/funds/...` (lista de fundos, config Performit)

**Sugestão de tags (nomes podem ser os do produto; o importante é não colapsar):**

| Área no código | Prefixo HTTP (exemplo) | Tag sugerida |
|----------------|------------------------|--------------|
| `adm-broker` | `/internal/v1/adm-broker` | `Administradoras` ou `ADM Broker` |
| `integrations/mellon` | `/internal/v1/integrations/mellon` | `Integrações Mellon` ou `Mellon` |
| `players` | `/internal/v1/players` | `Players` |
| `funds` | `/internal/v1/funds` | `Fundos` |

**Errado:** marcar todas as operações acima com `tags: [Fundos]` e uma única entrada em `tags[].description` citando “alinhado ao api-ivt-docs” — o agregado não define a taxonomia **deste** serviço.

**Certo:** quatro (ou mais) tags no `DocumentBuilder`/controllers, cada controller com `@ApiTags(...)` coerente com o módulo.

## Como generalizar

- **Go:** pacote ou prefixo de rota → nome da tag no swag.
- **FastAPI:** `APIRouter(..., tags=[...])` por módulo.
- **YAML manual:** uma seção comentada por domínio + mesma tag em todas as operações daquele prefixo.

Se dois módulos compartilharem **uma** tag, faça por decisão de produto (mesmo menu na UI), não por preguiça de documentação.
