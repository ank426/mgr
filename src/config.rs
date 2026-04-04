use clap::Args;
use serde::Serialize;

#[derive(Args, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[arg(long, default_value_t = 6.0)]
    pub prefetch_back: f32,

    #[arg(long, default_value_t = 8.0)]
    pub prefetch_forward: f32,

    #[arg(long, default_value_t = 500)]
    pub cursor_timeout: u32,

    #[arg(long, default_value_t = 200)]
    pub save_debounce: u32,

    #[arg(long, default_value_t = 100)]
    pub zoom: u32,
}
