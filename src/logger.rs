

pub struct Logger {
    pub  output: Vec<String>,
}

impl Logger {
    pub fn new() -> Self {
        Logger { output: vec![] }
    }
    pub fn log(&mut self, text: String) {
        self.output.push(text);
    }
    pub fn dump(&self) -> String {
        self.output.join("\n")
    }
}

