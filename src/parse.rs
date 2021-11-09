use select::{document::Document, predicate::{Class, Name}};

/**
 * Scrapes the html for any links and flags.
 */
pub fn scrape(html: String) -> (Vec<String>, Vec<String>) {
    let document = Document::from(html.as_str());
    let links = document
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .map(String::from).collect::<Vec<String>>();
    let flags = document
        .find(Class("blue-btn"))
        // .find(Class("secret_flag"))
        // .filter_map(|n| n.as_text())
        // .map(|flag| flag.split("FLAG: ").next().unwrap())
        .map(|n| format!("{:#?}", n.as_text()))
        .map(String::from)
        .collect::<Vec<String>>();

    (links, flags)
}