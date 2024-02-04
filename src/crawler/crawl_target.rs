use core::fmt;
use std::{hash::Hash, collections::HashSet};

use url::Host;

/// A crawl target - either a domain or a subdomain.
#[derive(Debug, Eq, Clone)]
pub struct CrawlTarget {

    host: String, // The target host
    host_depth: HostDepth
    
}

impl CrawlTarget {
    pub fn new(host: Host<&str>) -> Result<CrawlTarget, CrawlTargetError> {

        match host
        {
            Host::Domain(host) => {
                let host = host.trim_end_matches('.').to_owned(); // Remove potential dot characters at the end of the host name
                let host_depth = if host.matches('.').count() <= 1 { HostDepth::Domain } else { HostDepth::Subdomain } ;

                return Ok(CrawlTarget {
                    host, 
                    host_depth}
                )},

            _ => Err(CrawlTargetError::with_message("Crawl target cannot be an IP address."))
        }

    }

    /// Returns whether the crawl target is a domain or a subdomain.
    pub fn host_depth(&self) -> HostDepth {
        if self.host.matches('.').count() <= 1 {
            return HostDepth::Domain;
        }
        else {
            return HostDepth::Subdomain;
        }
    }

    /// Returns the host of the crawl target
    pub fn host(&self) -> &str {
        &self.host
    }
}

impl PartialEq for CrawlTarget {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host
    }
}

impl Hash for CrawlTarget {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.host.hash(state); // A crawl target is unique only if its host is
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum HostDepth {
    Domain,
    Subdomain
}

#[derive(Debug)]
pub struct CrawlTargetError {
    message: String
}

impl CrawlTargetError {
    pub fn new() -> CrawlTargetError {
        CrawlTargetError { message: String::from("Crawl Target Error") }
    }

    pub fn with_message(message: &str) -> CrawlTargetError {
        CrawlTargetError { message: String::from(message) }
    }
}

impl std::error::Error for CrawlTargetError {}

impl fmt::Display for CrawlTargetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}