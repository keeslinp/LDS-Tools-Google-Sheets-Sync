use crate::urls::*;
use playwright::Playwright;

pub async fn get_token(
    username: impl AsRef<str>,
    password: impl AsRef<str>,
) -> Result<String, playwright::Error> {
    let playwright = Playwright::initialize().await?;
    playwright.prepare()?;
    let chromium = playwright.chromium();
    let browser = chromium.launcher().headless(true).launch().await?;
    let context = browser.context_builder().build().await?;
    let page = context.new_page().await?;
    page.goto_builder(SIGN_IN_URL).goto().await?;
    page.fill_builder("input[name=\"username\"]", username.as_ref())
        .fill()
        .await?;
    page.click_builder("input[type=\"submit\"]").click().await?;

    page.fill_builder("input[name=\"password\"]", password.as_ref())
        .fill()
        .await?;
    page.click_builder("input[type=\"submit\"]").click().await?;
    // For some reason it isn't waiting properly for the nav to complete, so lets intentionally wait
    page.wait_for_selector_builder("input[name=\"password\"]")
        .state(playwright::api::frame::FrameState::Detached)
        .wait_for_selector()
        .await?;
    page.goto_builder(MEMBER_LIST_URL)
        .wait_until(playwright::api::DocumentLoadState::NetworkIdle)
        .goto()
        .await?;
    let auth_token = page
        .context()
        .cookies(&[])
        .await?
        .iter()
        .find(|cookie| cookie.name == AUTH_COOKIE)
        .expect("No auth cookie found")
        .value
        .clone();
    browser.close().await?;
    return Ok(auth_token);
}
