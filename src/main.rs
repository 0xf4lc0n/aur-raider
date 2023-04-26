mod models;
mod scrap;
mod selectors;

use std::sync::Arc;
use tokio::time::Instant;
use tracing::info;
use tracing_subscriber;

use crate::scrap::{get_page_and_scrap_packages, AurScraper, AUR_BASE_URL};

#[tokio::main]
async fn main() {
    let scraper = Arc::new(AurScraper::new());
    tracing_subscriber::fmt().init();

    let start = Instant::now();

    for i in 0..10 {
        let url = format!("{}/?SeB=nd&SB=p&O={}", AUR_BASE_URL, i * 50);
        get_page_and_scrap_packages(scraper.clone(), &url)
            .await
            .unwrap();
    }

    let duration = start.elapsed();
    info!("Scraped 10 pages of packages in {:?}", duration);
}
