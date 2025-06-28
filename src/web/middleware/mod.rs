pub mod auth;
pub mod res_map;

use crate::crypt::token::generate_web_token;
use tower_cookies::{Cookie, Cookies, cookie::SameSite};

use crate::web::error::ClientError;
use crate::web::error::{Error, Result};

pub const AUTH_TOKEN: &str = "auth-token";

//TODO: Set secure to true and samesite to none to enable HTTPS connection (when not running on
//localhost)

pub fn set_token_cookie(cookies: &Cookies, user: &str, salt: &str) -> Result<()> {
    let token = generate_web_token(user, salt)?;

    let mut cookie = Cookie::new(AUTH_TOKEN, token.to_string());
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookie.set_same_site(Some(SameSite::Lax));
    cookie.set_secure(false);

    cookies.add(cookie);

    Ok(())
}

pub fn remove_token_cookie(cookies: &Cookies) -> Result<()> {
    let mut cookie = Cookie::from(AUTH_TOKEN);
    cookie.set_path("/");
    cookie.set_same_site(Some(SameSite::Lax));
    cookie.set_secure(false);

    cookies.remove(cookie);

    Ok(())
}
