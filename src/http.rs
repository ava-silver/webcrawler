use std::{
    convert::TryFrom,
    io::{Read, Result},
    net::{TcpStream, ToSocketAddrs},
    str::from_utf8,
    sync::Arc,
};

use rustls::{ClientConfig, ClientConnection, OwnedTrustAnchor, RootCertStore, ServerName};
use url::Url;
use webpki_roots::TLS_SERVER_ROOTS;

use crate::DEBUG;

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

    let mut socket = TcpStream::connect(host)?;

    let mut root_store = RootCertStore::empty();
    root_store.add_server_trust_anchors(TLS_SERVER_ROOTS.0.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let rc_config = Arc::new(config);
    let server_addr = ServerName::try_from(hostname).expect("Invalid URL");
    let mut client = ClientConnection::new(rc_config, server_addr).unwrap();
    let mut buf = Vec::new();
    while buf.is_empty() {
        if client.wants_write() {
            client.write_tls(&mut socket).unwrap();
        }
        if client.wants_read() {
            client.read_tls(&mut socket).unwrap();
            client.process_new_packets().unwrap();
            client.reader().read_to_end(&mut buf).unwrap(); // should be blocking, figure out how to make it block
        }
    }

    // stream.write_all(&request.as_bytes())?;
    let res: String = from_utf8(buf.as_slice())
        .expect("error converting to utf8")
        .to_owned();

    assert!(res.len() > 0);
    Ok(res)
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
