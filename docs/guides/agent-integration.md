# Integração com Agentes (`ctx install` / `ctx uninstall`)

`ctx install --agent <name>` configura **dois mecanismos** no agente de codificação, juntos no mesmo `settings.json`:

1. **Hook `PreToolUse`** que redireciona comandos Bash cobertos para `ctx exec` automaticamente (compressão de output).
2. **MCP server `ctx`** expondo `ctx_exec`, `ctx_search`, `ctx_map`, `ctx_list` como tools nominais (chamadas explícitas pelo agente).

Os dois coexistem: o hook captura Bash calls existentes, o MCP server permite que o agente invoque tools por nome quando faz sentido (ex: `ctx_search` para busca semântica em docs indexadas).

Resultado prático: o agente roda `git status` normalmente; o hook reescreve para `ctx exec git status` antes da execução. Quando o agente quer buscar na wiki, chama `ctx_search` direto via MCP.

## Agentes suportados

| Agente | Status | Escopo padrão |
|--------|--------|---------------|
| Claude Code | ✅ disponível | `~/.claude/settings.json` |
| Claude Desktop | ✅ disponível | `~/*.../claude_desktop_config.json` |
| Cursor | 🚧 próxima entrega | — |
| Codex CLI | 🚧 próxima entrega | — |
| opencode | 🚧 próxima entrega | — |

## Uso

### Claude Code

```bash
# Instala no escopo de usuário (~/.claude/settings.json) — afeta todos os projetos
ctx install --agent claude-code

# Instala apenas no projeto atual (.claude/settings.json)
ctx install --agent claude-code --project

# Remove a instalação
ctx uninstall --agent claude-code

# Remove só do projeto
ctx uninstall --agent claude-code --project
```

### Claude Desktop

```bash
# Instala no aplicativo Claude Desktop
ctx install --agent claude-desktop

# Remove do aplicativo Claude Desktop
ctx uninstall --agent claude-desktop
```

Reinicie sessões abertas do agente para o hook entrar em vigor.

## O que é escrito no `settings.json`

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "ctx __hook claude-code-pre-tool-use",
            "_installer": "ctx"
          }
        ]
      }
    ]
  },
  "mcpServers": {
    "ctx": {
      "command": "ctx",
      "args": ["mcp", "serve"],
      "_installer": "ctx"
    }
  }
}
```

O campo `_installer: "ctx"` é o **marcador de propriedade**: o `uninstall` remove **apenas** entradas marcadas assim. Hooks e MCP servers que você ou outras ferramentas tenham configurado ficam intactos.

## Tools MCP expostas

Quando o agente cliente conecta via MCP, vê estas **10 tools**:

| Tool | Função |
|---|---|
| `ctx_exec` | Executa comando shell com filtro de compressão (mesma cobertura do hook PreToolUse) |
| `ctx_search` | Busca semântica em acervo do catalog (`collection`, `query`, `top_k`) |
| `ctx_map` | Gera repo map curado (`title`, `dirs`, `max_tokens`…) |
| `ctx_list` | Lista acervos catalogados disponíveis |
| `ctx_graph_index` | Indexa diretórios populando o grafo de símbolos |
| `ctx_callers` | Busca chamadores de um símbolo com relevância e budget de tokens |
| `ctx_callees` | Busca símbolos chamados a partir de um identificador qualificado |
| `ctx_trace` | Retorna a cadeia de callers até `depth` níveis |
| `ctx_impact` | Lista código impactado por mudanças (callers diretos e indiretos) |
| `ctx_node` | Localiza as definições de um símbolo no grafo |

Schemas de input são gerados automaticamente via `schemars` (validados no cliente antes da chamada).

Para listar via CLI: `ctx mcp tools`. Para subir o server standalone: `ctx mcp serve` (stdio long-running).

## Como o hook decide reescrever

Para cada Bash tool call, o hook (rodando como `ctx __hook claude-code-pre-tool-use`):

1. Lê JSON do stdin (`{"tool_name":"Bash","tool_input":{"command":"..."}}`).
2. Faz parse robusto do comando respeitando aspas (`shell-words`).
3. Consulta [`exec::registry::matches`](../../src/exec/registry.rs) — fonte única de verdade sobre comandos cobertos.
4. Se cobre → devolve `{"hookSpecificOutput":{"hookEventName":"PreToolUse","modifiedToolInput":{"command":"ctx exec <original>"}}}`.
5. Se não cobre → devolve `{}` (passthrough).

### Comandos não reescritos

- Comandos sem filtro registrado (ex: `echo`, `cat`)
- Comandos que já começam com `ctx exec` ou `ctx __hook` (evita loop infinito)
- Tool calls que não são `Bash`
- Input malformado (degradação silenciosa)

## Degradação suave

O handler do hook **sempre** sai com exit 0. Qualquer erro interno (parse falho, registry sem resposta, JSON malformado) vira passthrough silencioso (`{}`). Isso garante que uma sessão do Claude Code **nunca quebre** por causa do ctx — no pior caso, comandos rodam sem filtragem.

## Reverter manualmente

Se algo der errado e o uninstall não resolver, abra `~/.claude/settings.json` e remova manualmente entradas que tenham `_installer: "ctx"`. Não toque em outros hooks.

## Diferenças vs concorrentes

Ver [docs/competitors/](../competitors/) para a análise completa.

- **RTK** usa `rtk init -g` para escrever em arquivos por agente — mesma ideia do hook, mas RTK é proxy CLI separado e cobre 100+ comandos com regras hardcoded.
- **Context Mode** roda como MCP server e intercepta no protocolo, não via PreToolUse hook. Nosso `ctx` faz os dois (hook + MCP) coexistindo.
- **CodeGraph** é MCP-only com tools focadas em grafo de símbolos (callers/callees/trace) — eixo adjacente ao nosso.
