use lazy_static::lazy_static;
use scraper::Selector;

lazy_static! {
    pub static ref TABLE_RESULT_SELECTOR: Selector = Selector::parse("table.results").unwrap();
    pub static ref TABLE_PKGINFO_SELECTOR: Selector = Selector::parse("table#pkginfo").unwrap();
    pub static ref TBODY_SELECTOR: Selector = Selector::parse("tbody").unwrap();
    pub static ref TR_SELECTOR: Selector = Selector::parse("tr").unwrap();
    pub static ref TD_SELECTOR: Selector = Selector::parse("td").unwrap();
    pub static ref TH_SELECTOR: Selector = Selector::parse("th").unwrap();
    pub static ref A_SELECTOR: Selector = Selector::parse("a").unwrap();
    pub static ref UL_DEPS_SELECTOR: Selector = Selector::parse("ul#pkgdepslist").unwrap();
    pub static ref LI_SELECTOR: Selector = Selector::parse("li").unwrap();
    pub static ref EM_SELECTOR: Selector = Selector::parse("em").unwrap();
}
