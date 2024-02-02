mod cli;
mod crawler;
mod target;

use clap::Parser;
use cli::*;
use crawler::*;
use target::*;

use std::{ fs::File, io::{BufRead, BufReader}};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let targets_file = match File::open(&cli.targets) {
        Ok(file) => file,
        Err(error) => {eprintln!("Failed to open the file with the target URLs: {}", error.to_string()); return }
    };

    let targets_reader = BufReader::new(targets_file);
    let mut targets: Vec<CrawlTarget> = Vec::new();

    // Process target URLs from file
    for line in targets_reader.lines() {
        match line {
            Ok(line) => if let Ok(url) = reqwest::Url::parse(&line) {
                if CrawlTarget::is_url_scheme_supported(&url)
                {
                    targets.push(CrawlTarget::new(url));
                }
                else {
                    eprintln!("Scheme {} not supported.", url.scheme());
                }
                 
            }else {
                eprintln!("Failed to parse target URL: {}", line);
            },
            Err(error) => eprintln!("Failed to read targets from file: {}", error.to_string())
        };
    }

    let mut client_config = reqwest::Client::builder();

    client_config =
        client_config.user_agent(concat!(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")));

    if let Ok(client) = client_config.build() {
        
    } else {
        eprintln!("Failed to initialise the HTTP(S) client.")
    }
}
