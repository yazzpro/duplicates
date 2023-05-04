
#[derive(Debug,Serialize, Deserialize)]
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
impl ::std::default::Default for Settings {
    fn default() -> Self { Self {ignore_paths: Vec::new(), working_dir: ".".to_string(), delete_score: Vec::new(), action: "T".to_string(), watchdog: false, email_result_to: None, email_username: None, email_password: None, email_hostname: None} }
}
impl Settings {
    pub fn new() -> Result<Self, std::io::Error> {
        confy::load("config")
    }

}
