use std::net::{Ipv4Addr, Ipv6Addr};
use std::fmt;

use crate::dns::domain_name::{DomainName, DomainNameParseError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Host {
    Domain(DomainName),
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr)
}

impl Host {
    pub fn host_relationship(host1: &Self, host2: &Self) -> HostRelationship {
        match (host1, host2) {
            (Host::Domain(domain_name1), Host::Domain(domain_name2)) => {
                if domain_name1.eq(domain_name2) {
                    HostRelationship::Same
                } else {
                    if domain_name1.domain().eq(domain_name2.domain()) && domain_name1.tld().eq(domain_name2.tld()) {
                        HostRelationship::Related
                    } else {
                        HostRelationship::Unrelated
                    }
                }
            },

            // Both hosts are IPv4 addresses
            (Host::Ipv4(ip1), Host::Ipv4(ip2)) => {
                if ip1.eq(ip2) {
                    HostRelationship::Same
                } else {
                    HostRelationship::Unrelated
                }
            },

            // Both hosts are IPv6 address
            (Host::Ipv6(ip1), Host::Ipv6(ip2)) => {
                if ip1.eq(ip2) {
                    HostRelationship::Same
                } else {
                    HostRelationship::Unrelated
                }
            },

            // TODO: implement domain name resolution for the cases where one host is a domain and the other is an IP
            _ => HostRelationship::Unrelated,
        }
    }
}

impl TryFrom<url::Host<&str>> for Host {
    type Error = DomainNameParseError;

    fn try_from(value: url::Host<&str>) -> Result<Self, DomainNameParseError> {
        match value {
            url::Host::Domain(domain_name) => {
                match DomainName::parse(domain_name)
                {
                    Ok(domain_name) => { Ok(Host::Domain(domain_name)) },
                    Err(error) => Err(error)
                }
            },
            url::Host::Ipv4(ip) => Ok(Host::Ipv4(ip)),
            url::Host::Ipv6(ip) => Ok(Host::Ipv6(ip))
        }
    }
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Domain(domain) => write!(f, "{}", domain),
            Self::Ipv4(ip) => write!(f, "{}", ip),
            Self::Ipv6(ip) => write!(f, "{}", ip)
        }
        
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostRelationship {
    Same,      // The hosts are the same host
    Related,   // The hosts are related
    Unrelated, // The hosts are unrelated
}