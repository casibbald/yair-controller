use loco_rs::{app::Hooks, boot::StartMode, environment::Environment, prelude::*};
use std::time::Duration;
use tokio::signal;
use yapp::{
    app::App,
    controllers::kubecontroller::run,
    core::{environment::EnvironmentExt, kubecontroller::State},
};
async fn run_kubecontroller() {
    let state = State::default();
    run(state.clone()).await;
}

struct ServeParams {
    binding: String,
    port: u16,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // 1. Environment setup ---------------------------------------------------
    let environment = match std::env::var("ENVIRONMENT") {
        Ok(env) => <Environment as EnvironmentExt>::from_str(&env).unwrap_or(Environment::Development),
        Err(_) => Environment::Development,
    };
    println!("🚀 Starting in {} environment", environment);

    // 2. Config loading ------------------------------------------------------
    let config = App::load_config(&environment)
        .await
        .expect("Failed to load config");

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
        port: config.server.port as u16,
    };

    // 6. Concurrent tasks setup ----------------------------------------------
    let (tx, mut rx) = tokio::sync::oneshot::channel();

    // Server task
    let server_task = tokio::spawn({
        let ctx = boot.context().clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
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
        let rx_close = rx.closed();
        loop {
            tokio::select! {
                _ = rx_close => {
                    tracing::info!("Background task received shutdown signal");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
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
    App::on_shutdown(&boot.context()).await;

    Ok(())
}
