use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Config {
    pub prefetch: (f32, f32),
}
