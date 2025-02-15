use loco_rs::cli;
use tokio::signal;
use yair::{
    app::App,
    controllers::{kubecontroller::run, telemetry},
    core::kubecontroller::State,
};
use std::str::FromStr;
use loco_rs::{app::Hooks, boot::StartMode, environment::Environment, prelude::*};
use std::time::Duration;

};

#[tokio::main]
fn main() -> loco_rs::Result<()> {
    cli::main::<App>().await.expect("TODO: panic message");

    telemetry::init().await;

    // Initialize Kubernetes controller state
    let state = State::default();

    run(state.clone()).await;
}

struct ServeParams {
    binding: String,
    port: i32,
}

#[allow(clippy::redundant_pub_crate)]
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // 1. Environment setup ---------------------------------------------------
    let environment = std::env::var("ENVIRONMENT")
        .map_or(Environment::Development, |env| <Environment as FromStr>::from_str(&env).unwrap_or(Environment::Development));
    println!("🚀 Starting in {environment} environment");

    // 2. Config loading ------------------------------------------------------
    let config = App::load_config(&environment)
        .await
        .map_err(|e| {
            eprintln!("Failed to load config: {e:?}");
            e
        })?;

    // 3. Logger initialization -----------------------------------------------
    if !App::init_logger(&config, &environment).expect("Failed to initialize logger") {
        // Default Loco logger if not overridden
        tracing_subscriber::fmt()
            .with_target(false)
            .with_max_level(environment.log_level())
            .init();
    }

    // 4. Application boot ----------------------------------------------------
    let boot = App::boot(StartMode::ServerAndWorker, &environment)
        .await
        .expect("Failed to boot application");

    // 5. Server parameters ---------------------------------------------------
    let serve_params = ServeParams {
        binding: config.server.host.clone(),
        port: config.server.port,
    };

    // 6. Concurrent tasks setup ----------------------------------------------
    let (_tx, rx) = tokio::sync::oneshot::channel();

    // Server task
    let server_task = tokio::spawn({
        let ctx = boot.app_context.clone();
        let (tx, _rx) = tokio::sync::oneshot::channel();
        let router = boot.router.clone();
        async move {
            tracing::info!(
                "Starting server on {}:{}",
                serve_params.binding,
                serve_params.port
            );
            App::serve(router.expect("REASON"), &ctx)
                .await
                .expect("Server failed");

            tx.send(()).expect("Failed to send shutdown signal");
        }
    });

    // Background task
    let background_task = tokio::spawn(async move {
        tracing::info!("Background task started");
        let mut rx: tokio::sync::oneshot::Receiver<()> = rx;
        tokio::spawn(run_kubecontroller());
        loop {
            tokio::select! {
                _ = &mut rx => {
                    tracing::info!("Background task received shutdown signal");
                    break;
                }
                () = tokio::time::sleep(Duration::from_secs(5)) => {
                    tracing::info!("Background task heartbeat");
                }
            }
        }
    });

    // 7. Graceful shutdown ---------------------------------------------------
    tokio::select! {
        _ = signal::ctrl_c() => {
            tracing::info!("Received CTRL+C, shutting down");
        }
        _ = server_task => {
            tracing::info!("Server task completed");
        }
    }

    // 8. Cleanup -------------------------------------------------------------
    background_task.abort();
    App::on_shutdown(&boot.app_context).await;
    Ok(())
}
