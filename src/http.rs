use std::{io::{Read, Result, Write}, net::{TcpStream, ToSocketAddrs}};

use url::Url;
const LOG: bool = true;

/**
 * Performs a get request to the specified url with the optional additional headers.
 */
pub fn get(url: &str, headers: Option<Vec<String>>) -> Result<String> {
    let u = Url::parse(&url).unwrap();
    let hostname = u.host_str().unwrap();
    let host = format!("{}:{}", hostname, u.port_or_known_default().unwrap());
    if LOG { println!("hostname: {}; host ip: {:#?}", hostname, host.to_socket_addrs().unwrap().into_iter().next().unwrap()); }

    let mut h = vec![
        format!("Host: {}", hostname),
        String::from("Connection: close"),
        String::from("User-Agent: GetURL11/1.0")
    ];

    match headers {
        Some(mut hdrs) => h.append(&mut hdrs),
        None => (),
    }

    let request = format!("GET {} HTTP/1.1\r\n{}\r\n\r\n", u.path(), h.join("\r\n"));
    if LOG { println!("request: \n{}", request); }
    
    let mut stream = TcpStream::connect(host)?;
    stream.write_all(&request.as_bytes())?;
    let mut res = String::new();
    let size = stream.read_to_string(&mut res)?;
    assert!(size > 0);
    Ok(res)
}

/**
 * Parses the response code (and message) from a given http response.
 */
pub fn code(res: &String) -> (u32, String) {
    let header = res.split("\r\n").into_iter().next().unwrap();
    let code_msg: Vec<&str> = header.split(" ").into_iter().skip(1).take(2).collect();
    (code_msg[0].parse().unwrap(), String::from(code_msg[1]))
}   

/**
 * Parses the body from the given http response.
 */
pub fn body(res: &String) -> String {
    let sep = "\r\n\r\n";
    res.split(sep).into_iter().skip(1).collect::<Vec<&str>>().join(sep)
}


