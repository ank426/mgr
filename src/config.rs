use anyhow::ensure;
use clap::Args;
use serde::Serialize;

#[derive(Args, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(skip)]
    #[arg(short, long, default_value_t = 7169)]
    pub port: u16,

    #[serde(skip)]
    #[arg(short, long)]
    pub open: bool,

    #[serde(skip)]
    #[arg(long, default_value = ".mgr.toml")]
    pub readlist_file: String,

    #[arg(short, long, default_value_t = 100.0)]
    pub zoom: f32,

    #[arg(long, default_value_t = 10.0)]
    pub zoom_min: f32,

    #[arg(long, default_value_t = 500.0)]
    pub zoom_max: f32,

    #[arg(long, default_value_t = 500)]
    pub cursor_timeout: u32,

    #[arg(long, default_value_t = 200)]
    pub save_debounce: u32,

    #[arg(long, default_value_t = 6.0)]
    pub prefetch_back: f32,

    #[arg(long, default_value_t = 8.0)]
    pub prefetch_forward: f32,

    #[arg(long, default_value_t = 1)]
    pub volume_expand_back: u8,

    #[arg(long, default_value_t = 1)]
    pub volume_expand_forward: u8,
}

impl Config {
    pub fn validate(&self) -> anyhow::Result<()> {
        ensure!(self.zoom.is_finite() && self.zoom > 0.0, "zoom must be finite and positive");
        ensure!(self.zoom_min.is_finite() && self.zoom_min > 0.0, "zoom-min must be finite and positive");
        ensure!(self.zoom_max.is_finite() && self.zoom_max > 0.0, "zoom-max must be finite and positive");
        ensure!(self.zoom_min <= self.zoom_max, "zoom-min must be <= zoom-max");
        ensure!((self.zoom_min..=self.zoom_max).contains(&self.zoom), "zoom must be between zoom-min and zoom-max");
        ensure!(
            self.prefetch_back.is_finite() && self.prefetch_back >= 0.0,
            "prefetch-back must be finite and non-negative"
        );
        ensure!(
            self.prefetch_forward.is_finite() && self.prefetch_forward >= 0.0,
            "prefetch-forward must be finite and non-negative"
        );
        Ok(())
    }
}
