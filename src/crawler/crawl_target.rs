use std::hash::Hash;

use url::Host;

/// A crawl target
#[derive(Debug, Eq, Clone, Hash)]
pub struct CrawlTarget {
    host: Host<String>, // The target host
}

impl CrawlTarget {
    pub fn new(host: Host<&str>) -> CrawlTarget {

        match host
        {
            Host::Domain(host) => {
                let host = host.trim_end_matches('.').to_owned(); // Remove potential dot characters at the end of the host name
                
                CrawlTarget {
                    host: Host::Domain(host), 
                }
            },

            Host::Ipv4(ip) => CrawlTarget {
                host: Host::Ipv4(ip), 
            },

            Host::Ipv6(ip) => CrawlTarget {
                host: Host::Ipv6(ip), 
            }
        }

    }

    /// Returns the host of the crawl target
    pub fn host(&self) -> &Host<String> {
        &self.host
    }
}

impl PartialEq for CrawlTarget {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host
    }
}
