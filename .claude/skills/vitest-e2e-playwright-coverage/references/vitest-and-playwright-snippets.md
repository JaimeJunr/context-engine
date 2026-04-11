# Snippets de referência — Vitest, cobertura e E2E com Playwright

Conteúdo para carregar quando for necessário copiar configuração ou padrões concretos.

## Vitest — `vitest.config.ts` (cobertura mínima 75%)

```ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react'; // se for React; omitir em vanilla

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom', // ou 'happy-dom' conforme o projeto
    setupFiles: ['./src/test/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json-summary', 'html'],
      reportsDirectory: './coverage',
      thresholds: {
        lines: 75,
        functions: 75,
        branches: 75,
        statements: 75,
      },
      exclude: [
        '**/*.config.*',
        '**/dist/**',
        '**/e2e/**',
        '**/*.e2e.*',
        '**/mocks/**',
      ],
    },
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
  },
});
```

## Playwright — `playwright.config.ts` (E2E)

```ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: process.env.PLAYWRIGHT_TEST_BASE_URL ?? 'http://127.0.0.1:5173',
    trace: 'on-first-retry',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: {
    command: 'npm run dev',
    url: 'http://127.0.0.1:5173',
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
```

## Script npm sugerido

```json
{
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest",
    "test:coverage": "vitest run --coverage",
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui"
  }
}
```

## Setup Testing Library (React) — `src/test/setup.ts`

```ts
import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

afterEach(() => cleanup());
```

## E2E — spec Playwright (`e2e/example.spec.ts`)

```ts
import { test, expect } from '@playwright/test';

test('carrega a página inicial', async ({ page }) => {
  await page.goto('/');
  await expect(page).toHaveTitle(/.+/);
});
```

## CI — falhar se cobertura abaixo do limite

O Vitest com `thresholds` encerra com código de saída ≠ 0 quando a meta não é atingida. Garantir que o job rode `vitest run --coverage` (ou `npm run test:coverage`). Para E2E, `playwright test` em job separado ou estágio seguinte após build/preview.

## Alternativa ao runner `@playwright/test`

Se o repositório **precisa** acoplar E2E ao Vitest, usar projeto/config Vitest só para esses arquivos com `testTimeout` alto; ainda assim instalar browsers com Playwright. O padrão recomendado é manter **`playwright test`** para E2E e Vitest para unidade/componente.

## Cobertura E2E

Métrica de cobertura de **unidade/componente** permanece no Vitest (75%). Cobertura de código via instrumentação no browser (Playwright + Istanbul) é opcional e **não substitui** os thresholds do Vitest sobre o código-fonte da app.
