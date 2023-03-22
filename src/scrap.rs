use std::collections::HashMap;

use crate::{
    models::{AdditionalPackageData, BasicPackageData, PackageData, PackageDependency},
    selectors::*,
};
use anyhow::{Context, Result};
use reqwest::Client;
use scraper::Html;

pub struct AurScraper {
    http_client: Client,
}

impl AurScraper {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }

    pub async fn get_packages_from_page(&self, url: &str) -> Result<Vec<PackageData>> {
        let response = self.http_client.get(url).send().await?;
        let body = response.text().await?;
        let html_content = Html::parse_document(&body);

        let mut packages_data = vec![];

        for table in html_content.select(&TABLE_RESULT_SELECTOR) {
            for tbody in table.select(&TBODY_SELECTOR) {
                for tr in tbody.select(&TR_SELECTOR) {
                    let mut package_basic_info = vec![];
                    for td in tr.select(&TD_SELECTOR) {
                        if let Some(a) = td.select(&A_SELECTOR).next() {
                            package_basic_info.push(a.inner_html().trim().to_string());
                            // TODO: extract to separate function
                            package_basic_info.push(
                                a.value()
                                    .attr("href")
                                    .map_or("".to_string(), |s| s.to_string()),
                            );
                        } else {
                            package_basic_info.push(td.inner_html().trim().to_string());
                        }
                    }

                    let basic_package_data = BasicPackageData::try_from(package_basic_info)
                        .with_context(|| {
                            format!("Failed to scrap basic data for package from page {}", url)
                        })?;

                    let url_to_details =
                        format!("{}{}", url, basic_package_data.path_to_additional_data);

                    let additional_package_data =
                        self.get_additional_package_data(&url_to_details).await?;

                    let dependencies = self.get_dependencies(&url_to_details).await?;

                    packages_data.push(PackageData {
                        basic: basic_package_data,
                        additional: additional_package_data,
                        dependencies,
                    });
                }
            }
        }

        Ok(packages_data)
    }

    async fn get_additional_package_data(&self, url: &str) -> Result<AdditionalPackageData> {
        let response = self.http_client.get(url).send().await?;
        let body = response.text().await?;
        let html_content = Html::parse_document(&body);

        let mut package_data = HashMap::new();

        for table in html_content.select(&TABLE_PKGINFO_SELECTOR) {
            for tbody in table.select(&TBODY_SELECTOR) {
                for tr in tbody.select(&TR_SELECTOR) {
                    let key = tr
                        .select(&TH_SELECTOR)
                        .next()
                        .expect("Cannot scrap th tag")
                        .inner_html()
                        .trim()
                        .strip_suffix(':')
                        .expect("Value of the th tag is missing a ':'")
                        .split(' ')
                        .map(|s| s.to_lowercase())
                        .collect::<String>();

                    for td in tr.select(&TD_SELECTOR) {
                        if td.select(&A_SELECTOR).next().is_some() {
                            let mut buff = vec![];
                            for a in td.select(&A_SELECTOR) {
                                buff.push(a.inner_html().trim().to_string());
                            }
                            let links = buff.join(",");
                            package_data.insert(key.clone(), links);
                        } else {
                            package_data.insert(key.clone(), td.inner_html().trim().to_string());
                        }
                    }
                }
            }
        }

        AdditionalPackageData::try_from(package_data)
            .with_context(|| format!("Failed to scrap additional data for {} package", url))
    }

    async fn get_dependencies(&self, url: &str) -> Result<Vec<PackageDependency>> {
        let response = self.http_client.get(url).send().await?;
        let body = response.text().await?;
        let html_content = Html::parse_document(&body);

        let mut dependencies = vec![];

        for ul in html_content.select(&UL_DEPS_SELECTOR) {
            for li in ul.select(&LI_SELECTOR) {
                if let Some(a) = li.select(&A_SELECTOR).next() {
                    let group = a.inner_html().trim().to_string();

                    let mut packages = vec![];
                    for em in li.select(&EM_SELECTOR) {
                        for a in em.select(&A_SELECTOR) {
                            packages.push(a.inner_html().trim().to_string());
                        }
                    }

                    dependencies.push(PackageDependency { group, packages });
                }
            }
        }

        Ok(dependencies)
    }
}