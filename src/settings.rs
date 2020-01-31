use config::{ConfigError, Config, File};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub ignore_paths: Vec<String>,
    pub working_dir: String,
    pub delete_score: Vec<String>,
    pub action: String
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("config"))?;
        s.try_into()
    }
}