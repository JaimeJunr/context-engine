---
name: create
description: Criar USer Prompt
disable-model-invocation: true
---

# Criar USer Prompt

> **📚 Regra Pai**: Consulte `@prompt-engineering-rule.mdc` para fundamentos teóricos, metodologias avançadas e técnicas de otimização.

## 2. Parâmetros do Comando

### Parâmetros Obrigatórios

## 1. Contexto e Preparação

### 1.1 Carregamento Obrigatório de Regras

- [ ] **🚨 CRÍTICO**: **SEMPRE execute** `fetch_rules(["core/prompt-engineering-rule"])` como PRIMEIRO passo
- [ ] **SEMPRE confirme** que as regras foram carregadas com sucesso
- [ ] **SEMPRE aplique** diretrizes de templates e estrutura da regra pai

### 1.2 Análise de Requisitos

- [ ] **SEMPRE defina** objetivo específico e escopo do prompt

## 2. Parâmetros do Comando

### Parâmetros Obrigatórios

- **--nome**: Nome descritivo do prompt (sem espaços, use hífens)

- **--tipo**: Tipo de prompt (code-review, documentation, debugging, analysis, testing, deployment, maintenance)

### Parâmetros Opcionais

- **--template**: Template específico a usar (padrão: base)
- **--tags**: Tags adicionais separadas por vírgula
- **--autor**: Nome do autor do prompt
- **--versao**: Versão inicial (padrão: 1.0.0)
- **--descricao**: Descrição breve do prompt

## 3. Execução Principal

### 3.1 Validação de Parâmetros

- [ ] **SEMPRE valide** se nome contém apenas caracteres válidos (letras, números, hífens)
- [ ] **SEMPRE confirme** se data está no formato correto YYYY-MM-DD
- [ ] **SEMPRE verifique** se tipo é válido e suportado
- [ ] **SEMPRE valide** se template existe e está disponível

### 3.2 Geração do Arquivo

- [ ] **SEMPRE crie** arquivo com nome: `prompt-{nome}-{data}.md`
- [ ] **SEMPRE salve** em localização: `.cursor/prompts/`
- [ ] **SEMPRE aplique** template selecionado com parâmetros fornecidos
- [ ] **SEMPRE inclua** frontmatter com metadados completos

### 3.3 Aplicação de Template

- [ ] **SEMPRE use** template base para estrutura fundamental
- [ ] **SEMPRE adapte** template específico baseado no tipo
- [ ] **SEMPRE personalize** conteúdo com parâmetros fornecidos
- [ ] **SEMPRE inclua** seções obrigatórias: Objetivo, Instruções, Exemplos, Validação

## 4. Templates Disponíveis

> **📚 Templates Completos**: Consulte `@prompt-engineering-rule.mdc` para templates detalhados e exemplos completos.

### 4.1 Tipos Suportados

- **code-review**: Revisão sistemática de código
- **documentation**: Geração de documentação técnica
- **debugging**: Identificação e resolução de problemas
- **analysis**: Análise de dados e sistemas
- **testing**: Criação e execução de testes
- **deployment**: Deploy e configuração de ambientes
- **maintenance**: Manutenção e atualizações

### 4.2 Estrutura Base

Todos os templates seguem a estrutura padrão definida na regra pai:

- **Objetivo**: Definição clara do propósito
- **Contexto**: Preparação e pré-requisitos
- **Execução**: Passos principais da tarefa
- **Validação**: Verificação de qualidade
- **Exemplos**: Casos práticos de uso
- **Troubleshooting**: Resolução de problemas comuns

## 5. Validação e Finalização

### 5.1 Validação de Estrutura

- [ ] **SEMPRE execute** `fetch_rules(["core/prompt-engineering-rule"])` para validar estrutura
- [ ] **SEMPRE verifique** se frontmatter está correto
- [ ] **SEMPRE confirme** se hierarquia de títulos está adequada
- [ ] **SEMPRE valide** se markdown está sintaticamente correto
- [ ] **SEMPRE teste** se links e referências funcionam

### 5.2 Validação de Conteúdo

- [ ] **SEMPRE confirme** se objetivo está claro e específico
- [ ] **SEMPRE verifique** se instruções são acionáveis
- [ ] **SEMPRE valide** se exemplos são relevantes e funcionais
- [ ] **SEMPRE teste** se checklist é completo e verificável

### 5.3 Validação de Qualidade

- [ ] **SEMPRE execute** markdownlint para verificar sintaxe
- [ ] **SEMPRE valide** se formatação está consistente
- [ ] **SEMPRE confirme** se linguagem está clara e objetiva
- [ ] **SEMPRE teste** se prompt funciona como esperado

## 6. Exemplos de Uso

### Exemplo 1: Code Review

```bash
/create-prompt --nome="review-react-components" --data="2024-01-15" --tipo="code-review" --tags="react,frontend,components" --autor="Jaime"
```

### Exemplo 2: Documentação

```bash
/create-prompt --nome="api-documentation" --data="2024-01-15" --tipo="documentation" --template="api-docs" --tags="api,swagger,openapi"
```

### Exemplo 3: Debugging

```bash
/create-prompt --nome="debug-performance-issues" --data="2024-01-15" --tipo="debugging" --tags="performance,profiling,optimization"
```

## 7. Troubleshooting

### Problema: Template não encontrado

- [ ] **Sintoma**: Erro "Template não encontrado"
- [ ] **Causa**: Template especificado não existe
- [ ] **Solução**: Use template "base" ou verifique nome do template

### Problema: Nome inválido

- [ ] **Sintoma**: Erro "Nome contém caracteres inválidos"
- [ ] **Causa**: Nome contém espaços ou caracteres especiais
- [ ] **Solução**: Use apenas letras, números e hífens

### Problema: Data inválida

- [ ] **Sintoma**: Erro "Formato de data inválido"
- [ ] **Causa**: Data não está no formato YYYY-MM-DD
- [ ] **Solução**: Use formato correto, ex: 2024-01-15

## 8. Referências e Recursos

- [Engenharia de Prompt - Regras Completas](.cursor/rules/core/prompt-engineering-rule.mdc)
- [Formatação Markdown](.cursor/rules/core/markdown-rule.mdc)
- [Templates de Comandos](.cursor/commands/)
- [Exemplos de Prompts](.cursor/prompts/)

**Por que este comando existe**: Geração estruturada de prompts garante qualidade e consistência
**Automação**: Configure validação automática de prompts gerados
