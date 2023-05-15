mod cli;
mod html;
mod models;
mod scrap;
mod selectors;
mod serialization;

use clap::Parser;
use cli::{Cli, Commands, ToFsArgs};
use serialization::{save_to_binary_file, serialize_to_bson};
use std::fs::File;
use std::sync::Arc;
use tokio::time::Instant;
use tracing::{error, info, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::scrap::{get_page_and_scrap_packages, AurScraper, AUR_BASE_URL, AUR_PAGE_QUERY};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let scraper = Arc::new(AurScraper::new());

    let log_file = File::create("logs/errors.log").unwrap();

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(log_file.with_min_level(Level::ERROR))
                .with_filter(EnvFilter::new("error")),
        )
        .with(
            fmt::Layer::new().with_writer(
                std::io::stdout
                    .with_max_level(Level::INFO)
                    .with_min_level(Level::INFO),
            ),
        )
        .init();

    match &cli.command {
        Commands::ScrapToFs(fs_args) => scrap_and_save_to_fs(scraper, fs_args).await,
        Commands::ScrapToDb(_db_args) => todo!(),
        Commands::LoadFromFs(_db_args) => todo!(),
    }
}

async fn scrap_and_save_to_fs(scraper: Arc<AurScraper>, cfg: &ToFsArgs) {
    let end_page = cfg.end_page.unwrap_or(cfg.start_page + 1);
    let pages_raneg = (cfg.start_page - 1)..(end_page);
    let start = Instant::now();

    for i in pages_raneg {
        let url = format!("{}{}{}", AUR_BASE_URL, AUR_PAGE_QUERY, i * 250);

        match get_page_and_scrap_packages(scraper.clone(), &url).await {
            Ok(packages) => {
                let file_name = format!("{}/page_{}.bson", cfg.path, i + 1);
                let serialized = serialize_to_bson(packages).unwrap();
                save_to_binary_file(&file_name, &serialized).await.unwrap();
            }
            Err(e) => error!("{}", e),
        }
    }

    let duration = start.elapsed();
    info!(
        "Scraped {} pages of packages in {:?}",
        end_page * 250,
        duration
    );
}
