pub mod crawl_target;

use std::{collections::{HashMap, HashSet}, fs, path::PathBuf, sync::Arc, sync::Mutex, cell::RefCell};
use core::fmt;

use reqwest::Client;
use scraper::{Html, Selector};

use crawl_target::{CrawlTarget};
use tokio::{sync::mpsc, time::error::Elapsed};

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
        // Start crawling the initial targets
        for (target, status) in &mut self.crawl_targets {
            status.replace(CrawlStatus::InProgress); // Why though, Rust, why?

            tokio::spawn(Self::crawl_target(self.client.clone(), target.clone()));
        }

        let (tx, mut rx) = mpsc::channel::<CrawlTarget>(32);

        while let Some(new_potential_target) = rx.recv().await {
            if !self.crawl_targets.contains_key(&new_potential_target) {
                
                self.crawl_targets.insert(new_potential_target.clone(), RefCell::new(CrawlStatus::InProgress));

                tokio::spawn(Self::crawl_target(self.client.clone(), new_potential_target));
            }
        }
    }

    pub async fn crawl_target(client: reqwest::Client, crawl_target: CrawlTarget) {
        println!("Crawling target... {}", crawl_target.host());
    }

    async fn crawl_url(client: reqwest::Client, url: reqwest::Url, crawl_target: Arc<Mutex<CrawlTarget>>) -> ()
    {
        let new_links_to_crawl: HashSet<reqwest::Url> = HashSet::new();

        // Send get request
        if let Ok(response) = client.get(url).send().await
        {
            if let Ok(response_text) = response.text().await 
            {
                // Check content for links
                let document = Html::parse_document(&response_text);    
                let selector = Selector::parse("a").unwrap();

                let mut potential_new_links: HashSet<reqwest::Url> = HashSet::new();

                // Parse links from the webpage
                for element in document.select(&selector) {
                    // Try to get the href attribute
                    if let Some(href) = element.value().attr("href") {
                        if let Ok(link) = reqwest::Url::parse(href)
                        {
                            
                        }                    
                    }
                }

            }

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