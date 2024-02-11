pub mod crawl_target;

use core::fmt;
use std::collections::HashSet;

use futures::FutureExt;
use reqwest::{header, Client, Url};
use scraper::{Html, Selector};

use crawl_target::CrawlTarget;
use tokio::{sync::mpsc, task::JoinSet};

use crate::web::{
    host::{Host, HostRelationship},
    http,
};

pub struct Vdovitsa {
    crawl_targets: HashSet<CrawlTarget>,
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

        if let Ok(client) = client_config.build() {
            Ok(Vdovitsa {
                crawl_targets: initial_targets,
                client,
            })
        } else {
            Err(CrawlerError::with_message(
                "Failed to initialse web client.",
            ))
        }
    }

    pub async fn crawl(&mut self) {
        let (tx, mut new_targets) = mpsc::channel::<CrawlTarget>(64);
        let weak_tx = tx.downgrade();

        let mut crawl_target_tasks: JoinSet<()> = JoinSet::new();

        // Start crawling the initial targets
        for target in &self.crawl_targets {
            crawl_target_tasks.spawn(Self::crawl_target(
                self.client.clone(),
                target.clone(),
                weak_tx.upgrade().unwrap(),
            ));
        }

        drop(tx);

        // Process new potential targets
        while let Some(new_potential_target) = new_targets.recv().await {
            while let Some(Some(_)) = crawl_target_tasks.join_next().now_or_never() {} // Remove finished tasks from crawl_target_tasks
            if crawl_target_tasks.is_empty() {
                new_targets.close();
            }

            if !self.crawl_targets.contains(&new_potential_target) {
                self.crawl_targets.insert(new_potential_target.clone());

                crawl_target_tasks.spawn(Self::crawl_target(
                    self.client.clone(),
                    new_potential_target,
                    weak_tx.upgrade().unwrap(),
                ));
            }
        }

        println!("Crawling done");
    }

    async fn crawl_target(
        client: Client,
        crawl_target: CrawlTarget,
        new_targets: mpsc::Sender<CrawlTarget>,
    ) {
        let crawl_target_host = crawl_target.host().to_owned();
        println!("Crawling target... {}", crawl_target_host);

        let mut crawled_urls: HashSet<String> = HashSet::new();
        crawled_urls.insert(format!("{}", crawl_target.host()).clone());

        let (tx, mut new_links) = mpsc::channel(64);
        let mut crawl_url_tasks: JoinSet<()> = JoinSet::new();

        // Crawl the target host's main page
        crawl_url_tasks.spawn(Self::crawl_url(
            client.clone(),
            Url::parse(&format!("https://{}", crawl_target_host)).unwrap(),
            tx.clone(),
        ));

        while let Some(new_potential_link) = new_links.recv().await {
            // This is sus
            while let Some(Some(_)) = crawl_url_tasks.join_next().now_or_never() {} // Remove finished tasks from crawl_url_tasks
            if crawl_url_tasks.is_empty() {
                new_links.close();
            }

            for link in new_potential_link {
                // If the URL is relative
                if link.starts_with('/') {
                    let absolute_link =
                        format!("https://{}{}", crawl_target.host().to_string(), link);

                    if crawled_urls.insert(absolute_link.clone()) {
                        crawl_url_tasks.spawn(Self::crawl_url(
                            client.clone(),
                            Url::parse(&absolute_link).unwrap(),
                            tx.clone(),
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
                                    crawl_url_tasks.spawn(Self::crawl_url(
                                        client.clone(),
                                        parsed_url.clone(),
                                        tx.clone(),
                                    ));
                                }
                            },
                            // A new target to crawl
                            HostRelationship::Related => {
                                new_targets.send(CrawlTarget::new(parsed_url_host)).await.unwrap();
                            },
                            HostRelationship::Unrelated => { continue; }
                        }
                    }
                }
            }
        }

        println!("Finished crawling target: {}", crawl_target_host);
    }

    async fn crawl_url(client: Client, url: Url, new_links: mpsc::Sender<HashSet<String>>) {
        // Check if the URL returns an HTML page
        let Ok(response_headers) = http::get_url_response_headers(&client, url.clone()).await else { return; };
        let Some(content_type) = response_headers.get(header::CONTENT_TYPE) else { return; };
        let Ok(content_type) = content_type.to_str() else { return; };
        if !content_type.starts_with("text/html") {
            return;
        }

        // Send get request
        let mut new_links_to_crawl: HashSet<String> = HashSet::new();

        if let Ok(response) = http::get_url(&client, url).await {
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
            }
        }

        // Send the new links to the parent crawl_target
        if !new_links_to_crawl.is_empty() {
            new_links.send(new_links_to_crawl).await.unwrap();
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
