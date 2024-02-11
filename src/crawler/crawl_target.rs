use std::hash::Hash;

use crate::web::host::Host;

/// A crawl target
#[derive(Debug, Eq, Clone, Hash)]
pub struct CrawlTarget {
    host: Host // The target host
}

impl CrawlTarget {
    pub fn new(host: Host) -> CrawlTarget {

        match host
        {
            Host::Domain(host) => {
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
    pub fn host(&self) -> &Host {
        &self.host
    }
}

impl PartialEq for CrawlTarget {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host
    }
}
