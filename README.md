# aur-raider

Scraper written in Rust for AUR repository.

## Get started

Clone the repo:

```bash
git clone https://github.com/0xf4lc0n/aur-raider
```

Enter aur-raider directory and create required directories:

```bash
cd aur-raider
mkdir logs
```

Build the scraper:

```bash
cargo build --release
```

### Scraping to file system

In this mode packages are scraped and saved on your disk in the form of BSON files.
Each BSON file contains 250 packages the AUR repository.

In order to scrap to file system create directory for BSON files and run scraper:

```bash
mkdir bins
# OPTIONAL - adjust log level, this line may differ for your shell
let-env RUST_LOG = 'aur_raider=info'
./target/release/aur-raider scrap-to-fs --path bins/ --start-page 1 --end-page 363
```

### Loading packages from file system do databases

In this mode previously scraped packages are loaded to databases from BSON files.

Do the follwoing steps:

```bash
# Run databases
docker compose up
# OPTIONAL - adjust log level, this line may differ for your shell
let-env RUST_LOG = 'aur_raider=info'
# Load packages from file system to databases
./target/debug/aur-raider load-from-fs --path bins/ --start-page 1 --end-page 363
```

## Logging

All errors are additionaly dumped into logs directory as plaintext files.

To manupilate log level on stdout use RUST_LOG environment variable (default log level is set to ERROR so you may want to change it).

## Using as lib

To use this package as library invoke the following cargo command:

```bash
cargo add aur-raider --git https://github.com/0xf4lc0n/aur-raider --no-default-features --features models
```
