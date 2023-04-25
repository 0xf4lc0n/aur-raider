use std::collections::HashMap;

use crate::{
    models::{AdditionalPackageData, BasicPackageData, Comment, PackageData, PackageDependency},
    selectors::*,
};
use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use scraper::{ElementRef, Html};
use tokio::time::Instant;
use tracing::{debug, info};

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
                            package_basic_info.push(extract_attribute_value(a, "href"));
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

                    let response = self.http_client.get(&url_to_details).send().await?;
                    let body = response.text().await?;
                    let package_details_html = Html::parse_document(&body);

                    let additional_package_data = self
                        .get_additional_package_data(&package_details_html)
                        .await
                        .with_context(|| {
                            format!("Failed to scrap additional data for {} package", url)
                        })?;

                    let dependencies = self
                        .get_dependencies(&package_details_html)
                        .await
                        .with_context(|| {
                            format!("Failed to scrap dependencies for {} package", url)
                        })?;

                    let comments = self
                        .get_comments(&url_to_details)
                        .await
                        .with_context(|| format!("Failed to scrap comments for {} package", url))?;

                    packages_data.push(PackageData {
                        basic: basic_package_data,
                        additional: additional_package_data,
                        dependencies,
                        comments,
                    });

                    info!("Scrapped: {}", url_to_details);
                }
            }
        }

        Ok(packages_data)
    }

    async fn get_additional_package_data(
        &self,
        html_content: &Html,
    ) -> Result<AdditionalPackageData> {
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

    async fn get_dependencies(&self, html_content: &Html) -> Result<Vec<PackageDependency>> {
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

    pub async fn get_comments(&self, package_url: &str) -> Result<Vec<Comment>> {
        let response = self.http_client.get(package_url).send().await?;
        let body = response.text().await?;
        let html_content = Html::parse_document(&body);
        let mut comments = vec![];

        let last_comment_page_number = self.get_last_comment_page_number(html_content);

        let mut set = tokio::task::JoinSet::new();

        for idx in (0..=last_comment_page_number).step_by(10) {
            let comment_page_url = format!("{}?O={}", package_url, idx);

            set.spawn(Self::get_comments_from_page(
                self.http_client.clone(),
                comment_page_url,
            ));
        }

        while let Some(task_res) = set.join_next().await {
            let cmmnts = task_res.unwrap()?;
            comments.extend(cmmnts);
        }

        Ok(comments)
    }

    fn get_last_comment_page_number(&self, html_content: Html) -> usize {
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

    pub async fn get_comments_from_page(
        http_client: Client,
        page_url: String,
    ) -> Result<Vec<Comment>> {
        let start = Instant::now();
        let response = http_client.get(&page_url).send().await?;
        let body = response.text().await?;
        let html_content = Html::parse_document(&body);
        let duration = start.elapsed();
        debug!("{} -> {:?}", page_url, duration);

        let mut comments = vec![];

        for comments_container in html_content.select(&DIV_COMMENTS_SELECTOR) {
            for (comment_header, comment_content) in comments_container
                .select(&H4_COMMENT_HEADER_SELECTOR)
                .zip(comments_container.select(&DIV_COMMENT_CONTENT_SELECTOR))
                // Skip pinned comment
                .skip(1)
            {
                let header = Self::delete_tags(comment_header.inner_html());
                let content = Self::delete_tags(comment_content.inner_html());

                comments.push(Comment { header, content })
            }
        }

        Ok(comments)
    }

    fn delete_tags(mut html: String) -> String {
        html = html.lines().map(|l| l.trim()).collect();

        while let Some(s_idx) = html.find('<') {
            let e_idx = html[s_idx..].find('>').unwrap_or(html.len() - 1);
            html = html
                .chars()
                .take(s_idx)
                .filter(|c| c.is_ascii())
                .chain(html.chars().skip(s_idx + e_idx + 1))
                .collect::<String>();
        }

        html
    }
}

fn extract_attribute_value(el: ElementRef, attr_name: &str) -> String {
    el.value()
        .attr(attr_name)
        .map_or("".to_string(), |s| s.to_string())
}
