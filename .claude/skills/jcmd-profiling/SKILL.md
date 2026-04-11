---
name: jcmd-profiling
description: Use when the user needs to profile a running JVM/app using `jcmd` and `Thread.print -l` to capture thread dumps during latency, then analyze repeated "hot stacks" (DB/JDBC/ORM/locks/render/CPU symptoms) and propose evidence-based next steps. Use when the user mentions `jcmd`, `Thread.print`, JVM profiling, thread dumps, or "stack quente".
---

# JCMD Profiling (JVM Thread Dumps)

## Objetivo

- Coletar evidências durante um período de lentidão (stack traces repetidos).
- Identificar gargalo provável a partir de padrões ("hot stacks") e do estado das threads (RUNNABLE vs BLOCKED vs WAITING).
- Direcionar a próxima investigação com hipóteses testáveis (ex.: DB/JDBC, locks/contended monitors, pool starvation, código CPU-bound).

## Quando usar

Use esta skill quando:

- O usuário pedir "profiling" ou "capturar stacks" de uma aplicação Java.
- O usuário mencionar `jcmd`, `Thread.print -l`, "thread dump", "stack quente" ou lentidão intermitente/por endpoint.

## Pré-requisitos (dados que você deve pedir)

1. `PID` do processo Java/JVM.
2. Janela de captura (padrão recomendado: 20-60s, com amostra a cada ~1s).
3. `jcmd` correto:
   - Ideal: o `jcmd` do mesmo JDK que está em uso pela JVM (ou no mesmo ambiente/mesma versão).
   - Se não souber: peça para o usuário rodar `which jcmd` e/ou informar onde está o `jcmd`.
4. Se o `PID` não estiver claro: confirmar se `jcmd -l` lista a JVM local e mostra o `pid`/main class.

## Fase 1: Descobrir o `PID` (sem hardcode)

Se o usuário não fornecer o `PID`, sugira descobrir pelo processo "principal" da aplicação:

- Use um filtro por nome do serviço e/ou caminho do jar/classes.
- Exemplo (ajuste para o seu app): `ps aux | rg -i 'java|<nome-do-serviço>'`

Se a aplicação for forkada (ex.: Tomcat fork):

- Garanta que você está usando o `PID` do processo "Forked" (nunca hardcode).

Alternativa (quando `jcmd` consegue "enxergar" a JVM local):

- Rode `jcmd -l` (ou `"$JCMD" -l`) para listar alvos.
- Use o `pid` retornado para o comando `Thread.print -l`.

## Fase 2: Coletar evidência com `jcmd Thread.print -l`

### Comandos (template)

Use algo muito próximo do fluxo abaixo; substitua `PID` e `JCMD`:

```bash
PID="<PID_DA_JVM>"
JCMD="<CAMINHO_PARA_jcmd>" # opcional: se `which jcmd` funcionar, pode usar apenas `jcmd`
OUT_DIR="/tmp/jcmd-profiling"
OUT="$OUT_DIR/threads-$(date +%Y%m%d-%H%M%S).txt"

mkdir -p "$OUT_DIR"

# (opcional) informações rápidas da VM
"$JCMD" "$PID" VM.system_properties >> "$OUT" 2>&1 || true

for i in $(seq 1 40); do
  echo "\n===== SAMPLE $i $(date) =====" >> "$OUT"
  "$JCMD" "$PID" Thread.print -l >> "$OUT" 2>&1 || true
  sleep 1
done

echo "DONE $OUT"
```

### Dicas importantes

- Execute com permissões compatíveis com o usuário do processo (muitas vezes "mesmo usuário" ou via `sudo`).
- Não misture amostras de processos diferentes (confirmar `PID` antes de iniciar).
- Se houver muita lentidão mas poucos "picos", faça a captura alinhada ao momento ruim.

## Fase 3: Analisar "hot stacks" (padrões repetidos)

### O que procurar primeiro

1. **Threads travadas em DB/JDBC/ORM**
   - Exemplos comuns de padrões:
     - JDBC/SQL: `java.sql`, nomes de drivers (ex.: `com.mysql.*`, `org.postgresql.*`)
     - ORM: `org.hibernate.*`, `javax.persistence.*`
     - Pools: `com.zaxxer.hikari.*`
2. **Threads travadas em render/template**
   - `gsp`, `render`, `GroovyPagesServlet` (se for Grails/Groovy)
   - (genérico) encontre frames de "view layer" do seu framework
3. **Locks/contended monitors**
   - `java.util.concurrent.locks.*`
   - `Object.wait`, `sun.misc.Unsafe.park`, `Thread.sleep`
4. **Threads RUNNABLE que se repetem**
   - Se muitas threads ficam RUNNABLE repetindo os mesmos métodos, existe chance de CPU-bound.
   - Thread dump sozinho não mede CPU diretamente, mas o padrão ajuda.

### Como interpretar RUNNABLE vs BLOCKED/WAITING

No dump, procure o estado da thread (normalmente aparece como `java.lang.Thread.State: ...`):

- `RUNNABLE`: a thread está executando ou pronta para executar; padrões repetidos sugerem CPU-bound, trabalho constante ou loops.
- `BLOCKED`: provável contenção de monitor/lock (threads tentando sincronizar).
- `WAITING` / `TIMED_WAITING`: provável espera por I/O, timer, pool/condição, ou algum mecanismo de sincronização.

### Filtros práticos com `rg`

Após a captura, filtre pelo arquivo com termos relacionados ao seu stack.
Exemplo (genérico; adapte ao que você usa):

```bash
rg -n "java\.sql|org\.hibernate|javax\.persistence|com\.mysql\.jdbc|org\.postgresql|com\..zaxxer\.hikari|java\.util\.concurrent\.locks|Object\.wait|sun\.misc\.Unsafe\.park" "$OUT"
```

### Como concluir "stack quente"

Trate como "quente" quando:

- O mesmo grupo de frames aparece repetidamente em múltiplas amostras.
- O "corredor" (caminho do stack) até o ponto final (DB/lock/render) permanece consistente.
- O mesmo conjunto aparece em threads que pertencem aos mesmos tipos de trabalho (mesmas names de thread/headers do framework).

## Fase 4: Formular hipóteses e próximos passos (sem sair chutando)

Estruture assim:

1. **Hipótese principal** (1 frase): "A lentidão parece dominada por X porque Y aparece repetidamente".
2. **Evidência do dump**: cite 2–5 frames/linhas representativas.
3. **O que confirmar em seguida** (um passo por vez):
   - Se DB/JDBC: confirmar query específica (via logs/slow query/trace) e checar N+1/batching.
   - Se ORM: validar tamanho de fetch/associações e estratégias (lazy/eager), além de caching.
   - Se locks: identificar qual recurso (pool/monitor) e qual thread está segurando.
   - Se render: checar templates e chamadas repetidas por request.

## Fase 5: Validar melhoria (antes vs depois)

Depois de uma mudança (ex.: batching, cache, ajustar pool/locks, otimização de query), repita:

- A mesma janela de captura (mesma duração, mesmo intervalo).
- Compare:
  - Queda de frames "quentes" do gargalo suspeito.
  - Menor tempo em BLOCKED/WAITING (se o problema era lock/IO).
  - Menos recorrência do mesmo stack em novas amostras.

## Template de resposta que esta skill deve produzir

Quando a skill for usada, responda com:

- `Arquivo de evidência`: caminho do `OUT`.
- `Top padrões repetidos`: 3–6 grupos de stack (com exemplos de frames).
- `Estado das threads`: predominância (RUNNABLE vs BLOCKED vs WAITING) quando visível.
- `Gargalo provável`: DB/JDBC, ORM, locks, render ou CPU-bound.
- `Próximo passo de verificação`: 1 ação concreta para confirmar a hipótese.
