# Scripts da skill read-xml4-anbima (XML4 ANBIMA)

Scripts para leitura e análise de arquivos **XML4 ANBIMA** (formato `arquivoposicao_4_01`). Local: `.claude/skills/ivt/read-xml4-anbima/scripts/`. Validação alinhada ao fluxo Performit: Grails `AnbimaImportService`, frontend `AbstractAnbimaService`.

## Scripts disponíveis

### read-xml4-anbima.rb / read-xml4-anbima.sh

- **read-xml4-anbima.sh**: wrapper que verifica Ruby e Nokogiri e chama o script Ruby.
- **read-xml4-anbima.rb**: parsing, extração e validação.

#### Pré-requisitos

- Ruby instalado
- Gem Nokogiri: `gem install nokogiri`

#### Uso (raiz do workspace performit)

```bash
.claude/skills/ivt/read-xml4-anbima/scripts/read-xml4-anbima.sh --file="arquivo.xml"
ruby .claude/skills/ivt/read-xml4-anbima/scripts/read-xml4-anbima.rb --file="arquivo.xml"
```

#### Parâmetros

- **--file**: Caminho do XML (obrigatório)
- **--validate**: true|false (padrão: true)
- **--extract**: Seções separadas por vírgula (ex.: `header,provisao,caixa`)
- **--format**: summary|detailed|json (padrão: summary)

#### Exemplos

```bash
.claude/skills/ivt/read-xml4-anbima/scripts/read-xml4-anbima.sh --file="posicao.xml"
.claude/skills/ivt/read-xml4-anbima/scripts/read-xml4-anbima.sh --file="posicao.xml" --extract="header,provisao" --format=json
.claude/skills/ivt/read-xml4-anbima/scripts/read-xml4-anbima.sh --file="posicao.xml" --validate=false
```

#### Validações (alinhadas ao Performit)

1. **Estrutura**: raiz contém `arquivoposicao_4_01`; presença de `fundo/header` ou `carteira/header`.
2. **Campos obrigatórios**: nome, dtposicao, patliq, quantidade, valorcota no header.
3. **Cálculos**: quantidade × valorcota vs patliq (tolerância 0.01).

#### Estrutura

```
.claude/skills/ivt/read-xml4-anbima/scripts/
├── README.md               # Este arquivo
├── read-xml4-anbima.rb     # Script Ruby principal
└── read-xml4-anbima.sh     # Wrapper shell
```
