use std::env;
mod http;
mod login;

const BASE_URL: &str = "http://www.3700.network/fakebook/";
const LOGIN_URL: &str = "http://www.3700.network/accounts/login/?next=/fakebook/";

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 3);

    let username = &args[1];
    let password = &args[2];

    login::login()
    
    println!("{:?}, {:?}", &username, &password);
    
}
