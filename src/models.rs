use anyhow::anyhow;
use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
};
use thiserror::Error;

#[derive(Debug)]
pub struct PackageData {
    pub basic: BasicPackageData,
    pub additional: AdditionalPackageData,
    pub dependencies: Vec<PackageDependency>,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct AdditionalPackageData {
    pub git_clone_url: String,
    pub keywords: Option<String>,
    pub license: String,
    pub confilcts: Option<String>,
    pub provides: Option<String>,
    pub submitter: String,
    pub popularity: f32,
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
        let license = source
            .remove("licenses")
            .ok_or(ModelError::MissingSourceData { field: "license" })?;
        let confilcts = source.remove("conflicts");
        let provides = source.remove("provides");
        let submitter = source
            .remove("submitter")
            .ok_or(ModelError::MissingSourceData { field: "submitter" })?;
        let popularity = source
            .remove("popularity")
            .ok_or(ModelError::MissingSourceData {
                field: "popularity",
            })?
            .parse()
            .map_err(|e: ParseFloatError| ModelError::ParseError {
                field: "popularity",
                source: anyhow!(e),
            })?;
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
            popularity,
            first_submitted,
        })
    }
}

#[derive(Debug)]
pub struct PackageDependency {
    pub group: String,
    pub packages: Vec<String>,
}

#[derive(Debug)]
pub struct Comment {
    pub header: String,
    pub content: String,
}

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Source is missing data required to create struct")]
    MissingSourceData { field: &'static str },
    #[error("Cannot parse data for {field} field")]
    ParseError {
        field: &'static str,
        source: anyhow::Error,
    },
}
