use std::{
    collections::{HashSet, VecDeque},
    env::args,
};

mod http;
mod login;
mod parse;
use crate::{
    http::{get, valid_response},
    login::login,
    parse::{body, code, get_header, internal_url, scrape, DropUntilFirstOccurrence},
};

// const BASE_URL: &str = "https://www.3700.network/fakebook/";
const LOGIN_URL: &str = "https://fakebook.3700.network/accounts/login/?next=/fakebook/";
const TEST: &str = "http://www.softwareqatest.com/"; //http://www.http2demo.io/";
const TEST2: &str = "https://www.google.com/"; //http://www.http2demo.io/";
const DEBUG: bool = false;
fn main() {
    // collect the arguments
    let args: Vec<String> = args().collect();
    assert_eq!(args.len(), 3);

    let username = &args[1];
    let password = &args[2];
    if DEBUG {
        println!("Username: {:?}, Password: {:?}", &username, &password);
    }

    // login to the server
    let _res = login(TEST2, &username, &password);
    return;
    // begin the process of web scraping
    let mut visited_links: HashSet<String> = HashSet::new();
    let mut collected_flags: HashSet<String> = HashSet::new();
    let mut link_queue: VecDeque<String> = VecDeque::new();
    link_queue.push_back(String::from(TEST));
    while let Some(cur) = link_queue.pop_front() {
        if collected_flags.len() >= 5 {
            break;
        }
        if !visited_links.insert(cur.clone()) {
            continue;
        }

        // Get the webpage, skipping this site if the request errors
        // let headers: Vec<String> = vec![].into_iter().map(String::from).collect();
        let res = match get(cur.as_str(), None) {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Confirm valid response, if not try the next link
        let (res_code, res_message) = code(&res);
        match res_code {
            300..=399 => match get_header(&res, "Location") {
                Some(h) => link_queue.push_front(h.drop_to_fst_occ(" ")),
                None => (),
            },
            500..=599 => {
                link_queue.push_back(cur.clone());
                visited_links.remove(&cur);
            }
            _ => (),
        };
        match res_code {
            100..=299 => (),
            _ => {
                if DEBUG {
                    println!("Not-ok response: {}: {}", &res_code, &res_message)
                }
                continue;
            }
        };

        // Scrape the page and add the valid links and keys
        let html = body(&res);
        let (links, flags) = scrape(html);
        link_queue.extend(
            links
                .into_iter()
                .filter_map(|href| internal_url(&cur, &href).unwrap_or(None)),
        );
        collected_flags.extend(flags);

        if DEBUG {
            println!("Visited links so far: {:#?}\n", visited_links);
            std::io::stdin()
                .read_line(&mut String::new())
                .ok()
                .expect("Failed to read line");
        }
    }
    if DEBUG {
        println!("Visited links: {:#?}\n\nFLAGS:", visited_links);
    }
    for flag in collected_flags {
        println!("{}", flag);
    }
}
