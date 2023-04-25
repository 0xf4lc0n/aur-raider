mod models;
mod scrap;
mod selectors;

use reqwest::Client;
use scrap::AurScraper;
use tokio::time::Instant;
use tracing::info;
use tracing_subscriber;

static AUR_BASE_URL: &str = "https://aur.archlinux.org/packages";

#[tokio::main]
async fn main() {
    let client = Client::new();
    tracing_subscriber::fmt().init();

    let start = Instant::now();
    let _packages = AurScraper::get_packages_from_page(client, AUR_BASE_URL)
        .await
        .unwrap();
    let duration = start.elapsed();
    info!("Packages scrapped in: {:?}", duration);
}
