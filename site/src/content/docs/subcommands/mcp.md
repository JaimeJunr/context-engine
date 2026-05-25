---
title: ctx mcp
description: Native MCP server exposing ctx pipelines as 10 tools via stdio.
---

`ctx mcp serve` runs a Model Context Protocol server over stdio. Compatible MCP clients (Claude Code, Cursor, opencode, MCP Inspector) can connect and call ctx pipelines as tools.

## Subcommands

```bash
ctx mcp serve      # long-running stdio server
ctx mcp tools      # list exposed tools (debug)
```

## Tools exposed

| Tool | Wraps | Use case |
|---|---|---|
| `ctx_exec` | `pipelines::exec::run_proxy_capture` | Run a shell command with compression |
| `ctx_search` | `pipelines::catalog::search` | Semantic search in a catalog |
| `ctx_map` | `pipelines::map::run` | Generate a repo map |
| `ctx_list` | `pipelines::catalog::list_collections` | List indexed catalogs |
| `ctx_graph_index` | `pipelines::graph::index` | Index the symbol graph |
| `ctx_callers` | `pipelines::graph::callers` | Who calls symbol X? |
| `ctx_callees` | `pipelines::graph::callees` | What does symbol X call? |
| `ctx_trace` | `pipelines::graph::trace` | Caller chain up to depth |
| `ctx_impact` | `pipelines::graph::impact` | What breaks on change? |
| `ctx_node` | `pipelines::graph::node` | Where is X defined? |

## How it's registered with Claude Code

`ctx install --agent claude-code` writes this into `~/.claude/settings.json`:

```json
{
  "mcpServers": {
    "ctx": {
      "command": "ctx",
      "args": ["mcp", "serve"],
      "_installer": "ctx"
    }
  }
}
```

The `_installer: "ctx"` marker lets `ctx uninstall` remove only our entries without touching MCP servers added by other tools.

## Manual test

```bash
# Send initialize + tools/list to a running ctx mcp serve
(
  echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}'
  echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'
  echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
  sleep 1
) | ctx mcp serve
```

## Implementation

Built on top of [`rmcp 1.7`](https://docs.rs/rmcp) with `schemars 1.0` for automatic JSON schema generation. Each tool is an `async fn` annotated with `#[tool(...)]` and the schema is derived from the input struct.

See [`src/integrations/mcp/server.rs`](https://github.com/JaimeJunr/context-engine/blob/main/src/integrations/mcp/server.rs) for the full implementation (~250 lines for all 10 tools).
