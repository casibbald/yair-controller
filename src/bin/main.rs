use loco_rs::cli;
use yapp::app::App;
use yapp::controllers::lib::kubecontroller;
use yapp::controllers::lib::telemetry;



#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    cli::main::<App>().await
}