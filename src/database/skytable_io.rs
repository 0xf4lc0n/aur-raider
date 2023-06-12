use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use skytable::{
    actions::Actions,
    ddl::{Ddl, Keymap, KeymapType},
    pool::{self, Pool},
    types::{FromSkyhashBytes, IntoSkyhashBytes},
    SkyResult, Query,
};

const KEYSPACE: &str = "pkgs";
const BASIC_PKGS_TABLE: &str = "pkgs:basic";
const ADDITIONAL_PKGS_TABLE: &str = "pkgs:additional";
const COMMENTS_TABLE: &str = "pkgs:comments";
const DEPENDENCIES_TABLE: &str = "pkgs:dependencies";

use crate::models::{
    AdditionalPackageData, BasicPackageData, Comment, PackageData, PackageDependency,
};

use super::DatabasePackageIO;

#[derive(Debug, Serialize, Deserialize)]
struct Comments {
    data: Vec<Comment>
}

#[derive(Debug, Serialize, Deserialize)]
struct Dependencies {
    data: Vec<PackageDependency>
}

pub struct SkytableIO {
    pool: Pool,
}

impl SkytableIO {
    pub fn try_new() -> Result<Self> {
        let pool = pool::get("127.0.0.1", 2003, 16)?;
        Ok(Self { pool })
    }

    pub fn create_tables(&self) -> Result<()> {
        let mut conn = self.pool.get()?;
        check_err(conn.create_keyspace(KEYSPACE))?;

        let pkgs_table = Keymap::new(BASIC_PKGS_TABLE)
            .set_ktype(KeymapType::Str)
            .set_vtype(KeymapType::Binstr);

        check_err(conn.create_table(pkgs_table))?;

        let pkgs_table = Keymap::new(ADDITIONAL_PKGS_TABLE)
            .set_ktype(KeymapType::Str)
            .set_vtype(KeymapType::Binstr);

        check_err(conn.create_table(pkgs_table))?;

        let pkgs_table = Keymap::new(COMMENTS_TABLE)
            .set_ktype(KeymapType::Str)
            .set_vtype(KeymapType::Other("list<binstr>".to_owned()));

        check_err(conn.create_table(pkgs_table))?;

        let pkgs_table = Keymap::new(DEPENDENCIES_TABLE)
        .set_ktype(KeymapType::Str)
        .set_vtype(KeymapType::Other("list<binstr>".to_owned()));

        check_err(conn.create_table(pkgs_table))?;
        Ok(())
    }

    fn flushdb(&self) -> Result<()> {
        let mut conn = self.pool.get()?;
        conn.flushdb()?;
        Ok(())
    }

}

fn check_err<T>(res: SkyResult<T>) -> Result<()> {
    if let Err(e) = res {
        if !e.to_string().contains("already-exists") {
            return Err(e.into());
        }
    }

    Ok(())
}

#[async_trait]
impl DatabasePackageIO for SkytableIO {
    async fn health_check(&self) -> Result<()> {
        self.pool.get()?;
        Ok(())
    }

    async fn insert(&self, pkg: &PackageData) -> Result<()> {
        let mut conn = self.pool.get()?;
        let pkg_name = pkg.basic.name.clone();

        conn.switch(BASIC_PKGS_TABLE)?;
        conn.set(&pkg_name, &pkg.basic)?;

        conn.switch(ADDITIONAL_PKGS_TABLE)?;
        conn.set(&pkg_name, &pkg.additional)?;

        conn.switch(COMMENTS_TABLE)?;
        conn.run_query_raw(Query::new().arg("LSET").arg(&pkg.basic.name))?;
        conn.run_query_raw(Query::new().arg("LMOD").arg(&pkg.basic.name).arg("CLEAR"))?;

        for comment in &pkg.comments {
            let query = Query::new().arg("LMOD").arg(&pkg.basic.name).arg("PUSH").arg(comment);
            conn.run_query_raw(query)?;
        }

        conn.switch(DEPENDENCIES_TABLE)?;
        conn.run_query_raw(Query::new().arg("LSET").arg(&pkg.basic.name))?;
        conn.run_query_raw(Query::new().arg("LMOD").arg(&pkg.basic.name).arg("CLEAR"))?;

        for dependency in &pkg.dependencies {
            let query = Query::new().arg("LMOD").arg(&pkg.basic.name).arg("PUSH").arg(dependency);
            conn.run_query_raw(query)?;
        }
   
        Ok(())
    }

    async fn get(&self, name: &str) -> Result<crate::models::PackageData> {
        let mut conn = self.pool.get()?;

        conn.switch(BASIC_PKGS_TABLE)?;
        let basic: BasicPackageData = conn.get(name)?;

        conn.switch(ADDITIONAL_PKGS_TABLE)?;
        let additional: AdditionalPackageData = conn.get(name)?;

        conn.switch(COMMENTS_TABLE)?;
        let comments: Comments = conn.run_query(Query::new().arg("LGET").arg(name))?;

        conn.switch(DEPENDENCIES_TABLE)?;
        let dependencies: Dependencies = conn.run_query(Query::new().arg("LGET").arg(name))?;

        Ok(PackageData {
            basic,
            additional,
            comments: comments.data,
            dependencies: dependencies.data,
        })
    }
}

impl IntoSkyhashBytes for &BasicPackageData {
    fn as_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Cannot serialize PackageData to Vec<u8>")
    }
}

impl FromSkyhashBytes for BasicPackageData {
    fn from_element(element: skytable::Element) -> SkyResult<Self> {
        let bytes: Vec<u8> = element.try_element_into()?;
        serde_json::from_slice(&bytes)
            .map_err(|e| skytable::error::Error::ParseError(e.to_string()))
    }
}

impl IntoSkyhashBytes for &AdditionalPackageData {
    fn as_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Cannot serialize AdditionalPackageData to Vec<u8>")
    }
}

impl FromSkyhashBytes for AdditionalPackageData {
    fn from_element(element: skytable::Element) -> SkyResult<Self> {
        let bytes: Vec<u8> = element.try_element_into()?;
        serde_json::from_slice(&bytes)
            .map_err(|e| skytable::error::Error::ParseError(e.to_string()))
    }
}

impl IntoSkyhashBytes for &Comment {
    fn as_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Cannot serialize Comment to Vec<u8>")
    }
}

impl FromSkyhashBytes for Comment {
    fn from_element(element: skytable::Element) -> SkyResult<Self> {
        let bytes: Vec<u8> = element.try_element_into()?;
        serde_json::from_slice(&bytes)
            .map_err(|e| skytable::error::Error::ParseError(e.to_string()))
    }
}

impl FromSkyhashBytes for Comments {
    fn from_element(element: skytable::Element) -> SkyResult<Self> {
        let mut comments: Vec<Comment> = Vec::new();
        let comments_bytes: Vec<Vec<u8>> = element.try_element_into()?;
        for comment_bytes in &comments_bytes {
            let comment: Comment = serde_json::from_slice(comment_bytes)
                .map_err(|e| skytable::error::Error::ParseError(e.to_string()))?;
            comments.push(comment);
        }
        Ok(Comments { data: comments })
    }
}

impl FromSkyhashBytes for Dependencies {
    fn from_element(element: skytable::Element) -> SkyResult<Self> {
        let dependencies_bytes: Vec<Vec<u8>> = element.try_element_into()?;
        let mut dependencies: Vec<PackageDependency> = Vec::new();
        for dependency_bytes in dependencies_bytes {
            let dependency: PackageDependency = serde_json::from_slice(&dependency_bytes)
                .map_err(|e| skytable::error::Error::ParseError(e.to_string()))?;
            dependencies.push(dependency);
        }
        Ok(Dependencies { data: dependencies })
    }
}

impl IntoSkyhashBytes for &PackageDependency {
    fn as_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Cannot serialize PackageDependency to Vec<u8>")
    }
}

impl FromSkyhashBytes for PackageDependency {
    fn from_element(element: skytable::Element) -> SkyResult<Self> {
        let bytes: Vec<u8> = element.try_element_into()?;
        serde_json::from_slice(&bytes)
            .map_err(|e| skytable::error::Error::ParseError(e.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::SkytableIO;
    use crate::database::{
        shared::{assert_pkg, create_package_data},
        DatabasePackageIO,
    };
    use anyhow::Result;

    #[test]
    fn skytable_success_init_when_database_is_up() -> Result<()> {
        SkytableIO::try_new()?;
        Ok(())
    }

    #[test]
    fn skytable_success_connect_when_database_is_up() -> Result<()> {
        let sky = SkytableIO::try_new()?;
        sky.pool.get()?;
        Ok(())
    }

    #[tokio::test]
    async fn insert_data() -> Result<()> {
        // Arrange
        let skytable = SkytableIO::try_new()?;
        let generated_pkg = create_package_data();

        // Act
        skytable.flushdb()?;
        skytable.create_tables()?;
        skytable.insert(&generated_pkg).await?;
        let retreived_pkg = skytable.get("Test").await?;

        // Assert
        assert_pkg(&retreived_pkg, &generated_pkg);

        Ok(())
    }
}
