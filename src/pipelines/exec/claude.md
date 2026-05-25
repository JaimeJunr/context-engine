# Instruções: `src/exec/` — Compressão Inteligente de Output

Módulo de **compressão inteligente de output** — aplica filtros contextuais em saída de comandos (logs, errors, etc) para economizar tokens sem perder informação crítica.

## Estrutura

```
exec/
├── mod.rs         # API pública (compress)
├── pipeline.rs    # 8 estágios de filtragem (summarize, truncate, etc)
├── types.rs       # Configuração de filtros (FilterConfig, etc)
└── metrics.rs     # Tracking de economia de tokens
```

## Fluxo

```
Raw Output (logs, errors, etc)
    ↓ (pipeline.rs — 8 estágios)
Stage 1: Remove blank lines
Stage 2: Remove ansi codes
Stage 3: Summarize (deduplicate + aggregate)
Stage 4: Truncate lines
Stage 5: Filter by level
Stage 6: Group by category
Stage 7: Sample (1 de N)
Stage 8: Final truncate
    ↓
Compressed Output
    ↓ (metrics.rs)
Token Economy: 60-90%
```

## API Pública (mod.rs)

```rust
pub fn compress(
    raw_output: &str,
    config: &CompressConfig,
) -> Result<CompressedOutput>

pub struct CompressedOutput {
    pub compressed: String,
    pub metrics: CompressionMetrics,
}

pub struct CompressionMetrics {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub ratio: f32,  // output/input
    pub filters_applied: Vec<String>,
}
```

## Tipos (types.rs)

```rust
pub struct CompressConfig {
    pub max_output_tokens: usize,     // default: 2000
    pub max_line_length: usize,       // default: 120 chars
    pub remove_blank_lines: bool,     // default: true
    pub remove_ansi_codes: bool,      // default: true
    pub summarize: bool,              // default: true
    pub truncate_lines: bool,         // default: true
    pub filter_by_level: Option<Level>, // error, warn, info, debug
    pub group_by_category: bool,      // default: true
    pub sample_ratio: Option<f32>,    // 0.1 = 1 de 10, default: None
}

pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
}
```

## Pipeline (pipeline.rs)

### Estágio 1: Remove Blank Lines

```rust
fn remove_blank_lines(output: &str) -> String {
    output.lines()
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
```

### Estágio 2: Remove ANSI Codes

```rust
fn remove_ansi_codes(output: &str) -> String {
    // Remove colors, styles: \x1b[...m patterns
    let re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(output, "").to_string()
}
```

### Estágio 3: Summarize (Deduplicate + Aggregate)

```rust
fn summarize(output: &str) -> String {
    // Detect patterns: "repeated line X times"
    let lines = output.lines().collect::<Vec<_>>();
    let mut dedup = Vec::new();
    let mut count = 1;
    
    for (i, line) in lines.iter().enumerate() {
        if i > 0 && line == lines[i - 1] {
            count += 1;
        } else {
            if count > 1 {
                dedup.push(format!("{} (x{})", line, count));
            } else {
                dedup.push(line.to_string());
            }
            count = 1;
        }
    }
    
    dedup.join("\n")
}
```

**Exemplo:**

```
Input:
Error: timeout
Error: timeout
Error: timeout
Error: connection reset

Output:
Error: timeout (x3)
Error: connection reset
```

### Estágio 4: Truncate Lines

```rust
fn truncate_lines(output: &str, max_len: usize) -> String {
    output.lines()
        .map(|l| {
            if l.len() > max_len {
                format!("{}...", &l[..max_len])
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
```

### Estágio 5: Filter by Level

```rust
fn filter_by_level(output: &str, level: Level) -> String {
    let pattern = match level {
        Level::Error => "[ERROR]|error:",
        Level::Warn => "[WARN]|warning:",
        Level::Info => "[INFO]|info:",
        Level::Debug => "[DEBUG]|debug:",
    };
    
    let re = Regex::new(pattern).unwrap();
    output.lines()
        .filter(|l| re.is_match(l))
        .collect::<Vec<_>>()
        .join("\n")
}
```

### Estágio 6: Group by Category

Detecta padrões (file:line, error codes) e agrupa:

```
src/lib.rs:42: error: undefined variable
src/lib.rs:43: error: undefined variable
src/main.rs:10: error: type mismatch

→ Compressido:
src/lib.rs: 2 errors
src/main.rs: 1 error
```

### Estágio 7: Sample

Se output.len() > threshold, amostra 1 de N:

```rust
fn sample(output: &str, ratio: f32) -> String {
    let lines = output.lines().collect::<Vec<_>>();
    let step = (1.0 / ratio) as usize;
    
    lines.iter()
        .enumerate()
        .filter(|(i, _)| i % step == 0)
        .map(|(_, l)| *l)
        .collect::<Vec<_>>()
        .join("\n")
}
```

### Estágio 8: Final Truncate

Corta ao `max_output_tokens`:

```rust
fn final_truncate(output: &str, max_tokens: usize) -> String {
    let max_chars = max_tokens * 4;  // 1 token ≈ 4 chars
    
    if output.len() > max_chars {
        format!("{}...\n[truncated {} chars]",
            &output[..max_chars],
            output.len() - max_chars
        )
    } else {
        output.to_string()
    }
}
```

## Orquestração

Em `mod.rs`:

```rust
pub fn compress(raw: &str, cfg: &CompressConfig) -> Result<CompressedOutput> {
    let input_tokens = raw.len() / 4;
    
    let mut output = raw.to_string();
    let mut applied = Vec::new();
    
    // Pipeline
    if cfg.remove_blank_lines {
        output = pipeline::remove_blank_lines(&output);
        applied.push("remove_blank_lines".to_string());
    }
    
    if cfg.remove_ansi_codes {
        output = pipeline::remove_ansi_codes(&output);
        applied.push("remove_ansi_codes".to_string());
    }
    
    if cfg.summarize {
        output = pipeline::summarize(&output);
        applied.push("summarize".to_string());
    }
    
    if cfg.truncate_lines {
        output = pipeline::truncate_lines(&output, cfg.max_line_length);
        applied.push("truncate_lines".to_string());
    }
    
    if let Some(level) = &cfg.filter_by_level {
        output = pipeline::filter_by_level(&output, level);
        applied.push("filter_by_level".to_string());
    }
    
    if cfg.group_by_category {
        output = pipeline::group_by_category(&output);
        applied.push("group_by_category".to_string());
    }
    
    if let Some(ratio) = cfg.sample_ratio {
        output = pipeline::sample(&output, ratio);
        applied.push("sample".to_string());
    }
    
    output = pipeline::final_truncate(&output, cfg.max_output_tokens);
    applied.push("final_truncate".to_string());
    
    let output_tokens = output.len() / 4;
    let ratio = output_tokens as f32 / input_tokens as f32;
    
    Ok(CompressedOutput {
        compressed: output,
        metrics: CompressionMetrics {
            input_tokens,
            output_tokens,
            ratio,
            filters_applied: applied,
        },
    })
}
```

## Métricas (metrics.rs)

```rust
pub fn record_compression(
    filter_name: &str,
    input_len: usize,
    output_len: usize,
) {
    let reduction = 1.0 - (output_len as f32 / input_len as f32);
    println!("Compression[{}]: {:.1}% reduction",
        filter_name,
        reduction * 100.0
    );
}

pub fn economy_report(metrics: &CompressionMetrics) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Input:  {} tokens", metrics.input_tokens);
    println!("Output: {} tokens", metrics.output_tokens);
    println!("Saved:  {} tokens ({:.1}%)",
        metrics.input_tokens - metrics.output_tokens,
        (1.0 - metrics.ratio) * 100.0
    );
    println!("Filters: {:?}", metrics.filters_applied);
}
```

## Exemplo de Uso

```rust
use ctx::exec::{compress, CompressConfig, Level};

let raw_logs = "...[muito output]...";

let config = CompressConfig {
    max_output_tokens: 2000,
    max_line_length: 120,
    remove_blank_lines: true,
    remove_ansi_codes: true,
    summarize: true,
    truncate_lines: true,
    filter_by_level: Some(Level::Error),
    group_by_category: true,
    sample_ratio: Some(0.5),  // 1 de 2 linhas
};

let result = compress(&raw_logs, &config)?;
println!("{}", result.compressed);
println!("\nSaved: {} tokens ({:.0}%)",
    result.metrics.input_tokens - result.metrics.output_tokens,
    (1.0 - result.metrics.ratio) * 100.0
);
```

## Testes

```bash
cargo test exec::
cargo test exec::pipeline::tests
cargo test exec::metrics::tests
```

### Teste Padrão

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_deduplicates() {
        let input = "error\nerror\nerror\nwarning";
        let output = summarize(input);
        assert!(output.contains("error (x3)"));
    }

    #[test]
    fn test_final_truncate_respects_limit() {
        let long_output = "x".repeat(10000);
        let config = CompressConfig {
            max_output_tokens: 500,
            ..Default::default()
        };
        
        let result = compress(&long_output, &config)?;
        assert!(result.metrics.output_tokens <= 500);
    }

    #[test]
    fn test_filter_by_level_preserves_errors() {
        let input = "[INFO] Starting\n[ERROR] Failed\n[DEBUG] Trace";
        let output = filter_by_level(input, Level::Error);
        assert!(output.contains("[ERROR]"));
        assert!(!output.contains("[INFO]"));
    }
}
```

## Performance

### Complexidade

- `remove_blank_lines`: O(N)
- `summarize`: O(N log N) — sorting
- `truncate_lines`: O(N)
- **Total**: O(N log N) — aceitável para outputs típicos (<1MB)

### Parallelization

Se output > 10MB, processar chunks em paralelo com `rayon`:

```rust
pub fn compress_large_file(
    file_path: &Path,
    config: &CompressConfig,
) -> Result<CompressedOutput> {
    let content = std::fs::read_to_string(file_path)?;
    
    // Split em chunks
    let chunks = content.split("\n\n")
        .par_iter()
        .map(|c| compress(c, config))
        .collect::<Result<Vec<_>>>()?;
    
    // Merge
    let merged = chunks.iter()
        .map(|c| &c.compressed)
        .collect::<Vec<_>>()
        .join("\n");
    
    Ok(CompressedOutput { compressed: merged, .. })
}
```

---

**Última atualização**: 2026-04-14
