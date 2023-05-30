use anyhow::Result;
use async_trait::async_trait;

use crate::models::PackageData;

mod redis_io;
mod skytable_io;
mod surreal_io;
mod shared;

#[async_trait]
pub trait DatabasePackageIO {
    async fn health_check(&self) -> Result<()>;
    async fn insert(&self, pkg: &PackageData) -> Result<()>;
    async fn get(&self, name: &str) -> Result<PackageData>;
}
