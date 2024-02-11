mod cli;
mod crawler;
mod web;
mod dns;
mod util;

use clap::Parser;
use cli::*;
use crawler::*;
use crawl_target::*;
use dns::domain_name::DomainName;
use web::host::Host;

use std::{ fs::File, io::{BufRead, BufReader}, collections::HashSet};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let targets_file = match File::open(&cli.targets) {
        Ok(file) => file,
        Err(error) => {eprintln!("Failed to open the file with the target URLs: {}", error.to_string()); return }
    };

    let targets_reader = BufReader::new(targets_file);
    let mut initial_targets: HashSet<CrawlTarget> = HashSet::new();

    console_subscriber::init();

    // Process target hosts from file
    for line in targets_reader.lines() {
        match line {
            Ok(line) => if let Ok(domain_name) = DomainName::parse(&line) {
                initial_targets.insert(CrawlTarget::new(Host::Domain(domain_name)));
            }else {
                eprintln!("Failed to parse target URL: {}", line);
            },
            Err(error) => eprintln!("Failed to read targets from file: {}", error.to_string())
        };
    }

    match Vdovitsa::new(initial_targets)
    {
        Ok(mut crawler) => {
            crawler.crawl().await;
        },
        Err(error) => { eprintln!("{}", error.to_string()); }
    }
    

}
