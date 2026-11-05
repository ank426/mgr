use std::env;
use std::path::PathBuf;

use anyhow::ensure;
use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(skip)]
    #[arg(short, long, default_value_t = 7169)]
    pub port: u16,

    #[serde(skip)]
    #[arg(short, long, num_args = 0..=1, default_missing_value = "true", default_value_t = false, hide_possible_values = true)]
    pub open: bool,

    #[serde(skip)]
    #[arg(long, default_value = ".mgr.toml")]
    pub readlist_file: String,

    #[arg(short, long, default_value_t = 100.0)]
    pub zoom: f32,

    #[arg(long, default_value_t = true, action = clap::ArgAction::Set, hide_possible_values = true)]
    pub dynamic_colors: bool,

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

    #[arg(skip)]
    pub keybindings: Keybindings,
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
        self.keybindings.validate()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct Keybindings {
    #[serde(alias = "zoom_in")]
    pub zoom_in: String,
    #[serde(alias = "zoom_out")]
    pub zoom_out: String,
    #[serde(alias = "scroll_down")]
    pub scroll_down: String,
    #[serde(alias = "scroll_up")]
    pub scroll_up: String,
    #[serde(alias = "previous_volume")]
    pub previous_volume: String,
    #[serde(alias = "next_volume")]
    pub next_volume: String,
    #[serde(alias = "first_page")]
    pub first_page: String,
    #[serde(alias = "last_page")]
    pub last_page: String,
    #[serde(alias = "invert_colors")]
    pub invert_colors: String,
    #[serde(alias = "page_progress")]
    pub page_progress: String,
    #[serde(alias = "scroll_progress")]
    pub scroll_progress: String,
    #[serde(alias = "volume_progress")]
    pub volume_progress: String,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            zoom_in: "=".into(),
            zoom_out: "-".into(),
            scroll_down: "j".into(),
            scroll_up: "k".into(),
            previous_volume: "h".into(),
            next_volume: "l".into(),
            first_page: "g".into(),
            last_page: "G".into(),
            invert_colors: "i".into(),
            page_progress: "s".into(),
            scroll_progress: "d".into(),
            volume_progress: "S".into(),
        }
    }
}

impl Keybindings {
    fn validate(&self) -> anyhow::Result<()> {
        let bindings = [
            ("zoom_in", &self.zoom_in),
            ("zoom_out", &self.zoom_out),
            ("scroll_down", &self.scroll_down),
            ("scroll_up", &self.scroll_up),
            ("previous_volume", &self.previous_volume),
            ("next_volume", &self.next_volume),
            ("first_page", &self.first_page),
            ("last_page", &self.last_page),
            ("invert_colors", &self.invert_colors),
            ("page_progress", &self.page_progress),
            ("scroll_progress", &self.scroll_progress),
            ("volume_progress", &self.volume_progress),
        ];
        let mut seen = std::collections::HashMap::new();
        for (action, key) in bindings {
            if key.is_empty() {
                continue;
            }
            ensure!(
                !seen.contains_key(key),
                "key `{key}` is assigned to both `{}` and `{action}`",
                seen.get(key).unwrap_or(&"")
            );
            seen.insert(key, action);
        }
        Ok(())
    }
}

pub fn file_args() -> anyhow::Result<(Vec<String>, Option<Keybindings>)> {
    let path = config_dir()?.join("config.toml");
    if !path.exists() {
        return Ok((vec![], None));
    }
    let mut table = std::fs::read_to_string(&path)?.parse::<toml::Table>()?;
    let keybindings = table.remove("keybindings").map(toml::Value::try_into).transpose()?;
    let args = table
        .into_iter()
        .flat_map(|(key, value)| [format!("--{}", key.replace('_', "-")), value.to_string()])
        .collect();
    Ok((args, keybindings))
}

pub fn config_dir() -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(env::var("XDG_CONFIG_HOME").or_else(|_| env::var("HOME").map(|h| h + "/.config"))?).join("mgr"))
}
