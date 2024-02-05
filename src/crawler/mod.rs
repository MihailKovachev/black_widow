pub mod crawl_target;

use core::fmt;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    net::Ipv4Addr,
};

use reqwest::{Client, Url};
use scraper::{Html, Selector};

use crawl_target::CrawlTarget;
use tokio::sync::mpsc;
use url::Host;

pub struct Vdovitsa {
    crawl_targets: HashMap<CrawlTarget, RefCell<CrawlStatus>>,
    client: Client,
}

impl Vdovitsa {
    /// Create a Vdovitsa crawler with initial targets.
    pub fn new(initial_targets: HashSet<CrawlTarget>) -> Result<Vdovitsa, CrawlerError> {
        // Configure the web client
        let client_config = Client::builder().user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ));

        // Set up the crawl targets
        let mut crawl_targets = HashMap::with_capacity(initial_targets.len());
        for target in initial_targets {
            crawl_targets.insert(target, RefCell::new(CrawlStatus::Pending));
        }

        if let Ok(client) = client_config.build() {
            Ok(Vdovitsa {
                crawl_targets: crawl_targets,
                client,
            })
        } else {
            Err(CrawlerError::with_message(
                "Failed to initialse web client.",
            ))
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

        // Process new potential targets
        while let Some(new_potential_target) = rx.recv().await {
            if !self.crawl_targets.contains_key(&new_potential_target) {
                self.crawl_targets.insert(
                    new_potential_target.clone(),
                    RefCell::new(CrawlStatus::InProgress),
                );
                println!("Adding new target: {}", new_potential_target.host());

                let client = self.client.clone();
                let tx = tx.clone();

                tokio::spawn(Self::crawl_target(client, new_potential_target, tx));
            }
        }

        println!("Crawling done");
    }

    pub async fn crawl_target(
        client: Client,
        crawl_target: CrawlTarget,
        new_targets: mpsc::Sender<CrawlTarget>,
    ) {
        println!("Crawling target... {}", crawl_target.host());

        let mut crawled_urls: HashSet<String> = HashSet::new();
        crawled_urls.insert(format!("{}", crawl_target.host()).clone());

        let (tx, mut new_links) = mpsc::channel(32);

        tokio::spawn(Self::crawl_url(
            client.clone(),
            Url::parse(&format!("https://{}", crawl_target.host())).unwrap(),
            tx.clone(),
        ));

        while let Some(new_potential_link) = new_links.recv().await {
            for link in new_potential_link {
                if let Ok(parsed_url) = Url::parse(&link) {
                    // Only HTTP and HTTPS are supported
                    if parsed_url.scheme().eq("https") || parsed_url.scheme().eq("http") {
                        match parsed_url.host() {
                            Some(parsed_url_host) => {
                                match Self::compare_hosts(
                                    &parsed_url_host.to_owned(),
                                    crawl_target.host(),
                                ) {
                                    HostRelation::Same => {
                                        // The link belongs to the current target
                                        let normalized_url: String = parsed_url
                                            .to_string()
                                            .trim_end_matches('/')
                                            .split_once("://")
                                            .unwrap()
                                            .1
                                            .to_string();
                                        if (!crawled_urls.contains(&normalized_url)) {
                                            println!(
                                                "Adding URL {} for target {}",
                                                parsed_url.to_string(),
                                                crawl_target.host().to_string()
                                            );
                                            crawled_urls.insert(normalized_url);
                                            tokio::spawn(Self::crawl_url(
                                                client.clone(),
                                                parsed_url,
                                                tx.clone(),
                                            ));
                                        }
                                    }
                                    HostRelation::Related => {
                                        // The link points to a new potential target
                                        new_targets
                                            .send(CrawlTarget::new(parsed_url_host.clone()))
                                            .await
                                            .unwrap();
                                    }

                                    HostRelation::Unrelated => (), // Skip links to unrelated hosts
                                }
                            }
                            None => {}
                        }
                    } else {
                        continue;
                    }
                }
            }
        }
    }

    async fn crawl_url(client: Client, url: Url, new_links: mpsc::Sender<HashSet<String>>) -> () {
        let mut new_links_to_crawl: HashSet<String> = HashSet::new();

        println!(
            "Crawling URL {}",
            url.to_string(),
        );

        // Send get request
        if let Ok(response) = client.get(url).send().await {
            if let Ok(response_text) = response.text().await {
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
    fn compare_hosts(host1: &Host<String>, host2: &Host<String>) -> HostRelation {
        match (host1, host2) {
            (Host::Domain(domain1), Host::Domain(domain2)) => {
                if domain1.eq(domain2) {
                    HostRelation::Same
                } else {
                    let host1_parts: Vec<&str> = domain1.split('.').rev().take(2).collect();
                    let host2_parts: Vec<&str> = domain2.split('.').rev().take(2).collect();

                    if host1_parts.eq(&host2_parts) {
                        HostRelation::Related
                    } else {
                        HostRelation::Unrelated
                    }
                }
            }

            // Both hosts are IPv4 addresses
            (Host::Ipv4(ip1), Host::Ipv4(ip2)) => {
                if ip1.eq(ip2) {
                    HostRelation::Same
                } else {
                    HostRelation::Unrelated
                }
            }

            // Both hosts are IPv6 address
            (Host::Ipv6(ip1), Host::Ipv6(ip2)) => {
                if ip1.eq(ip2) {
                    HostRelation::Same
                } else {
                    HostRelation::Unrelated
                }
            }

            // TODO: implement domain name resolution for the cases where one host is a domain and the other is an IP
            _ => HostRelation::Unrelated,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HostRelation {
    Same,      // The hosts are the same host
    Related,   // The hosts are related
    Unrelated, // The hosts are unrelated
}

#[derive(Debug, Clone)]
pub enum CrawlStatus {
    Pending,
    InProgress,
    Finished,
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
