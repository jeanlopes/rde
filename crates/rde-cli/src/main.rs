//! RDE CLI — Entry point for the Rust Debugger Engine.

use clap::{Parser, Subcommand};
use rde_core::DebugEngine;
use rde_repl;
use rde_win32::WindowsBackend;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(name = "rde-cli")]
#[command(about = "Rust Debugger Engine — Native Windows debugger")]
struct Args {
    /// Subcommand (e.g., cargo debug)
    #[command(subcommand)]
    command: Option<Commands>,

    /// Target executable to debug (optional, can be specified via REPL)
    target: Option<String>,

    /// Arguments to pass to the target
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    target_args: Vec<String>,

    /// Run with TUI interface instead of text REPL
    #[arg(long)]
    tui: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Debug a Cargo project
    Cargo {
        #[command(subcommand)]
        command: CargoCommands,
    },
}

#[derive(Subcommand, Debug)]
enum CargoCommands {
    /// Launch debug session from a Cargo project
    Debug {
        /// Package to build (for workspaces)
        #[arg(long)]
        package: Option<String>,
        /// Target to build
        #[arg(long)]
        target: Option<String>,
        /// Build profile
        #[arg(long, default_value = "dev")]
        profile: String,
        /// Features to enable
        #[arg(long)]
        features: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.tui {
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("rde-debug.log")
            .expect("failed to open log file");
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .with_writer(move || log_file.try_clone().unwrap())
            .finish();
        tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    } else {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .finish();
        tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    }

    info!("RDE CLI starting");

    let backend = WindowsBackend::new();
    let (mut engine, command_tx, event_rx) = DebugEngine::new(backend);

    let engine_handle = tokio::spawn(async move {
        if let Err(e) = engine.run().await {
            eprintln!("Engine error: {e}");
        }
    });

    match args.command {
        Some(Commands::Cargo { command: CargoCommands::Debug { package, target, profile, features } }) => {
            let manifest_path = std::env::current_dir()
                .unwrap_or_default()
                .join("Cargo.toml");
            let _ = command_tx.send(rde_core::EngineCommand::CargoLaunch {
                manifest_path,
                package,
                target,
                profile,
                features,
            });
        }
        None if args.target.is_some() => {
            let target = args.target.unwrap();
            let path = std::path::PathBuf::from(&target);
            let path = if path.exists() {
                std::fs::canonicalize(&path).unwrap_or(path)
            } else {
                let cargo_example = std::path::PathBuf::from(format!("target/debug/examples/{}.{}",
                    path.file_stem().and_then(|s| s.to_str()).unwrap_or(&target),
                    path.extension().and_then(|s| s.to_str()).unwrap_or("exe")
                ));
                if cargo_example.exists() {
                    std::fs::canonicalize(&cargo_example).unwrap_or(cargo_example)
                } else {
                    eprintln!("Erro: executável não encontrado: {}", path.display());
                    eprintln!("Dica: compile com 'cargo build --example <nome>' e use o caminho completo.");
                    std::process::exit(1);
                }
            };
            let _ = command_tx.send(rde_core::EngineCommand::Launch {
                path,
                args: args.target_args,
            });
        }
        _ => {}
    }

    if args.tui {
        if let Err(e) = rde_tui::run(event_rx, command_tx).await {
            eprintln!("TUI error: {e}");
        }
    } else {
        rde_repl::run(event_rx, command_tx).await;
    }

    let _ = engine_handle.await;
    info!("RDE CLI exiting");
}
