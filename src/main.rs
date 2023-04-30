mod cli;
mod html;
mod models;
mod scrap;
mod selectors;
mod serialization;

use clap::Parser;
use cli::{Cli, Commands, ToFsArgs};
use serialization::{save_to_binary_file, serialize_to_bson};
use std::sync::Arc;
use tokio::time::Instant;
use tracing::info;

use crate::scrap::{get_page_and_scrap_packages, AurScraper, AUR_BASE_URL, AUR_PAGE_QUERY};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let scraper = Arc::new(AurScraper::new());
    tracing_subscriber::fmt().init();

    match &cli.command {
        Commands::ScrapToFs(fs_args) => scrap_and_save_to_fs(scraper, fs_args).await,
        Commands::ScrapToDb(_db_args) => todo!(),
        Commands::LoadFromFs(_db_args) => todo!(),
    }
}

async fn scrap_and_save_to_fs(scraper: Arc<AurScraper>, cfg: &ToFsArgs) {
    let pages_raneg = (cfg.start_page - 1)..(cfg.end_page.unwrap_or(cfg.start_page + 1));
    let start = Instant::now();

    for i in pages_raneg {
        let url = format!("{}{}{}", AUR_BASE_URL, AUR_PAGE_QUERY, i * 250);
        let packages = get_page_and_scrap_packages(scraper.clone(), &url)
            .await
            .unwrap();

        let file_name = format!("{}/page_{}.bson", cfg.path, i + 1);
        let serialized = serialize_to_bson(packages).unwrap();
        save_to_binary_file(&file_name, &serialized).await.unwrap();
    }

    let duration = start.elapsed();
    info!("Scraped 250 pages of packages in {:?}", duration);
}
