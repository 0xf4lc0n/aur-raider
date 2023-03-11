use std::error::Error;

pub struct PackageData {
    pub basic: BasicPackageData,
    pub additional: AdditionalPackageData,
}

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
        let name = iter.next().ok_or(ModelError::MissingSourceData)?;
        let path_to_additional_data = iter.next().ok_or(ModelError::MissingSourceData)?;
        let version = iter.next().ok_or(ModelError::MissingSourceData)?;
        let votes = iter
            .next()
            .ok_or(ModelError::MissingSourceData)?
            .parse()
            .map_err(|e| ModelError::ParseError(Box::new(e)))?;
        let popularity = iter
            .next()
            .ok_or(ModelError::MissingSourceData)?
            .parse()
            .map_err(|e| ModelError::ParseError(Box::new(e)))?;
        let description = iter.next().ok_or(ModelError::MissingSourceData)?;
        let maintainer = iter.next().ok_or(ModelError::MissingSourceData)?;
        let last_updated = iter.next().ok_or(ModelError::MissingSourceData)?;

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

pub struct AdditionalPackageData {
    pub git_clone_url: String,
    pub license: String,
    pub confilcts: Option<String>,
    pub provides: Option<String>,
    pub submitter: String,
    pub popularity: f32,
    // TODO: use chrono
    pub first_submitted: String,
}

#[derive(Debug)]
pub enum ModelError {
    MissingSourceData,
    ParseError(Box<dyn Error>),
}
