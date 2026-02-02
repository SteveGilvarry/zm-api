#![allow(clippy::result_large_err)]
use crate::{configure::AppConfig, error::AppResult};

pub mod database;
pub mod http;

pub trait ClientBuilder: Sized {
    fn build_from_config(config: &AppConfig) -> AppResult<Self>;
}
