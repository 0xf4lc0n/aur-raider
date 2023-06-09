use anyhow::Result;
use async_trait::async_trait;
use skytable::{
    actions::Actions,
    ddl::{Ddl, Keymap, KeymapType},
    pool::{self, Pool},
    types::{FromSkyhashBytes, IntoSkyhashBytes},
    SkyResult,
};

const BASIC_PKGS_TABLE: &str = "basic";
const ADDITIONAL_PKGS_TABLE: &str = "additional";
const COMMENTS_TABLE: &str = "comments";
const DEPENDENCIES_TABLE: &str = "dependencies";

use crate::models::{
    AdditionalPackageData, BasicPackageData, Comment, PackageData, PackageDependency,
};

use super::DatabasePackageIO;

pub struct SkytableIO {
    pool: Pool,
}

impl SkytableIO {
    pub fn try_new() -> Result<Self> {
        let pool = pool::get("127.0.0.1", 2003, 16)?;

        Ok(Self { pool })
    }

    fn flushdb(&self) -> Result<()> {
        let mut conn = self.pool.get()?;
        conn.flushdb()?;
        Ok(())
    }

    fn create_tables(&self, keyspace: &str) -> Result<(String, String, String, String)> {
        let mut conn = self.pool.get()?;
        check_err(conn.create_keyspace(keyspace))?;

        let basic_table = format!("{}:{}", keyspace, BASIC_PKGS_TABLE);
        let advanced_table = format!("{}:{}", keyspace, ADDITIONAL_PKGS_TABLE);
        let comments_table = format!("{}:{}", keyspace, COMMENTS_TABLE);
        let deps_table = format!("{}:{}", keyspace, DEPENDENCIES_TABLE);

        let pkgs_table = Keymap::new(&basic_table)
            .set_ktype(KeymapType::Str)
            .set_vtype(KeymapType::Binstr);

        check_err(conn.create_table(pkgs_table))?;

        let pkgs_table = Keymap::new(&advanced_table)
            .set_ktype(KeymapType::Str)
            .set_vtype(KeymapType::Binstr);

        check_err(conn.create_table(pkgs_table))?;

        let pkgs_table = Keymap::new(&comments_table)
            .set_ktype(KeymapType::Str)
            .set_vtype(KeymapType::Binstr);

        check_err(conn.create_table(pkgs_table))?;

        let pkgs_table = Keymap::new(&deps_table)
            .set_ktype(KeymapType::Str)
            .set_vtype(KeymapType::Binstr);

        check_err(conn.create_table(pkgs_table))?;
        Ok((basic_table, advanced_table, comments_table, deps_table))
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

        //TODO: create only once
        let (basic, additional, comments, deps) = self.create_tables("pkgs")?;

        conn.switch(basic)?;
        conn.set(&pkg_name, &pkg.basic)?;

        conn.switch(additional)?;
        conn.set(&pkg_name, &pkg.additional)?;

        conn.switch(comments)?;
        for (idx, comment) in pkg.comments.iter().enumerate() {
            let name = (idx + 1).to_string();
            conn.set(name, comment)?;
        }

        conn.switch(deps)?;
        for (idx, dep) in pkg.dependencies.iter().enumerate() {
            let name = (idx + 1).to_string();
            conn.set(name, dep)?;
        }

        Ok(())
    }

    async fn get(&self, name: &str) -> Result<crate::models::PackageData> {
        let mut conn = self.pool.get()?;

        let basic_table = format!("{}:{}", name, BASIC_PKGS_TABLE);
        let advanced_table = format!("{}:{}", name, ADDITIONAL_PKGS_TABLE);
        let comments_table = format!("{}:{}", name, COMMENTS_TABLE);
        let deps_table = format!("{}:{}", name, DEPENDENCIES_TABLE);

        conn.switch(basic_table)?;
        let basic: BasicPackageData = conn.get(name)?;

        conn.switch(advanced_table)?;
        let additional: AdditionalPackageData = conn.get(name)?;

        conn.switch(comments_table)?;
        let comment_keys: Vec<String> = conn.lskeys(1_000_000 as u64)?;

        let mut comments = vec![];

        for key in comment_keys {
            let cmnt: Comment = conn.get(key)?;
            comments.push(cmnt);
        }

        conn.switch(deps_table)?;
        let dep_keys: Vec<String> = conn.lskeys(1_000_000 as u64)?;

        let mut dependencies = vec![];

        for key in dep_keys {
            let dep: PackageDependency = conn.get(key)?;
            dependencies.push(dep);
        }

        Ok(PackageData {
            basic,
            additional,
            dependencies,
            comments,
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
        skytable.insert(&generated_pkg).await?;
        let retreived_pkg = skytable.get("Test").await?;

        // Assert
        assert_pkg(&retreived_pkg, &generated_pkg);

        Ok(())
    }
}
