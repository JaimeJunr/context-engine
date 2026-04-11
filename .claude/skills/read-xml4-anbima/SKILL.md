---
name: read-xml4-anbima
description: Ferramentas para leitura e análise de arquivos XML4 ANBIMA (arquivoposicao_4_01). Use quando precisar ler, validar ou extrair seções de XML4. Fornece scripts executáveis e validação alinhada ao fluxo Performit (AnbimaImportService, AbstractAnbimaService).
---

# Read XML4 ANBIMA: Ferramentas para XML4 ANBIMA

Ferramentas para ler, analisar e validar arquivos **XML4 ANBIMA** (formato `arquivoposicao_4_01`). O agente deve **executar os scripts** desta skill em vez de reimplementar a lógica. A validação (`--validate=true`) segue o mesmo critério que o sistema Performit usa ao ler XML4: Grails (`AnbimaImportService`) e frontend (`AbstractAnbimaService`).

## Ferramentas disponíveis

Todas as ferramentas ficam em `.claude/skills/ivt/read-xml4-anbima/scripts/`. Usar a partir da raiz do workspace **performit**.

### 1. Análise completa (recomendado)

**Script**: `read-xml4-anbima.sh` (wrapper) ou `read-xml4-anbima.rb`

Executa parsing, extração de seções (header, provisão, caixa, ativos) e **validação ANBIMA4** (estrutura, campos obrigatórios, cálculos). Alinhado ao fluxo de leitura do sistema: root `arquivoposicao_4_01`, `fundo/header` ou `carteira/header` (nome, dtposicao, patliq, quantidade, valorcota), PL e cotas.

```bash
# Da raiz do workspace performit
.claude/skills/ivt/read-xml4-anbima/scripts/read-xml4-anbima.sh --file="caminho/para/arquivo.xml"
```

**Parâmetros**

| Parâmetro    | Obrigatório | Descrição |
|-------------|-------------|-----------|
| `--file=FILE` | Sim        | Caminho do arquivo XML |
| `--validate=true\|false` | Não (padrão: true) | Rodar validação completa (estrutura + campos + cálculos, alinhada ao Performit) |
| `--extract=SEÇÕES` | Não | Seções separadas por vírgula: `header`, `provisao`, `caixa`, `acoes`, `titpublico`, `titprivado`, `debenture`, `imoveis`, `cotas` |
| `--format=summary\|detailed\|json` | Não (padrão: summary) | Formato da saída |

**Exemplos**

```bash
# Análise com validação e resumo
.claude/skills/ivt/read-xml4-anbima/scripts/read-xml4-anbima.sh --file="posicao.xml"

# Só header e provisão, saída JSON
.claude/skills/ivt/read-xml4-anbima/scripts/read-xml4-anbima.sh --file="posicao.xml" --extract="header,provisao" --format=json

# Parsing rápido sem validação
.claude/skills/ivt/read-xml4-anbima/scripts/read-xml4-anbima.sh --file="posicao.xml" --validate=false
```

### 2. Quando usar cada ferramenta

- **Ler/analisar um XML4 ANBIMA** → executar `read-xml4-anbima.sh --file=...` (formato `summary` ou `json` conforme necessidade).
- **Validar conformidade ANBIMA4** → usar `--validate=true` (padrão); a seção "Validações" reflete as exigências de estrutura e cálculo que o sistema usa na importação.
- **Extrair só algumas seções** → usar `--extract=header,provisao,caixa` (ou outras seções).
- **Integrar com outro código** → usar `--format=json` e parsear a saída.

## Pré-requisitos

- Ruby instalado.
- Gem Nokogiri: `gem install nokogiri`.

O `read-xml4-anbima.sh` verifica Ruby e Nokogiri antes de chamar o Ruby.

## Regras que o agente deve seguir

1. **SEMPRE** validar que o arquivo existe e é legível antes de chamar o script.
2. **SEMPRE** usar encoding adequado (UTF-8 ou o declarado no XML; o Grails usa `encodeXmlContent`).
3. Para interpretar erros ou campos, consultar o [reference.md](reference.md) desta skill (estrutura, códigos ANBIMA 4, validações).

## Referência detalhada

- [reference.md](reference.md): estrutura XML4, XPath, mapeamento de seções, códigos de provisão e formatos de saída.
- [scripts/README.md](scripts/README.md): uso dos scripts, parâmetros e exemplos.

A skill permanece em `.claude/skills/ivt/read-xml4-anbima/`.
