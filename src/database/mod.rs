use anyhow::Result;
use async_trait::async_trait;

use crate::models::PackageData;

mod redis_io;
pub use redis_io::RedisIO;

mod skytable_io;
pub use skytable_io::SkytableIO;

mod surreal_io;
pub use surreal_io::SurrealIO;

mod shared;

#[async_trait]
pub trait DatabasePackageIO {
    async fn health_check(&self) -> Result<()>;
    async fn insert(&self, pkg: &PackageData) -> Result<()>;
    async fn get(&self, name: &str) -> Result<PackageData>;
    fn get_name(&self) -> &'static str;
}
