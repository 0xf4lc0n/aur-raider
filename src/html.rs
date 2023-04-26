use scraper::ElementRef;

pub fn delete_tags(mut html: String) -> String {
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

pub fn extract_attribute_value(el: ElementRef, attr_name: &str) -> String {
    el.value()
        .attr(attr_name)
        .map_or("".to_string(), |s| s.to_string())
}
