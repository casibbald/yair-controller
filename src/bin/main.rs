use loco_rs::cli;
use yapp::{
    app::App,
    controllers::{kubecontroller::run, lib::kubecontroller::State, telemetry},
};


#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    cli::main::<App>().await.expect("TODO: panic message");

    telemetry::init().await;

    // Initialize Kubernetes controller state
    let state = State::default();
    run(state.clone()).await; // Ensure `run` is awaited
    Ok(())
}
