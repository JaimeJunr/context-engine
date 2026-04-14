# ctx — Mapa de Repositório + Busca Semântica

**Objetivo**: Economizar tokens (60-90%) e encontrar código rapidamente em repos grandes.

**Dois modos principais**:
1. **`ctx map`** — Entender estrutura/fluxo de código
2. **`ctx search`** — Buscar padrões em documentação

---

## 🎯 FLUXO PRÁTICO: Identificar Module em Projeto Grande

### Fase 1: Orientação Rápida (Topologia)

```bash
# Entender arquitetura geral
ctx map --title "entender arquitetura geral" --dirs . --max-tokens 4096
```

**O que você vê:**
- Diretórios principais (src, app, lib, tests)
- Padrões de arquitetura (MVC, layered, microservices)
- Arquivos mais importantes por relevância

**Quando usar:**
- Primeira vez no projeto
- Precisa entender fluxo geral
- Vai refatorar um módulo inteiro

---

### Fase 2: Zoom em Área Específica (Personalized PageRank)

```bash
# Exemplo: Preciso modificar autenticação em MVC
ctx map --title "fluxo de autenticação" --dirs . --seeds src/auth,src/middleware,src/controllers
```

**O que acontece:**
- Prioriza arquivos próximos aos `seeds` (autenticação)
- Mostra Controller → Service → Model
- Revela dependências de forma inteligente

**Exemplo output:**
```
src/controllers/AuthController.ts
src/services/AuthService.ts
src/models/User.ts
src/middleware/AuthMiddleware.ts
src/utils/tokenService.ts
```

---

### Fase 3: Localizar Arquivo Específico (Grep/Find)

```bash
# Se busco o UserController:
grep -r "class UserController" src/controllers
# → Acha: src/controllers/UserController.ts

# Ou se busco por padrão de arquivo:
find src/controllers -name "*UserController*"
```

**Padrão de nomes no MVC:**
- Controllers: `src/controllers/*Controller.ts`
- Services: `src/services/*Service.ts`
- Models: `src/models/*Model.ts`
- Routes: `src/routes/*Routes.ts`

---

### Fase 4: Rastrear Dependências (Read + Grep)

```bash
# 1. Abrir o Controller
# (Read to see imports)

# 2. Ver quais Services usa
grep "import.*Service" src/controllers/UserController.ts
grep "this\\..*Service" src/controllers/UserController.ts
# → output: this.userService.createUser()

# 3. Localizar o Service correspondente
find src/services -name "*UserService*"

# 4. Ver métodos do Service
grep "createUser" src/services/UserService.ts
```

**Pattern típico no MVC:**
```typescript
// UserController.ts
export class UserController {
  constructor(private userService: UserService) {}
  
  async register(req, res) {
    const user = await this.userService.createUser(req.body);
    res.json(user);
  }
}
```

---

## 📊 MATRIZ: Quando Usar Cada Comando

| Situação | Comando | Exemplo |
|----------|---------|---------|
| **Entender arquitetura geral** | `ctx map` | `ctx map --title "arquitetura" --dirs .` |
| **Focar em módulo específico** | `ctx map --seeds` | `ctx map --title "auth" --dirs . --seeds src/auth` |
| **Achar um arquivo** | `grep` ou `find` | `grep -r "class AuthService" src/` |
| **Ver fluxo Controller→Service→Model** | `Read` + `Grep` | `Read UserController.ts`, depois `grep "service\."` |
| **Buscar padrão em docs** | `ctx search` | `ctx search docs "como validar email"` |

---

## 💡 EXEMPLO REAL: "Adicionar Validação de Email no Cadastro"

**Passo 1:** Encontrar o Controller de cadastro
```bash
grep -r "register\|signup\|create.*user" src/controllers --include="*.ts"
# Output: src/controllers/UserController.ts
```

**Passo 2:** Abrir e ler o UserController
```
→ Ver: import { UserService } from ../services
→ Ver: this.userService.createUser(data)
```

**Passo 3:** Localizar o UserService
```bash
find src/services -name "*UserService*"
# Output: src/services/UserService.ts
```

**Passo 4:** Abrir e ler UserService.createUser()
```
→ Ver: onde os dados são processados
→ Ver: quais validações já existem
```

**Passo 5:** Escrever teste (TDD)
```typescript
// UserService.test.ts
test("deve rejeitar email inválido", () => {
  expect(() => userService.createUser({ email: "invalid" }))
    .toThrow("Email inválido");
});
```

**Passo 6:** Implementar validação no Service
**Passo 7:** Executar testes
**Passo 8:** Verificar se Controller passa dados corretamente

---

## 🚀 COMANDOS ESSENCIAIS (Copy-Paste)

```bash
# 1. Ver estrutura geral
ctx map --title "arquitetura" --dirs . --max-tokens 4096

# 2. Focar em módulo (ex: autenticação)
ctx map --title "auth module" --dirs . --seeds src/auth

# 3. Achar Controller específico
grep -r "class.*Controller" src/controllers | grep -i "user\|auth\|product"

# 4. Ver imports/dependências
grep "import\|require" src/controllers/UserController.ts | head -20

# 5. Rastrear método específico
grep -n "createUser\|register" src/services/UserService.ts

# 6. Buscar em documentação (se houver wiki/docs indexado)
ctx search docs "validação de email"

# 7. Ver testes relacionados
find . -path "*/test*" -name "*User*" -type f
```

---

## ⚡ PRO TIPS

- **ctx map é rápido**: Usa análise local, sem rede. Execute sempre que não tiver certeza.
- **Personalized PageRank (--seeds)**: Prioriza arquivos próximos aos seed directories. Muito útil em MVC.
- **Budget**: Use `--max-tokens 8000` para repos grandes; padrão é 4096.
- **Top N**: Use `--top 20` se só quer os 20 arquivos mais relevantes (ignora budget).
- **Seeds múltiplos**: `--seeds src/auth,src/middleware,src/utils` para priorizar vários dirs.

---

## ❌ ANTI-PATTERNS (O que NÃO fazer)

- ❌ Ler todos os arquivos com `ls -la src/` → Use `ctx map` em vez disso
- ❌ `grep -r ".*"` sem padrão → Seja específico: `grep -r "class.*Service"`
- ❌ Explorar aleatorimente → Use `ctx map --seeds <dir>` para navegar inteligente
- ❌ Esperar entender tudo de uma vez → Use Fase 1 → Fase 2 → Fase 3 progressivamente
