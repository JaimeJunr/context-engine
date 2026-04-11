---
name: read-xml5-anbima
description: Ferramentas para leitura e análise de arquivos XML5 ANBIMA. Use quando o agente ariel-xml5-anbima precisar ler, validar ou extrair seções de XML5 (ISO 20022, RCVM 175). Scripts mostram o comportamento atual do sistema Performit (não necessariamente o oficial do manual ANBIMA XML5). Para referências oficiais, use a skill PDF para ler o Manual do Anbima XML 5.
---

# Read XML5 ANBIMA: Ferramentas para XML5 ANBIMA

Ferramentas para o **agente ariel-xml5-anbima** ler, analisar e validar arquivos **XML5 ANBIMA**. O agente deve **executar os scripts** desta skill em vez de reimplementar a lógica.

**Importante:** Os scripts desta skill servem para **ver o comportamento atual do sistema** (Performit) ao ler e validar XML5 — ou seja, o que o Grails (`Anbima5ImportService`), Rails (`Anbima5ReaderService`) e frontend (`AbstractAnbima5Service`) fazem hoje. **Não representam necessariamente** a especificação oficial do manual ANBIMA XML 5. Para conferir regras oficiais, códigos e estrutura conforme o manual, use a **skill PDF** sobre o manual da ANBIMA (ver seção "Referências oficiais" abaixo).

A validação (`--validate=true`) segue o mesmo critério que o sistema Performit usa ao ler XML5.

## Ferramentas disponíveis

Todas as ferramentas ficam em `.claude/skills/read-xml5-anbima/scripts/`. Usar a partir da raiz do workspace **performit**.

### 1. Análise completa (recomendado)

**Script**: `read-xml5-anbima.sh` (wrapper) ou `read-xml5-anbima.rb`

Executa parsing, extração de seções (BAH, paginação, prestadores, carteira, ativos, despesas) e **validação** (estrutura, campos obrigatórios, cálculos) **conforme o comportamento atual do sistema Performit**. A validação replica o que o fluxo de leitura do sistema faz hoje (wrapper `PosicaoAtivosCarteira` > `Document` > `SctiesBalAcctgRpt`, PL em `AcctBaseCcyTtlAmts/TtlHldgsValOfStmt`, primeiro `BalForAcct` com `AggtBal` e `PricDtls`); não substitui a consulta ao manual oficial ANBIMA para regras oficiais.

```bash
# Da raiz do workspace performit
.claude/skills/read-xml5-anbima/scripts/read-xml5-anbima.sh --file="caminho/para/arquivo.xml"
```

**Parâmetros**

| Parâmetro    | Obrigatório | Descrição |
|-------------|-------------|-----------|
| `--file=FILE` | Sim        | Caminho do arquivo XML |
| `--validate=true\|false` | Não (padrão: true) | Rodar validação completa (estrutura + campos + cálculos, alinhada ao Performit) |
| `--extract=SEÇÕES` | Não | Seções separadas por vírgula: `BalForAcct`, `SubAcctDtls`, `bsnsMsg`, `Pgntn`, `StmtGnlDtls` |
| `--format=summary\|detailed\|json` | Não (padrão: summary) | Formato da saída |

**Exemplos**

```bash
# Análise com validação e resumo
.claude/skills/read-xml5-anbima/scripts/read-xml5-anbima.sh --file="posicao.xml"

# Só carteira e ativos, saída JSON
.claude/skills/read-xml5-anbima/scripts/read-xml5-anbima.sh --file="posicao.xml" --extract="BalForAcct,SubAcctDtls" --format=json

# Parsing rápido sem validação
.claude/skills/read-xml5-anbima/scripts/read-xml5-anbima.sh --file="posicao.xml" --validate=false
```

### 2. Quando usar cada ferramenta

- **Ler/analisar um XML5 ANBIMA** → executar `read-xml5-anbima.sh --file=...` (formato `summary` ou `json` conforme necessidade).
- **Ver como o sistema Performit valida hoje** → usar `--validate=true` (padrão); a seção "Validações" da saída reflete as mesmas exigências de estrutura e cálculo que o sistema usa na importação (comportamento atual do sistema, não a especificação oficial do manual ANBIMA).
- **Extrair só algumas seções** → usar `--extract=BalForAcct,SubAcctDtls` (ou outras seções).
- **Integrar com outro código** → usar `--format=json` e parsear a saída.

## Pré-requisitos

- Ruby instalado.
- Gem Nokogiri: `gem install nokogiri`.

O `read-xml5-anbima.sh` verifica Ruby e Nokogiri antes de chamar o Ruby.

## Regras que o agente deve seguir

1. **SEMPRE** validar que o arquivo existe e é legível antes de chamar o script.
2. **SEMPRE** usar encoding UTF-8 no arquivo XML (o script remove BOM como no Grails).
3. Para interpretar erros ou campos, o agente pode consultar a regra `ariel-xml5-rule` e o [reference.md](reference.md) desta skill (XPath, códigos, validações).
4. Para **referências oficiais** (manual ANBIMA Arquivo de Posição 5.0), **usar a skill PDF** para ler o PDF desta skill (ver seção "Referências oficiais (manual em PDF)" acima).

## Referências oficiais (manual em PDF)

Para **referências oficiais** do XML5 (códigos, estrutura, significados de campos, regras PREVIC/ANBIMA), o agente deve usar a **skill PDF** para ler o manual oficial da ANBIMA:

- **Arquivo**: `Manual do Anbima XML 5.pdf`
- **Caminho na skill**: `.claude/skills/ivt/read-xml5-anbima/Manual do Anbima XML 5.pdf`

**Quando usar**: sempre que for preciso confirmar ou buscar no manual oficial — códigos de tipo de ativo/despesa, descrição de tags, regras de preenchimento, exemplos oficiais ou dúvidas que o [reference.md](reference.md) não cubra.

**Como usar**: invocar a **skill PDF** (ler/extrair texto do PDF) sobre esse arquivo; extrair as páginas ou trechos relevantes (por exemplo com `pdfplumber` ou `pypdf` conforme a skill PDF). O agente ariel-xml5-anbima pode pedir ao orquestrador que use a skill PDF antes de responder com base no manual.

## Referência detalhada

- [reference.md](reference.md): namespace, XPath, mapeamento de seções, validações e formatos de saída.
- [scripts/README.md](scripts/README.md): uso dos scripts, parâmetros e exemplos.

## Integração com o agente ariel-xml5-anbima

O subagent **ariel-xml5-anbima** (`.claude/agents/ariel-xml5-anbima.md`) é o especialista em XML5 ANBIMA. Ao ser invocado para ler, validar ou explicar um arquivo XML5:

1. Use as **ferramentas** desta skill (scripts em `read-xml5-anbima/scripts/`) para obter análise e validação **do comportamento atual do sistema** — não do manual oficial.
2. Use o **conhecimento** do agente (estrutura, códigos, PREVIC/ANBIMA) para explicar resultados, localizar problemas (tag/XPath) e sugerir correções.
3. Quando precisar de **referência oficial** (códigos, regras de preenchimento, descrição de campos), use a **skill PDF** para ler o manual `.claude/skills/ivt/read-xml5-anbima/Manual do Anbima XML 5.pdf`.
4. Resposta no formato: **Problema → Local (caminho XML) → Solução**.

A skill permanece em `.claude/skills/read-xml5-anbima/`.
