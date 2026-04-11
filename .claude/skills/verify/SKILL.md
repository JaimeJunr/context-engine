---
name: verify
description: Verificação completa do codebase antes de commit ou PR. Executa build, types, lint, testes, auditoria de console.log e git status. Modos quick/full/pre-commit/pre-pr.
disable-model-invocation: true
---

# Verificação do Codebase

Executa checklist de verificação completo no estado atual do projeto.

## Quando Usar

- Antes de abrir um PR
- Antes de commit final
- Ao finalizar uma feature ou bugfix
- Quando o usuário pedir `/verify`

## Fluxo

Executar verificações nesta ordem exata:

### 1. Build Check
- Executar o comando de build do projeto
- Se falhar, reportar erros e PARAR

### 2. Type Check
- Executar TypeScript/type checker (se aplicável)
- Reportar todos os erros com file:line

### 3. Lint Check
- Executar linter
- Reportar warnings e erros

### 4. Test Suite
- Executar todos os testes
- Reportar pass/fail count
- Reportar cobertura se disponível

### 5. Console.log Audit
- Buscar console.log em arquivos fonte (não em testes)
- Reportar localizações

### 6. Git Status
- Mostrar mudanças não commitadas
- Mostrar arquivos modificados desde último commit

## Output

Produzir relatório conciso:

```
VERIFICAÇÃO: [PASS/FAIL]

Build:    [OK/FAIL]
Types:    [OK/X erros]
Lint:     [OK/X issues]
Testes:   [X/Y passed, Z% cobertura]
Logs:     [OK/X console.logs]

Pronto para PR: [SIM/NÃO]
```

Se houver issues críticos, listar com sugestões de correção.

## Argumentos

$ARGUMENTS pode ser:
- `quick` - Apenas build + types
- `full` - Todas as verificações (padrão)
- `pre-commit` - Verificações relevantes para commits
- `pre-pr` - Verificações completas + security scan
