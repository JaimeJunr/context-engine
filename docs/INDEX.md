# Documentação — Context Engine

> **Objetivo**: Guia central para navegação e compreensão da documentação do `ctx` (context-engine).

## Índice de Documentação

### Produto e Visão

- [Produto](produto.md) — por que o ctx existe, evolução, visão de futuro (3 Horizontes)
- [Roadmap](roadmap.md) — próximas features priorizadas por horizonte

### Engenharia

- [Patterns](patterns.md) — filosofia, invariantes, como adicionar comandos e linguagens
- [Referência Técnica](arquitetura.md) — pipeline completo, módulos, cache, ranking, CLI

### Features Implementadas

- [Especificação RAG](especificacao-rag.md) — sistema de recuperação semântica de conhecimento, regras de negócio, entidades
- [Implementação RAG](implementacao-rag.md) — arquivos, testes, uso básico do módulo catalog

### Pesquisa

- [State of the Art: Code Search](pesquisa/code-search-state-of-art.md) — survey de técnicas de busca em código
- [Decisões de Implementação](pesquisa/decisoes-implementacao.md) — trade-offs e escolhas técnicas

## Como Usar Esta Documentação

| Se você quer... | Leia |
|---|---|
| Entender o projeto do zero | [`README.md`](../README.md) → [`produto.md`](produto.md) |
| Contribuir com código | [`patterns.md`](patterns.md) → [`arquitetura.md`](arquitetura.md) |
| Adicionar uma linguagem | [`patterns.md`](patterns.md) — seção "Como adicionar suporte a nova linguagem" |
| Adicionar um novo comando | [`patterns.md`](patterns.md) — seção "Como adicionar um novo comando" |
| Entender decisões de design | [`pesquisa/decisoes-implementacao.md`](pesquisa/decisoes-implementacao.md) |
| Planejar o que vem a seguir | [`produto.md`](produto.md) + [`roadmap.md`](roadmap.md) |

## Contribuindo com a Documentação

- Mantenha documentação atualizada junto com o código
- Use português para documentação, inglês para código
- Atualize este índice ao adicionar novos documentos
