---
name: tela-k-reconcile
description: >
  Skill especializada na Tela K (Conciliação) do Performit. Use quando o usuário perguntar sobre,
  debugar ou implementar funcionalidades relacionadas à tela de conciliação (#/reconcile),
  fluxos de "Acatar fundos de zeragem", "Acatar todos os fundos", "Abrir Dia", batch de reconciliação,
  controllers Rails/Grails da conciliação, jobs de reconciliação, ou qualquer coisa relacionada
  à Tela K. Também use ao investigar bugs como botões sem resposta na conciliação.
---

# Tela K – Skill de Conciliação

Use esta skill ao trabalhar com qualquer aspecto da **Tela K (Conciliação)** do Performit.

## Arquitetura Rápida

A Tela K é uma SPA AngularJS servida pelo Rails (`http://localhost:3000/app#/reconcile`) com backend **híbrido Rails + Grails**:

- **Rails (porta 3000)**: proxy, API `/api/*`, jobs Sidekiq
- **Grails (porta 8080, via proxy `/g/*`)**: regras de negócio, cotas, posições, contábil

Veja o mapa completo em `references/fluxo-completo.md`.

---

## Fluxos Principais

### 1. "Acatar fundos de zeragem" → **100% Rails**

```
position.html.slim → acceptAllMoneyfunds()
  → POST /api/reconcile/moneyfunds/:fundId/:date
  → Api::Reconcile::MoneyfundsController#update
  → AdministratorPortfolio.new(fund, date).create_adm_portfolio
    (tenta: Pactual → Anbima → Mellon → Anbima5 → Anbima5galgo)
  → Reconcile::AdmFundShare.new(user, fund, adm_portfolio).execute
    (diff de qtd por ISIN → CustomSecurityTransaction, create_or_update_quote)
  → { success: true } → loadSystemPositions(), broadcast 'cash-dirty'
```

### 2. "Acatar todos os fundos" → **100% Grails**

```
position.html.slim → acceptAllFundPrices() → createPriceRequests(funds)
  → POST /g/quote/rest  [{ fund, day, close, security }, ...]
  → Grails: QuoteController#rest → processQuoteJson(item)
    (find/create Quote, openingService.markDirty)
  → { ok: true, processed: N } → compare(), broadcast 'price-changed'
```

### 3. "Abrir Dia" → **Rails (batch) → Grails**

```
general-actions.html.slim → open()
  → POST /api/fund_reconcile_batch/process_batch
    { batch_action: 'open_day', fund_ids: [...], date, ws_id }
  → FundReconcileBatchJob (Sidekiq)
    → execute_open_day(fund)
    → PerformitGrails.new.open_day(fund.id, date)
    → Grails: DailyService.openDay(date, fund, user, ...)
  → Broadcast ActionCable: fund_reconcile_ws_#{ws_id}
```

### 4. Outras ações Batch

`POST /api/fund_reconcile_batch/process_batch` com `batch_action`:

| batch_action | O que faz |
|---|---|
| `open_day` | Abre o dia no Grails |
| `calculate_quote` | Calcula NAV no Grails |
| `accept_quote` | Aceita cota (atualiza Quote Rails) |
| `share_quote` | Compartilha cota |
| `mark_conciliated` | Marca conciliado no Grails |
| `unmark_conciliated` | Desmarca conciliado no Grails |

---

## Mapa de Arquivos Críticos

### Frontend (Rails assets – AngularJS)

| Arquivo | Responsabilidade |
|---|---|
| `app/assets/templates/reconcile.html.slim` | Template principal (`ng-controller="Reconcile"`) |
| `app/assets/templates/reconcile/position.html.slim` | Tabelas de posição + botões Acatar |
| `app/assets/templates/reconcile/general-actions.html.slim` | Botões Abrir Dia, Marcar Conciliado, Aprovar Cota |
| `app/assets/templates/reconcile-new/position.html.slim` | Versão alternativa (controlada por `Setting.reconcile_session`) |
| `app/assets/javascripts/app/reconcile/reconcile.controller.js.coffee` | Controller `Reconcile` |
| `app/assets/javascripts/app/reconcile/position.controller.js.coffee` | Controller `Position`: `acceptAllMoneyfunds`, `acceptAllFundPrices` |
| `app/assets/javascripts/app/reconcile/general.controller.js.coffee` | Controller `General`: `open`, `toggleMark`, `toggleApprove` |

### Backend Rails

| Arquivo | Responsabilidade |
|---|---|
| `app/controllers/api/reconcile/moneyfunds_controller.rb` | `update` – acatar fundos de zeragem |
| `app/controllers/api/fund_reconcile_batch_controller.rb` | `merged_data`, `process_batch`, `latest_opening_date` |
| `app/controllers/api/open_day_controller.rb` | `do_open` – abrir dia de fundo único |
| `app/models/administrator_portfolio.rb` | Factory de leitores de carteira ADM |
| `app/models/reconcile/adm_fund_share.rb` | Executa acato de zeragem (diffs + Quote + Transaction) |
| `app/services/fund_reconcile_service.rb` | `build_merged_fund_reconcile_data` (dados da tela batch) |
| `app/jobs/fund_reconcile_batch_job.rb` | Job Sidekiq para ações em lote |

### Backend Grails

| Arquivo | Responsabilidade |
|---|---|
| `grails-app/controllers/.../QuoteController.groovy` | `rest` (POST) – acatar preços em lote |
| `grails-app/controllers/.../FundTransactionController.groovy` | `reconcile` – cotizar fundos |
| `grails-app/controllers/.../AccountingController.groovy` | `index`, `bySource` |
| `grails-app/controllers/.../OpeningController.groovy` | `recalculate`, `reopenFunds` |
| `grails-app/controllers/.../ReconcileController.groovy` | Cash: `entries`, `save`, `update`, `listTrades`, `acceptFx`, `acceptFee` |
| `grails-app/services/.../DailyService.groovy` | `openDay`, `openFunds` – abre o dia com validações |
| `grails-app/services/.../OpeningService.groovy` | `markDirty` – invalida cache de abertura |
| `grails-app/domain/.../CashEntry.groovy` | Lançamento de caixa |

---

## Debugging

### Botões sem resposta (ex.: CAP-8183)

**"Acatar fundos de zeragem"** (Rails):
1. Verificar resposta/status de `POST /api/reconcile/moneyfunds/:fund_id/:date`
2. Checar `AdministratorPortfolio.create_adm_portfolio` – qual leitor é usado?
3. Inspecionar `Reconcile::AdmFundShare#execute` – exceções ou transações não criadas?

**"Acatar todos os fundos"** (Grails):
1. Verificar se `POST /g/quote/rest` chega ao Grails (status 200?)
2. Checar `Quote.withTransaction / processQuoteJson` – exceção silenciosa?
3. Verificar CORS/proxy e formato do body (`fund.id`, `day`, `close`, `security.symbol`)

**"Abrir Dia"** (Batch):
1. Verificar se job foi enfileirado: `POST /api/fund_reconcile_batch/process_batch`
2. Checar logs do Sidekiq – job falhou?
3. WebSocket conectado? `fund_reconcile_ws_#{ws_id}` – ActionCable

### Comportamentos importantes

- `AdministratorPortfolio` tenta 5 leitores em sequência; se nenhum funcionar lança exceção
- `Reconcile::AdmFundShare` converte moeda automaticamente se currencies diferentes
- `FundReconcileBatchJob` não falha tudo se um fundo falha – captura erro por fundo
- `DailyService.openDay` valida fins de semana, feriados e sequência de abertura
- `Setting.reconcile_session` controla qual conjunto de templates é usado (`reconcile/` vs `reconcile-new/`)

---

## Endpoints de Referência Rápida

| Fluxo | Método | Endpoint |
|---|---|---|
| Acatar fundos de zeragem | POST | `/api/reconcile/moneyfunds/:fund_id/:date` |
| Acatar todos os fundos | POST | `/g/quote/rest` |
| Abrir Dia (lote) | POST | `/api/fund_reconcile_batch/process_batch` |
| Abrir Dia (único) | POST | `/api/open_day/do_open` |
| Dados batch | GET | `/api/fund_reconcile_batch/merged_data?date=` |
| Última data abertura | GET | `/api/fund_reconcile_batch/latest_opening_date` |
| Cotizar fundos | POST | `/g/fundTransaction/reconcile` |
| Liquidar contábil | POST | `/g/accountingEntry/liquidateAjax/:id` |

---

## Referências

- Fluxo completo com diagramas: `references/fluxo-completo.md`
- Doc original do projeto: `docs/tela-k-fluxo-reconcile.md`
