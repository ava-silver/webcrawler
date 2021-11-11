use std::{
    io::{Read, Result, Write},
    net::{TcpStream, ToSocketAddrs},
};

use crate::DEBUG;
use native_tls::TlsConnector;

use url::Url;

fn fetch(
    url: &str,
    method: &str,
    headers: Option<Vec<String>>,
    data: Option<String>,
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

    let mut h = vec![
        format!("Host: {}", hostname),
        String::from("Connection: close"),
        String::from("User-Agent: AvaSilverWebScraper/1.0"),
    ];

    match headers {
        Some(mut hdrs) => h.append(&mut hdrs),
        None => (),
    }

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
pub fn get(url: &str, headers: Option<Vec<String>>) -> Result<String> {
    fetch(url, "GET", headers, None)
}

pub fn post(url: &str, headers: Option<Vec<String>>, data: String) -> Result<String> {
    fetch(url, "POST", headers, Some(data))
}

pub fn valid_response(code: u32) -> bool {
    200 <= code && code < 400
}
