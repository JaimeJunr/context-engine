# Scripts da skill read-xml5-anbima (XML5 ANBIMA)

Scripts utilitários para o agente **ariel-xml5-anbima** e para uso via skill **read-xml5-anbima**. Local: `.claude/skills/read-xml5-anbima/scripts/`. A validação é focada em ANBIMA5 e alinhada ao fluxo de leitura do Performit (Grails, Rails, frontend).

## Scripts disponíveis

### read-xml5-anbima.rb / read-xml5-anbima.sh

Script para leitura e análise de arquivos XML5 ANBIMA (skill **read-xml5-anbima**).

#### Pré-requisitos

- Ruby instalado
- Gem Nokogiri instalada: `gem install nokogiri`

#### Uso

Executar a partir da **raiz do workspace performit**:

```bash
# Wrapper shell (recomendado) — usado pelo agente ariel-xml5-anbima
.claude/skills/read-xml5-anbima/scripts/read-xml5-anbima.sh --file="arquivo.xml"

# Ou diretamente com Ruby
ruby .claude/skills/read-xml5-anbima/scripts/read-xml5-anbima.rb --file="arquivo.xml"
```

#### Parâmetros

- **--file**: Caminho do arquivo XML a ser lido (obrigatório)
- **--validate**: Executar validação ANBIMA5 (estrutura, campos, cálculos alinhados ao Performit) — opcional, padrão: `true`
- **--extract**: Extrair seções específicas do XML (opcional, ex: `--extract="BalForAcct,SubAcctDtls"`)
- **--format**: Formato de saída - `summary` | `detailed` | `json` (opcional, padrão: `summary`)

#### Exemplos

```bash
# Leitura básica com validação padrão
.claude/skills/read-xml5-anbima/scripts/read-xml5-anbima.sh --file="arquivo.xml"

# Extrair apenas seções específicas
.claude/skills/read-xml5-anbima/scripts/read-xml5-anbima.sh --file="arquivo.xml" --extract="BalForAcct,SubAcctDtls"

# Leitura sem validação completa (apenas parsing)
.claude/skills/read-xml5-anbima/scripts/read-xml5-anbima.sh --file="arquivo.xml" --validate=false

# Saída em formato JSON
.claude/skills/read-xml5-anbima/scripts/read-xml5-anbima.sh --file="arquivo.xml" --format=json

# Saída detalhada
.claude/skills/read-xml5-anbima/scripts/read-xml5-anbima.sh --file="arquivo.xml" --format=detailed
```

#### Funcionalidades

- ✅ Parsing seguro de XML com Nokogiri (remove BOM como no Grails)
- ✅ Aceita wrapper ANBIMA5 `PosicaoAtivosCarteira` > `Document` > `SctiesBalAcctgRpt`
- ✅ Validação de estrutura (elementos usados pelo sistema: BalForAcct, StmtGnlDtls, AcctBaseCcyTtlAmts, BAH, Pgntn, SfkpgAcct)
- ✅ Validação de campos obrigatórios (BAH 001-008, data posição 013, etc.)
- ✅ Validação de cálculos (PL em AcctBaseCcyTtlAmts/TtlHldgsValOfStmt; consistência com AggtBal × PricDtls no primeiro BalForAcct)
- ✅ Extração de seções (BAH, Paginação, Prestadores, Carteira, Ativos, Despesas)
- ✅ Múltiplos formatos de saída (summary, detailed, json)

#### Validações Realizadas (alinhadas ao Performit)

1. **Estrutura**: Aceita root `PosicaoAtivosCarteira` ou raiz com namespace semt; exige SctiesBalAcctgRpt, BalForAcct, StmtGnlDtls, AcctBaseCcyTtlAmts; BAH, Pgntn, SfkpgAcct para conformidade ANBIMA5.
2. **Campos Obrigatórios**: BAH (informante, CNPJ 14 dígitos, msgDefIdr `semt.003.001.04`, bizSvc `Arquivo de Posição 5.0`), data posição (StmtGnlDtls/FrDtToDt/FrDt).
3. **Cálculos**: PL em TtlHldgsValOfStmt (Amt/Sgn); primeiro BalForAcct: AggtBal Qty × PricDtls Val Amt consistente com PL declarado.

#### Saída

O script fornece análise estruturada do arquivo XML5 ANBIMA, incluindo:

- **BAH (Business Application Header)**: Informante, CNPJ, Destinatário, Mensagem, Serviço
- **Paginação**: Página atual, última página
- **Prestadores**: Administrador, Gestor, Custodiante
- **Detalhes Gerais**: Data posição, Operação, Frequência, Tipo atualização
- **Carteira**: ISIN, CNPJ, Quantidade de cotas, Valor da cota, Total de ativos
- **Ativos**: ISIN, Nome, Quantidade, Valor, Tipo
- **Despesas**: Tipo, Valor, Descrição
- **Validação**: Estrutura (e erros), Campos obrigatórios, Cálculos

#### Tratamento de Erros

- ❌ Arquivo não encontrado ou não legível
- ❌ XML mal formado (syntax errors)
- ❌ Erros de encoding (UTF-8 obrigatório)
- ❌ Estrutura inválida (wrapper/mensagem, elementos obrigatórios)
- ❌ Campos obrigatórios inválidos
- ❌ Divergências em cálculos (PL vs cotas × valor cota)

#### Notas Importantes

- **SEMPRE use** encoding UTF-8 para arquivos XML (BOM é removido automaticamente).
- Com wrapper **PosicaoAtivosCarteira**, o namespace do root não precisa ser semt; a mensagem interna é validada.
- **SEMPRE siga** regras da `ariel-xml5-rule` para interpretação; use a saída do script para **Problema → Local (XPath) → Solução**.

## Estrutura

```
.claude/skills/read-xml5-anbima/scripts/
├── README.md             # Este arquivo
├── read-xml5-anbima.rb   # Script Ruby principal (skill read-xml5-anbima)
└── read-xml5-anbima.sh   # Wrapper shell (usado pelo agente ariel-xml5-anbima)
```

## Integração com o agente

O subagent **ariel-xml5-anbima** (`.claude/agents/ariel-xml5-anbima.md`) usa estes scripts como ferramentas. Ao invocar o agente para ler/validar XML5 ANBIMA, ele deve executar `read-xml5-anbima.sh` com os parâmetros adequados e interpretar a saída com o conhecimento de domínio (estrutura, códigos, PREVIC/ANBIMA). A validação reflete o que o sistema Performit exige ao importar o XML.
