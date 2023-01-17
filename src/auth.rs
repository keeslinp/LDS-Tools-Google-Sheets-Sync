use crate::urls::*;
use anyhow::Result;
use headless_chrome::{Browser, LaunchOptions};

pub fn get_token(username: impl AsRef<str>, password: impl AsRef<str>) -> Result<String> {
    let browser = Browser::new(LaunchOptions::default_builder().headless(true).build()?)?;
    let tab = browser.wait_for_initial_tab()?;
    tab.navigate_to(SIGN_IN_URL)?;
    tab.wait_until_navigated()?;
    tab.wait_for_element("input[name=\"username\"]")?
        .type_into(username.as_ref())?;
    tab.wait_for_element("input[type=\"submit\"]")?.click()?;
    tab.wait_for_element("input[name=\"password\"]")?
        .type_into(password.as_ref())?;
    tab.wait_for_element("input[type=\"submit\"]")?.click()?;
    tab.wait_until_navigated()?;
    tab.navigate_to(MEMBER_LIST_URL)?;
    tab.wait_until_navigated()?;
    let auth_token = tab
        .get_cookies()?
        .iter()
        .find(|cookie| cookie.name == AUTH_COOKIE)
        .expect("No Auth cookie found")
        .value
        .clone();

    // Leaving commented because for some reason this fails in headless mode.
    // tab.close_target()?;

    Ok(auth_token)
}
