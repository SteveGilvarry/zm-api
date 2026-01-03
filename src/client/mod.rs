#![allow(clippy::result_large_err)]
use crate::{configure::AppConfig, error::AppResult};

pub mod database;
pub mod email;
pub mod http;
pub mod webrtc_signaling;

pub trait ClientBuilder: Sized {
    fn build_from_config(config: &AppConfig) -> AppResult<Self>;
}
