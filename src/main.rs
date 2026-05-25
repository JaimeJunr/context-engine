use anyhow::Result;
use clap::{Parser, Subcommand};
use context_engine::integrations::agents::{installer_for, AgentName, Scope};
use context_engine::pipelines::catalog::{self, types::Collection};
use context_engine::pipelines::map::run;
use context_engine::shared::cache;

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
        /// Profundidade máxima de scan de diretórios (default: 15)
        #[arg(long, default_value = "15")]
        max_depth: usize,
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

    /// Configura interativamente endpoint e modelos LLM
    #[command(name = "setup", alias = "init-llm")]
    Setup,

    /// Detecta stack do projeto e gera .ctx/config.toml local
    #[command(name = "init")]
    WorkspaceInit {
        /// Diretório alvo (padrão: diretório atual)
        #[arg(long, default_value = ".")]
        path: String,
        /// Sobrescreve .ctx/config.toml se já existir
        #[arg(long)]
        force: bool,
    },

    /// Gerencia configuração global (~/.ctx/config.toml)
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
    },

    /// Registra e indexa um diretório em um único passo (add + index)
    Bootstrap {
        /// Diretório a indexar
        #[arg(long)]
        path: String,
        /// Nome da collection (padrão: nome do diretório)
        #[arg(long)]
        name: Option<String>,
    },

    /// Proxy de comandos com economia de tokens
    Exec {
        #[command(subcommand)]
        cmd: ExecCommand,
    },

    /// Instala integração (hooks) em um agente de codificação
    Install {
        /// Agente alvo (ex: claude-code)
        #[arg(long, value_enum)]
        agent: AgentName,
        /// Instala no projeto atual (.claude/) em vez do escopo de usuário (~/.claude/)
        #[arg(long)]
        project: bool,
        /// Sobrescreve configuração existente sem perguntar
        #[arg(long)]
        force: bool,
    },

    /// Remove a integração instalada por `ctx install` em um agente
    Uninstall {
        /// Agente alvo
        #[arg(long, value_enum)]
        agent: AgentName,
        /// Remove do projeto atual em vez do escopo de usuário
        #[arg(long)]
        project: bool,
    },

    /// Handler interno de hook (não invocar manualmente)
    #[command(name = "__hook", hide = true)]
    Hook {
        /// Nome do hook (ex: claude-code-pre-tool-use)
        name: String,
    },

    /// MCP server (Model Context Protocol)
    Mcp {
        #[command(subcommand)]
        cmd: McpCommand,
    },

    /// Grafo de símbolos (callers/callees/trace/impact/node)
    Graph {
        #[command(subcommand)]
        cmd: GraphCommand,
    },
}

#[derive(Subcommand)]
enum GraphCommand {
    /// Indexa diretórios populando o grafo
    Index {
        /// Diretórios alvo separados por vírgula
        #[arg(long, default_value = ".")]
        dirs: String,
        /// Profundidade máxima de scan (default: 15)
        #[arg(long, default_value_t = 15)]
        max_depth: usize,
    },
    /// Quem chama o símbolo?
    Callers {
        /// Nome do símbolo
        name: String,
        /// Query para ranquear resultados
        #[arg(long)]
        query: Option<String>,
        /// Budget de tokens
        #[arg(long = "max-tokens")]
        max_tokens: Option<usize>,
    },
    /// O que esta função chama?
    Callees {
        /// Identificador qualificado (file::name)
        qualified: String,
        #[arg(long)]
        query: Option<String>,
        #[arg(long = "max-tokens")]
        max_tokens: Option<usize>,
    },
    /// Cadeia de callers até este símbolo
    Trace {
        name: String,
        #[arg(long, default_value_t = 3)]
        depth: usize,
        #[arg(long)]
        query: Option<String>,
        #[arg(long = "max-tokens")]
        max_tokens: Option<usize>,
    },
    /// O que quebra se eu mudar este símbolo?
    Impact {
        name: String,
        #[arg(long, default_value_t = 2)]
        depth: usize,
    },
    /// Detalhes de um símbolo (onde está definido)
    Node { name: String },
}

#[derive(Subcommand)]
enum McpCommand {
    /// Sobe o MCP server em stdio (long-running)
    Serve,
    /// Lista as tools expostas pelo MCP server
    Tools,
}

#[derive(Subcommand)]
enum ExecCommand {
    /// Relatório de economia de tokens
    Report {
        /// Formato JSON
        #[arg(long)]
        json: bool,
        /// Filtrar por projeto (diretório)
        #[arg(long)]
        project: Option<String>,
        /// Janela de dias (padrão: todos)
        #[arg(long)]
        days: Option<u32>,
    },
    /// Executa qualquer comando, aplicando filtro se disponível (passthrough caso contrário)
    #[command(external_subcommand)]
    Run(Vec<String>),
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// Define um valor de configuração (ex: llm.endpoint)
    Set { key: String, value: String },
    /// Lê um valor de configuração
    Get { key: String },
    /// Lista toda a configuração atual
    List,
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
            max_depth,
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

            let result = run(
                &title,
                &dirs,
                top,
                max_tokens,
                seeds.as_deref(),
                &fmt,
                max_depth,
            );
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

        Commands::Setup => {
            context_engine::shared::config::run_init_wizard()?;
        }

        Commands::WorkspaceInit { path, force } => {
            match context_engine::shared::workspace::run_workspace_init(&path, force) {
                Ok(report) => println!("{}", report),
                Err(e) => {
                    eprintln!("erro: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Config { cmd } => {
            let mut cfg = context_engine::shared::config::load()?;
            match cmd {
                ConfigCmd::Set { key, value } => {
                    context_engine::shared::config::set_key(&mut cfg, &key, &value)?;
                    context_engine::shared::config::save(&cfg)?;
                    println!("{} = {}", key, value);
                }
                ConfigCmd::Get { key } => {
                    match context_engine::shared::config::get_key(&cfg, &key)? {
                        Some(v) => println!("{}", v),
                        None => println!("(não definido)"),
                    }
                }
                ConfigCmd::List => {
                    let path = context_engine::shared::config::config_path();
                    println!("Config: {}\n", path.display());
                    println!(
                        "llm.endpoint = {}",
                        cfg.llm.endpoint.as_deref().unwrap_or("(não definido)")
                    );
                    println!(
                        "llm.embedder = {}",
                        cfg.llm.embedder.as_deref().unwrap_or("(não definido)")
                    );
                    println!(
                        "llm.reranker = {}",
                        cfg.llm.reranker.as_deref().unwrap_or("(não definido)")
                    );
                }
            }
        }

        Commands::Bootstrap { path, name } => {
            let path = std::path::Path::new(&path);
            if !path.exists() {
                anyhow::bail!("diretório '{}' não encontrado", path.display());
            }
            let stats = catalog::bootstrap(path, name.as_deref())?;
            println!(
                "Bootstrap '{}': {} arquivos descobertos, {} chunks indexados.",
                stats.collection_name, stats.files_discovered, stats.chunks_indexed
            );
            println!(
                "Próximo passo: ctx embed {} --batch-size 50",
                stats.collection_name
            );
        }

        Commands::Exec { cmd } => match cmd {
            ExecCommand::Report {
                json,
                project,
                days,
            } => {
                use context_engine::pipelines::exec::metrics;
                use dirs::home_dir;
                use rusqlite::Connection;

                let db_path = home_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join(".cache")
                    .join("context_engine")
                    .join("catalog.db");
                std::fs::create_dir_all(db_path.parent().unwrap())?;
                let conn = Connection::open(&db_path)?;
                metrics::migrate(&conn)?;

                let summary = metrics::aggregate_summary(&conn, project.as_deref(), days)?;

                if json {
                    println!("{}", serde_json::to_string_pretty(&summary)?);
                } else {
                    println!("=== Relatório de Economia de Tokens ===");
                    println!("Comandos executados: {}", summary.total_commands);
                    println!("Tokens entrada:      {}", summary.total_input_tokens);
                    println!("Tokens saída:        {}", summary.total_output_tokens);
                    println!("Tokens economizados: {}", summary.total_saved_tokens);
                    println!("Economia média:      {:.1}%", summary.avg_savings_percent);
                    if !summary.breakdown_by_command.is_empty() {
                        println!("\nPor comando:");
                        for b in &summary.breakdown_by_command {
                            println!(
                                "  {:30} {:5}x  -{} tokens ({:.0}%)",
                                b.command_name, b.count, b.saved_tokens, b.avg_savings_percent
                            );
                        }
                    }
                }
            }
            ExecCommand::Run(argv) => {
                use context_engine::pipelines::exec::run_proxy;
                let exit_code = run_proxy(argv)?;
                std::process::exit(exit_code);
            }
        },

        Commands::Install {
            agent,
            project,
            force,
        } => {
            let scope = if project { Scope::Project } else { Scope::User };
            let installer = installer_for(agent);
            let report = installer.install(scope, force)?;
            if report.already_installed {
                println!(
                    "{} já estava instalado em {}",
                    installer.name(),
                    report.settings_path.display()
                );
            } else {
                println!(
                    "{} instalado em {}",
                    installer.name(),
                    report.settings_path.display()
                );
                println!("Reinicie sessões abertas do agente para o hook entrar em vigor.");
            }
        }

        Commands::Uninstall { agent, project } => {
            let scope = if project { Scope::Project } else { Scope::User };
            let installer = installer_for(agent);
            let report = installer.uninstall(scope)?;
            if report.removed {
                println!(
                    "{} removido de {}",
                    installer.name(),
                    report.settings_path.display()
                );
            } else {
                println!(
                    "Nenhuma instalação do {} encontrada em {}",
                    installer.name(),
                    report.settings_path.display()
                );
            }
        }

        Commands::Hook { name } => {
            context_engine::integrations::agents::hook_handlers::dispatch(&name)?;
        }

        Commands::Graph { cmd } => {
            use context_engine::pipelines::graph::{self, QueryOptions};
            match cmd {
                GraphCommand::Index { dirs, max_depth } => {
                    let dirs: Vec<String> = dirs
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    let stats = graph::index(&dirs, max_depth)?;
                    println!(
                        "{} arquivos varridos, {} indexados, {} símbolos, {} calls, {} erros",
                        stats.files_scanned,
                        stats.files_indexed,
                        stats.symbols,
                        stats.calls,
                        stats.errors
                    );
                }
                GraphCommand::Callers {
                    name,
                    query,
                    max_tokens,
                } => {
                    let conn = graph::store::open_default()?;
                    let result = graph::callers(
                        &conn,
                        &name,
                        &QueryOptions {
                            query,
                            max_tokens,
                            depth: None,
                        },
                    )?;
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                GraphCommand::Callees {
                    qualified,
                    query,
                    max_tokens,
                } => {
                    let conn = graph::store::open_default()?;
                    let result = graph::callees(
                        &conn,
                        &qualified,
                        &QueryOptions {
                            query,
                            max_tokens,
                            depth: None,
                        },
                    )?;
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                GraphCommand::Trace {
                    name,
                    depth,
                    query,
                    max_tokens,
                } => {
                    let conn = graph::store::open_default()?;
                    let result = graph::trace(
                        &conn,
                        &name,
                        &QueryOptions {
                            query,
                            max_tokens,
                            depth: Some(depth),
                        },
                    )?;
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                GraphCommand::Impact { name, depth } => {
                    let conn = graph::store::open_default()?;
                    let result = graph::impact(
                        &conn,
                        &name,
                        &QueryOptions {
                            depth: Some(depth),
                            ..Default::default()
                        },
                    )?;
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                GraphCommand::Node { name } => {
                    let conn = graph::store::open_default()?;
                    let nodes = graph::node(&conn, &name)?;
                    println!("{}", serde_json::to_string_pretty(&nodes)?);
                }
            }
        }

        Commands::Mcp { cmd } => match cmd {
            McpCommand::Serve => {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(context_engine::integrations::mcp::serve())?;
            }
            McpCommand::Tools => {
                println!("Tools expostas pelo MCP server:");
                for name in context_engine::integrations::mcp::tool_names() {
                    println!("  {}", name);
                }
            }
        },
    }

    Ok(())
}
