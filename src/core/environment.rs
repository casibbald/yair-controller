use loco_rs::environment::Environment;

pub trait EnvironmentExt {
    fn log_level(&self) -> tracing::Level;

    /// Converts a string to an `Environment`.
    ///
    /// # Errors
    ///
    /// Returns an error if the string does not match any known environment.
    fn from_str(env: &str) -> loco_rs::Result<Self, String>
    where
        Self: Sized;
}

impl EnvironmentExt for Environment {
    fn log_level(&self) -> tracing::Level {
        match self {
            Self::Development => tracing::Level::DEBUG,
            Self::Test => tracing::Level::INFO,
            Self::Production => tracing::Level::WARN,
            Self::Any(_) => tracing::Level::ERROR,
        }
    }

    fn from_str(env: &str) -> loco_rs::Result<Self, String> {
        match env {
            "Development" => Ok(Self::Development),
            "Production" => Ok(Self::Production),
            _ => Err(format!("Unknown environment: {env}")),
        }
    }
}
