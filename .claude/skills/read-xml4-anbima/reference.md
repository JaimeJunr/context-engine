# Referência: Leitura e Análise XML4 ANBIMA

Detalhes de implementação para uso quando for preciso extrair, validar ou interpretar XML4 ANBIMA. Os scripts da skill **read-xml4-anbima** (`read-xml4-anbima.sh` / `read-xml4-anbima.rb`) seguem o fluxo Performit: Grails `AnbimaImportService`, frontend `AbstractAnbimaService`.

## Identificação do formato

- **Root**: elemento cujo nome contém `arquivoposicao_4_01` (ex.: `<arquivoposicao_4_01>`). O Grails aceita se os primeiros 150 caracteres do arquivo contêm `<arquivoposicao_4_01`.
- **Sem namespace**: XML4 não usa namespaces; tags são acessadas por nome direto.

## Estrutura principal (Grails)

- **Header**: `fundo[0].header[0]` ou `carteira[0].header[0]`
- **Campos do header**: `nome`, `dtposicao` (yyyyMMdd), `patliq`, `quantidade`, `valorcota`
- **Encoding**: o Grails usa `encodeXmlContent(file)` (detecta encoding pela declaração do XML ou assume UTF-8).

## Elementos usados pelo sistema

### Header (obrigatório para importação)

- `nome` — nome do fundo
- `dtposicao` — data da posição (yyyyMMdd)
- `patliq` — patrimônio líquido
- `quantidade` — quantidade de cotas
- `valorcota` — valor da cota

### Raiz (frontend checkNav)

- `patliq` — PL total
- `vlcotasemitir` — valor cotas a emitir
- `vlcotasresgatar` — valor cotas a resgatar

### Provisão (contábeis)

- `provisao`: cada item com `coddespesa` ou `codprov`, `valor`, `credeb` (C/D), `dt` (yyyyMMdd)

### Caixa

- `caixa`: cada item com `isininstituicao`, `saldo`

### Seções de ativos (frontend)

- `acoes`, `titpublico`, `titprivado`, `debenture`, `imoveis`, `termorv`, `termorf`, `cotas`, `exotics`, `opcoesacoes`, `opcoesflx`, `forwardsmoedas`

## Mapeamento para --extract

| Valor --extract | Chave na análise |
|-----------------|------------------|
| header          | header           |
| provisao        | provisao         |
| caixa           | caixa            |
| acoes           | acoes            |
| titpublico       | titpublico       |
| titprivado       | titprivado       |
| debenture       | debenture        |
| imoveis         | imoveis          |
| cotas           | cotas            |

## Códigos ANBIMA 4 (provisão)

Alguns códigos usados no frontend: 1–16 (despesas administrativas/taxas), 18–22 (títulos, derivativos), 31 (empréstimo ação), 34 (taxa administração), 35 (taxa performance), 47 (A receber do Master), 999 (Diversos), 2001 (NDF Câmbio).

## Validações (alinhadas ao Performit)

- **Estrutura**: root contém `arquivoposicao_4_01`; presença de `fundo` ou `carteira`; header com nome, dtposicao, patliq, quantidade, valorcota.
- **Campos**: dtposicao em yyyyMMdd; patliq, quantidade, valorcota numéricos.
- **Cálculos**: quantidade × valorcota ≈ patliq (tolerância 0.01), como no Grails.

## Formato de saída

- `summary`: Markdown com Header, Caixa/Provisão (resumo) e Validações.
- `detailed` / `json`: análise completa em JSON (header, provisao, caixa, seções extraídas, validacao).
