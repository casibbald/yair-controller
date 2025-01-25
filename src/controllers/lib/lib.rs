#![allow(unused_imports, unused_variables)]
use loco_rs::{Error as LocoError};
use kube::runtime::finalizer::Error;
use serde_json;

/// Unified Result type using loco_rs::Error
pub type Result<T> = std::result::Result<T, LocoError>;

/// Utility for creating and converting errors
pub struct ErrorWrapper;

impl ErrorWrapper {
    pub fn from_serde(err: serde_json::Error) -> LocoError {
        LocoError::wrap(err)
    }

    pub fn from_kube<T>(err: T) -> LocoError
    where
        T: std::error::Error + Send + Sync + 'static,
    {
        LocoError::wrap(err)
    }

    pub fn from_custom(err: &str) -> LocoError {
        LocoError::wrap(std::io::Error::new(std::io::ErrorKind::Other, err))
    }
}

/// Extend loco_rs::Error with utility methods
pub trait LocoErrorExt {
    fn metric_label(&self) -> String;
}

impl LocoErrorExt for LocoError {
    fn metric_label(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}
