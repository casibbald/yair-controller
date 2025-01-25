use loco_rs::cli;
use yapp::app::App;

pub mod crdgen;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    cli::main::<App>().await
}
