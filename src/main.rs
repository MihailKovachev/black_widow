mod cli;
mod crawler;
mod dns;
mod util;
mod web;

use cli::{args::Args, Cli};
use crawl_target::*;
use crawler::{crawler_config::CrawlerConfig, *};
use dns::domain_name::DomainName;
use rusqlite::{params, Connection};
use web::host::Host;

use clap::Parser;
use std::{
    collections::HashSet, fs::{self, File}, io::{BufRead, BufReader}, path::PathBuf, process
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let targets_file = File::open(&args.targets)?;

    let targets_reader = BufReader::new(targets_file);
    let mut initial_targets: HashSet<CrawlTarget> = HashSet::new();

    console_subscriber::init();

    // Process target hosts from file
    for line in targets_reader.lines() {
        match line {
            Ok(line) => {
                if let Ok(domain_name) = DomainName::parse(&line) {
                    initial_targets.insert(CrawlTarget::new(Host::Domain(domain_name)));
                } else {
                    eprintln!("Failed to parse target URL: {}", line);
                }
            }
            Err(error) => eprintln!("Failed to read targets from file: {}", error.to_string()),
        };
    }

    let db_path = path_clean::clean(std::env::current_dir()?.join(&args.output_file));

    // Set up the output database
    let db = Connection::open(&args.output_file)?;
    
    // Create initial targets table
    db.execute("CREATE TABLE IF NOT EXISTS targets (
        id INTEGER PRIMARY KEY,
        host TEXT)", ())?;

    db.close().unwrap();

    let crawler_config = CrawlerConfig {
        initial_targets,
        crawl_subdomains: args.crawl_subdomains,
        db_path,
    };

    let mut crawler = Crawler::new(crawler_config)?;
    let crawler_task = tokio::spawn(async move {
        crawler.crawl().await;
    });

    crawler_task.await.unwrap();

    Ok(())
}
