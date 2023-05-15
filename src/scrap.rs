use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    html::{delete_tags, extract_attribute_value},
    models::{AdditionalPackageData, BasicPackageData, Comment, PackageData, PackageDependency},
    selectors::*,
};
use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use scraper::{ElementRef, Html};
use tokio::task::JoinSet;
use tracing::{error, info};

pub static AUR_BASE_URL: &str = "https://aur.archlinux.org/packages";
pub static AUR_PAGE_QUERY: &str = "?PP=250&SeB=nd&SB=p&O=";

pub struct AurScraper {
    http_client: Client,
}

impl AurScraper {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }

    async fn get_parsed_page(&self, url: &str) -> Result<Html> {
        Self::get_parsed_page_with_client(self.http_client.clone(), url).await
    }

    async fn get_parsed_page_with_client(http_client: Client, url: &str) -> Result<Html> {
        let response = http_client
            .get(url)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;
        let body = response.text().await?;
        Ok(Html::parse_document(&body))
    }

    pub async fn get_packages_basic_data_from_page(
        &self,
        url: &str,
    ) -> Result<Vec<BasicPackageData>> {
        let html_content = self.get_parsed_page(url).await?;
        scrap_packages_from_page(html_content)
    }

    #[allow(unused)]
    pub async fn get_package_details_from_page(
        &self,
        url: &str,
    ) -> Result<(AdditionalPackageData, Vec<PackageDependency>)> {
        let html_content = self.get_parsed_page(url).await?;
        let (additional, dependencies) = scrap_package_details(&html_content)
            .with_context(|| format!("Failed to scrap details for {}", url))?;

        Ok((additional, dependencies))
    }

    #[allow(unused)]
    pub async fn get_package_comments_from_page(&self, url: &str) -> Result<Vec<Comment>> {
        let html_content = self.get_parsed_page(url).await?;
        scrap_package_comments(html_content)
    }

    pub async fn get_package_details_with_comments_from_page(
        &self,
        url: &str,
    ) -> Result<(AdditionalPackageData, Vec<PackageDependency>, Vec<Comment>)> {
        let html_content = self.get_parsed_page(url).await?;

        let (additional, dependencies) = scrap_package_details(&html_content)
            .with_context(|| format!("Failed to scrap details for {}", url))?;

        let comments = scrap_package_comments(html_content)?;

        Ok((additional, dependencies, comments))
    }
}

pub async fn get_page_and_scrap_packages(
    scraper: Arc<AurScraper>,
    url: &str,
) -> Result<Vec<PackageData>> {
    let start = Instant::now();
    let packages_basic_data = scraper.get_packages_basic_data_from_page(url).await?;

    let mut set = JoinSet::new();

    for basic in &packages_basic_data {
        let url = format!("{}{}", AUR_BASE_URL, basic.path_to_additional_data);
        let scraper = scraper.clone();

        set.spawn(async move {
            let (details, deps, comments) = scraper
                .get_package_details_with_comments_from_page(&url)
                .await?;

            Result::<(AdditionalPackageData, Vec<PackageDependency>, Vec<Comment>)>::Ok((
                details, deps, comments,
            ))
        });
    }

    let mut details_and_comments = vec![];

    while let Some(task_result) = set.join_next().await {
        let task_result = task_result.map_err(|e| anyhow!(e)).and_then(|tr| tr);

        match task_result {
            Ok((details, deps, comments)) => details_and_comments.push((details, deps, comments)),
            Err(e) => error!("{}", e),
        }
    }

    let duration = start.elapsed();

    let packages: Vec<PackageData> = packages_basic_data
        .into_iter()
        .zip(details_and_comments)
        .map(
            |(basic, (additional, dependencies, comments))| PackageData {
                basic,
                additional,
                dependencies,
                comments,
            },
        )
        .collect();

    info!("Scraped packages from {} in: {:?}", url, duration);

    Ok(packages)
}

fn scrap_packages_from_page(html_content: Html) -> Result<Vec<BasicPackageData>> {
    let mut packages_basic_data = vec![];

    for table in html_content.select(&TABLE_RESULT_SELECTOR) {
        for tbody in table.select(&TBODY_SELECTOR) {
            for tr in tbody.select(&TR_SELECTOR) {
                let basic_package_data = scrap_package_basic_data(tr)?;
                packages_basic_data.push(basic_package_data);
            }
        }
    }

    Ok(packages_basic_data)
}

fn scrap_package_basic_data(tr: ElementRef<'_>) -> Result<BasicPackageData> {
    let mut package_basic_info = vec![];

    for td in tr.select(&TD_SELECTOR) {
        if let Some(a) = td.select(&A_SELECTOR).next() {
            package_basic_info.push(a.inner_html().trim().to_string());
            package_basic_info.push(extract_attribute_value(a, "href"));
        } else {
            package_basic_info.push(td.inner_html().trim().to_string());
        }
    }

    BasicPackageData::try_from(package_basic_info).map_err(|e| anyhow!(e))
}

fn scrap_package_details(
    package_details: &Html,
) -> Result<(AdditionalPackageData, Vec<PackageDependency>)> {
    let additional = scrap_package_additional_data(package_details)
        .with_context(|| "Failed to scrap additional data for package".to_string())?;

    let dependencies = scrap_package_dependencies(package_details)
        .with_context(|| "Failed to scrap dependencies for package".to_string())?;

    Ok((additional, dependencies))
}

fn scrap_package_additional_data(html_content: &Html) -> Result<AdditionalPackageData> {
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

    AdditionalPackageData::try_from(package_data).map_err(|err| anyhow!(err))
}

fn scrap_package_dependencies(html_content: &Html) -> Result<Vec<PackageDependency>> {
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

fn scrap_package_comments(html_content: Html) -> Result<Vec<Comment>> {
    let mut comments = vec![];

    for comments_container in html_content.select(&DIV_COMMENTS_SELECTOR) {
        for (comment_header, comment_content) in comments_container
            .select(&H4_COMMENT_HEADER_SELECTOR)
            .zip(comments_container.select(&DIV_COMMENT_CONTENT_SELECTOR))
            // Skip pinned comment
            .skip(1)
        {
            let header = delete_tags(comment_header.inner_html());
            let content = delete_tags(comment_content.inner_html());

            comments.push(Comment { header, content })
        }
    }

    Ok(comments)
}

#[allow(unused)]
fn get_last_comment_page_number(html_content: Html) -> usize {
    let comment_nav = html_content.select(&P_COMMENT_HEADER_NAV_SELECTOR).next();

    // Case when there is only one comment page
    let Some(comment_nav) = comment_nav else { return 1; };

    let comment_pages = comment_nav
        .select(&A_PAGE_SELECTOR)
        .collect::<Vec<ElementRef>>();
    let next_a = comment_pages.iter().last().unwrap();

    extract_attribute_value(*next_a, "href")
        .split('=')
        .last()
        .unwrap()
        .parse::<usize>()
        .unwrap()
}
