#[allow(unused_imports)]
use yair::{
    app::App,
    controllers::{kubecontroller::run, telemetry},
    core::kubecontroller::State,
};

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    let loco_rs_handle = tokio::spawn(async {
        if let Err(e) = run_loco_rs().await {
            eprintln!("Error in loco_rs: {e:?}");
        }
    });

    telemetry::init().await;
    let kubecontroller_handle = tokio::spawn(async {
        if let Err(e) = run_kubecontroller().await {
            eprintln!("Error in loco_rs: {e:?}");
        }
    });

    // Await all tasks to complete
    let _ = tokio::try_join!(kubecontroller_handle, loco_rs_handle)?;

    Ok(())
}

async fn run_kubecontroller() -> Result<(), Box<dyn std::error::Error>> {
    let state = State::default();
    run(state.clone()).await;
    Ok(())
}

async fn run_loco_rs() -> loco_rs::Result<()> {
    println!("Starting loco_rs...");

    let start_mode = loco_rs::boot::StartMode::ServerOnly;
    let environment = loco_rs::environment::Environment::Development;
    let boot_result = loco_rs::boot::create_app::<App>(start_mode, &environment).await?;

    let server_params = loco_rs::boot::ServeParams {
        port: 8080,
        binding: "localhost".to_string(),
    };
    loco_rs::boot::start::<App>(boot_result, server_params, true).await?;
    Ok(())
}
