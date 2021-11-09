use std::{env};

use crate::http::{body, get, code};
mod http;
mod login;
mod parse;

// const BASE_URL: &str = "http://www.3700.network/fakebook/";
// const LOGIN_URL: &str = "http://www.3700.network/accounts/login/?next=/fakebook/";
const TEST: &str = "http://www.http2demo.io/";

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 3);

    let username = &args[1];
    let password = &args[2];
    println!("{:?}, {:?}", &username, &password);
    // let headers: Vec<String> = vec![].into_iter().map(String::from).collect();
    
    let res = get(TEST, None).unwrap();
    let (res_code, res_message) = code(&res);
    println!("{}: {}", res_code, &res_message);

    let html = body(&res);
    // println!("{:#?}", &html[..400]);
    let (links, flags) = parse::scrape(html);
    println!("links: {:#?}\nflags: {:#?}", links, flags);
    // login::login()
    
}
