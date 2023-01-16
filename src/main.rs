use anyhow::{anyhow, Result};
use lds_api::LDSApi;

mod auth;
mod lds_api;
mod sheets;
mod urls;

#[tokio::main]
async fn main() -> Result<()> {
    let auth_token = auth::get_token(
        std::env::var("LDS_USERNAME").expect("Need LDS_USERNAME env"),
        std::env::var("LDS_PASSWORD").expect("Need LDS_PASSWORD env"),
    )
    .await?;
    let client = LDSApi::new(auth_token)?;
    let sheets_api = sheets::SheetsApi::new().await?;
    let spread_sheet = sheets_api
        .get_spread_sheet(std::env::var("DOCUMENT_ID").expect("Need DOCUMENT_ID env"))
        .await?;
    let sheet = std::env::var("SHEET_NAME")
        .map_err(|e| anyhow!(e))
        .and_then(|name| {
            spread_sheet
                .sheet_by_title(name)
                .ok_or(anyhow!("No sheet by that title"))
        })
        .unwrap_or_else(|_| spread_sheet.sheet_at(0));
    sheet.clear("A1:A999").await?;
    let members = client.get_member_list().await?;
    sheet
        .append(
            "A1",
            members
                .iter()
                .map(|member| vec![member.given_name(), member.family_name(), member.uuid()])
                .collect::<Vec<Vec<&str>>>(),
        )
        .await?;

    Ok(())
}
