# Exemplo Java — unitário, integração e E2E (Playwright)

Ilustração **JVM comum**; ajustar para Quarkus/Micronaut se o projeto usar.

## Unitário — JUnit 5 + Mockito

```java
import org.junit.jupiter.api.Test;
import org.mockito.Mockito;

import static org.assertj.core.api.Assertions.assertThatThrownBy;

class OrderValidatorTest {

  @Test
  void rejectsEmptyId() {
    assertThatThrownBy(() -> new OrderValidator().validate(""))
      .isInstanceOf(IllegalArgumentException.class);
  }
}
```

## Integração — Spring Boot Test + MockMvc (HTTP in-process)

```java
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.autoconfigure.web.servlet.AutoConfigureMockMvc;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.test.web.servlet.MockMvc;

import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.status;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.jsonPath;

@SpringBootTest
@AutoConfigureMockMvc
class HealthIntegrationTest {

  @Autowired MockMvc mvc;

  @Test
  void healthOk() throws Exception {
    mvc.perform(get("/health"))
      .andExpect(status().isOk())
      .andExpect(jsonPath("$.ok").value(true));
  }
}
```

## Integração — REST Assured (JAR em execução ou porta aleatória)

Quando preferir cliente HTTP real contra `TestRestTemplate` ou `@LocalServerPort`:

```java
import io.restassured.RestAssured;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.test.web.server.LocalServerPort;

import static org.hamcrest.Matchers.equalTo;

@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
class HealthRestAssuredTest {

  @LocalServerPort int port;

  @BeforeEach
  void setup() {
    RestAssured.baseURI = "http://localhost";
    RestAssured.port = port;
  }

  @Test
  void health() {
    RestAssured.when().get("/health")
      .then()
      .statusCode(200)
      .body("ok", equalTo(true));
  }
}
```

## Integração — Testcontainers (Postgres, Kafka, etc.)

Usar quando a dúvida é **SQL real**, migrações ou driver — não mockar o banco nessa camada.

Padrão: `@Testcontainers` + `@Container` estático, `DynamicPropertySource` para Spring DataSource. (Detalhes variam por versão; seguir documentação oficial do Testcontainers e do framework.)

## E2E — Playwright

Duas abordagens alinhadas à skill:

1. **Playwright Java** (`com.microsoft.playwright:playwright`): abrir browser ou usar `APIRequest` no mesmo processo JVM que dispara o JAR — útil quando o time padroniza Java end-to-end.
2. **Playwright Node** em job CI: sobe o artefato Spring Boot (`java -jar`), define `E2E_API_URL`, roda `npx playwright test` com specs só de `request` — mesma ideia do exemplo JS.

Escolher uma via e documentar no README/CI; evitar duplicar os dois sem necessidade.

### Esboço Playwright Java (API)

```java
import com.microsoft.playwright.*;
import org.junit.jupiter.api.*;

public class ApiE2E {
  static Playwright pw;
  static APIRequestContext request;

  @BeforeAll
  static void before() {
    pw = Playwright.create();
    String base = System.getenv().getOrDefault("E2E_API_URL", "http://127.0.0.1:8080");
    request = pw.request().newContext(new APIRequest.NewContextOptions().setBaseURL(base));
  }

  @AfterAll
  static void after() {
    pw.close();
  }

  @Test
  void health() {
    APIResponse res = request.get("/health");
    Assertions.assertTrue(res.ok());
  }
}
```

(Antes: dependência Maven `com.microsoft.playwright:playwright` e instalação de browsers conforme documentação Playwright Java.)

## Agnosticidade

Os **papéis** são os mesmos da skill principal: Mockito/JUnit para unidade, Boot+MockMvc/REST Assured+Testcontainers para integração, Playwright para E2E black-box. Em projetos **sem Spring**, usar equivalentes (Helidon, Quarkus `@QuarkusTest`, etc.) mantendo a separação de camadas.
