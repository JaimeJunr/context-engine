# 🏗️ Architecture Guidelines

## 🚨 COMPORTAMENTO OBRIGATÓRIO

### Ações Críticas de Desenvolvimento

1. **SEMPRE sigam** princípios fundamentais de Clean Code, TDD e DDD.
2. **SEMPRE usem** refatoração contínua para manter qualidade.
3. **SEMPRE apliquem** técnicas apropriadas para cada domínio.
4. **SEMPRE mantenham** consistência entre regras especializadas.

## 🎯 REGRAS ESPECIALIZADAS (RESUMO)

### 1. Clean Code

- Código limpo e legível.
- Nomes significativos e responsabilidade única (SRP).
- Eliminação de duplicação e encapsulamento.
- Priorize manutenibilidade.

### 2. Test-Driven Development (TDD)

**🚨 CRÍTICO**: **SEMPRE escreva pelo menos 1 teste ANTES de escrever o código**.

#### Obrigatório antes de começar

- **SEMPRE comunique** ao usuário que seguirá TDD.
- **SEMPRE confirme**: existe estrutura de testes configurada?
  - Se não houver, **pergunte** antes de implementar.
  - Registre a exceção quando testes não forem aplicáveis.

#### Ciclo Red-Green-Refactor

1. **🔴 RED**: teste descrevendo o comportamento esperado (inicialmente falha).
2. **🟢 GREEN**: código mínimo para o teste passar.
3. **♻️ REFACTOR**: melhoria mantendo testes verdes.

#### Como o teste deve descrever o comportamento

- **O QUE** deve acontecer
- **QUANDO** deve acontecer
- **COMO** deve acontecer (resultado esperado)

#### Cobertura

- **SEMPRE mantenha** cobertura mínima de 80% em funcionalidades críticas.
- **SEMPRE busque** >90% em regras de negócio complexas.
- **SEMPRE garanta** que TODOS os testes passem antes de concluir.

### 3. Domain-Driven Design (DDD)

- Modelagem de domínio e linguagem ubíqua.
- Entidades, agregados e value objects.
- Serviços de domínio e bounded contexts.

#### Quando ficar muito complexo

- **SEMPRE considere** usar Clean Architecture/Onion/Hexagonal para apoiar a modelagem.

### 4. SOLID Principles

- **SEMPRE apliquem** SRP, OCP, LSP, ISP, DIP.
- Evite over-engineering.
- Aplique design patterns somente com benefício real.
- Use YAGNI: não implemente o que não é necessário agora.

### 5. Clean Architecture

- Separe em camadas: Entities, Use Cases, Interface Adapters e Frameworks/Drivers.
- Dependency Rule: dependências apontam para dentro.
- Isole regras de negócio de frameworks, UI, banco e agências externas.
- Testabilidade: lógica de negócio deve ser testável sem dependências externas.

### 6. Arquiteturas por Camada

#### 6.1 Backend: MVC

- Use MVC quando a base do projeto exigir padrão tradicional.
- Model: regras de negócio e acesso a dados.
- Controller: orquestração, validações e coordenação.
- View: serialização/presentação.
- Se a complexidade aumentar, isole regras com DDD e aplique Clean Architecture sobre módulos críticos.

#### 6.2 Frontend: Arquitetura de Features (React)

- Organize por features autocontidas (cada feature encapsula sua lógica).
- Agrupe: `components/`, `hooks/`, `services/`, `types/` e testes dentro da feature.
- Use `shared/` apenas para o que realmente é compartilhável.
- Isole lógica de negócio e evite dependências entre features.

#### 6.3 Frontend: Clean Architecture (para APIs complexas)

- Separe: `domain/`, `application/`, `infrastructure/`, `presentation/`.
- Isole comunicação HTTP e mappers em `infrastructure/`.
- Mantenha regras de negócio fora de `infrastructure/`.

### 7. Refatoração

- Refatorar com segurança mantendo funcionalidade.
- Eliminar code smells (métodos longos, muitos parâmetros, duplicação).
- Refatoração incremental e guiada por testes.

### 8. Qualidade de Código e Ferramentas Automatizadas

- Configure linting e formatação adequados (ex.: ESLint/Prettier).
- Integre análise estática quando disponível.
- Execute verificações antes de concluir e corrija problemas encontrados.

### 8.1 Type Safety

- Use strict mode em TypeScript quando aplicável.
- Tipos explícitos para funções/props/retornos.
- Evite `any`; prefira tipos específicos/`unknown`.
- Valide em runtime quando necessário (ex.: Zod/Yup).

### 9. Git e Versionamento

- Conventional Commits.
- Padrão de branch: `<prefixo>-CAP-<número>-<descrição>` (ou `CAP-<número>-<descrição>`).
- Use hooks/validações automáticas quando existirem.

### 10. Documentação de Decisões Arquiteturais (ADRs)

- **SEMPRE documente** decisões que impactam a arquitetura.
- Inclua: contexto, decisão, consequências e alternativas consideradas.

### 11. Trilhas de Auditoria (Audit Trails)

- Para operações sensíveis: registre quem/quando/o quê e dados antes/depois.
- Logs devem ser imutáveis e protegidos para auditoria.
- Nunca exponha dados sensíveis nos logs.

## 🔧 INTEGRAÇÃO ENTRE REGRAS

- Princípios devem funcionar juntos como ciclo contínuo: Clean Code -> TDD -> DDD -> SOLID -> Clean Architecture -> Refatoração.
- Git e Debugging suportam rastreabilidade e correções.

## 📋 EXEMPLOS (CURTOS)

### Nova funcionalidade

1. Comunique TDD e confirme estrutura de testes.
2. Escreva um teste simples (RED).
3. Implemente mínimo (GREEN).
4. Refatore mantendo testes verdes.
5. Reaplique Clean Code/DDD/SOLID/Clean Architecture conforme o domínio.

### Correção de bug

1. Investigue sistematicamente.
2. Comunique e garanta que exista um teste (ou exceção documentada).
3. Crie teste que reproduz o bug (RED).
4. Corrija mínimo necessário (GREEN) e refatore (Refactor).

## 🚫 LIMITES ABSOLUTOS

- **🚨 CRÍTICO**: **NUNCA implementem** código sem pelo menos 1 teste (ou exceção confirmada).
- **🚨 CRÍTICO**: **NUNCA iniciem** TDD sem comunicar claramente.
- **NUNCA ignorem** princípios de qualidade e necessidade de Clean Architecture quando a aplicação fica complexa.
- **NUNCA misturem** lógica de negócio com infraestrutura no frontend.
- **NUNCA deixem** de criar `infrastructure/` para APIs complexas no frontend.

## 📚 REFERÊNCIAS

- Core Rule: `core-rule.mdc`
- Clean Code, TDD, DDD, SOLID, Clean Architecture, Refatoração, Debugging, Git (conforme regras especializadas).

## ✅ VERIFICAÇÃO FINAL (MÍNIMA)

Antes de concluir qualquer tarefa:

- TDD foi comunicado e pelo menos 1 teste foi escrito antes da implementação (ou exceção documentada).
- Todos os testes passam.
- Regras especializadas foram consultadas.
- Clean Code e SOLID foram aplicados.
- Camadas e Dependency Rule de Clean Architecture foram respeitadas.
- Frontend seguiu MVC/Features/Clean Architecture conforme o caso.
- Infra de APIs complexas foi isolada em `infrastructure/`.
- Se houve decisão arquitetural relevante, existe ADR.
- Se houve operação sensível, existe audit trail.
