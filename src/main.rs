use std::{
    collections::{HashMap, HashSet, VecDeque},
    env::args,
};

mod http;
mod login;
mod parse;
use crate::{
    http::{get, UpdateCookies},
    login::login,
    parse::{body, code, get_header, internal_url, scrape, DropUntilFirstOccurrence},
};

const BASE_URL: &str = "https://fakebook.3700.network/fakebook/";
const LOGIN_URL: &str = "https://fakebook.3700.network/accounts/login/?next=/fakebook/";
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

    let mut cookies: HashMap<String, String> = HashMap::new();
    let mut visited_links: HashSet<String> = HashSet::new();
    let mut collected_flags: HashSet<String> = HashSet::new();
    let mut link_queue: VecDeque<String> = VecDeque::new();

    // login to the server
    let login_res = login(LOGIN_URL, &username, &password, &mut cookies).unwrap();
    assert_eq!(code(&login_res).0, 302);

    link_queue.push_front(BASE_URL.to_owned());

    // begin the process of web scraping
    while let Some(cur) = link_queue.pop_front() {
        if collected_flags.len() >= 5 {
            break;
        }
        if !visited_links.insert(cur.clone()) {
            continue;
        }
        println!("{}:{}", collected_flags.len(), cur);

        // Get the webpage, skipping this site if the request errors
        let res = match get(cur.as_str(), None, &mut cookies) {
            Ok(r) => r,
            Err(_) => continue,
        };
        cookies.update_cookies(&res);

        // Confirm valid response, if not try the next link
        let (res_code, _) = code(&res);
        match res_code {
            300..=399 => {
                if let Some(hdr) = get_header(&res, "Location").into_iter().next() {
                    if let Some(url) = internal_url(&cur, &hdr.drop_to_fst_occ(" ")).unwrap_or(None)
                    {
                        link_queue.push_front(url);
                    }
                }
            }
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
                    println!("Not-ok response: {}: {}", &res_code, &res)
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
        println!("Done! Visited links: {:#?}\n\nFLAGS:", visited_links);
    }
    for flag in collected_flags {
        println!("{}", flag);
    }
}
