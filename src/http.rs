use std::{
    collections::HashMap,
    io::{Read, Result, Write},
    net::{TcpStream, ToSocketAddrs},
};

use crate::{
    parse::{get_header, DropUntilFirstOccurrence},
    DEBUG,
};
use native_tls::TlsConnector;

use url::Url;

/**
 * Performs a HTTP request at `url` with the given method, extra headers, and data.
 * By default, performs the request on http 1.1 with TLS,
 * and adds the following headers: Host, Connection (close), and User-Agent.
 */
fn fetch(
    url: &str,
    method: &str,
    headers: Option<Vec<String>>,
    data: Option<String>,
    cookies: &HashMap<String, String>,
) -> Result<String> {
    let u = Url::parse(&url).unwrap();
    let hostname = u.host_str().unwrap();
    let host = format!("{}:{}", hostname, u.port_or_known_default().unwrap());

    if DEBUG {
        println!(
            "hostname: {}; host ip: {:#?}",
            hostname,
            host.to_socket_addrs().unwrap().into_iter().next().unwrap()
        );
    }

    // create initial headers
    let mut h = vec![
        format!("Host: {}", hostname),
        String::from("Connection: close"), //TODO this is the issue? may have to make full cookie manager like seen in devtools
        String::from("User-Agent: AvaSilverWebScraper/1.0"),
    ];

    if let Some(mut hdrs) = headers {
        h.append(&mut hdrs);
    }

    h.append_cookies(cookies);

    let request = format!(
        "{} {} HTTP/1.1\r\n{}\r\n\r\n{}",
        &method,
        u.path(),
        h.join("\r\n"),
        data.unwrap_or("".to_owned())
    );
    if DEBUG {
        println!("request: \n{}", request);
    }
    let connector = TlsConnector::new().unwrap();
    let stream = TcpStream::connect(host)?;
    let mut stream = connector.connect(hostname, stream).unwrap();

    stream.write_all(request.as_bytes()).unwrap();
    let mut res = vec![];
    stream.read_to_end(&mut res).unwrap();
    Ok(String::from_utf8_lossy(&res).to_string())
}

/**
 * Performs a get request to the specified url with the optional additional headers.
 */
pub fn get(
    url: &str,
    headers: Option<Vec<String>>,
    cookies: &HashMap<String, String>,
) -> Result<String> {
    fetch(url, "GET", headers, None, cookies)
}

pub fn post(
    url: &str,
    headers: Option<Vec<String>>,
    data: String,
    cookies: &HashMap<String, String>,
) -> Result<String> {
    fetch(url, "POST", headers, Some(data), cookies)
}

pub trait UpdateCookies {
    /**
     * Updates the cookies based on the Set-Cookie headers in the given response from the server
     */
    fn update_cookies(&mut self, response: &String) -> usize;
}

impl UpdateCookies for HashMap<String, String> {
    fn update_cookies(&mut self, response: &String) -> usize {
        let cookies = get_header(response, "Set-Cookie")
            .into_iter()
            .map(|c| {
                let cookie = c.drop_to_fst_occ(" ");
                let mut s = cookie.split(|c| c == '=' || c == ';');
                let name = s.next().unwrap_or("");
                let val = s.next().unwrap_or("");
                (name.to_owned(), val.to_owned())
            })
            .collect::<Vec<(String, String)>>();
        for cookie in &cookies {
            if cookie.0.len() > 0 {
                self.insert(cookie.0.to_owned(), cookie.1.to_owned());
            }
        }
        cookies.len()
    }
}

pub trait AppendCookies {
    fn append_cookies(&mut self, cookies: &HashMap<String, String>);
}

impl AppendCookies for Vec<String> {
    fn append_cookies(&mut self, cookies: &HashMap<String, String>) {
        let cookie = cookies
            .into_iter()
            .map(|(a, b)| format!("{}={}", a, b))
            .collect::<Vec<String>>()
            .join("; ");
        self.push(format!("Cookie: {}", cookie));
    }
}
