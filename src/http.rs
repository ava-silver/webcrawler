use crate::parse::{get_header, DropUntilFirstOccurrence};
use core::panic;
use native_tls::{TlsConnector, TlsStream};
use std::{
    collections::HashMap,
    io::{Read, Result, Write},
    net::TcpStream,
};
use url::Url;

pub struct HttpClient {
    url: Url,
    stream: TlsStream<TcpStream>,
    cookies: HashMap<String, String>,
}

impl HttpClient {
    /**
     * Creates a new HttpClient which can send requests to the server at the given URL.
     */
    pub fn new(addr: &str) -> HttpClient {
        let url = Url::parse(&addr).unwrap();
        let hostname = url.host_str().unwrap().to_owned();
        let host = format!("{}:{}", hostname, url.port_or_known_default().unwrap());

        let connector = TlsConnector::new().unwrap();
        let stream = TcpStream::connect(&host).unwrap();
        let stream = connector.connect(&hostname, stream).unwrap();

        HttpClient {
            url,
            stream,
            cookies: HashMap::new(),
        }
    }

    /**
     * Reconnects the client to the server, shutting down the previous connection to create a new one.
     */
    pub fn reconnect(&mut self) {
        self.stream.shutdown().unwrap_or_else(|e| {
            println!("Shutdown failed: {}", e.to_string());
            panic!("Shutdown Failed");
        });
        let hostname = self.url.host_str().unwrap().to_owned();
        let host = format!("{}:{}", hostname, self.url.port_or_known_default().unwrap());

        let connector = TlsConnector::new().unwrap();
        let stream = TcpStream::connect(&host).unwrap();
        self.stream = connector.connect(&hostname, stream).unwrap();
    }

    /**
     * Performs a HTTP request at `url` with the given method, extra headers, and data.
     * By default, performs the request on http 1.1 with TLS,
     * and adds the following headers: Host, Connection (close), and User-Agent.
     */
    fn fetch(
        &mut self,
        url: &str,
        method: &str,
        headers: Option<Vec<String>>,
        data: Option<String>,
    ) -> Result<String> {
        let u = Url::parse(url).unwrap();
        // create initial headers
        let mut h = vec![
            format!("Host: {}", self.url.host_str().unwrap()),
            String::from("Connection: keep-alive"),
            String::from("User-Agent: AvaSilverWebScraper/1.0"),
        ];

        if let Some(mut hdrs) = headers {
            h.append(&mut hdrs);
        }

        self.append_cookies(&mut h);

        let request = format!(
            "{} {} HTTP/1.1\r\n{}\r\n\r\n{}",
            method,
            u.path(),
            h.join("\r\n"),
            data.unwrap_or("".to_owned())
        );
        self.stream.write_all(request.as_bytes())?;

        let res = self.read_message()?;

        self.update_cookies(&res);
        Ok(res)
    }

    /**
     * Reads a response from the server, only reading enough to get the message, for keep-alive functionality.
     */
    fn read_message(&mut self) -> Result<String> {
        let mut res = String::new();

        while !res.ends_with("\r\n\r\n") {
            let mut buf = [0; 1];
            self.stream.read_exact(&mut buf)?;
            res.push(char::from(buf[0]));
        }

        let l = get_header(&res, "Content-Length")
            .into_iter()
            .next()
            .unwrap()
            .drop_to_fst_occ(" ")
            .parse::<usize>()
            .unwrap();

        let mut buf = vec![0u8; l];
        self.stream.read_exact(&mut buf)?;
        res.push_str(String::from_utf8(buf).unwrap().as_str());
        Ok(res)
    }

    /**
     * Performs a get request to the specified url with the optional additional headers.
     */
    pub fn get(&mut self, url: &str, headers: Option<Vec<String>>) -> Result<String> {
        self.fetch(url, "GET", headers, None)
    }

    /**
     * Performs a post request to the specified url with the optional additional headers and data.
     */
    pub fn post(
        &mut self,
        url: &str,
        headers: Option<Vec<String>>,
        data: String,
    ) -> Result<String> {
        let adtl_hdrs = vec![
            format!("Referer: {}", url),
            format!("Content-Length: {}", data.len()),
        ];
        let hdrs = match headers {
            Some(h) => Some(h.into_iter().chain(adtl_hdrs.into_iter()).collect()),
            None => Some(adtl_hdrs),
        };

        self.fetch(url, "POST", hdrs, Some(data))
    }

    /**
     * Updates the cookies based on the Set-Cookie headers in the given response from the server
     */
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
                self.cookies
                    .insert(cookie.0.to_owned(), cookie.1.to_owned());
            }
        }
        cookies.len()
    }

    /**
     * Appends the client's cookies to the given list of headers.
     */
    fn append_cookies(&self, headers: &mut Vec<String>) {
        let cookie = self
            .cookies
            .iter()
            .map(|(a, b)| format!("{}={}", a, b))
            .collect::<Vec<String>>()
            .join("; ");
        headers.push(format!("Cookie: {}", cookie));
    }
}
