pub mod crawl_target;

use std::{collections::{HashMap, HashSet}, fs, path::PathBuf, sync::Arc, sync::Mutex, cell::RefCell};
use core::fmt;

use reqwest::Client;
use scraper::{Html, Selector};

use crawl_target::{CrawlTarget};
use tokio::{sync::mpsc, time::error::Elapsed};
use url::Url;

pub struct Vdovitsa {
    crawl_targets: HashMap<CrawlTarget, RefCell<CrawlStatus>>,
    client: Client
}

impl Vdovitsa {

    /// Create a Vdovitsa crawler with initial targets.
    pub fn new(initial_targets: HashSet<CrawlTarget>) -> Result<Vdovitsa, CrawlerError> {
        
        // Configure the web client
        let mut client_config = Client::builder()
        .user_agent(concat!( env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")));

        // Set up the crawl targets
        let mut crawl_targets = HashMap::with_capacity(initial_targets.len());
        for target in initial_targets {
            crawl_targets.insert(target, RefCell::new(CrawlStatus::Pending));
        }

        if let Ok(client) = client_config.build() {

            Ok(Vdovitsa {
                crawl_targets: crawl_targets,
                client
            })
        }
        else {
            Err(CrawlerError::with_message("Failed to initialse web client."))
        }
    }

    pub async fn crawl(&mut self) {
        let (tx, mut rx) = mpsc::channel::<CrawlTarget>(32);

        // Start crawling the initial targets
        for (target, status) in &mut self.crawl_targets {
            status.replace(CrawlStatus::InProgress); // Why though, Rust, why?

            let client = self.client.clone();
            let target = target.clone();
            let tx = tx.clone();
            
            tokio::spawn(Self::crawl_target(client, target, tx));
        }

        while let Some(new_potential_target) = rx.recv().await {
            if !self.crawl_targets.contains_key(&new_potential_target) {
                
                self.crawl_targets.insert(new_potential_target.clone(), RefCell::new(CrawlStatus::InProgress));
                println!("Crawling new target: {}", new_potential_target.host());

                let client = self.client.clone();
                let tx = tx.clone();
                
                tokio::spawn(Self::crawl_target(client, new_potential_target, tx));
            }
        }

        println!("Crawling done");
    }

    pub async fn crawl_target(client: Client, crawl_target: CrawlTarget, new_targets: mpsc::Sender<CrawlTarget>) {
        println!("Crawling target... {}", crawl_target.host());
        let crawl_target_url = format!("https://{}", crawl_target.host());

        let mut crawled_urls: HashSet<String> = HashSet::new();
        crawled_urls.insert(crawl_target_url.clone());
        
        let (tx, mut new_links) = mpsc::channel(32);

        tokio::spawn(Self::crawl_url(client.clone(), Url::parse(&crawl_target_url).unwrap(), tx.clone()));
        
        while let Some(new_potential_link) = new_links.recv().await {
            for link in new_potential_link {
                if let Ok(parsed_url) = Url::parse(&link)
                {
                    if let Some(parsed_host) = parsed_url.domain() {

                        if parsed_host.eq(crawl_target.host()) {
                            
                            crawled_urls.insert(parsed_host.to_string());
                            println!("Crawling URL {} for target {}", parsed_url.to_string(), crawl_target.host());

                            tokio::spawn(Self::crawl_url(client.clone(), parsed_url, tx.clone()));
                        
                        }else if Self::are_hosts_related(crawl_target.host(), parsed_host){
                            // New crawl target has been found
                            if let Ok(new_target) = CrawlTarget::new(url::Host::Domain(parsed_host)) {
                                new_targets.send(new_target).await;
                            }
                        }
                    }
                }
            }
        }
    }

    async fn crawl_url(client: Client, url: Url, new_links: mpsc::Sender<HashSet<String>>) -> ()
    {
        let mut new_links_to_crawl: HashSet<String> = HashSet::new();

        // Send get request
        if let Ok(response) = client.get(url).send().await
        {
            if let Ok(response_text) = response.text().await 
            {
                // Check content for links
                let document = Html::parse_document(&response_text);    
                let selector = Selector::parse("a").unwrap();

                // Parse links from the webpage
                for element in document.select(&selector) {
                    // Try to get the href attribute
                    if let Some(href) = element.value().attr("href") {
                        new_links_to_crawl.insert(href.to_string());
                    }
                }
            }
        }

        new_links.send(new_links_to_crawl).await.unwrap();

    }

    /// Returns whether two hosts are related.
    fn are_hosts_related(host1: &str, host2: &str) -> bool {
        if host1.eq(host2) { return true; }
        else {
            let host1_parts: Vec<&str> = host1.split('.').rev().take(2).collect();
            let host2_parts: Vec<&str> = host2.split('.').rev().take(2).collect();

            return host1_parts.eq(&host2_parts);
        }

    }
}

#[derive(Debug, Clone)]
pub enum CrawlStatus {
    Pending,
    InProgress,
    Finished
}

#[derive(Debug)]
pub struct CrawlerError {
    message: String
}

impl CrawlerError {
    fn new() -> CrawlerError {
        CrawlerError { message: String::from("Crawl Target Error") }
    }

    fn with_message(message: &str) -> CrawlerError {
        CrawlerError { message: String::from(message) }
    }
}

impl std::error::Error for CrawlerError {}

impl fmt::Display for CrawlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}