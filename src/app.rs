use std::path::Path;
use async_trait::async_trait;
use loco_rs::{
    Result,
    app::{AppContext, Hooks},
    bgworker::Queue,
    boot::{BootResult, StartMode, create_app},
    config::Config,
    controller::AppRoutes,
    environment::Environment,
    task::Tasks,
};
use jsonwebtoken::errors;
use loco_rs::environment::{resolve_from_env, DEFAULT_ENVIRONMENT};
use loco_rs::prelude::DatabaseConnection;
use crate::controllers;
#[allow(unused_imports)] use crate::tasks;

pub struct App;

impl App {
    #[allow(clippy::redundant_closure)]
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::missing_errors_doc)]
    pub async fn load_config(environment: &Environment) -> Result<Config> {
        let config_path = format!("{environment}");
        println!("Loading configuration from path: {config_path}");
        let config_contents = tokio::fs::read_to_string(config_path).await?;
        let config = serde_yaml::from_str(&config_contents)
            .map_err(|e| {
                eprintln!("Failed to parse config: {e}");
                loco_rs::Error::from(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            })?;
        println!("Loaded config: {}", serde_json::to_string_pretty(&config).unwrap());
        Ok(config)
    }
}


#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(mode: StartMode, environment: &str) -> Result<BootResult> {
        create_app::<Self, M>(mode, environment).await
    }


    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::empty() // controller routes below
            .add_route(controllers::metrics::routes())
            .add_route(controllers::health::routes())
            .add_route(controllers::home::routes())
    }

    async fn connect_workers(_ctx: &AppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn register_tasks(tasks: &mut Tasks) {
        // tasks.register(TASK);
    }

    async fn truncate(db: &DatabaseConnection) -> Result<()> {
        todo!()
    }

    async fn seed(db: &DatabaseConnection, path: &Path) -> Result<()> {
        todo!()
    }
}

