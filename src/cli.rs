use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scraps the AUR page and saves scraped objects to the local file system as the BSON files
    ScrapToFs(ToFsArgs),
    /// Scraps the AUR page and saves scraped objects to the database
    ScrapToDb(ToDbArgs),
    /// Reads scraped objects from the file system and loads them to the database
    LoadFromFs(FromFsArgs),
}

#[derive(Args)]
pub struct ToFsArgs {
    /// Page number from which scraping will start
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..))]
    pub start_page: u32,
    /// Page number to which scraping will process
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
    pub end_page: Option<u32>,
    /// Path to directory where BSON files will be stored
    #[arg(long)]
    pub path: String,
}
#[derive(Args)]
pub struct ToDbArgs {
    /// Database connection string
    #[arg(long)]
    pub cs: Vec<String>,
}

#[derive(Args)]
pub struct FromFsArgs {
    /// Path to directory where BSON are stored
    #[arg(long)]
    pub path: String,
}
