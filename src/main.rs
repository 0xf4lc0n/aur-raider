use scraper::{Html, Selector};

static AUR_BASE_URL: &str = "https://aur.archlinux.org/packages";

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();

    let response = client
        .get(AUR_BASE_URL)
        .send()
        .await
        .expect("Cannot execute a GET request to the AUR repository");

    let body = response.text().await.unwrap();

    let html_content = Html::parse_document(&body);

    let table_selector = Selector::parse("table.results").unwrap();
    let tbody_selector = Selector::parse("tbody").unwrap();
    let tr_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let a_selector = Selector::parse("a").unwrap();

    let mut packages_data = vec![];

    for table in html_content.select(&table_selector) {
        for tbody in table.select(&tbody_selector) {
            for tr in tbody.select(&tr_selector) {
                let mut package_basic_info = vec![];
                for td in tr.select(&td_selector) {
                    if let Some(a) = td.select(&a_selector).next() {
                        package_basic_info.push(a.inner_html().trim().to_string());
                    } else {
                        package_basic_info.push(td.inner_html().trim().to_string());
                    }
                }
                packages_data.push(package_basic_info);
            }
        }
    }

    for package_details in packages_data {
        println!("{}", package_details[0]);
    }
}
