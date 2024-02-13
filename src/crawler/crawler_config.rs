use std::collections::HashSet;

use crate::{cli::args::Args, CrawlTarget};


#[derive(Debug)]
pub struct CrawlerConfig {
    pub initial_targets: HashSet<CrawlTarget>,
    pub crawl_subdomains: bool
}