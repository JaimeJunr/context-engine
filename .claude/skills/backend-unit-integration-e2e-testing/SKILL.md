---
name: backend-unit-integration-e2e-testing
description: >-
  Use when defining or implementing backend test strategy across languages; when unit,
  integration, or E2E layers are missing, duplicated, or flaky; when choosing tools for
  HTTP APIs, databases, message brokers, or black-box flows; when wiring Playwright
  (browser or API requests) against local, Docker, or deployed services; when CI needs
  separated test jobs, Testcontainers, Supertest, REST Assured, or contract tests; or
  when the user mentions testes de integração, suite lenta, smoke backend, or
  cobertura além da aplicação web.
---

# Backend: testes unitários, integração e E2E (stack agnóstica + Playwright)

## Princípio

Separar **três camadas** com propósitos distintos. A linguagem e o framework mudam; as responsabilidades não.

| Camada         | Objetivo                                                                               | O que valida                                         | Velocidade   |
| -------------- | -------------------------------------------------------------------------------------- | ---------------------------------------------------- | ------------ |
| **Unitária**   | Unidades isoladas (funções, classes, handlers) com dependências substituídas           | Regras de domínio, parsing, branches                 | Muito rápida |
| **Integração** | Limites reais entre o serviço e infraestrutura (DB, fila, HTTP in-process, filesystem) | SQL, migrations, serializers, adapters               | Média        |
| **E2E**        | Sistema como caixa-preta: rede real (ou próxima de produção), ambiente subido          | Contratos HTTP, fluxos críticos, regressão de deploy | Mais lenta   |

**Playwright** entra na camada **E2E**: via **navegador** (fluxo usuário batendo no backend) ou via **`APIRequestContext`** (chamadas HTTP sem UI). Não substitui testes unitários nem integração de repositório/fila.

Antes de implementar: ler `package.json` / build (`pom.xml`, `build.gradle`, `go.mod`, etc.), como o servidor sobe, e padrões já usados no repositório.

## Stack agnóstica — como escolher ferramentas

- **Unitário**: runner e assertions nativos da linguagem (ex.: Vitest/Jest, JUnit 5, `go test`, `pytest`, .NET xUnit).
- **Integração HTTP**: cliente in-process ou contra app de teste (ex.: Supertest, Spring `MockMvc`, `httptest` em Go, `rack-test`).
- **Integração persistência/fila**: banco real efêmero (Testcontainers, SQLite in-memory, embedded), ou serviço fake só quando o contrato for estável.
- **E2E**: **Playwright** (Node com `@playwright/test` é o caso de referência na organização; [Playwright para Java](https://playwright.dev/java/) existe quando o time padroniza JVM). Alternativas (Karate, REST Assured puro, k6) só se o repo já as adota.

Definir **comandos CI separados** (ex.: `test:unit`, `test:integration`, `test:e2e`) para falhar cedo e em paralelo quando fizer sentido.

## Quando começar do zero

1. Inventariar entrypoints (HTTP, jobs, CLI) e dependências externas.
2. Criar pastas ou sufixos claros (`*.test.ts`, `src/test/java/...`, `tests/integration/`).
3. Configurar unitário com mocks nos limites; sem rede externa por padrão.
4. Configurar integração com DB/fila real ou container; variáveis de ambiente só para teste.
5. Configurar E2E: subir app (script, `webServer` do Playwright, Compose) e apontar `baseURL` ou URL absoluta.
6. Documentar como rodar cada camada localmente e no CI.

## Quando completar uma suite existente

1. Mapear buracos por camada (não “mais testes genéricos”).
2. Priorizar caminhos críticos de negócio e regressões recentes.
3. Evitar E2E para cada ramo — isso pertence a unitário/integração.
4. Estabilizar: dados determinísticos, isolamento, teardown de recursos.
5. Manter Playwright E2E enxuto; usar `request` fixture para APIs quando não houver UI.

## Regras — unitário

- Sem I/O de rede ou disco acidental; mock/stub em fronteiras.
- Um comportamento principal por teste; nomes descrevem cenário.
- Relógio e random: controlar com APIs da linguagem ou biblioteca de teste.

## Regras — integração

- Validar contratos que unitário não vê (tipos SQL, headers, serialização).
- Ambiente isolado por execução ou suite quando possível (schema limpo, topics dedicados).
- Timeouts explícitos em esperas; evitar sleep fixo.

## Regras — E2E com Playwright

- **Com UI**: fluxos que exercitam o backend de ponta a ponta (login, checkout crítico).
- **Só API**: `request.newContext()` + `get`/`post` com asserções de status e corpo; baseURL do serviço backend.
- Dados: usuários/fixtures criados via API ou seed idempotente.
- CI: instalar browsers (`npx playwright install --with-deps` no job) quando usar browser; job API-only pode ser mais leve.

## Erros comuns

- E2E cobrindo validação de campo que deveria ser unitário.
- Integração acoplada a ordem global de testes ou estado compartilhado.
- Playwright acoplado a implementação interna (importar módulos do servidor no teste E2E).
- Um único comando `test` que mistura tudo e torna feedback lento.

## Sinais de alerta

- “Só E2E porque é mais confiável” — pirâmide invertida gera fragilidade e custo.
- Desligar testes integração em CI “porque flakam” sem corrigir isolamento ou dados.
- Mockar o banco em E2E e chamar de integração.

## Exemplos por linguagem

- **JavaScript/TypeScript (referência explícita):** Vitest (unitário + integração in-process), Supertest (HTTP contra app de teste), Playwright (E2E API ou browser) — ver `references/examples-js-vitest-supertest-playwright.md`.
- **Java (referência explícita):** JUnit 5 + Mockito (unitário), Spring Boot Test / REST Assured / Testcontainers (integração), Playwright Java ou Playwright Node apontando para o JAR subido — ver `references/examples-java-backend-testing.md`.

Para outras linguagens, transportar os **mesmos papéis** (runner, HTTP in-process ou container, E2E black-box com Playwright ou ferramenta já padronizada no repo).

## Fluxo de decisão (alto nível)

```text
Nova regra ou endpoint
        │
        ├─► Lógica pura / ramos ──────────────► teste unitário
        │
        ├─► Persistência, fila, HTTP real ───► integração
        │
        └─► Contrato público ou jornada crítica ► E2E (Playwright: API ou browser)
```
