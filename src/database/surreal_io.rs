use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

use crate::models::PackageData;

use super::DatabasePackageIO;

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}

type SurResult<T> = Result<T, surrealdb::Error>;

struct SurrealIO {
    db: Surreal<Client>,
}

impl SurrealIO {
    pub async fn try_new() -> Result<Self> {
        let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;

        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await?;

        db.use_ns("aur").use_db("packages").await?;

        Ok(Self { db })
    }

    async fn delete(&self, name: &str) -> Result<()> {
        let _: Option<PackageData> = self.db.delete(("pkgs", name)).await?;
        Ok(())
    }
}

fn skip_allready_exist_error<T>(res: SurResult<T>) -> Result<()> {
    if let Err(e) = res {
        if !e.to_string().contains("already exists") {
            return Err(e.into());
        }
    }

    Ok(())
}

#[async_trait]
impl DatabasePackageIO for SurrealIO {
    async fn health_check(&self) -> Result<()> {
        self.db.health().await?;
        Ok(())
    }

    async fn insert(&self, pkg: &PackageData) -> Result<()> {
        let res: SurResult<Record> = self
            .db
            .create(("pkgs", &pkg.basic.name))
            .content(&pkg)
            .await;
        skip_allready_exist_error(res)?;

        Ok(())
    }

    async fn get(&self, name: &str) -> Result<PackageData> {
        let pkg: PackageData = self.db.select(("pkgs", name)).await?;
        Ok(pkg)
    }
}

#[cfg(test)]
mod test {
    use super::SurrealIO;
    use crate::database::{shared::create_package_data, DatabasePackageIO};
    use anyhow::Result;

    #[tokio::test]
    async fn success_surrealio_init_when_database_is_up() {
        SurrealIO::try_new().await.unwrap();
    }

    #[tokio::test]
    async fn insert_data() -> Result<()> {
        // Arrange
        let db = SurrealIO::try_new().await?;
        let generated_pkg = create_package_data();

        // Act
        db.insert(&generated_pkg).await?;
        let retreived_pkg = db.get("Test").await?;
        db.delete("Test").await?;

        // Assert
        assert_eq!(retreived_pkg.basic.name, generated_pkg.basic.name);
        assert_eq!(retreived_pkg.comments.len(), generated_pkg.comments.len());
        assert_eq!(
            retreived_pkg.dependencies.len(),
            generated_pkg.dependencies.len()
        );

        Ok(())
    }
}
