use reqwest::{header::{HeaderMap, HeaderValue}, Client, Response};
use url::Url;

/// Perform a HEAD request to the specified URL
pub async fn head_url(client: &Client, url: Url) -> Result<Response, reqwest::Error> {
    client.head(url).send().await
}

/// Perform a GET request to the specified URL
pub async fn get_url(client: &Client, url: Url) -> Result<Response, reqwest::Error >{
    client.get(url).send().await
}

/// Obtain the headers of the response to a GET request
pub async fn get_url_response_headers(client: &Client, url: Url) -> Result<HeaderMap<HeaderValue>, reqwest::Error> {
    match (client.head(url).send().await) {
        Ok(response) => {
            Ok(response.headers().to_owned())
        },
        Err(error) => Err(error)
    }
}

