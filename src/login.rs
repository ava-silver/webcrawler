use crate::{http::HttpClient, parse::get_csrf_middleware_token};

/**
 * Logs into Fakebook, returning the response, or Errors on any errors.
 */
pub fn login(
    url: &str,
    username: &str,
    password: &str,
    client: &mut HttpClient,
) -> Result<String, ()> {
    let res = client.get(url, None).or(Err(()))?;

    let csrf_m_tok = get_csrf_middleware_token(&res).ok_or(())?;
    let data = format!(
        "username={}&password={}&csrfmiddlewaretoken={}&next=%2Ffakebook%2F",
        username, password, csrf_m_tok
    );

    let mut hdrs: Vec<String> = vec![
        "Origin: https://fakebook.3700.network",
        "Content-Type: application/x-www-form-urlencoded",
        "Accept: text/html",
        "Accept-Language: en-US,en;q=0.9",
        "Cache-Control: max-age=0",
        "Sec-GPC: 1",
        "Sec-Fetch-Site: same-origin",
        "Sec-Fetch-Mode: navigate",
        "Sec-Fetch-User: ?1",
        "Sec-Fetch-Dest: document",
        "Accept-Language: en-US,en;q=0.9",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    hdrs.push(format!("X-CSRFToken: {}", csrf_m_tok));

    let res = client.post(url, Some(hdrs), data).or(Err(()))?;
    Ok(res)
}
