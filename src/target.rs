pub struct CrawlTarget {
    pub url: reqwest::Url,
    top_level_domain: String
}

impl CrawlTarget {
    pub fn new(url: reqwest::Url) -> CrawlTarget {
        CrawlTarget {
            url,
            top_level_domain: "".to_string()
        }
    }

    pub fn is_url_scheme_supported(url: &reqwest::Url) -> bool {
        let url_scheme = url.scheme();
        
        url_scheme.eq("http") || url_scheme.eq("https")
    }
}