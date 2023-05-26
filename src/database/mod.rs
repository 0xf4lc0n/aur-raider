use anyhow::Result;

use crate::models::PackageData;

mod redis_io;
mod skytable_io;
mod dummy;

pub trait DatabasePackageIO {
    fn health_check(&self) -> Result<()>;
    fn insert(&self, pkg: PackageData) -> Result<()>;
    fn get(&self, name: &str) -> Result<PackageData>;
}
