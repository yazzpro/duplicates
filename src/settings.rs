use std::fs;


#[derive(Default,Debug,Serialize, Deserialize)]
pub struct Settings {
    pub ignore_paths: Vec<String>,
    pub working_dir: String,
    pub delete_score: Vec<String>,
    /// action for duplicates:
    /// D - Delete (all except one)
    /// T - Test delete (write which would be deleted but don't delete)
    /// S - Stop - display like for T but ends program execution
    /// All the rest - just write duplicates
    pub action: String,

    pub watchdog: bool,

    pub email_result_to: Option<String>,
    pub email_username: Option<String>,
    pub email_password: Option<String>,
    pub email_hostname: Option<String>
}
impl Settings {
    pub fn new() -> Result<Self, std::io::Error> {
       let s = fs::read_to_string("config.toml")?;
       
       let r = toml::from_str::<Settings>(s.as_str());
       if r.is_err() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Unable to read config"));
       }
       Ok(r.unwrap())
    }

}
