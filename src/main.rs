mod models;
mod scrap;
mod selectors;

use scrap::AurScraper;

static AUR_BASE_URL: &str = "https://aur.archlinux.org/packages";

#[tokio::main]
async fn main() {
    let scraper = AurScraper::new();

    let packages = scraper.get_packages_from_page(AUR_BASE_URL).await.unwrap();

    for package in packages {
        println!("{package:?}");
    }
}
