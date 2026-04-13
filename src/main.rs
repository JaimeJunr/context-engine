use anyhow::Result;
use clap::{Parser, Subcommand};
use context_engine::catalog::types::Collection;
use context_engine::{cache, catalog, run};

#[derive(Parser)]
#[command(
    name = "ctx",
    version,
    about = "CLI para repo map e recuperação semântica de conhecimento"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Gera repo map curado para LLMs
    Map {
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
    },

    /// Registra ou atualiza um acervo documental
    Add {
        /// Nome único do acervo
        name: String,
        /// Diretório ou glob de origem
        #[arg(long = "source", short = 's')]
        sources: Vec<String>,
        /// Padrão glob de inclusão, ex: "**/*.md"
        #[arg(long = "include")]
        include: Vec<String>,
        /// Padrão glob de exclusão, ex: "**/node_modules/**"
        #[arg(long = "exclude")]
        exclude: Vec<String>,
        /// Modelo de embedding (padrão: nomic-embed-text)
        #[arg(long)]
        embedder_model: Option<String>,
        /// Modelo de reranking (padrão: llama3.2)
        #[arg(long)]
        reranker_model: Option<String>,
        /// Comando a executar antes de indexar (ex: "git pull")
        #[arg(long)]
        pre_index_cmd: Option<String>,
        /// Endpoint do servidor LLM (ex: http://192.168.1.10:8080)
        #[arg(long)]
        llm_endpoint: Option<String>,
    },

    /// Cataloga documentos do acervo (novos e modificados)
    Index {
        /// Nome do acervo
        name: String,
        /// Também gera embeddings para chunks pendentes após indexar
        #[arg(long)]
        with_embed: bool,
        /// Tamanho do lote de embeddings (padrão: 50)
        #[arg(long, default_value_t = 50)]
        batch_size: usize,
    },

    /// Gera embeddings para chunks pendentes
    Embed {
        /// Nome do acervo
        name: String,
        /// Tamanho do lote (padrão: 50)
        #[arg(long, default_value_t = 50)]
        batch_size: usize,
    },

    /// Busca semântica no acervo
    Search {
        /// Nome do acervo
        name: String,
        /// Query de busca (prefixos: exact:, conceptual:, expanded:)
        query: String,
        /// Número máximo de resultados (padrão: 10)
        #[arg(long, short = 'k', default_value_t = 10)]
        top_k: usize,
        /// Exibe o fragmento completo de cada resultado
        #[arg(long)]
        full: bool,
    },

    /// Lista todos os acervos registrados
    List,

    /// Exibe relatório de saúde do acervo
    Status {
        /// Nome do acervo
        name: String,
    },

    /// Compacta o repositório interno, removendo dados obsoletos
    Compact {
        /// Nome do acervo
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = execute(cli) {
        eprintln!("ERRO: {:#}", e);
        std::process::exit(1);
    }
}

fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Map {
            title,
            dirs,
            top,
            max_tokens,
            fmt,
            seeds,
            no_cache,
        } => {
            if no_cache {
                cache::NO_CACHE.store(true, std::sync::atomic::Ordering::Relaxed);
            }

            let dirs: Vec<String> = dirs
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if dirs.is_empty() {
                anyhow::bail!("nenhum diretório válido fornecido");
            }

            let seeds: Option<Vec<String>> = seeds.map(|s| {
                s.split(',')
                    .map(|x| x.trim().to_string())
                    .filter(|x| !x.is_empty())
                    .collect()
            });

            let result = run(&title, &dirs, top, max_tokens, seeds.as_deref(), &fmt);
            print!("{}", result);
        }

        Commands::Add {
            name,
            sources,
            include,
            exclude,
            embedder_model,
            reranker_model,
            pre_index_cmd,
            llm_endpoint,
        } => {
            let col = Collection {
                name: name.clone(),
                sources,
                include_patterns: include,
                exclude_patterns: exclude,
                path_contexts: vec![],
                pre_index_cmd,
                embedder_model,
                reranker_model,
                llm_endpoint,
            };
            catalog::add_collection(col)?;
            println!("Acervo '{}' registrado com sucesso.", name);
        }

        Commands::Index {
            name,
            with_embed,
            batch_size,
        } => {
            print!("Indexando '{}'... ", name);
            let stats = catalog::index(&name)?;
            println!(
                "concluído. {} varridos, {} indexados, {} sem alteração, {} erros.",
                stats.scanned, stats.indexed, stats.skipped, stats.errors
            );

            if with_embed {
                println!("Gerando embeddings...");
                let embedded = catalog::embed_pending(&name, batch_size)?;
                println!("{} chunks processados.", embedded);
            }
        }

        Commands::Embed { name, batch_size } => {
            println!("Gerando embeddings para '{}'...", name);
            let embedded = catalog::embed_pending(&name, batch_size)?;
            println!("{} chunks processados.", embedded);
        }

        Commands::Search {
            name,
            query,
            top_k,
            full,
        } => {
            let results = catalog::search(&name, &query, top_k)?;

            if results.is_empty() {
                println!("Nenhum resultado encontrado.");
                return Ok(());
            }

            for (i, r) in results.iter().enumerate() {
                println!("\n[{}] {:.4} — {}", i + 1, r.score, r.doc_path);
                if let Some(ctx) = &r.context {
                    println!("    Contexto: {}", ctx);
                }
                if full {
                    println!(
                        "    Fragmento (offset {}):\n    {}",
                        r.chunk_offset,
                        r.chunk_text.replace('\n', "\n    ")
                    );
                } else {
                    let preview: String = r.chunk_text.chars().take(200).collect();
                    let preview = if r.chunk_text.len() > 200 {
                        format!("{}…", preview)
                    } else {
                        preview
                    };
                    println!("    {}", preview.replace('\n', " "));
                }
            }
            println!();
        }

        Commands::List => {
            let collections = catalog::list_collections()?;
            if collections.is_empty() {
                println!("Nenhum acervo registrado.");
                return Ok(());
            }
            println!("{:<30} ÚLTIMA INDEXAÇÃO", "ACERVO");
            println!("{}", "-".repeat(55));
            for (name, last) in &collections {
                let last_str = last.as_deref().unwrap_or("nunca");
                println!("{:<30} {}", name, last_str);
            }
        }

        Commands::Status { name } => {
            let h = catalog::health(&name)?;
            println!("Acervo:              {}", h.name);
            println!("Documentos:          {}", h.total_documents);
            println!("Embeddings pendentes: {}", h.pending_embeddings);
            println!(
                "Última indexação:    {}",
                h.last_indexed.as_deref().unwrap_or("nunca")
            );
            println!(
                "Consistente:         {}",
                if h.consistent { "sim" } else { "não" }
            );
        }

        Commands::Compact { name } => {
            catalog::compact(&name)?;
            println!("Compactação de '{}' concluída.", name);
        }
    }

    Ok(())
}
