use crate::{
    http::get,
    parse::{body, headers},
};

pub fn login(url: &str, username: &str, password: &str) -> Result<i32, ()> {
    let res = get(&url, None).or(Err(()))?;
    let h = headers(&res);
    let b = body(&res);
    println!("{:#?}\n{:#?}", h, b);

    Ok(8)
}
