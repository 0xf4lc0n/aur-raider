mod models;
mod scrap;
mod selectors;

use scrap::AurScraper;
use tokio::time::Instant;
use tracing_subscriber;
use tracing::info;

static AUR_BASE_URL: &str = "https://aur.archlinux.org/packages";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let scraper = AurScraper::new();

    let start = Instant::now();
    let _packages = scraper.get_packages_from_page(AUR_BASE_URL).await.unwrap();
    let duration = start.elapsed();
    info!("Packages scrapped in: {:?}", duration);
}
