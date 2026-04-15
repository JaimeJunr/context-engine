# Algoritmo de Ranking

## BM25 (Probabilistic Relevance Framework)

Score baseado em:
- Term Frequency (TF) — quantas vezes termo aparece no arquivo
- Inverse Document Frequency (IDF) — raridade do termo no corpus
- Document Length Normalization — penalidade para docs muito grandes

Fórmula:
```
score = Σ IDF(qi) * (f(qi) * (k1 + 1)) / (f(qi) + k1 * (1 - b + b * |D|/avgdl))
```

Parâmetros padrão:
- k1 = 1.5 (saturação de TF)
- b = 0.75 (normalização de tamanho)

## Personalized PageRank (PPR)

Ranking estrutural baseado em grafo de dependências.

Quando você fornece `--seeds`, o algoritmo:
1. Constrói grafo: nó = arquivo, edge = import/dependency
2. Executa PPR partindo dos seed directories
3. Nós próximos aos seeds recebem ranking mais alto

Uso comum:
```bash
# "Quero arquivos relacionados a autenticação"
ctx map --title "2FA" --dirs . --seeds src/auth --max-tokens 4000
```

## Combinação: BM25 + PPR

Score final = α * bm25_score + (1 - α) * ppr_score

Onde α = 0.7 (default, 70% léxico, 30% estrutural)

Você pode ajustar via configuração global:
```bash
ctx config set ranking.alpha 0.5  # 50/50 mix
```
