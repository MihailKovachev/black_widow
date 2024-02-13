use std::{collections::HashSet, path::PathBuf};

use crate::CrawlTarget;


#[derive(Debug)]
pub struct CrawlerConfig {
    pub initial_targets: HashSet<CrawlTarget>,
    pub crawl_subdomains: bool,
}