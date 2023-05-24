use std::collections::HashMap;

use anyhow::{anyhow, Result};
use redis::{self, Client, Commands, Connection};

use crate::models::{Comment, PackageData};

// TODO: Implement DatabaseIO trait
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

    pub fn insert_package_data(&self, pkg: PackageData) -> Result<()> {
        let mut conn = self.connect()?;

        let res: Result<()> = conn
            .hset_multiple(
                format!("pkgs:{}", pkg.basic.name),
                &[
                    ("popularity", pkg.basic.popularity.to_string()),
                    ("last_updated", pkg.basic.last_updated),
                    ("description", pkg.basic.description),
                    ("maintainer", pkg.basic.maintainer),
                    ("version", pkg.basic.version),
                    ("votes", pkg.basic.votes.to_string()),
                    ("path_to_additional_data", pkg.basic.path_to_additional_data),
                    ("firstsubmitted", pkg.additional.first_submitted),
                    ("gitcloneurl", pkg.additional.git_clone_url),
                    ("submitter", pkg.additional.submitter),
                    (
                        "confilcts",
                        pkg.additional.confilcts.unwrap_or_else(|| String::new()),
                    ),
                    (
                        "provides",
                        pkg.additional.provides.unwrap_or_else(|| String::new()),
                    ),
                    (
                        "keywords",
                        pkg.additional.keywords.unwrap_or_else(|| String::new()),
                    ),
                    (
                        "license",
                        pkg.additional.license.unwrap_or_else(|| String::new()),
                    ),
                ],
            )
            .map_err(|e| anyhow!(e));

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

        for dependency in pkg.dependencies {
            for dep in dependency.packages {
                conn.rpush(
                    format!("pkgs:{}:deps:{}:pkgs", pkg.basic.name, dependency.group),
                    dep,
                )?;
            }

            // TODO: Add a set of deps groups
        }

        Ok(())
    }

    pub fn get_package_data(&self, name: &str) -> Result<PackageData> {
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

        return Ok(pkg);
    }

    fn flushdb(&self) -> Result<()> {
        let mut conn = self.connect()?;
        redis::cmd("flushdb").query(&mut conn)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::models::{
        AdditionalPackageData, BasicPackageData, Comment, PackageData, PackageDependency,
    };

    use super::RedisIO;

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

    #[test]
    fn insert_data() {
        // Arrange
        let redis = RedisIO::try_new().unwrap();
        let pkg = PackageData {
            basic: BasicPackageData {
                name: "Test".into(),
                votes: 100,
                version: "1.2".into(),
                popularity: 6.2,
                maintainer: "Tester".into(),
                description: "Sample description".into(),
                last_updated: "2012".into(),
                path_to_additional_data: "/test".into(),
            },
            additional: AdditionalPackageData {
                license: None,
                keywords: None,
                provides: None,
                confilcts: None,
                submitter: "Tester".into(),
                git_clone_url: "some git url".into(),
                first_submitted: "2011".into(),
            },
            comments: vec![
                Comment {
                    header: "Someone wrote at 14:15".into(),
                    content: "Cool package".into(),
                },
                Comment {
                    header: "Foo wrote at 20:30".into(),
                    content: "Not bad".into(),
                },
            ],

            dependencies: vec![PackageDependency {
                group: "abc".into(),
                packages: vec!["aaa".into(), "bbb".into(), "ccc".into()],
            }],
        };

        // Act
        redis.flushdb().unwrap();
        redis.insert_package_data(pkg).unwrap();
        let pkg = redis.get_package_data("Test").unwrap();

        // Assert
        assert_eq!(pkg.basic.name, "Test");
        assert!(pkg.comments.len() == 2);
    }
}
