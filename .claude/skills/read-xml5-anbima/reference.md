# Referência: Leitura e Análise XML5 ANBIMA

Detalhes de implementação para uso quando o agente precisar de regras de extração, validação ou formato. O agente **ariel-xml5-anbima** usa os scripts da skill **read-xml5-anbima** (`read-xml5-anbima.sh` / `read-xml5-anbima.rb`) como ferramentas; consulte este arquivo para lógica interna ou quando for gerar código equivalente.

**Referência oficial**: para códigos, regras de preenchimento e descrição de campos conforme a ANBIMA, use a **skill PDF** para ler o manual `Manual do Anbima XML 5.pdf` nesta mesma pasta da skill.

## Namespace e constantes

- **Namespace**: `urn:iso:std:iso:20022:tech:xsd:semt.003.001.04`
- **Mensagem**: `semt.003.001.04`
- **Serviço**: `Arquivo de Posição 5.0`
- Encoding: UTF-8 obrigatório

## Elementos obrigatórios (estrutura)

- `//xmlns:bsnsMsg` (BAH)
- `//xmlns:Pgntn`
- `//xmlns:StmtGnlDtls`
- `//xmlns:SfkpgAcct` (Custodiante)

## Mapeamento de seções para --extract

| Valor --extract | Chave na análise |
|-----------------|------------------|
| BalForAcct      | carteira         |
| SubAcctDtls     | ativos           |
| bsnsMsg         | bah              |
| Pgntn           | paginacao        |
| StmtGnlDtls     | detalhes_gerais  |

## XPath principais (Nokogiri com xmlns)

- BAH informante: `.//xmlns:fr//xmlns:nm`, CNPJ: `.//xmlns:fr//xmlns:id//xmlns:othr//xmlns:id`
- Paginação: `//xmlns:Pgntn`, PgNb, LastPgInd
- Prestadores: `//xmlns:AcctOwnr`, `//xmlns:AcctSvcr`, `//xmlns:SfkpgAcct`
- Carteira: `//xmlns:BalForAcct` com FinInstrmId/ISIN, Bal/Qty, Bal/Valtn/Amt
- Ativos: `//xmlns:SubAcctDtls` com FinInstrmId, FinInstrmAttrbts/Nm, Bal/Qty, Bal/Valtn/Amt
- Despesas: `//xmlns:Bal//xmlns:BalTp[.='EXPN']` (e MANF, EQUL, CUST, BRKF, TAXS, OTHR)

## Validações

- **Estrutura**: namespace da raiz, presença dos 4 elementos obrigatórios.
- **Campos**: 001–002 (informante, CNPJ 14 chars), 005 msgDefIdr, 006 bizSvc, 013 data posição.
- **Cálculos**: PL = Qty × Valor da cota; conferir com total declarado (tolerância 0.01).

## Formato de saída

- `summary`: Markdown com Informações Gerais, Carteira e Validações.
- `detailed` / `json`: análise completa em JSON (bah, paginacao, prestadores, detalhes_gerais, carteira, ativos, despesas, validacao).
