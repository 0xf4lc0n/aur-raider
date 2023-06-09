mod cli;
mod database;
mod html;
mod models;
mod scrap;
mod selectors;
mod serialization;

use clap::Parser;
use cli::{Cli, Commands, FromFsArgs, ToFsArgs};
use database::{DatabasePackageIO, RedisIO, SkytableIO, SurrealIO};
use serialization::{read_binary_file_and_deserialize, save_to_binary_file, serialize_to_bson};
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
        .with(fmt::layer().with_filter(EnvFilter::from_default_env()))
        .init();

    match &cli.command {
        Commands::ScrapToFs(fs_args) => scrap_and_save_to_fs(scraper, fs_args).await,
        Commands::ScrapToDb(_db_args) => todo!(),
        Commands::LoadFromFs(from_fs_args) => {
            load_from_file_system_to_databases(from_fs_args).await
        }
    }
}

async fn scrap_and_save_to_fs(scraper: Arc<AurScraper>, cfg: &ToFsArgs) {
    let start_page = cfg.start_page - 1;
    let end_page = cfg.end_page.unwrap_or(cfg.start_page);
    let pages_range = start_page..(end_page);
    let start = Instant::now();

    for i in pages_range {
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
        end_page - start_page,
        duration
    );
}

async fn load_from_file_system_to_databases(cfg: &FromFsArgs) {
    let start_page = cfg.start_page;
    let end_page = cfg.end_page.unwrap_or(cfg.start_page) + 1;
    let pages_range = start_page..(end_page);

    for i in pages_range {
        let file_path = format!("{}/page_{}.bson", cfg.path, i);
        let packages = read_binary_file_and_deserialize(&file_path)
            .expect(&format!("Cannot read and deserialize file {}", file_path));

        let redis = RedisIO::try_new().expect("Cannot create RedisIO");
        let surreal = SurrealIO::try_new().await.expect("Cannot create SurrealIO");
        let skytable = SkytableIO::try_new().expect("Cannot create SkytableIO");
        skytable.create_tables().expect("Cannot create tables in Skytable");

        for pkg in packages {
            if let Err(e) = redis.insert(&pkg).await {
                error!(
                    "Failed to insert {} to Redis database. Caused by: {}",
                    &pkg.basic.name, e
                );
            }

            if let Err(e) = skytable.insert(&pkg).await {
                error!(
                    "Failed to insert {} to Skytable database. Caused by: {}",
                    &pkg.basic.name, e
                );
            }

            if let Err(e) = surreal.insert(&pkg).await {
                error!(
                    "Failed to insert {} to Surreal database. Caused by: {}",
                    &pkg.basic.name, e
                );
            }
            info!("Loaded {} package to all databases", &pkg.basic.name);
        }
    }
}
