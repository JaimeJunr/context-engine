# Tela K – Fluxo Completo e Arquitetura

## Arquitetura Geral

```
┌─────────────────────────────────────────────────────────────┐
│  USUÁRIO (Browser)                                           │
│  http://localhost:3000/app#/reconcile                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  RAILS (porta 3000)                                          │
│  • GET /app → AppController#index → SPA AngularJS           │
│  • #/reconcile → reconcile.html.slim                        │
│  • Session: reconcile ou reconcile-new (Setting)            │
└─────────────────────────────────────────────────────────────┘
         │ /api/*                       │ /g/* (proxy Grails)
         ▼                              ▼
┌──────────────────┐         ┌──────────────────────────────┐
│  RAILS API       │         │  GRAILS (porta 8080)          │
│  moneyfunds      │         │  /g/quote/rest                │
│  fund_reconcile  │         │  /g/fundTransaction/reconcile │
│  _batch          │         │  /g/accountingEntry/...       │
│  open_day        │         │  /g/openDay                   │
└──────────────────┘         └──────────────────────────────┘
         │                              │
         ▼                              ▼
┌──────────────────┐         ┌──────────────────────────────┐
│  Rails Models    │         │  Grails Services/Domains      │
│  AdmPortfolio    │         │  QuoteService                 │
│  AdmFundShare    │         │  FundTransactionService       │
│  FundReconcile   │         │  OpeningService               │
│  Service/Job     │         │  DailyService                 │
└──────────────────┘         └──────────────────────────────┘
```

---

## Fluxo 1: "Acatar fundos de zeragem"

**Trigger**: `ng-click="acceptAllMoneyfunds()"` em `position.html.slim`

```
Position Controller (position.controller.js.coffee)
  acceptAllMoneyfunds()
    disableMoneyfunds = true
    POST /api/reconcile/moneyfunds/:fundId/:date   ← date em YYYY-MM-DD
    ↓
Api::Reconcile::MoneyfundsController#update (Rails)
    fund = Fund.find(params[:fund_id])
    date = Date.parse(params[:date])
    adm_portfolio = AdministratorPortfolio.new(fund, date).create_adm_portfolio
    Reconcile::AdmFundShare.new(current_user, fund, adm_portfolio).execute
    render json: { success: true }
    ↓
AdministratorPortfolio#create_adm_portfolio
    Tenta em ordem:
      1. Pactual::XlsPortfolio
      2. Anbima::XmlPortfolio
      3. Mellon::TxtPortfolio
      4. Anbima5::XmlPortfolio
      5. Anbima5galgo::XmlPortfolio
    Retorna: { date:, moneyfund_positions: [...] }
    ↓
Reconcile::AdmFundShare#execute
    sys_isins  = posições do sistema (security_type MONEYFUND)
    adm_isins  = posições do portfolio ADM
    Para cada ISIN (union):
      diff_quantity = adm_qty - sys_qty
      Se diff != 0 → CustomSecurityTransaction.save_all
      Se há preço   → create_or_update_quote(security, price)
    PerformitGrails.new.clear_cache
    ↓
Resposta ao Frontend:
    disableMoneyfunds = false
    msg.success("Posições e preços acatados.")
    $rootScope.$broadcast 'cash-dirty'
    loadSystemPositions()
```

---

## Fluxo 2: "Acatar todos os fundos"

**Trigger**: `ng-click="acceptAllFundPrices()"` em `position.html.slim`

```
Position Controller
  acceptAllFundPrices()
    → createPriceRequests($scope.funds)
    Monta array: [{ fund: {id}, day, close, security: {symbol ou isin} }, ...]
    POST /g/quote/rest   (contextPath = /g)
    ↓
Grails: QuoteController#rest (POST)
    request.JSON (lista de objetos)
    Quote.withTransaction {
      json.each { processQuoteJson(item) }
    }
    render { ok: true, processed: N, errors: [] }
    ↓
processQuoteJson(item) [método privado]
    Quote bindData from json (exclude: security, manual)
    security = Security.findBySymbol ou findByIsin
    fund = Fund.get(json.fund?.id)
    Se Quote existente (fund+day+security): bindData, markDirty, save
    Senão: new Quote, openingService.markDirty(fund, day), save
    ↓
Resposta ao Frontend:
    msg.success "Preços acatados."
    $rootScope.$broadcast 'price-changed'
    compare()
```

---

## Fluxo 3: "Abrir Dia" (Lote)

**Trigger**: botão em `reconcile/general-actions.html.slim` → `open()` no controller `General`

```
General Controller
  open()
    POST /api/fund_reconcile_batch/process_batch
    {
      batch_action: 'open_day',
      fund_ids: [...],
      date: 'YYYY-MM-DD',
      ws_id: uuid
    }
    ↓
Api::FundReconcileBatchController#process_batch (Rails)
    Valida batch_action (whitelist)
    FundReconcileBatchJob.perform_async(batch_action, fund_ids, date, ws_id, user_id, ...)
    render json: { status: 'Processando', message: '...' }
    ↓
FundReconcileBatchJob (Sidekiq)
    Para cada fund_id:
      fund = Fund.find(id)
      execute_open_day(fund)
        → PerformitGrails.new.open_day(fund.id, date_formatted)
        → Grails: POST /ws/openDay?fundId=...&openingDate=DD/MM/YYYY
    Broadcast ActionCable: fund_reconcile_ws_#{ws_id}
    ↓
Grails: DailyService.openDay(date, fund, user, jobProgress, session)
    Valida: não é fim de semana, não é feriado
    Valida: sequência de abertura (respeita dias anteriores)
    Abre o dia (recalcula posições, cotas, etc.)
    ↓
Frontend:
    WebSocket escuta canal fund_reconcile_ws_#{ws_id}
    Atualiza status fundo a fundo em tempo real
```

---

## Fluxo 4: "Abrir Dia" (Fundo Único)

```
POST /api/open_day/do_open
  { fund_id, date }
  ↓
Api::OpenDayController#do_open
  PerformitGrails.new.open_day(fund_id, date)
  ↓
Grails: DailyService.openDay
  ↓
{ success: true/false, message: '...' }
```

---

## Fluxo 5: Batch – Outras Ações

```
POST /api/fund_reconcile_batch/process_batch
  { batch_action, fund_ids, date, ws_id, adm_quotes (opcional) }

batch_action         │ método no Job             │ O que chama
─────────────────────┼───────────────────────────┼────────────────────────────
open_day             │ execute_open_day           │ Grails /ws/openDay
calculate_quote      │ execute_calculate_quote    │ Grails calc_nav_per_share
accept_quote         │ execute_accept_quote       │ Rails Quote.close = adm_quote
share_quote          │ execute_share_quote        │ Rails FundService.share
mark_conciliated     │ execute_mark_conciliated   │ Grails mark_conciliated
unmark_conciliated   │ execute_unmark_conciliated │ Grails unmark_conciliated
```

---

## Fluxo 6: "Cotizar Fundos" (acceptFundQuotes)

```
POST /g/fundTransaction/reconcile
  { id: fundId, date: params.date }
  ↓
Grails: FundTransactionController#reconcile
  FundTransaction.findAllWhere(fund, shareCalculationDate: session.date)
  Para cada transaction:
    fundTransactionService.quoteFund(transaction)
      → ajusta quantity/value por cota, custódia, contábil, custodyEntry
```

---

## Controllers Rails – Assinatura dos Métodos

### MoneyfundsController

```ruby
# POST /api/reconcile/moneyfunds/:fund_id/:date
def update
  fund = Fund.find(params[:fund_id])
  date = Date.parse(params[:date])
  adm_portfolio = AdministratorPortfolio.new(fund, date).create_adm_portfolio
  Reconcile::AdmFundShare.new(current_user, fund, adm_portfolio).execute
  render json: { success: true }
end
```

### FundReconcileBatchController

```ruby
# GET /api/fund_reconcile_batch/merged_data?date=YYYY-MM-DD
def merged_data
  # Retorna: { data: [...], ws_id: uuid }
  data = FundReconcileService.build_merged_fund_reconcile_data(date, current_user)
  render json: { data: data, ws_id: SecureRandom.uuid }
end

# POST /api/fund_reconcile_batch/process_batch
def process_batch
  # Params: batch_action, fund_ids, date, ws_id, adm_quotes (opcional)
  # Ações permitidas: open_day, calculate_quote, accept_quote, share_quote,
  #                   mark_conciliated, unmark_conciliated
  FundReconcileBatchJob.perform_async(...)
  render json: { status: 'Processando', message: '...' }
end

# GET /api/fund_reconcile_batch/latest_opening_date
def latest_opening_date
  # Retorna: { latest_date: 'YYYY-MM-DD', success: true }
end
```

---

## Parâmetros do QuoteController (Grails)

```
POST /g/quote/rest
Content-Type: application/json

[
  {
    "fund": { "id": 123 },
    "day": "2024-01-15",
    "close": 10.5432,
    "security": {
      "symbol": "PERF11"   // ou "isin": "BRPERF1CTA01"
    }
  },
  ...
]
```

---

## Estados do Frontend

```
$scope.funds             → lista de fundos com dados de reconciliação
$scope.params.date       → data selecionada
$scope.disableMoneyfunds → bloqueia botão durante processamento
$scope.disableFundPrices → bloqueia botão durante processamento
Setting.reconcile_session → 'reconcile' ou 'reconcile-new' (templates)
```

---

## WebSocket (ActionCable)

O batch usa WebSocket para atualizações em tempo real:

```javascript
// Frontend assina canal:
App.cable.subscriptions.create(
  { channel: "FundReconcileChannel", ws_id: wsId },
  { received: function(data) { /* atualiza UI */ } }
)

// Job publica:
ActionCable.server.broadcast("fund_reconcile_ws_#{ws_id}", {
  fund_id: fund.id,
  status: 'done',
  message: 'Dia aberto com sucesso'
})
```

---

## CashEntry (Conciliação de Caixa)

Domínio Grails para lançamentos de caixa:

```groovy
class CashEntry {
  Account account
  Date day
  String currency
  BigDecimal value
  EntryType entryType    // OTHERS, LOAN_REBATE, etc.
  String description
  String sourceType      // ex.: "br.com.investtools.funds.CustomSecurityTransaction"
  Long sourceId
  Security security
  CashEntryGroup group

  def beforeDelete() {
    // Remove discrepâncias e transações de liquidação vinculadas
  }
}
```

`ReconcileController` (Grails) gerencia: `entries()`, `save()`, `update()` (reconcile, undo, liquidation, etc.), `delete()`, `listTrades()`, `acceptFx()`, `acceptFee()`.

---

## Pontos de Atenção para Desenvolvimento

1. **Template condicional**: `Setting.reconcile_session` controla se usa `reconcile/` ou `reconcile-new/`
2. **Leitores ADM em cascata**: se o leitor errado for tentado primeiro, pode gerar erro silencioso
3. **Conversão cambial** em `AdmFundShare`: automática quando moedas diferem
4. **Batch não falha tudo**: job captura erro por fundo individualmente
5. **Cache Grails**: `PerformitGrails.new.clear_cache` é chamado após acato de zeragem
6. **WebSocket obrigatório para batch**: sem WS conectado, feedback em tempo real não chega
7. **`openingService.markDirty`**: sempre chamado após criar/atualizar Quote no Grails
