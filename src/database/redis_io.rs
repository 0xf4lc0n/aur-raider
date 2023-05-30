use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use redis::{self, Client, Commands, Connection};

use crate::models::{Comment, PackageData, PackageDependency};

use super::DatabasePackageIO;

pub struct RedisIO {
    client: Client,
}

impl RedisIO {
    pub fn try_new() -> Result<Self> {
        let client = redis::Client::open("redis://localhost")?;
        Ok(Self { client })
    }

    fn connect(&self) -> Result<Connection> {
        self.client.get_connection().map_err(|e| anyhow!(e))
    }

    fn flushdb(&self) -> Result<()> {
        let mut conn = self.connect()?;
        redis::cmd("flushdb").query(&mut conn)?;
        Ok(())
    }
}

#[async_trait]
impl DatabasePackageIO for RedisIO {
    async fn health_check(&self) -> Result<()> {
        self.connect()?;
        Ok(())
    }

    async fn insert(&self, pkg: &PackageData) -> Result<()> {
        let mut conn = self.connect()?;

        conn.hset_multiple(
            format!("pkgs:{}", pkg.basic.name),
            &[
                ("popularity", pkg.basic.popularity.to_string().as_str()),
                ("last_updated", pkg.basic.last_updated.as_str()),
                ("description", pkg.basic.description.as_str()),
                ("maintainer", pkg.basic.maintainer.as_str()),
                ("version", pkg.basic.version.as_str()),
                ("votes", pkg.basic.votes.to_string().as_str()),
                ("path_to_additional_data", pkg.basic.path_to_additional_data.as_str()),
                ("firstsubmitted", pkg.additional.first_submitted.as_str()),
                ("gitcloneurl", pkg.additional.git_clone_url.as_str()),
                ("submitter", pkg.additional.submitter.as_str()),
                (
                    "confilcts",
                    pkg.additional.confilcts.as_ref().map(|s| s.as_str()).unwrap_or(""),
                ),
                (
                    "provides",
                    pkg.additional.provides.as_ref().map(|s| s.as_str()).unwrap_or(""),

                ),
                (
                    "keywords",
                    pkg.additional.keywords.as_ref().map(|s| s.as_str()).unwrap_or(""),
                ),
                (
                    "license",
                    pkg.additional.license.as_ref().map(|s| s.as_str()).unwrap_or(""),
                ),
            ],
        )?;

        for (idx, comment) in pkg.comments.iter().enumerate() {
            conn.hset_multiple(
                format!("pkgs:{}:cmnts:{}", pkg.basic.name, idx + 1),
                &[("header", &comment.header), ("content", &comment.content)],
            )?;

            conn.sadd(
                format!("pkgs:{}:cmnts", pkg.basic.name),
                format!("pkgs:{}:cmnts:{}", pkg.basic.name, idx + 1),
            )?;
        }

        for dependency in &pkg.dependencies {
            for dep in &dependency.packages {
                conn.rpush(
                    format!("pkgs:{}:deps:{}", pkg.basic.name, dependency.group),
                    dep,
                )?;
            }

            conn.sadd(
                format!("pkgs:{}:deps", pkg.basic.name),
                format!("pkgs:{}:deps:{}", pkg.basic.name, dependency.group),
            )?;
        }

        Ok(())
    }

    async fn get(&self, name: &str) -> Result<PackageData> {
        let mut conn = self.connect()?;

        let mut pkg_dict: HashMap<String, String> = conn.hgetall(format!("pkgs:{}", name))?;
        pkg_dict.insert("name".into(), name.into());

        let mut pkg = PackageData::try_from(pkg_dict).map_err(|e| anyhow!(e))?;

        let cmnts_list: Vec<String> = conn.smembers(format!("pkgs:{}:cmnts", pkg.basic.name))?;

        let mut comments = vec![];

        for cmnt in cmnts_list {
            let cmnt_dict: HashMap<String, String> = conn.hgetall(cmnt)?;
            comments.push(Comment::try_from(cmnt_dict)?);
        }

        pkg.comments = comments;

        let group_list: Vec<String> = conn.smembers(format!("pkgs:{}:deps", pkg.basic.name))?;

        let mut dependencies = vec![];

        for group in group_list {
            let packages: Vec<String> = conn.lrange(&group, 0, -1)?;

            dependencies.push(PackageDependency { group, packages });
        }

        pkg.dependencies = dependencies;

        Ok(pkg)
    }

    fn get_name(&self) -> &'static str {
        "Redis"
    }
}

#[cfg(test)]
mod test {
    use super::RedisIO;
    use crate::database::{
        shared::{assert_pkg, create_package_data},
        DatabasePackageIO,
    };
    use anyhow::Result;

    #[test]
    fn success_init_when_database_is_up() {
        // Act
        let redis = RedisIO::try_new();

        // Assert
        assert_eq!(redis.is_ok(), true);
    }

    #[test]
    fn success_connect_when_database_is_up() {
        // Arrange
        let redis = RedisIO::try_new().unwrap();

        // Act
        let con = redis.connect();

        // Assert
        assert_eq!(con.is_ok(), true);
    }

    #[tokio::test]
    async fn insert_data() -> Result<()> {
        // Arrange
        let redis = RedisIO::try_new()?;
        let generated_pkg = create_package_data();

        // Act
        redis.flushdb()?;
        redis.insert(&generated_pkg).await?;
        let retreived_pkg = redis.get("Test").await?;

        // Assert
        assert_pkg(&retreived_pkg, &generated_pkg);

        Ok(())
    }
}
