use std::{fmt::{self, Display}};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DomainName {
    tld: String,
    domain: String,
    subdomains: Option<Vec<String>>,
}

impl DomainName {
    pub fn parse(domain_name: &str) -> Result<Self, DomainNameParseError> {
        
        let domain_levels: Vec<&str> = domain_name.trim_end_matches('.').split('.').collect();
        
        let domain_levels_len = domain_levels.len();

        if domain_levels_len < 2 {
            return Err(DomainNameParseError);
        }
        else if domain_levels_len == 2 {
            return Ok(Self {
                tld: domain_levels.last().unwrap().to_owned().to_string(),
                domain: domain_levels.first().unwrap().to_owned().to_string(),
                subdomains: None
            });
        }
        else {
            return Ok(Self {
                tld: domain_levels.last().unwrap().to_owned().to_string(),
                domain: domain_levels[domain_levels_len - 2].to_string(),
                subdomains: Some(domain_levels[0..domain_levels.len() - 2].iter().map(|v| v.to_string()).collect())
            });
        }
    }

    /// Returns whether the domain name is a subdomain of another domain name.
    pub fn is_subdomain_of(&self, other: &DomainName) -> bool {
        if self.domain.ne(other.domain()) || self.tld.ne(other.tld()) { return false; }

        match (&self.subdomains, &other.subdomains) {
            (Some(self_subdomains), Some(other_subdomains)) => {
                if self_subdomains.len() <= other_subdomains.len() { return false; }
                
                let mut i = 0;
                for (self_subdomain, other_subdomain) in self_subdomains.iter().rev().zip(other_subdomains.iter().rev())
                {
                    if self_subdomain.ne(other_subdomain) {
                        return false;
                    }

                    i += 1;

                    if i == other_subdomains.len() {
                        break;
                    }
                }
                true
            },
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => false
        }
    }

    /// Returns whether the domain name is superdomain of another domain name
    pub fn is_superdomain_of(&self, other: &DomainName) -> bool {
        other.is_subdomain_of(&self)
    }

    /// Returns the top-level domain of the domain name.
    pub fn tld(&self) -> &str {
        &self.tld
    }

    /// Returns the domain of the domain name.
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Returns the subdomains of the domain, if such exist.
    pub fn subdomains(&self) -> &Option<Vec<String>> {
        &self.subdomains
    }

}

impl Display for DomainName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(subdomains) = &self.subdomains {
            write!(f, "{}.{}.{}", subdomains.join("."), self.domain, self.tld)
        }
        else {
            write!(f, "{}.{}", self.domain, self.tld)
        }
        
    }
}

#[derive(Debug)]
pub struct DomainNameParseError;

impl std::error::Error for DomainNameParseError {}

impl fmt::Display for DomainNameParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to parse domain name!")
    }
}
