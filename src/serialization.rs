use anyhow::Result;
use bson::{doc, Bson, Document};
use serde::Serialize;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::models::PackageData;

#[derive(Serialize)]
struct Test {
    name: String,
    surname: String,
}

pub fn serialize_to_bson(packages: Vec<PackageData>) -> Result<Vec<u8>> {
    let bson_vec: Vec<Bson> = packages
        .into_iter()
        .map(|pkg| bson::to_bson(&pkg).unwrap())
        .collect();

    let bson_doc = doc!("packages": bson_vec);

    let mut buffer = vec![];
    bson_doc.to_writer(&mut buffer)?;
    Ok(buffer)
}

pub async fn save_to_binary_file(file_name: &str, bytes: &[u8]) -> Result<()> {
    let mut file = File::create(file_name).await?;
    file.write_all(bytes).await?;
    file.sync_all().await?;
    Ok(())
}

pub fn read_binary_file_and_deserialize(path: &str) -> Result<Vec<PackageData>> {
    let file = std::fs::File::open(path)?;
    let document = Document::from_reader(file)?;
    let pkgs = document.get("packages").unwrap().to_owned();
    let packages: Vec<PackageData> = bson::from_bson(pkgs)?;
    Ok(packages)
}
