# Troubleshooting

## Problemas Comuns

### `ctx map` retorna erro "arquivo não encontrado"
Verifique se os diretórios existem e estão com permissão de leitura:
```bash
ls -la src/auth src/models
```

### Embeddings não estão sendo gerados
Certifique-se que Ollama está rodando:
```bash
ollama serve
# Em outro terminal:
ollama pull nomic-embed-text
```

### Cache desatualizado
Force re-parse:
```bash
ctx map --no-cache --title "..." --dirs ...
```

### Performance lenta
- Use `--no-cache` apenas quando necessário
- Reduza `--max-tokens` se a saída for grande
- Verifique disponibilidade de memória (rayon paraleliza parsing)
