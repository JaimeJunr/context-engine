---
name: ux-reference
description: Use when reviewing, designing, or evaluating user interfaces — applying Nielsen's 10 Heuristics, UX Laws (Fitts, Hick, Jakob, Miller, Gestalt, Von Restorff, Peak-End Rule), or general UX best practices. Also triggers when discussing usability issues, accessibility, cognitive load, form design, navigation patterns, or creating acceptance criteria for frontend tasks.
---

# UX Reference — Heurísticas, Leis e Boas Práticas

Referência estruturada para avaliação e design de interfaces. Cobre as 10 Heurísticas de Nielsen, as principais Leis de UX e boas práticas gerais.

## Quando Ativar

- Revisando ou avaliando uma interface (heuristic evaluation)
- Projetando novos componentes ou fluxos de usuário
- Tomando decisões sobre navegação, formulários, feedback, erros
- Discutindo trade-offs de UX com o time de produto
- Criando critérios de aceite para tarefas de frontend

---

## Parte 1 — 10 Heurísticas de Nielsen

Desenvolvidas por Jakob Nielsen (Nielsen Norman Group). São os princípios mais usados para avaliação heurística de interfaces.

---

### H1 — Visibilidade do Status do Sistema
**Descrição:** O sistema deve sempre manter os usuários informados sobre o que está acontecendo, por meio de feedback adequado e em tempo razoável.

**Como aplicar:**
- Exibir indicadores de carregamento (spinners, barras de progresso) durante operações assíncronas
- Confirmar ações executadas ("Salvo com sucesso", "Email enviado")
- Mostrar o estado atual do sistema claramente (aba ativa, item selecionado, etapa atual de um wizard)
- Responder a interações em menos de 400ms sempre que possível

**Exemplos concretos:**
- Upload de arquivo com barra de progresso percentual
- Botão que muda de "Salvar" para "Salvando..." durante requisição
- Breadcrumb mostrando onde o usuário está na hierarquia
- Badge de notificação mostrando quantidade de itens pendentes

**Erros comuns a evitar:**
- Ações sem nenhum feedback visual (o usuário clica e nada acontece aparentemente)
- Loaders infinitos sem estimativa de tempo ou mensagem
- Botões que não desabilitam durante o processamento (double submit)
- Status de sistema escondido em lugares não óbvios

---

### H2 — Correspondência entre Sistema e Mundo Real
**Descrição:** O sistema deve usar palavras, frases e conceitos familiares ao usuário — não jargão técnico interno. Seguir convenções do mundo real, apresentando informações em ordem natural e lógica.

**Como aplicar:**
- Usar linguagem do domínio do usuário, não da tecnologia
- Organizar informações como o usuário naturalmente pensa sobre elas
- Usar metáforas visuais reconhecíveis (ícone de lixeira para deletar, envelope para email)
- Datas no formato local do usuário

**Exemplos concretos:**
- "Pasta" e "Arquivo" em vez de "Directory" e "Object"
- Formulário de endereço com campos na ordem: CEP → Rua → Número → Complemento → Cidade → Estado
- Ícone de impressora para imprimir (mesmo que poucos imprimam hoje)
- "Saldo disponível" em vez de "account_balance_available"

**Erros comuns a evitar:**
- Mensagens de erro com códigos internos ou stack traces para o usuário final
- Formulários com campos em ordem técnica (id, created_at, updated_at)
- Jargão de sistema ("null", "undefined", "404") exposto diretamente
- Usar termos diferentes para o mesmo conceito em partes distintas da interface

---

### H3 — Controle e Liberdade do Usuário
**Descrição:** Usuários frequentemente escolhem funções por engano. O sistema precisa ter "saídas de emergência" claramente marcadas para deixar estados indesejados sem necessidade de diálogos extensos.

**Como aplicar:**
- Sempre fornecer Desfazer (Ctrl+Z) e Refazer para ações reversíveis
- Botão "Cancelar" em todos os modais e formulários longos
- Confirmação antes de ações destrutivas irreversíveis
- Permitir que o usuário feche/descarte sem perder progresso (draft automático)

**Exemplos concretos:**
- "Desfazer" após deletar um item (Gmail, Notion)
- Modal de confirmação "Tem certeza? Esta ação não pode ser desfeita" antes de excluir conta
- Botão X para fechar modal sem salvar
- Rascunho automático em formulários longos

**Erros comuns a evitar:**
- Deletar imediatamente sem confirmação nem possibilidade de desfazer
- Modais sem botão de fechar ou cancelar
- Navegação que perde dados de formulário sem aviso
- Processos sem possibilidade de voltar à etapa anterior

---

### H4 — Consistência e Padrões
**Descrição:** Usuários não devem ter que se perguntar se palavras, situações ou ações diferentes significam a mesma coisa. Seguir convenções da plataforma.

**Como aplicar:**
- Usar o mesmo vocabulário para o mesmo conceito em toda a interface
- Botões primários sempre na mesma posição e com mesmo estilo
- Ícones com significado consistente em todo o produto
- Seguir padrões da plataforma (iOS Human Interface Guidelines, Material Design, etc.)

**Exemplos concretos:**
- Botão "Confirmar" sempre à direita do "Cancelar" (ou sempre à esquerda — o importante é ser consistente)
- Cor vermelha sempre associada a ações destrutivas/erros
- Atalhos de teclado seguindo convenções do OS (Ctrl+S = salvar)
- Data sempre no mesmo formato em toda a aplicação

**Erros comuns a evitar:**
- Chamar a mesma ação de "Salvar" em um lugar e "Confirmar" em outro
- Botões primários e secundários trocando de posição entre telas
- Ícones com significados diferentes dependendo do contexto
- Misturar paradigmas de UI (parte do app segue Material, parte segue Fluent)

---

### H5 — Prevenção de Erros
**Descrição:** Ainda melhor que boas mensagens de erro é um design cuidadoso que previne problemas de acontecer. Eliminar condições propensas a erros ou checar por elas e apresentar confirmação antes de executar.

**Como aplicar:**
- Validar inputs em tempo real (não apenas no submit)
- Desabilitar botões até que os requisitos mínimos sejam atendidos
- Confirmar ações com consequências graves
- Usar defaults inteligentes que minimizam erro
- Formatar inputs automaticamente (máscara de CPF, telefone)

**Exemplos concretos:**
- Campo de email que valida formato enquanto o usuário digita
- Botão "Enviar" desabilitado até que todos os campos obrigatórios estejam preenchidos
- Seletor de data que não permite datas inválidas (ex.: 31/02)
- Confirmação por digitação ("Digite DELETE para confirmar")

**Erros comuns a evitar:**
- Validar somente no submit, acumulando múltiplos erros
- Permitir input de formato inválido (ex.: letras em campo numérico)
- Ações destrutivas de um clique sem nenhuma barreira
- Defaults em branco para campos críticos que têm valor óbvio

---

### H6 — Reconhecimento em Vez de Memorização
**Descrição:** Minimizar a carga de memória do usuário tornando objetos, ações e opções visíveis. O usuário não deve ter que lembrar informações de uma parte da interface para usar outra.

**Como aplicar:**
- Mostrar opções disponíveis em vez de exigir que o usuário as lembre (menus, autocomplete)
- Histórico de buscas recentes
- Contexto sempre visível (qual arquivo está aberto, em qual etapa está)
- Labels visíveis nos campos de formulário (não apenas placeholder)

**Exemplos concretos:**
- Dropdown com todas as opções listadas (não campo de texto livre)
- Autocomplete em campo de busca com sugestões
- Indicador de etapa em wizard ("Etapa 2 de 5")
- Labels persistentes acima do campo, não apenas como placeholder que desaparece

**Erros comuns a evitar:**
- Placeholders como substituto de labels (desaparecem ao digitar)
- Forçar o usuário a lembrar dados de telas anteriores
- Menus colapsados por padrão em contextos onde o usuário precisa de acesso frequente
- Remover contexto ao navegar para subpáginas

---

### H7 — Flexibilidade e Eficiência de Uso
**Descrição:** Aceleradores — invisíveis para o usuário novato — permitem que usuários experientes realizem interações mais rapidamente. Permitir que usuários personalizem ações frequentes.

**Como aplicar:**
- Atalhos de teclado para ações frequentes
- Macros ou templates para tarefas repetitivas
- Ações em lote (bulk actions)
- Comandos rápidos (command palette estilo VS Code)

**Exemplos concretos:**
- Atalhos de teclado (Ctrl+K para busca rápida, G+I para ir a Issues no GitHub)
- Filtros e buscas salvas
- Bulk select + delete/move para gerenciar múltiplos itens
- Sugestões inteligentes baseadas em comportamento anterior

**Erros comuns a evitar:**
- Interface que só funciona via mouse, sem atalhos de teclado
- Exigir N cliques para uma ação frequente (deveria ter atalho)
- Não permitir customização de fluxos repetitivos
- Ignorar power users em favor de apenas usuários novatos

---

### H8 — Design Estético e Minimalista
**Descrição:** Diálogos não devem conter informações irrelevantes ou raramente necessárias. Cada unidade de informação extra compete com informações relevantes e diminui sua visibilidade relativa.

**Como aplicar:**
- Remover elementos decorativos que não comunicam nada
- Hierarquia visual clara — nem tudo pode ser igualmente importante
- Progressive disclosure: mostrar detalhes apenas quando necessário
- Espaço em branco como elemento de design

**Exemplos concretos:**
- Dashboard que mostra apenas KPIs principais, com drill-down para detalhes
- Formulário de onboarding dividido em etapas, não tudo de uma vez
- Modal de confirmação com apenas texto essencial e dois botões
- Email marketing com uma única CTA principal

**Erros comuns a evitar:**
- Dashboards sobrecarregados com 20+ métricas sem hierarquia
- Formulários com todos os campos opcionais na tela inicial
- Múltiplos CTAs igualmente proeminentes na mesma tela
- Textos longos onde um ícone + label curto resolveria

---

### H9 — Ajudar Usuários a Reconhecer, Diagnosticar e Recuperar de Erros
**Descrição:** Mensagens de erro devem ser expressas em linguagem simples (sem código de erro), indicar precisamente o problema e sugerir construtivamente uma solução.

**Como aplicar:**
- Mensagens de erro em linguagem humana, não técnica
- Indicar exatamente onde está o erro (próximo ao campo, não no topo da página)
- Sugerir como corrigir ("Use pelo menos 8 caracteres" em vez de "Senha inválida")
- Manter o conteúdo já preenchido após o erro

**Exemplos concretos:**
- "O email já está em uso. Tente fazer login ou redefinir sua senha." (com link)
- Validação inline: campo bordado em vermelho com "Formato inválido: use nome@exemplo.com"
- "Não foi possível conectar. Verifique sua internet e tente novamente." com botão Tentar Novamente
- Form que mantém todos os dados preenchidos após erro de validação

**Erros comuns a evitar:**
- "Error 500: Internal Server Error" para o usuário final
- Mensagem de erro no topo sem indicar qual campo tem problema
- Limpar todos os campos após erro de submit
- "Campo inválido" sem explicar o formato esperado

---

### H10 — Ajuda e Documentação
**Descrição:** Mesmo que seja melhor que o sistema possa ser usado sem documentação, pode ser necessário fornecer ajuda. Essa informação deve ser fácil de buscar, focada na tarefa do usuário, listar etapas concretas e não ser muito extensa.

**Como aplicar:**
- Tooltips contextuais para campos ou ações complexas
- Documentação buscável e organizada por tarefas, não por features
- Onboarding progressivo para usuários novos
- Links para ajuda próximos às áreas relevantes (não apenas no rodapé)

**Exemplos concretos:**
- Tooltip "?" ao lado de campo técnico explicando o que colocar
- Central de ajuda com busca por palavras-chave ("como exportar relatório")
- Checklist de onboarding que guia o usuário nas primeiras ações
- "Saiba mais" inline em contexto, abrindo documentação relevante

**Erros comuns a evitar:**
- Documentação organizada por estrutura interna do produto (não por tarefa do usuário)
- Help center sem buscas ou com busca ruim
- Tooltips longos demais (mais de 2 linhas)
- Esconder toda a ajuda atrás de um único link genérico "FAQ"

---

## Parte 2 — Leis de UX

Compiladas por Jon Yablonski em [lawsofux.com](https://lawsofux.com). Princípios da psicologia cognitiva e comportamental aplicados ao design de interfaces.

---

### Lei de Fitts
**Nome completo:** Fitts's Law
**Descrição:** O tempo para alcançar um alvo é uma função da distância até o alvo e do tamanho do alvo. Alvos maiores e mais próximos são mais fáceis de clicar.

**Fórmula simplificada:** `T = a + b × log₂(1 + D/W)` onde D = distância, W = largura do alvo.

**Como aplicar:**
- Botões de ação primária devem ser grandes e próximos do cursor esperado
- Menus de contexto aparecem onde o usuário já está (próximos ao clique)
- CTAs no final de formulários (onde o olho naturalmente termina)
- Cantos e bordas da tela são "infinitamente" grandes (cursor para naturalmente)

**Exemplos concretos:**
- Dock do macOS: ícones grandes e na borda inferior
- Botão "Confirmar" grande ao final de um fluxo de checkout
- Menu de contexto aparece no ponto do clique direito
- Botão flutuante (FAB) grande em mobile, fácil de atingir com o polegar

**Erros comuns a evitar:**
- Botões minúsculos para ações primárias
- Links de texto inline para ações críticas
- Botão "Confirmar" pequeno longe do botão "Cancelar" grande
- Hit targets menores que 44x44px em mobile (guideline Apple/Google)

---

### Lei de Hick
**Nome completo:** Hick's Law (Hick-Hyman Law)
**Descrição:** O tempo para tomar uma decisão aumenta logaritmicamente conforme o número e a complexidade das escolhas aumentam. O dobro de opções não significa o dobro do tempo — é logarítmico.

**Como aplicar:**
- Reduzir opções de menu a um mínimo necessário
- Categorizar opções em grupos lógicos
- Progressive disclosure: mostrar opções avançadas somente quando solicitado
- Onboarding com escolhas simples em cada etapa

**Exemplos concretos:**
- Netflix: destaca um conteúdo "Para você" em vez de apenas listar tudo
- Menu de navegação com 5-7 itens principais, não 15
- Wizard de configuração que coleta uma informação por vez
- "Planos" com 3 opções (Basic, Pro, Enterprise) em vez de 10 variações

**Erros comuns a evitar:**
- Dropdowns com 50+ opções sem busca ou agrupamento
- Menus de navegação com dezenas de itens no mesmo nível
- Formulários com muitos campos opcionais apresentados de uma vez
- Apresentar todas as features de um produto na tela inicial

---

### Lei de Jakob
**Nome completo:** Jakob's Law
**Descrição:** Usuários passam a maior parte do tempo em outros sites. Isso significa que preferem que seu site funcione da mesma forma que todos os outros que já conhecem.

**Como aplicar:**
- Seguir padrões estabelecidos de UI (logo no canto superior esquerdo, nav no topo)
- Usar ícones universalmente reconhecidos para ações comuns
- Adotar convenções de plataforma (iOS, Android, Web)
- Não reinventar padrões de interação sem razão clara

**Exemplos concretos:**
- Ícone de hambúrguer (☰) para menu mobile é imediatamente reconhecido
- Logo no canto superior esquerdo leva à homepage
- Ícone de lupa para busca, envelope para email, sino para notificações
- Checkout seguindo o padrão: carrinho → endereço → pagamento → confirmação

**Erros comuns a evitar:**
- Inventar novos padrões de navegação sem necessidade real
- Ícones não convencionais para ações comuns sem label
- Colocar elementos em posições inesperadas por razão puramente estética
- Reinventar o scroll, hover, ou comportamentos nativos do browser

---

### Lei de Miller
**Nome completo:** Miller's Law (The Magical Number Seven, Plus or Minus Two)
**Descrição:** A pessoa média consegue manter apenas 7 (±2) itens na memória de trabalho. Grupos maiores excedem a capacidade cognitiva e causam erros e frustração.

**Como aplicar:**
- Limitar itens de menu a 5-9 por nível
- Agrupar informações relacionadas (chunking)
- Dividir formulários longos em seções temáticas
- Não exibir mais de 7 opções em um único grupo sem hierarquia

**Exemplos concretos:**
- Número de telefone formatado como (11) 9 8765-4321 em vez de 11987654321
- Cartão de crédito dividido em grupos de 4 dígitos: 1234 5678 9012 3456
- Menu de navegação com 5-7 itens principais
- Formulário dividido em seções: "Dados pessoais", "Endereço", "Pagamento"

**Erros comuns a evitar:**
- Listas de opções com 20+ itens sem agrupamento
- Formulários com 30 campos numa única tela sem separação visual
- Números longos sem formatação (CEP, telefone, CPF)
- Tabelas com 15+ colunas sem priorização ou ocultação opcional

---

### Princípios Gestalt
**Origem:** Escola de psicologia alemã (início do séc. XX). "O todo é diferente da soma das partes."

Os princípios Gestalt mais relevantes para UX:

#### Proximidade
Objetos próximos tendem a ser percebidos como um grupo.
- Agrupar visualmente campos relacionados de um formulário
- Separar seções com espaço em branco (não apenas linhas)
- Labels próximas ao campo que pertencem

#### Similaridade
Elementos com aparência similar são percebidos como relacionados.
- Botões primários têm cor e estilo consistentes
- Links têm o mesmo estilo em toda a interface
- Itens da mesma categoria têm o mesmo tratamento visual

#### Continuidade
O olho tende a seguir linhas e curvas, agrupando elementos ao longo de uma trajetória.
- Sliders, carousels e listas horizontais indicam que há mais conteúdo
- Progress bars indicam direção e progresso
- Layout em grade cria linhas imaginárias que organizam o olhar

#### Fechamento (Closure)
O cérebro completa formas incompletas para criar um todo significativo.
- Carrossel onde a última imagem visível é parcialmente cortada indica que há mais
- Conteúdo cortado na dobra da tela indica scroll
- Ícones simplificados ainda são reconhecidos mesmo sem todos os detalhes

#### Figura e Fundo (Figure-Ground)
O cérebro separa automaticamente elementos em "figura" (foco) e "fundo" (contexto).
- Modais com fundo escurecido isolam o conteúdo principal
- Cards com sombra se destacam do fundo
- Contraste adequado define o que é conteúdo vs. estrutura

#### Região Comum
Elementos dentro de uma área delimitada são percebidos como grupo.
- Cards agrupam informações relacionadas
- Caixas, bordas e fundos coloridos definem seções
- Formulários com fieldsets claramente delimitados

**Erros comuns a evitar (Gestalt):**
- Labels distantes dos campos (viola Proximidade)
- Elementos com estilos inconsistentes no mesmo nível hierárquico (viola Similaridade)
- Conteúdo que não indica se há mais itens além da dobra (viola Continuidade/Fechamento)
- Baixo contraste entre figura e fundo (viola Figura-Fundo)

---

### Lei de Von Restorff (Efeito de Isolamento)
**Descrição:** Quando múltiplos objetos similares estão presentes, aquele que difere dos demais é mais provável de ser lembrado.

**Como aplicar:**
- Destacar o plano recomendado em uma tabela de preços
- Usar cor/tamanho diferente para a CTA principal
- Notificações e badges chamam atenção por serem diferentes do contexto

**Erros comuns a evitar:**
- Destacar tudo (quando tudo é especial, nada é)
- Usar o efeito para enganar usuários ("dark patterns")

---

### Efeito de Posição Serial
**Descrição:** Usuários tendem a lembrar melhor os primeiros e últimos itens de uma série (primazia e recência). Itens no meio são menos lembrados.

**Como aplicar:**
- Colocar itens mais importantes no início e no fim de listas e menus
- CTA principal no final de uma página (recência) ou no início (primazia)
- Informações críticas não devem ficar apenas no meio de um conteúdo longo

**Erros comuns a evitar:**
- Enterrar a ação mais importante no meio de um menu
- Informações críticas de segurança apenas no corpo de um texto longo

---

### Lei de Tesler (Lei da Conservação da Complexidade)
**Descrição:** Para qualquer sistema existe uma certa quantidade de complexidade que não pode ser reduzida. Se não for tratada no design, será repassada ao usuário.

**Como aplicar:**
- Absorver complexidade no sistema (backend, defaults inteligentes) para simplificar a UI
- Aceitar que formulários complexos de domínio complexo não podem ser simplificados além de um ponto
- Não simplificar removendo features que o usuário precisa

**Erros comuns a evitar:**
- Simplificar a UI transferindo trabalho para o usuário (ex.: forçar formato manual em vez de converter automaticamente)
- Remover funcionalidades em nome da "simplicidade" sem entender o caso de uso

---

### Lei de Parkinson
**Descrição:** Qualquer tarefa se expande até ocupar todo o tempo disponível.

**Como aplicar (UX):**
- Definir timeouts e limites em fluxos críticos
- Progress indicators criam senso de urgência e conclusão
- Formulários com estimativa de tempo ("Leva 2 minutos") incentivam conclusão

---

### Efeito de Pico-Final (Peak-End Rule)
**Descrição:** Pessoas julgam uma experiência principalmente por como se sentiram no pico (melhor ou pior momento) e no final, não pela média de todos os momentos.

**Como aplicar:**
- Investir na experiência do momento de sucesso (onboarding completo, primeira compra, etc.)
- Finalizar fluxos com celebração e reforço positivo
- Tratar momentos de erro como prioridade (são potenciais "picos negativos")

**Exemplos concretos:**
- Animação de confetti ao completar onboarding (Duolingo, Notion)
- Email de confirmação de compra bem elaborado
- Tratamento cuidadoso de páginas de erro 404

---

## Parte 3 — Boas Práticas Gerais de UX

### Hierarquia Visual
- Usar tamanho, peso, cor e espaço para comunicar importância relativa
- Uma única CTA primária por tela; demais são secundárias ou terciárias
- Contraste mínimo WCAG AA: 4.5:1 para texto normal, 3:1 para texto grande

### Feedback e Resposta do Sistema
- < 100ms: ação parece instantânea
- 100ms – 1s: usuário percebe o delay, mas não perde o fio
- 1s – 10s: mostrar indicador de progresso
- > 10s: mostrar progresso com estimativa de tempo e possibilidade de cancelar

### Mobile First
- Hit targets mínimos: 44x44px (Apple) / 48x48dp (Google)
- Zona de conforto do polegar: parte inferior e central da tela
- Evitar gestos não-descobertos sem affordance visual

### Acessibilidade (a11y)
- Contraste de cor adequado (WCAG AA mínimo)
- Todos os elementos interativos acessíveis por teclado
- Labels descritivos para screen readers (não apenas "Clique aqui")
- Não depender apenas de cor para comunicar estado (usar também ícone ou texto)

### Carga Cognitiva
- Reduzir o número de decisões por tela
- Defaults inteligentes baseados em comportamento comum
- Progressive disclosure: mostrar complexidade gradualmente
- Evitar jargão técnico, preferir linguagem simples

### Formulários
- Labels sempre visíveis (não apenas como placeholder)
- Validação inline em tempo real
- Agrupar campos relacionados
- Campos opcionais marcados como "(opcional)", não os obrigatórios com "*" sem explicação
- Mensagens de erro próximas ao campo, descritivas e com solução sugerida

### Navegação
- Indicar claramente a localização atual
- Breadcrumbs em hierarquias profundas
- Máximo 3 cliques para qualquer conteúdo importante (diretriz prática)
- Links e botões com labels descritivos do destino/ação

### Micro-interações
- Animações com propósito: comunicar estado, não apenas decorar
- Duração ideal: 200-500ms (rápido o suficiente para não frustrar, lento o suficiente para ser percebido)
- Evitar animações que bloqueiam a interação

---

## Checklist de Avaliação Heurística

Antes de concluir um design ou revisar uma interface:

**Nielsen Heuristics:**
- [ ] H1: Feedback claro para todas as ações e estados do sistema
- [ ] H2: Linguagem do domínio do usuário, não jargão técnico
- [ ] H3: Saídas de emergência visíveis (cancelar, desfazer)
- [ ] H4: Vocabulário e padrões consistentes em toda a interface
- [ ] H5: Validações previnem erros antes do submit
- [ ] H6: Opções visíveis, sem exigir memorização
- [ ] H7: Atalhos e eficiência para usuários experientes
- [ ] H8: Sem informações desnecessárias; hierarquia clara
- [ ] H9: Mensagens de erro claras, em linguagem humana, com solução
- [ ] H10: Ajuda contextual disponível onde necessário

**Leis de UX:**
- [ ] Fitts: alvos de ação grandes o suficiente e próximos ao contexto de uso
- [ ] Hick: número de escolhas reduzido ao necessário, agrupado logicamente
- [ ] Jakob: padrões estabelecidos respeitados, sem reinvenção desnecessária
- [ ] Miller: não mais de 7±2 itens por grupo, chunking aplicado
- [ ] Gestalt: proximidade, similaridade e continuidade aplicados na composição

**Geral:**
- [ ] Contraste WCAG AA em todos os textos
- [ ] Hit targets ≥ 44x44px em mobile
- [ ] Feedback de sistema em < 1s ou com indicador de progresso
- [ ] Todos os fluxos testados com usuários reais (ou ao menos com heuristic walkthrough)

---
