pub mod crawl_target;
pub mod crawler_config;

use core::fmt;
use std::fs::{self, File};
use std::sync::{Arc, Mutex};
use std::{collections::HashSet};

use reqwest::{header, Client, Url};
use rusqlite::{params, Connection};
use scraper::{Html, Selector};
use tokio::sync::mpsc;

use crate::{
    util::ChannelPacket,
    web::{
        host::{Host, HostRelationship},
        http,
    },
};
use crawl_target::CrawlTarget;

use self::crawler_config::CrawlerConfig;

pub struct Crawler {
    crawl_targets: HashSet<CrawlTarget>,
    client: Client,
    config: Arc<CrawlerConfig>,
}

impl Crawler {
    /// Create a Vdovitsa crawler with initial targets.
    pub fn new(config: CrawlerConfig) -> Result<Crawler, CrawlerError> {
        // Configure the web client
        let client_config = Client::builder().user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ));

        if let Ok(client) = client_config.build() {
            Ok(Crawler {
                crawl_targets: config.initial_targets.clone(),
                client,
                config: Arc::new(config),
            })
        } else {
            Err(CrawlerError::with_message(
                "Failed to initialse web client.",
            ))
        }
    }

    pub async fn crawl(&mut self) {
        let (tx, mut new_targets) = mpsc::channel::<ChannelPacket<CrawlTarget>>(64);

        // Set up URLs table
        let Ok(db) = Connection::open(&self.config.db_path) else { eprintln!("Failed to open DB!"); return; };
    
        // Create initial targets table
        db.execute("CREATE TABLE IF NOT EXISTS urls (
            id INTEGER PRIMARY KEY,
            url TEXT NOT NULL,
            target TEXT NOT NULL,
            response_code INTEGER,
            response_body BLOB)
            ", ()).unwrap();

        db.close().unwrap();

        // Start crawling the initial targets
        for target in &self.crawl_targets {
            tokio::spawn(Self::crawl_target(
                self.client.clone(),
                target.clone(),
                tx.clone(),
                Arc::clone(&self.config),
            ));
        }

        drop(tx);

        // Process new potential targets
        while let Some(new_potential_target) = new_targets.recv().await {
            if self.crawl_targets.insert(new_potential_target.data.clone()) {
                tokio::spawn(Self::crawl_target(
                    self.client.clone(),
                    new_potential_target.data,
                    new_potential_target.sender,
                    Arc::clone(&self.config),
                ));
            }
        }

        println!("Crawling done");
    }

    async fn crawl_target(
        client: Client,
        crawl_target: CrawlTarget,
        new_targets: mpsc::Sender<ChannelPacket<CrawlTarget>>,
        config: Arc<CrawlerConfig>,
    ) {
        let crawl_target_host = crawl_target.host().to_owned();
        println!("Crawling target... {}", crawl_target_host);

        let mut crawled_urls: HashSet<String> = HashSet::new();
        crawled_urls.insert(format!("{}", crawl_target.host()).clone());

        let (tx, mut new_links) = mpsc::channel::<ChannelPacket<HashSet<String>>>(64);

        // Create DB table for the target
        let Ok(db) = Connection::open(&config.db_path) else { eprintln!("Failed to create database table for: {}", crawl_target_host); return;};
        let db = Arc::new(Mutex::new(db));


        // Crawl the target host's main page
        tokio::spawn(Self::crawl_url(
            client.clone(),
            Url::parse(&format!("https://{}/", crawl_target_host)).unwrap(),
            tx.clone(),
            Arc::clone(&db),
        ));

        drop(tx);

        while let Some(new_potential_links) = new_links.recv().await {
            for link in new_potential_links.data {
                // If the URL is relative
                if link.starts_with('/') && link.len() > 1 {
                    let absolute_link = format!("{}{}", crawl_target.host().to_string(), link);

                    if crawled_urls.insert(absolute_link.clone()) {
                        tokio::spawn(Self::crawl_url(
                            client.clone(),
                            Url::parse(&format!("https://{}", absolute_link)).unwrap(),
                            new_potential_links.sender.clone(),
                            Arc::clone(&db),
                        ));
                    }
                } else {
                    let Ok(parsed_url) = Url::parse(&link) else { continue; };
                    // Only HTTP and HTTPS are supported

                    if parsed_url.scheme().eq("https") || parsed_url.scheme().eq("http") {
                        let Some(parsed_url_host) = parsed_url.host() else { continue; };
                        let Ok(parsed_url_host) = Host::try_from(parsed_url_host) else { continue; };

                        match Host::host_relationship(crawl_target.host(), &parsed_url_host) {
                            // A new link to crawl
                            HostRelationship::Same => {
                                if crawled_urls.insert(parsed_url.to_string()) {
                                    tokio::spawn(Self::crawl_url(
                                        client.clone(),
                                        parsed_url.clone(),
                                        new_potential_links.sender.clone(),
                                        Arc::clone(&db),
                                    ));
                                }
                            }

                            // A new target to crawl
                            HostRelationship::Related => {
                                if config.crawl_subdomains {
                                    new_targets
                                        .send(ChannelPacket {
                                            sender: new_targets.clone(),
                                            data: CrawlTarget::new(parsed_url_host),
                                        })
                                        .await
                                        .unwrap();
                                }
                            }

                            HostRelationship::Unrelated => {
                                continue;
                            }
                        }
                    }
                }
            }
        }

        println!("Finished crawling target: {}", crawl_target_host);
    }

    async fn crawl_url(
        client: Client,
        url: Url,
        new_links: mpsc::Sender<ChannelPacket<HashSet<String>>>,
        db: Arc<Mutex<Connection>>
    ) {
        let mut new_links_to_crawl: HashSet<String> = HashSet::new();

        // Send get request
        let Ok(response) = http::get_url(&client, url.clone()).await else { return; };
        
        let status_code = response.status();
        if !status_code.is_success() { return; }

        // Check if the URL returns an HTML page
        let Some(content_type) = response.headers().get(header::CONTENT_TYPE) else { return; };
        let Ok(content_type) = content_type.to_str() else { return; };
        if !content_type.starts_with("text/html") {
            return;
        }

        if let Ok(response_text) = response.text().await {
            // Check content for links
            let document = Html::parse_document(&response_text);
            let selector = Selector::parse("a").unwrap();

            // Parse links from the webpage
            for element in document.select(&selector) {
                // Try to get the href attribute
                if let Some(href) = element.value().attr("href") {
                    new_links_to_crawl.insert(href.to_owned());
                }
            }

            match db.lock() {
                Ok(db) => {
                    if let Err(error) = db.execute(
                        "INSERT INTO urls (url, target, response_code, response_body) VALUES (?1, ?2, ?3, ?4)", 
                        params![url.to_string(), url.host_str().unwrap(), status_code.as_u16(), response_text]
                    ) {
                        eprintln!("Failed to update DB: {}", error);
                        return;
                    }
                }
                Err(error) => {
                    eprintln!("Failed to obtain mutex lock: {}", error);
                    return;
                }
            }
        }

        // Send the new links to the parent crawl_target
        if !new_links_to_crawl.is_empty() {
            new_links
                .send(ChannelPacket {
                    sender: new_links.clone(),
                    data: new_links_to_crawl,
                })
                .await
                .unwrap();
        }
    }
}

#[derive(Debug)]
pub struct CrawlerError {
    message: String,
}

impl CrawlerError {
    fn new() -> CrawlerError {
        CrawlerError {
            message: String::from("Crawl Target Error"),
        }
    }

    fn with_message(message: &str) -> CrawlerError {
        CrawlerError {
            message: String::from(message),
        }
    }
}

impl std::error::Error for CrawlerError {}

impl fmt::Display for CrawlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
