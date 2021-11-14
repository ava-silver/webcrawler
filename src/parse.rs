use select::{
    document::Document,
    predicate::{And, Attr, Class, Name},
};
use url::{ParseError, Url};

pub trait DropUntilFirstOccurrence {
    /**
     * Returns a string that contains the current up until
     * the pattern `s`, not including `s`.
     */
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
    let mut code_msg = header.split(" ").into_iter().skip(1);
    (
        code_msg.next().unwrap().parse().unwrap(),
        String::from(code_msg.collect::<Vec<&str>>().join(" ")),
    )
}

/**
 * Collects all the headers (excluding the inital line) into a vector.
 */
pub fn headers(res: &String) -> Vec<String> {
    res.split("\r\n\r\n")
        .next()
        .unwrap()
        .split("\r\n")
        .skip(1)
        .map(String::from)
        .collect::<Vec<String>>()
}

/**
 * Finds the header which matches the name `hdr` in the headers of `res`,
 * performing case-insensitive match for header name.
 */
pub fn get_header(res: &String, hdr: &str) -> Vec<String> {
    headers(res)
        .into_iter()
        .filter(|h| match h.split(":").next() {
            Some(s) => s.eq_ignore_ascii_case(hdr),
            None => false,
        })
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

/**
 * Finds the csrf middleware token contained in the html body of the response,
 * or None if there is none.
 */
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

    #[test]
    fn test_get_header() {
        let res = "HTTP/1.1 200 OK\r\nContent-Length: 1750\r\nConnection: keep-alive\r\nSet-Cookie: csrftoken=zDBlSPOhReFdO0AcXi66edtGEwazJQHL9q4owxc0fREhY2AtOZMCUqxmXzxEfS6i; expires=Sun, 13 Nov 2022 17:01:55 GMT; Max-Age=31449600; Path=/; SameSite=Lax".to_owned();
        let mut hdrs: Vec<String> = Vec::new();
        assert_eq!(hdrs, get_header(&res, "nothing"));
        hdrs.push("Content-Length: 1750".to_owned());
        assert_eq!(hdrs, get_header(&res, "content-length"));
        assert_eq!(hdrs, get_header(&res, "Content-Length"));
    }

    #[test]
    fn test_code() {
        let res = "HTTP/1.1 200 OK\r\nContent-Length: 1750\r\nConnection: keep-alive\r\nSet-Cookie: csrftoken=zDBlSPOhReFdO0AcXi66edtGEwazJQHL9q4owxc0fREhY2AtOZMCUqxmXzxEfS6i; expires=Sun, 13 Nov 2022 17:01:55 GMT; Max-Age=31449600; Path=/; SameSite=Lax".to_owned();
        assert_eq!((200, "OK".to_owned()), code(&res));
        let res2 = "HTTP/1.1 404 Not Found\r\n".to_owned();
        assert_eq!((404, "Not Found".to_owned()), code(&res2));
    }

    #[test]
    fn test_headers() {
        let res = "HTTP/1.1 200 OK\r\nContent-Length: 1750\r\nConnection: keep-alive\r\nSet-Cookie: csrftoken=zDBlSPOhReFdO0AcXi66edtGEwazJQHL9q4owxc0fREhY2AtOZMCUqxmXzxEfS6i; expires=Sun, 13 Nov 2022 17:01:55 GMT; Max-Age=31449600; Path=/; SameSite=Lax\r\n\r\n".to_owned();
        assert_eq!(vec![
            "Content-Length: 1750".to_owned(),
            "Connection: keep-alive".to_owned(),
            "Set-Cookie: csrftoken=zDBlSPOhReFdO0AcXi66edtGEwazJQHL9q4owxc0fREhY2AtOZMCUqxmXzxEfS6i; expires=Sun, 13 Nov 2022 17:01:55 GMT; Max-Age=31449600; Path=/; SameSite=Lax".to_owned()
        ], headers(&res));
    }

    #[test]
    fn test_drop() {
        assert_eq!(
            "kjahsdkjash dkjaskjdh asdh kjashd"
                .to_owned()
                .drop_to_fst_occ(" "),
            "dkjaskjdh asdh kjashd".to_owned()
        );
        assert_eq!(
            "Content-Length: 1750".to_owned().drop_to_fst_occ(" "),
            "1750".to_owned()
        );
        assert_eq!(
            "Content-Length: 1750".to_owned().drop_to_fst_occ("8"),
            "".to_owned()
        );
    }
}
