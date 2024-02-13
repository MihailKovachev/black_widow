mod cli;
mod crawler;
mod web;
mod dns;
mod util;

use cli::{args::Args, Cli};
use crawler::{crawler_config::CrawlerConfig, *};
use crawl_target::*;
use crossterm::Command;
use dns::domain_name::DomainName;
use web::host::Host;

use std::{ fs::File, io::{BufRead, BufReader}, collections::HashSet};
use clap::Parser;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let targets_file = match File::open(&args.targets) {
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

    let crawler_config = CrawlerConfig {
        initial_targets,
        crawl_subdomains: args.crawl_subdomains
    };

    match Crawler::new(crawler_config)
    {
        Ok(mut crawler) => {
            let crawler = tokio::spawn(async move { crawler.crawl().await; });
        

            crawler.await.unwrap();
        },
        Err(error) => { eprintln!("{}", error.to_string()); }
    }
    

}
