use std::io::stdout;

pub mod args;

#[derive(Debug)]
pub struct Cli {
    stdout: std::io::Stdout
}

impl Cli {
    pub fn init() -> Self {
        Cli {
            stdout: stdout()
        }
    }

    pub fn clear(&self) {
        
    }
}
