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
    pub static ref DIV_COMMENTS_SELECTOR: Selector = Selector::parse("div.comments").unwrap();
    pub static ref H4_COMMENT_HEADER_SELECTOR: Selector =
        Selector::parse("h4.comment-header").unwrap();
    pub static ref DIV_COMMENT_CONTENT_SELECTOR: Selector =
        Selector::parse("div.article-content").unwrap();
    pub static ref P_SELECTOR: Selector = Selector::parse("p").unwrap();
    pub static ref P_COMMENT_HEADER_NAV_SELECTOR: Selector =
        Selector::parse("p.comments-header-nav").unwrap();
    pub static ref A_PAGE_SELECTOR: Selector = Selector::parse("a.page").unwrap();
}
