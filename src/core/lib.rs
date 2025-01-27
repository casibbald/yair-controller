#![allow(unused_imports, unused_variables)]
use kube::runtime::finalizer::Error;
use loco_rs::Error as LocoError;
use serde_json;

pub type Result<T> = std::result::Result<T, LocoError>;

pub struct ErrorWrapper;

impl ErrorWrapper {
    #[must_use]
    pub fn from_serde(err: serde_json::Error) -> LocoError {
        LocoError::wrap(err)
    }

    pub fn from_kube<T>(err: T) -> LocoError
    where
        T: std::error::Error + Send + Sync + 'static,
    {
        LocoError::wrap(err)
    }

    #[must_use]
    pub fn from_custom(err: &str) -> LocoError {
        LocoError::wrap(std::io::Error::new(std::io::ErrorKind::Other, err))
    }
}

pub trait LocoErrorExt {
    fn metric_label(&self) -> String;
}

impl LocoErrorExt for LocoError {
    fn metric_label(&self) -> String {
        format!("{self:?}").to_lowercase()
    }
}
