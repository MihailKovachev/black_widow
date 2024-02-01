mod cli;
mod crawler;

use clap::Parser;
pub use cli::*;
pub use crawler::*;

#[tokio::main]
async fn main() {
    
    let cli = Cli::parse();

    if let Ok(base_url) = reqwest::Url::parse(&cli.base_url)
    {
        let mut client_config = reqwest::Client::builder();

        client_config = client_config.user_agent(concat!(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")));

        if let Ok(client) = client_config.build()
        {
            Crawler::crawl(&client, base_url).await;
        }
        else {
            eprintln!("Failed to initialise the HTTP(S) client.")
        }
    }
    else {
        eprintln!("The specified URL is not valid.");
    }
}
