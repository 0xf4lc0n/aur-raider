use anyhow::Result;
use bson::{Bson, doc};
use serde::Serialize;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::models::PackageData;

#[derive(Serialize)]
struct Test {
    name: String,
    surname: String,
}

pub fn serialize_to_bson(packages: Vec<PackageData>) -> Result<Vec<u8>>
where
{
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
    file.write_all(&bytes).await?;
    file.sync_all().await?;
    Ok(())
}
