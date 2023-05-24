use anyhow::Result;

mod redis_io;

pub trait DatabaseIO {
    fn health_check(&self) -> Result<()>;
    fn insert(&self) -> Result<()>;
}
