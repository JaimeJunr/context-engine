# Especificação Funcional Agnóstica
## Sistema de Recuperação Semântica de Conhecimento

**Versão:** 1.0  
**Data:** 2026-04-11  
**Classificação:** Documento de Regras de Negócio — Uso Interno

---

## 1. Visão Executiva

Este módulo entrega à organização uma **capacidade de recuperação inteligente de conhecimento corporativo**. Dado um acervo documental heterogêneo — composto por manuais, especificações, guias operacionais e artefatos de código — o sistema permite que colaboradores e agentes automatizados localizem, em linguagem natural, as informações mais pertinentes ao seu contexto de trabalho.

O valor central reside na combinação de três dimensões de relevância:
- **Relevância textual**: o documento contém os termos exatos da consulta?
- **Relevância conceitual**: o documento trata do mesmo assunto, mesmo que em outras palavras?
- **Relevância qualitativa**: um avaliador especializado confirma que o documento responde à necessidade?

O resultado é uma experiência de busca que supera ferramentas tradicionais baseadas em palavras-chave, aproximando-se da compreensão humana da intenção por trás de uma pergunta.

O sistema é inteiramente operado de forma local, sem envio de dados para serviços externos, garantindo privacidade e soberania sobre o acervo documental.

---

## 2. Catálogo de Entradas e Saídas (Interface de Negócio)

### 2.1 Entradas do Sistema

| Conceito de Dado | Propósito | Natureza da Informação |
|---|---|---|
| Localização do Acervo | Define o repositório físico de documentos a ser catalogado | Referência de localização |
| Padrão de Inclusão | Especifica quais tipos de documento devem ser incorporados ao acervo | Categórico / Filtro |
| Padrão de Exclusão | Define categorias de documentos a serem ignorados durante a catalogação | Categórico / Filtro |
| Consulta de Recuperação | A pergunta ou intenção do usuário em linguagem natural | Textual livre |
| Descrição de Contexto | Informação adicional associada a uma seção do acervo, que orienta a avaliação de pertinência | Textual descritivo |
| Identificador de Documento | Referência direta a um documento específico do acervo | Referência categórica |
| Padrão de Recuperação em Lote | Expressão que seleciona múltiplos documentos simultaneamente | Padrão de filtro |
| Instrução de Atualização | Operação a ser executada antes de reindexar o acervo (ex.: sincronização com repositório externo) | Instrução operacional |
| Configuração de Acervo | Agrupamento de parâmetros que define um acervo nomeado, incluindo localização, filtros e contexto | Registro de configuração |
| Perfil de Avaliador de Pertinência | Define qual modelo especializado é utilizado para gerar julgamentos qualitativos | Referência de configuração |
| Perfil de Geração de Assinatura Semântica | Define qual modelo é utilizado para converter fragmentos em representações numéricas | Referência de configuração |
| Perfil de Geração de Variantes | Define qual modelo é utilizado para reformular consultas e expandir intenções | Referência de configuração |
| Instrução de Compactação | Comando de manutenção que solicita a reorganização e compactação do repositório interno de dados, recuperando espaço de armazenamento não utilizado | Instrução operacional |

### 2.2 Saídas do Sistema

| Conceito de Dado | Propósito | Natureza da Informação |
|---|---|---|
| Lista de Resultados Ranqueados | Documentos ordenados por grau de pertinência à consulta | Coleção ordenada quantitativa |
| Pontuação de Pertinência | Valor numérico de 0 a 1 que indica o grau de adequação de cada resultado | Quantitativo contínuo |
| Fragmento Relevante | Trecho do documento considerado mais pertinente, com indicação de posição no documento original | Textual com referência de localização |
| Relatório de Saúde do Acervo | Diagnóstico sobre o estado de catalogação: documentos indexados, pendentes de processamento, inconsistências | Relatório de status categórico/quantitativo |
| Documento Completo | Conteúdo integral de um item específico do acervo, recuperado por identificador | Textual estruturado |
| Variantes de Consulta | Reformulações geradas automaticamente da pergunta original, usadas internamente para ampliar a cobertura da busca | Textual / uso interno |
| Registro de Acervos | Listagem de todos os acervos catalogados com seus metadados e estado de atualização | Coleção de metadados |

---

## 3. Inventário de Regras de Decisão

### 3.1 Catalogação de Documentos

**RD-01 — Inclusão por Correspondência de Padrão**  
Um documento só é elegível para incorporação ao acervo se seu nome e localização corresponderem ao padrão de inclusão definido para aquela coleção. Documentos que correspondam também ao padrão de exclusão são descartados, mesmo que atendam ao padrão de inclusão.

**RD-02 — Controle de Duplicidade por Impressão Digital**  
Antes de processar um documento, o sistema verifica se seu conteúdo foi alterado desde a última catalogação. Se o conteúdo for idêntico ao já catalogado, o documento é ignorado no ciclo atual, evitando reprocessamento desnecessário.

**RD-03 — Segmentação Inteligente por Limites Semânticos**  
Documentos extensos são divididos em fragmentos de tamanho padronizado para viabilizar a recuperação precisa. A divisão prioriza pontos de ruptura natural do conteúdo, respeitando a seguinte hierarquia de preferência:
- Títulos de primeiro nível (prioridade máxima)
- Títulos de segundo nível
- Delimitadores de blocos de exemplos ou ilustrações
- Parágrafos
- Quebras de linha simples (prioridade mínima)

Se nenhum ponto de ruptura natural for encontrado dentro da janela de análise, a divisão ocorre no último espaço disponível antes do limite.

**RD-04 — Sobreposição entre Fragmentos Consecutivos**  
Para preservar a continuidade do raciocínio entre fragmentos adjacentes, cada fragmento retém aproximadamente 15% do conteúdo do fragmento anterior. Isso garante que informações contextuais presentes em uma divisão não se percam.

**RD-05 — Fragmentação Orientada à Estrutura de Código**  
Para documentos que contenham código-fonte, a segmentação considera a hierarquia estrutural do código ao determinar os pontos de corte, seguindo a prioridade:
- Definições de tipos e estruturas de dados compostas (prioridade máxima)
- Definições de rotinas ou procedimentos
- Declarações de tipos simples e enumerações
- Instruções de importação ou referência externa (prioridade mínima)

Quando a análise estrutural não for aplicável ao tipo de documento, aplica-se a regra RD-03.

### 3.2 Geração de Representações Semânticas

**RD-06 — Priorização de Processamento**  
Fragmentos recém-incorporados ou modificados têm prioridade de processamento sobre os demais. O sistema mantém uma fila de itens pendentes de geração de representação semântica.

**RD-07 — Formato de Entrada para Geração de Representação**  
A representação semântica é gerada a partir do fragmento acompanhado de seu título de documento. O título funciona como âncora de contexto, melhorando a qualidade da representação.

**RD-08 — Validade da Representação em Cache**  
Uma representação semântica gerada anteriormente permanece válida enquanto o conteúdo do fragmento não for alterado. A invalidade é detectada pela mudança na impressão digital do documento (conforme RD-02).

### 3.3 Processo de Busca e Recuperação

**RD-09 — Desdobramento da Intenção de Consulta**  
Antes de iniciar a busca, o sistema gera automaticamente 2 a 3 variações da consulta original, usando reformulações semânticas. O objetivo é capturar diferentes ângulos da intenção do usuário e ampliar a cobertura da recuperação.

**RD-09-A — Direcionamento Explícito de Modalidade de Busca**  
O usuário pode prefixar sua consulta com um qualificador de modalidade para controlar qual dimensão de relevância será priorizada:

| Qualificador | Comportamento |
|---|---|
| Correspondência exata | Força busca exclusivamente por correspondência textual; as etapas de busca conceitual e avaliação qualitativa são suprimidas |
| Similaridade conceitual | Força busca exclusivamente por proximidade conceitual |
| Expansão de intenção | Aplica expansão de intenção da consulta antes da busca, gerando hipótese de resposta ideal para orientar a recuperação |

Na ausência de qualificador, aplica-se o fluxo padrão de fusão descrito em RD-11. Qualificadores não alteram o contrato de saída — o resultado continua sendo uma lista ranqueada com pontuações de pertinência.

**RD-10 — Ponderação da Consulta Original**  
Nos resultados de busca combinada, os documentos recuperados pela consulta original recebem peso duas vezes superior ao dos documentos recuperados exclusivamente pelas variações geradas. Isso garante que a intenção declarada pelo usuário seja sempre o critério dominante.

**RD-11 — Fusão de Rankings por Reciprocidade de Posição**  
Os resultados provenientes da busca por correspondência textual e da busca por proximidade conceitual são combinados usando a posição relativa de cada documento em cada lista. A contribuição de cada lista para a pontuação final é calculada como o inverso da posição (com constante de ajuste de 60), somando as contribuições de todas as listas ponderadas.

**RD-12 — Bônus de Posição Privilegiada**  
O documento classificado em primeiro lugar em qualquer lista individual recebe um acréscimo de 0,05 pontos na pontuação de fusão. Os documentos classificados nas posições 2 e 3 recebem acréscimo de 0,02 pontos.

**RD-13 — Limite de Candidatos para Avaliação Qualitativa**  
Após a fusão de rankings, apenas os 30 documentos com maior pontuação são submetidos à avaliação qualitativa por avaliador especializado. Documentos além deste limite são descartados.

**RD-14 — Avaliação Qualitativa Binária com Confiança**  
O avaliador especializado emite um julgamento binário (pertinente / não pertinente) para cada candidato, acompanhado de um índice de confiança derivado da probabilidade associada à decisão.

**RD-15 — Integração Posição-Dependente do Julgamento Qualitativo**  
A influência do julgamento qualitativo na pontuação final varia conforme a posição do candidato no ranking intermediário:

| Faixa de Posição | Peso da Fusão de Rankings | Peso do Julgamento Qualitativo |
|---|---|---|
| 1ª a 3ª posição | 75% | 25% |
| 4ª a 10ª posição | 60% | 40% |
| 11ª posição em diante | 40% | 60% |

Justificativa: candidatos bem posicionados pela fusão de rankings merecem confiança maior nos critérios quantitativos; candidatos com posição intermediária ou baixa se beneficiam mais da avaliação qualitativa para subir ou descer no ranking final.

**RD-16 — Contexto de Acervo como Orientação para o Avaliador**  
Se um acervo possuir descrições de contexto associadas a seções específicas, essas descrições são fornecidas ao avaliador qualitativo como informação complementar, melhorando a precisão da avaliação para aquele domínio específico.

### 3.4 Gestão de Acervos e Coleções

**RD-17 — Unicidade de Nome de Acervo**  
Cada acervo deve possuir um identificador único no sistema. A tentativa de registrar um acervo com nome já existente deve resultar em operação de atualização, não em duplicação.

**RD-18 — Rastreamento de Estado de Atualização**  
O sistema registra o instante da última catalogação de cada acervo. Essa informação é exibida no relatório de saúde e usada para identificar acervos desatualizados.

**RD-19 — Propagação de Contexto por Hierarquia de Caminho**  
Descrições de contexto associadas a um caminho de nível superior aplicam-se automaticamente a todos os documentos em subcaminhos, salvo se uma descrição mais específica substituir.

**RD-30 — Compactação do Repositório Interno**  
O sistema oferece operação de manutenção que reorganiza o repositório interno de dados, removendo registros obsoletos (documentos excluídos, fragmentos de versões anteriores, entradas de cache expiradas) e compactando o espaço de armazenamento recuperado. Esta operação não altera nenhum dado ativo nem invalida assinaturas semânticas existentes. Recomenda-se sua execução periódica em acervos com alta taxa de rotatividade de documentos.

### 3.5 Gestão de Recursos e Desempenho

**RD-20 — Carregamento Tardio de Recursos Computacionais Intensivos**  
Os modelos de avaliação qualitativa e de geração de representações semânticas não são carregados ao iniciar o sistema. Seu carregamento ocorre apenas no momento da primeira operação que os requisitar.

**RD-21 — Liberação Automática por Inatividade**  
Se nenhuma operação demandar os modelos por um período superior a 5 minutos, o sistema os descarrega automaticamente da memória ativa, liberando recursos para outros processos.

**RD-22 — Armazenamento em Cache de Resultados de Processamento**  
Resultados de operações custosas — como variações de consulta e julgamentos de pertinência — são armazenados associados a uma impressão digital da entrada. Se a mesma entrada for processada novamente, o resultado em cache é retornado sem reprocessamento.

**RD-23 — Modos de Operação do Canal de Comunicação**  
O sistema suporta dois modos de exposição do canal de comunicação com agentes externos:
- **Modo integrado**: o sistema é iniciado sob demanda pelo agente solicitante e comunica-se através de fluxos padrão de entrada e saída do processo. É encerrado junto com o processo cliente.
- **Modo persistente**: o sistema opera como um processo de longa duração, aceitando requisições de múltiplos clientes simultaneamente por uma porta de rede local. Neste modo, aplica-se a regra RD-21 para liberação de recursos por inatividade.

A escolha do modo é feita no momento da implantação e não requer alteração no contrato de interface.

**RD-29 — Independência entre Perfis de Modelo**  
Os três perfis de modelo (geração de assinatura semântica, avaliação qualitativa e geração de variantes de consulta) são configurados e substituídos de forma independente. A troca de um perfil não invalida resultados produzidos pelos demais, desde que o perfil substituído seja da mesma categoria funcional.

### 3.6 Ciclo de Especialização de Avaliadores

**RD-24 — Coleta de Pares de Treinamento**  
O sistema registra, para cada julgamento qualitativo emitido com confirmação explícita do usuário, o par (consulta + fragmento candidato → pertinente/não pertinente). Esses pares constituem o acervo de especialização.

**RD-25 — Limiar Mínimo para Início de Especialização**  
O processo de especialização de um avaliador só pode ser iniciado quando o acervo de especialização contiver no mínimo 100 pares confirmados. Tentativas abaixo desse limiar devem ser rejeitadas com indicação do número de pares disponíveis.

**RD-26 — Validação Cruzada da Especialização**  
Após cada ciclo de especialização, o sistema avalia o desempenho do avaliador especializado em um subconjunto reservado dos pares de treinamento (mínimo 20%). O ciclo só é aceito se a métrica de qualidade do julgamento superar a linha de base do avaliador original.

**RD-27 — Reversibilidade da Especialização**  
O avaliador original é preservado mesmo após a especialização. O operador pode reverter para o avaliador original a qualquer momento, sem perda dos pares de treinamento acumulados.

**RD-28 — Exportação de Avaliador Especializado**  
Um avaliador após especialização pode ser exportado em formato portável, permitindo sua reutilização em outras instâncias do sistema sem necessidade de repetir o ciclo de especialização.

---

## 4. Entidades de Informação

### 4.1 Registro de Acervo

Documento de configuração que define uma coleção gerenciada de conhecimento. Contém:
- **Identificador único** do acervo (informação categórica)
- **Localização física** do repositório de documentos (referência de localização)
- **Critérios de inclusão e exclusão** de documentos (filtros)
- **Marca temporal** da última catalogação (temporal)
- **Instrução de sincronização** opcional, executada antes de reindexar (operacional)
- **Indicador de participação padrão** em operações de busca global (booleano)

### 4.2 Registro de Documento

Ficha cadastral de um documento incorporado ao acervo. Contém:
- **Identificador curto** gerado por impressão digital do caminho (referência)
- **Localização no acervo** (referência de caminho)
- **Título inferido** do documento (textual)
- **Impressão digital do conteúdo** para controle de integridade (hash)
- **Estado de ativação** no acervo (categórico: ativo/inativo)

### 4.3 Fragmento de Conteúdo com Assinatura Semântica

Unidade mínima de recuperação do sistema. Contém:
- **Vínculo com o Registro de Documento** pai (referência)
- **Número de sequência** dentro do documento (quantitativo ordinal)
- **Posição de início** no documento original, em unidades de conteúdo (quantitativo)
- **Texto do fragmento** (textual)
- **Assinatura semântica** — representação numérica multidimensional que codifica o significado do fragmento (vetor numérico, 768 dimensões)

### 4.4 Registro de Contexto Documental

Descrição de negócio associada a uma localização lógica dentro de um acervo. Contém:
- **Caminho lógico** da seção do acervo (referência hierárquica)
- **Descrição em linguagem natural** do tipo de conteúdo presente naquela seção (textual)

Exemplo de uso: associar a descrição "Guias de resolução de problemas operacionais" ao caminho `/suporte`, orientando o avaliador qualitativo ao processar documentos dessa seção.

### 4.5 Registro de Cache de Processamento

Resultado armazenado de uma operação custosa, para reaproveitamento futuro. Contém:
- **Impressão digital da entrada** — identifica unicamente a combinação de consulta e parâmetros (hash)
- **Tipo de operação** armazenada (categórico: variantes de consulta / julgamento de pertinência)
- **Resultado serializado** da operação (estruturado)
- **Marca temporal** de geração (temporal)

### 4.6 Resultado de Recuperação

Unidade de resposta ao usuário após uma operação de busca. Contém:
- **Fragmento de conteúdo** recuperado (textual)
- **Pontuação de pertinência final** (quantitativo contínuo, 0 a 1)
- **Localização no documento original** (referência de posição)
- **Metadados do documento** pai: título, localização no acervo, identificador (referência + textual)
- **Decomposição da pontuação** (opcional): contribuição da busca textual, da busca conceitual e do julgamento qualitativo (quantitativo analítico)

### 4.7 Relatório de Saúde do Acervo

Diagnóstico instantâneo do estado do sistema. Contém, para cada acervo:
- **Nome do acervo** (categórico)
- **Total de documentos catalogados** (quantitativo)
- **Documentos com assinatura semântica pendente** (quantitativo)
- **Data da última catalogação** (temporal)
- **Indicador de consistência** — presença de fragmentos sem par de assinatura semântica (categórico)

### 4.8 Registro de Avaliador Especializado

Snapshot de um avaliador de pertinência após especialização. Contém:
- **Identificador único** do avaliador especializado (referência)
- **Categoria funcional** que o avaliador atende (categórico: geração de assinaturas / avaliação qualitativa / geração de variantes)
- **Acervo de pares de treinamento** utilizados na especialização (referência a conjunto)
- **Métrica de desempenho** do avaliador especializado em validação cruzada (quantitativo, 0 a 1)
- **Métrica de linha de base** do avaliador original, para comparação (quantitativo, 0 a 1)
- **Marca temporal** da conclusão da especialização (temporal)
- **Disponibilidade de reversão** para o avaliador original (booleano)

---

*Este documento foi produzido seguindo o protocolo de Desconstrução Funcional Clean-room. Nenhum identificador técnico do sistema de origem foi preservado. A leitura deste documento não permite a inferência da linguagem de programação, framework, bibliotecas ou padrões arquiteturais utilizados na implementação original.*
