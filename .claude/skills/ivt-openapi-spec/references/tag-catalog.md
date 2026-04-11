# Anexo — tags do `api-ivt-docs` (somente merge no agregado)

Este arquivo **não** define tags para outros repositórios. Ele lista o que já existe no **`v1/swagger.yaml` agregado** para quando você **contribui** com operações que entram nesse arquivo.

Para serviços isolados (ex.: control server, BFF, microserviços), ignore esta lista e use `references/tags-from-codebase.md` + `SKILL.md`.

## Tags atuais no agregado

| Tag | Uso resumido no YAML agregado |
|-----|-------------------------------|
| **Autenticação** | OAuth2 Cognito |
| **Ativos** | Ativos, emissores, custódia |
| **Clientes** | Cotistas, passivo |
| **Contas** | Contas liquidação/bancárias |
| **Corretoras** | Corretoras |
| **Distribuidores** | Distribuidores, rebate |
| **Eventos Corporativos** | Eventos |
| **Fundos** | Fundos e outros itens hoje agrupados no Cadastro do Redoc |
| **Mercados** | Mercados |
| **Moedas** | Moedas |
| **Usuários** | Usuários |
| **Abertura & Conciliação** | Dia, reconciliação |
| **Lançamentos** | Lançamentos |
| **Movimentações** | Movimentações |
| **Cotações** | Cotações |
| **Câmbio** | Câmbio |
| **Índices** | Índices |
| **Derivativos** | Derivativos |
| **Feriados** | Feriados |

## `x-tagGroups` (Redoc)

| Grupo | Tags |
|-------|------|
| Autenticação | Autenticação |
| Cadastro | Ativos, Clientes, Contas, Corretoras, Distribuidores, Eventos Corporativos, Fundos, Mercados, Moedas, Usuários |
| Conciliação | Abertura & Conciliação, Lançamentos, Movimentações |
| Preços | Câmbio, Cotações, Índices |
| Operações | Derivativos |
| Utilitários | Feriados |

Nova tag no agregado: atualizar `tags`, `x-tagGroups` e manter PT-BR consistente.
