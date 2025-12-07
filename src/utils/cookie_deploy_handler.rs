// use cookie::Cookie;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use time;

pub async fn deploy_auth_cookie(cookies: Cookies, cookie_value: String) {
    // let cookie = Cookie::build(("name", "value"))
    //     .domain("www.rustychat.com")
    //     .path("/")
    //     .secure(true)
    //     .http_only(true);
    //
    // jar.add(cookie);
    // // jar.remove(Cookie::build("name").path("/"));

    // Create a basic cookie
    let mut cookie = Cookie::new("rusty_chat_auth_cookie", cookie_value);

    // Set cookie attributes for security
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_secure(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);

    // Optional: set expiration
    cookie.set_max_age(time::Duration::hours(24));

    cookies.add(cookie);
}