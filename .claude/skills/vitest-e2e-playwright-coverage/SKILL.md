---
name: vitest-e2e-playwright-coverage
description: >-
  Use when adding or finishing frontend tests with Vitest, browser E2E automation
  (Playwright), or enforcing coverage in CI; when coverage is below 75%, tests are
  missing for new components, E2E is flaky or absent, vitest.config needs thresholds,
  jsdom/happy-dom setup, Testing Library, Playwright fixtures/timeouts, or the user
  mentions unit tests, integração visual, cobertura, coverage, relatório html lcov,
  test:e2e, playwright.config, or smoke flows in the browser.
---

# Testes de frontend: Vitest, E2E com Playwright e cobertura ≥ 75%

## Princípio obrigatório

**Cobertura global mínima: 75%** em linhas, funções, branches e statements (ajustar apenas com decisão explícita de produto/arquitetura documentada — não por conveniência). E2E **complementa** testes de unidade/componente; não substitui a obrigação de cobertura no código da aplicação.

Antes de implementar: inspecionar o repositório (`package.json`, `vitest.config.*`, `playwright.config.*`, pastas `e2e/`, `tests/e2e/`, `src/**/*.test.*`) e alinhar à stack já adotada.

## Referência rápida

| Camada | Ferramenta | Foco |
|--------|------------|------|
| Unidade / componente | Vitest + (opcional) Testing Library | Lógica, props, eventos, hooks |
| Cobertura | `@vitest/coverage-v8` (ou `istanbul`) | Thresholds no config |
| E2E / browser | `@playwright/test` (recomendado) | Fluxos críticos, integração real do bundle |
| Runner E2E | `playwright test` ou projeto Vitest dedicado | Timeouts e `webServer` no config; não poluir suite unitária |

Se o repositório ainda usa **Puppeteer**, a mesma disciplina de E2E aplica-se; o padrão desta skill é **Playwright**. O requisito de **75% de cobertura** continua na suite Vitest principal.

## Quando começar do zero

1. Instalar dependências: `vitest`, `@vitest/coverage-v8`, ambiente DOM (`jsdom` ou `happy-dom`), e `@playwright/test` para E2E (`npx playwright install` para browsers em CI/local).
2. Criar ou estender `vitest.config.ts` com `coverage.thresholds` em **75** para `lines`, `functions`, `branches`, `statements`.
3. Definir `coverage.exclude` para não distorcer métricas (configs, builds, pastas só E2E, mocks globais).
4. Adicionar `setupFiles` se usar Testing Library (`jest-dom`/vitest).
5. Adicionar `playwright.config.ts`: `baseURL`, `webServer` (comando de dev/preview + URL), `timeout`/`expect.timeout` conforme o fluxo.
6. Organizar specs E2E (ex.: `e2e/**/*.spec.ts`) e scripts em `package.json`: `test`, `test:coverage`, `test:e2e` → `playwright test` (ou variação do monorepo).

Detalhes e snippets: `references/vitest-and-playwright-snippets.md`.

## Quando completar testes existentes

1. Rodar `vitest run --coverage` e identificar arquivos abaixo do threshold ou não cobertos.
2. Priorizar: (a) código novo ou alterado em PR, (b) módulos de domínio crítico, (c) UI com lógica condicional.
3. Preferir testes estáveis: dados explícitos, evitar dependência de timers reais quando der para usar `vi.useFakeTimers()` com critério.
4. Para lacunas difíceis de unitar (canvas, third-party), isolar em módulos testáveis ou limitar exclusões de cobertura com comentário justificado (`istanbul`/`v8` ignore) — **exceção rara**, não padrão.
5. E2E: cobrir poucos fluxos **críticos** bem escolhidos; evitar duplicar tudo que já está coberto em componente.

## Regras — Vitest

- Usar convenções do repo para nome e local dos testes (`*.test.ts`, `__tests__/`, etc.).
- Ambiente: `jsdom`/`happy-dom` para componentes; não usar browser real na suite principal salvo projeto separado.
- Mocks: `vi.mock` com caminhos estáveis; resetar estado em `beforeEach`/`afterEach` quando houver singletons ou módulos com cache.
- Asserções assíncronas: sempre `await findBy*` / `waitFor` (Testing Library) em vez de `getBy*` imediato quando o DOM atualiza após efeito.

## Regras — E2E com Playwright

- Subir a aplicação de forma reprodutível: preferir `webServer` em `playwright.config.ts` ou documentar `BASE_URL` / `PLAYWRIGHT_TEST_BASE_URL`.
- Seletores: preferir `getByRole`, `getByTestId` e rótulos acessíveis; evitar classes CSS só de estilo.
- Sincronização: usar auto-waiting do Playwright e asserções `expect(locator)` com retry; evitar `waitForTimeout` fixo como estratégia principal.
- `headless: true` em CI; `headed` / `debug` só para depuração local.
- Isolar E2E da suite unitária (comandos e configs distintos) para não multiplicar tempo de feedback.
- Gravação e rastreio: `npx playwright codegen` e trace sob demanda para falhas intermitentes.

## Critério de pronto

- [ ] `vitest run --coverage` passa com thresholds ≥ **75%** (ou justificativa escrita e aprovada para exceção pontual).
- [ ] Fluxos E2E críticos passam com `playwright test` (ou script documentado), incluindo como o servidor sobe (`webServer` ou passo explícito).
- [ ] Nenhum teste novo depende de ordem global frágil ou estado compartilhado não resetado.

## Erros comuns

- Threshold só em `lines` — as outras métricas ficam em 0% ou baixas e o relatório engana.
- Incluir `node_modules` ou `dist` na cobertura sem excluir.
- E2E sem servidor: `goto` falha de forma obscura; documentar pré-requisito ou usar `webServer`.
- Mocks que nunca são `vi.clearAllMocks` / `mockRestore` entre testes e vazam comportamento.
- Centenas de E2E lentos duplicando cobertura de componente — manter pirâmide equilibrada.

## Sinais de alerta — parar e corrigir

- Propor abaixar o mínimo de **75%** “só desta vez” sem decisão registrada.
- Excluir grandes pastas de `coverage.exclude` só para passar no CI.
- Substituir testes de unidade por E2E “porque é mais real” — E2E não cobre linhas do bundle da mesma forma.
- `page.waitForTimeout` / sleeps fixos como solução principal de sincronização.

## Fluxo de decisão (alto nível)

```text
Mudança em UI ou lógica de frontend
        │
        ├─► Comportamento isolável? → Vitest (+ Testing Library se React/Vue compatível)
        │
        ├─► Integração multi-página / fluxo real crítico? → Playwright (@playwright/test)
        │
        └─► Cobertura após implementação < 75%? → ampliar testes unitários/componente até passar
```
