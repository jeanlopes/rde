//! RDE CLI — Entry point for the Rust Debugger Engine.

use clap::Parser;
use rde_core::DebugEngine;
use rde_repl;
use rde_win32::WindowsBackend;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(name = "rde-cli")]
#[command(about = "Rust Debugger Engine — Native Windows debugger")]
struct Args {
    /// Target executable to debug (optional, can be specified via REPL)
    target: Option<String>,

    /// Arguments to pass to the target
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    target_args: Vec<String>,
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();
    info!("RDE CLI starting");

    let backend = WindowsBackend::new();
    let (mut engine, command_tx, event_rx) = DebugEngine::new(backend);

    // Spawn engine in background
    let engine_handle = tokio::spawn(async move {
        if let Err(e) = engine.run().await {
            eprintln!("Engine error: {e}");
        }
    });

    // If target provided on command line, auto-launch
    if let Some(target) = args.target {
        let path = std::path::PathBuf::from(&target);
        let path = std::fs::canonicalize(&path).unwrap_or(path);
        let _ = command_tx.send(rde_core::EngineCommand::Launch {
            path,
            args: args.target_args,
        });
    }

    // Run REPL
    rde_repl::run(event_rx, command_tx).await;

    // Wait for engine to finish
    let _ = engine_handle.await;
    info!("RDE CLI exiting");
}
