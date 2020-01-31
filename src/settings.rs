use config::{ConfigError, Config, File};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub ignore_paths: Vec<String>,
    pub working_dir: String,
    pub delete_score: Vec<String>,
    /// action for duplicates:
    /// D - Delete (all except one)
    /// T - Test delete (write which would be deleted but don't delete)
    /// S - Stop - display like for T but ends program execution
    /// All the rest - just write duplicates
    pub action: String
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("config"))?;
        s.try_into()
    }
}