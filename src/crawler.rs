
pub struct Crawler {

}

impl Crawler {
    pub async fn crawl(client: &reqwest::Client, url: reqwest::Url) -> () {
        let response = client.get(url).send().await.expect("Request failed.");

        println!("{}", response.text().await.unwrap());
    }
}