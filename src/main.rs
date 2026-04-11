use clap::Parser;
use context_engine::{cache, run};

/// context_engine — Repo Map curado para o planejador LLM
#[derive(Parser)]
#[command(name = "ctx", version)]
struct Cli {
    /// Título ou descrição do ticket
    #[arg(long, required = true)]
    title: String,

    /// Diretórios alvo separados por vírgula
    #[arg(long, required = true)]
    dirs: String,

    /// Número fixo de arquivos retornados (0 = usar --max-tokens)
    #[arg(long, default_value_t = 0)]
    top: usize,

    /// Budget máximo de tokens para o repo_map
    #[arg(long = "max-tokens", default_value_t = 4096)]
    max_tokens: usize,

    /// Formato de saída: text ou json
    #[arg(long = "format", default_value = "text")]
    fmt: String,

    /// Dirs seed para PPR separados por vírgula (ativa Personalized PageRank)
    #[arg(long)]
    seeds: Option<String>,

    /// Ignorar cache (forçar re-parse)
    #[arg(long)]
    no_cache: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.no_cache {
        cache::NO_CACHE.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    let dirs: Vec<String> = cli
        .dirs
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if dirs.is_empty() {
        eprintln!("ERRO: nenhum diretório válido fornecido.");
        std::process::exit(1);
    }

    let seeds: Option<Vec<String>> = cli.seeds.map(|s| {
        s.split(',')
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect()
    });

    let result = run(
        &cli.title,
        &dirs,
        cli.top,
        cli.max_tokens,
        seeds.as_deref(),
        &cli.fmt,
    );

    print!("{}", result);
}
