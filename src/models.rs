use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageData {
    pub basic: BasicPackageData,
    pub additional: AdditionalPackageData,
    pub dependencies: Vec<PackageDependency>,
    pub comments: Vec<Comment>,
}

impl TryFrom<HashMap<String, String>> for PackageData {
    type Error = ModelError;

    fn try_from(mut source: HashMap<String, String>) -> Result<Self, Self::Error> {
        let mut getter = |k| get_obligatory_field(&mut source, k);

        let name = getter("name")?;
        let path_to_additional_data = getter("path_to_additional_data")?;
        let version = getter("version")?;
        let votes =
            getter("votes")?
                .parse()
                .map_err(|e: ParseIntError| ModelError::ParseError {
                    field: "votes",
                    source: anyhow!(e),
                })?;
        let popularity = getter("popularity")?
            .parse()
            .map_err(|e: ParseFloatError| ModelError::ParseError {
                field: "popularity",
                source: anyhow!(e),
            })?;
        let description = getter("description")?;
        let maintainer = getter("maintainer")?;
        let last_updated = getter("last_updated")?;

        let basic = BasicPackageData {
            name,
            path_to_additional_data,
            version,
            votes,
            popularity,
            description,
            maintainer,
            last_updated,
        };

        let additional = AdditionalPackageData::try_from(source)?;

        Ok(Self {
            basic,
            additional,
            comments: vec![],
            dependencies: vec![],
        })
    }
}

fn get_obligatory_field(
    source: &mut HashMap<String, String>,
    key: &'static str,
) -> Result<String, ModelError> {
    source
        .remove(key)
        .ok_or(ModelError::MissingSourceData { field: key })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicPackageData {
    pub name: String,
    pub version: String,
    pub path_to_additional_data: String,
    pub votes: i32,
    pub popularity: f32,
    pub description: String,
    pub maintainer: String,
    // TODO: use chrono
    pub last_updated: String,
}

impl TryFrom<Vec<String>> for BasicPackageData {
    type Error = ModelError;

    fn try_from(source: Vec<String>) -> Result<Self, Self::Error> {
        let mut iter = source.into_iter();
        let name = iter
            .next()
            .ok_or(ModelError::MissingSourceData { field: "name" })?;

        let mut path_to_additional_data = iter.next().ok_or(ModelError::MissingSourceData {
            field: "path_to_additional_data",
        })?;

        if let Some(idx) = path_to_additional_data.rfind('/') {
            path_to_additional_data = path_to_additional_data[idx..].to_string();
        }

        let version = iter
            .next()
            .ok_or(ModelError::MissingSourceData { field: "version" })?;

        let votes = iter
            .next()
            .ok_or(ModelError::MissingSourceData { field: "votes" })?
            .parse()
            .map_err(|e: ParseIntError| ModelError::ParseError {
                field: "votes",
                source: anyhow!(e),
            })?;

        let popularity = iter
            .next()
            .ok_or(ModelError::MissingSourceData {
                field: "popularity",
            })?
            .parse()
            .map_err(|e: ParseFloatError| ModelError::ParseError {
                field: "popularity",
                source: anyhow!(e),
            })?;

        let description = iter.next().ok_or(ModelError::MissingSourceData {
            field: "description",
        })?;

        let maintainer = iter.next().ok_or(ModelError::MissingSourceData {
            field: "maintainer",
        })?;

        let last_updated = iter.next().ok_or(ModelError::MissingSourceData {
            field: "last_updated",
        })?;

        Ok(BasicPackageData {
            name,
            path_to_additional_data,
            version,
            votes,
            popularity,
            description,
            maintainer,
            last_updated,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdditionalPackageData {
    pub git_clone_url: String,
    pub keywords: Option<String>,
    pub license: Option<String>,
    pub confilcts: Option<String>,
    pub provides: Option<String>,
    pub submitter: String,
    // TODO: use chrono
    pub first_submitted: String,
}

impl TryFrom<HashMap<String, String>> for AdditionalPackageData {
    type Error = ModelError;

    fn try_from(mut source: HashMap<String, String>) -> Result<Self, Self::Error> {
        let git_clone_url = source
            .remove("gitcloneurl")
            .ok_or(ModelError::MissingSourceData {
                field: "git_clone_url",
            })?;
        let keywords = source.remove("keywords");
        let license = source.remove("licenses");
        let confilcts = source.remove("conflicts");
        let provides = source.remove("provides");
        let submitter = source
            .remove("submitter")
            .ok_or(ModelError::MissingSourceData { field: "submitter" })?;
        let first_submitted =
            source
                .remove("firstsubmitted")
                .ok_or(ModelError::MissingSourceData {
                    field: "first_submitted",
                })?;

        Ok(Self {
            git_clone_url,
            keywords,
            license,
            confilcts,
            provides,
            submitter,
            first_submitted,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageDependency {
    pub group: String,
    pub packages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    pub header: String,
    pub content: String,
}

impl TryFrom<HashMap<String, String>> for Comment {
    type Error = ModelError;

    fn try_from(mut source: HashMap<String, String>) -> Result<Self, Self::Error> {
        let mut getter = |k| get_obligatory_field(&mut source, k);

        let header = getter("header")?;
        let content = getter("content")?;

        Ok(Self { header, content })
    }
}

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Source lacks of data required to create struct. Missing field: {field}")]
    MissingSourceData { field: &'static str },
    #[error("Cannot parse data for {field} field")]
    ParseError {
        field: &'static str,
        source: anyhow::Error,
    },
}
