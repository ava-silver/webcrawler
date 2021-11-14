use select::{
    document::Document,
    predicate::{And, Attr, Class, Name},
};
use url::{ParseError, Url};

pub trait DropUntilFirstOccurrence {
    fn drop_to_fst_occ(&self, s: &str) -> String;
}

impl DropUntilFirstOccurrence for String {
    fn drop_to_fst_occ(&self, s: &str) -> String {
        self.split(s)
            .skip(1)
            .map(String::from)
            .collect::<Vec<String>>()
            .join(s)
    }
}

/**
 * Scrapes the html for any links and flags.
 */
pub fn scrape(html: String) -> (Vec<String>, Vec<String>) {
    let document = Document::from(html.as_str());
    let links = document
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .map(String::from)
        .collect::<Vec<String>>();
    let flags = document
        .find(And(Class("secret_flag"), Name("h2")))
        // .find(Class("selected"))
        .map(|n| n.text())
        .map(|flag| flag.drop_to_fst_occ(" "))
        .collect::<Vec<String>>();

    (links, flags)
}

/**
 * Parses the response code (and message) from a given http response.
 */
pub fn code(res: &String) -> (u32, String) {
    let header = res.split("\r\n").into_iter().next().unwrap();
    let code_msg: Vec<&str> = header.split(" ").into_iter().skip(1).take(2).collect();
    (code_msg[0].parse().unwrap(), String::from(code_msg[1]))
}

pub fn headers(res: &String) -> Vec<String> {
    res.split("\r\n\r\n")
        .next()
        .unwrap()
        .split("\r\n")
        .skip(1)
        .map(String::from)
        .collect::<Vec<String>>()
}

pub fn get_header(res: &String, hdr: &str) -> Vec<String> {
    headers(res)
        .into_iter()
        .filter(|h| h.split(":").next() == Some(hdr))
        .collect()
}

/**
 * Parses the body from the given http response.
 */
pub fn body(res: &String) -> String {
    res.drop_to_fst_occ("\r\n\r\n")
}

/**
 * Returns the href as a full URL based on the current one,
 * or None if it is an external URL or cannot be converted to a correct URL
 */
pub fn internal_url(cur: &String, href: &String) -> Result<Option<String>, ParseError> {
    let cur_url = Url::parse(cur.as_str())?;
    // Try to parse the URL, or as an extension of the hostname
    let mut next = Url::parse(href.as_str()).or(Url::parse(
        format!(
            "{}://{}{}",
            cur_url.scheme(),
            cur_url.host_str().unwrap(),
            href
        )
        .as_str(),
    ))?;

    if next.path().contains("logout") {
        return Ok(None);
    }
    if let Some(host) = next.host_str() {
        if cur_url.host_str() != Some(host) {
            return Ok(None);
        }
    } else {
        next.set_host(cur_url.host_str())?;
        if next.set_scheme(cur_url.scheme()).is_err() {
            return Ok(None);
        }
    }

    Ok(Some(String::from(next.as_str())))
}

pub fn get_csrf_middleware_token(res: &String) -> Option<String> {
    let b = body(res);
    let document = Document::from(b.as_str());
    let tokens = document
        .find(Attr("name", "csrfmiddlewaretoken"))
        .filter_map(|n| n.attr("value"))
        .map(String::from)
        .collect::<Vec<String>>();
    tokens.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_internal_url() {
        let cur = "https://fakebook.3700.network/fakebook/651267983/".to_owned();

        assert_eq!(
            internal_url(&cur, &"https://foo.com/fakebook/489574850/".to_owned()),
            Ok(None)
        );
        assert_eq!(
            internal_url(&cur, &"/fakebook/489574850/".to_owned()),
            Ok(Some(
                "https://fakebook.3700.network/fakebook/489574850/".to_owned()
            ))
        );
        assert_eq!(
            internal_url(&cur, &"/fakebook/489574850/friends/1/".to_owned()),
            Ok(Some(
                "https://fakebook.3700.network/fakebook/489574850/friends/1/".to_owned()
            ))
        );
    }
}
