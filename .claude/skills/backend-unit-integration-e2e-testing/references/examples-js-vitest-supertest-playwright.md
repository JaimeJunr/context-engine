# Exemplo JavaScript/TypeScript — Vitest, Supertest, Playwright

Stack de referência para backend Node: **unitário e integração HTTP** no mesmo runner (Vitest), **E2E** em job separado com Playwright.

## Dependências (exemplo)

```bash
npm add -D vitest supertest @types/supertest @playwright/test
```

## Unitário — Vitest (`src/auth/validate.test.ts`)

Foco em funções puras ou use cases com dependências mockadas.

```ts
import { describe, it, expect, vi } from 'vitest';
import { validateToken } from './validate';

describe('validateToken', () => {
  it('rejeita token vazio', () => {
    expect(() => validateToken('')).toThrow('token obrigatório');
  });
});
```

## Integração HTTP — Vitest + Supertest

App exportada para teste sem escutar porta real (padrão com Express/Fastify adaptável).

```ts
// app.ts — exportar instância sem listen()
import express from 'express';
export function createApp() {
  const app = express();
  app.use(express.json());
  app.get('/health', (_req, res) => res.status(200).json({ ok: true }));
  return app;
}
```

```ts
// health.integration.test.ts
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import request from 'supertest';
import { createApp } from './app';

describe('GET /health', () => {
  const app = createApp();

  it('retorna 200', async () => {
    const res = await request(app).get('/health').expect(200);
    expect(res.body).toEqual({ ok: true });
  });
});
```

Configurar Vitest com `testMatch` ou projetos separados se quiser isolar integração mais lenta:

```ts
// vitest.config.ts (trecho)
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    projects: [
      { test: { name: 'unit', include: ['**/*.test.ts'], exclude: ['**/*.integration.test.ts'] } },
      { test: { name: 'integration', include: ['**/*.integration.test.ts'], testTimeout: 30_000 } },
    ],
  },
});
```

## E2E — Playwright (somente API)

Útil quando não há UI ou o foco é o backend publicado.

```ts
// e2e/api.spec.ts
import { test, expect } from '@playwright/test';

test.describe('API black-box', () => {
  test('health na instância subida', async ({ request }) => {
    const base = process.env.E2E_API_URL ?? 'http://127.0.0.1:3000';
    const res = await request.get(`${base}/health`);
    expect(res.ok()).toBeTruthy();
    await expect(res).toBeOK();
    const body = await res.json();
    expect(body).toMatchObject({ ok: true });
  });
});
```

`playwright.config.ts`: usar `webServer` para subir `node dist/server.js` ou `npm run start` antes dos testes, ou documentar URL externa em CI.

## E2E — Playwright (navegador + backend)

Quando o backend só é exercitado pela UI, o spec vive no front ou em repo e2e compartilhado; o princípio é o mesmo: serviço real, dados de teste, asserções em critérios de aceite.

## Scripts `package.json` (exemplo)

```json
{
  "scripts": {
    "test:unit": "vitest run --project unit",
    "test:integration": "vitest run --project integration",
    "test:e2e": "playwright test"
  }
}
```
