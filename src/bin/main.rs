use yapp::app::App::{self as App};
use yapp::core::logger;
use jsonwebtoken::errors;
use yapp::app::Hooks;

// #[tokio::main]
// async fn main() -> loco_rs::Result<()> {
//     cli::main::<App>().await.expect("TODO: panic message");
//
//     telemetry::init().await;
//
//     // Initialize Kubernetes controller state
//     let state = State::default();
//     run(state.clone()).await; // Ensure `run` is awaited
//     Ok(())
// }


use clap::Parser;
use loco_rs::{
    boot::{create_app, create_context, list_endpoints, start, ServeParams, StartMode},
    cli::{Cli, Commands},
    environment::{resolve_from_env, Environment},
    prelude::*,
};
use loco_rs::app::Hooks;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let environment: Environment = cli.environment.clone().unwrap_or_else(resolve_from_env).into();

    // Load application configuration
    let config = App::load_config(&environment).await?;

    // Initialize logger
    if !App::init_logger(&config, &environment)? {
        logger::init::<App>(&config.logger)?;
    }

    // Handle CLI commands
    match cli.command {
        Commands::Start {
            worker,
            server_and_worker,
            binding,
            port,
            no_banner,
        } => {
            let start_mode = if worker {
                StartMode::WorkerOnly
            } else if server_and_worker {
                StartMode::ServerAndWorker
            } else {
                StartMode::ServerOnly
            };

            let boot_result = create_app::<App, M>().await?;

            let serve_params = ServeParams {
                port: port.unwrap_or(boot_result.app_context.config.server.port),
                binding: binding.unwrap_or_else(|| boot_result.app_context.config.server.binding.to_string()),
            };

            start::<App>(boot_result, serve_params, no_banner).await?;
        }
        Commands::Routes {} => {
            let app_context = create_context::<App>(&environment).await?;
            list_endpoints::<App>(&app_context);
        }
        // Add other command handlers as needed...
        _ => unimplemented!("Command not yet implemented"),
    }

    Ok(())
}