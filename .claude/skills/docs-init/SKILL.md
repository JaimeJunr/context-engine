---
name: docs-init
description: Inicializa a estrutura de documentação do projeto criando diretórios, README.md e docs/INDEX.md seguindo o padrão IVT.
disable-model-invocation: true
---

# Inicializar Documentação do Projeto

Inicializa a estrutura de documentação do projeto criando diretórios, README.md e docs/INDEX.md seguindo o padrão IVT.

## Parâmetros

- **--backup**: Criar backup automático antes da inicialização (padrão: true)
- **--validate**: Executar validação completa após inicialização (padrão: true)
- **--dry-run**: Apenas mostrar mudanças sem aplicar (opcional)

## 1. Contexto e Preparação

- [ ] **SEMPRE identifique** se a estrutura de docs já existe
- [ ] **SEMPRE faça** backup de arquivos existentes antes de qualquer alteração
- [ ] **SEMPRE verifique** se a estrutura de diretórios está adequada

## 2. Análise e Diagnóstico

- [ ] **SEMPRE verifique** se arquivos essenciais existem:
  - [ ] `README.md` na raiz do projeto
  - [ ] Pasta `docs/` com documentação
  - [ ] `docs/INDEX.md` como guia central
- [ ] **SEMPRE documente** estado atual antes de iniciar

## 3. Criação da Estrutura

### 3.1 Diretórios

```bash
mkdir -p docs
```

### 3.2 README.md do Projeto

- [ ] **SEMPRE verifique** se `README.md` existe na raiz
- [ ] **SEMPRE crie** `README.md` se não existir:

````markdown
# [Nome do Projeto]

> **Descrição**: [Descrição breve do projeto]

## Objetivo

[Objetivo principal do projeto]

## Tecnologias

- [Tecnologia 1]
- [Tecnologia 2]
- [Tecnologia 3]

## Estrutura do Projeto

```text
projeto/
├── docs/           # Documentação do projeto
├── src/            # Código fonte
├── tests/          # Testes
└── README.md       # Este arquivo
```

## Instalação e Uso

### Pré-requisitos

- [Pré-requisito 1]
- [Pré-requisito 2]

### Instalação

```bash
# Comando de instalação
```

### Uso

```bash
# Comando de uso
```

## Documentação

Consulte a pasta `docs/` para documentação completa do projeto.

## Contribuição

[Instruções de contribuição]

## Licença

[Informações de licença]
````

### 3.3 docs/INDEX.md

- [ ] **SEMPRE crie** `docs/INDEX.md` se não existir:

```markdown
# Documentação do Projeto

> **Objetivo**: Guia central para navegação e compreensão da documentação do projeto.

## Índice de Documentação

### Arquitetura e Design

- [Arquitetura do Sistema](arquitetura/sistema.md)
- [Decisões de Design](arquitetura/decisoes.md)
- [Padrões Utilizados](arquitetura/padroes.md)

### Desenvolvimento

- [Guia de Desenvolvimento](desenvolvimento/guia.md)
- [Padrões de Código](desenvolvimento/padroes.md)
- [Convenções](desenvolvimento/convencoes.md)

### Testes

- [Estratégia de Testes](testes/estrategia.md)
- [Guia de Testes](testes/guia.md)
- [Cobertura](testes/cobertura.md)

### Deploy e Infraestrutura

- [Guia de Deploy](deploy/guia.md)
- [Configuração de Ambiente](deploy/ambiente.md)
- [Monitoramento](deploy/monitoramento.md)

### APIs e Integrações

- [Documentação da API](api/documentacao.md)
- [Endpoints](api/endpoints.md)
- [Autenticação](api/autenticacao.md)

### Ferramentas e Utilitários

- [Ferramentas de Desenvolvimento](ferramentas/desenvolvimento.md)
- [Scripts Úteis](ferramentas/scripts.md)
- [Configurações](ferramentas/configuracoes.md)

## Como Usar Esta Documentação

1. **Para Desenvolvedores**: Comece com [Guia de Desenvolvimento](desenvolvimento/guia.md)
2. **Para Arquitetos**: Consulte [Arquitetura do Sistema](arquitetura/sistema.md)
3. **Para DevOps**: Veja [Guia de Deploy](deploy/guia.md)
4. **Para Testadores**: Acesse [Estratégia de Testes](testes/estrategia.md)

## Contribuindo com a Documentação

- [ ] Mantenha documentação atualizada
- [ ] Use linguagem clara e objetiva
- [ ] Inclua exemplos práticos
- [ ] Valide links e referências
```

## 4. Validação

- [ ] Arquivos essenciais criados
- [ ] Estrutura de diretórios adequada
- [ ] `README.md` na raiz
- [ ] `docs/` criada com `INDEX.md`
