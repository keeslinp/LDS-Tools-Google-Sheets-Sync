use anyhow::Result;
use gcp_auth::AuthenticationManager;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};

pub struct SheetsApi {
    client: reqwest::Client,
}

#[derive(Deserialize, Debug)]
struct SpreadSheetProperties {
    title: String,
}

#[derive(Deserialize, Debug)]
struct SheetProperties {
    #[serde(rename = "sheetId")]
    sheet_id: usize,
    title: String,
    index: usize,
}

#[derive(Deserialize, Debug)]
struct SheetData {
    properties: SheetProperties,
}

pub struct Sheet<'a, 'b> {
    data: &'b SheetData,
    spread_sheet: &'a SpreadSheet<'a>,
}

#[derive(Deserialize, Debug)]
struct SpreadSheetData {
    properties: SpreadSheetProperties,
    sheets: Vec<SheetData>,
}

pub struct SpreadSheet<'a> {
    data: SpreadSheetData,
    id: String,
    api: &'a SheetsApi,
}

impl SpreadSheetData {
    pub fn title<'a>(&'a self) -> &'a str {
        &self.properties.title
    }
}

fn default_headers(token: impl AsRef<str>) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(format!("Bearer {}", token.as_ref()).as_str())?,
    );
    Ok(headers)
}

#[derive(Serialize)]
struct AppendBody<'a> {
    values: Vec<Vec<&'a str>>,
}

impl SheetsApi {
    pub async fn new() -> Result<SheetsApi> {
        let authentication_manager = AuthenticationManager::new().await?;
        let token = authentication_manager
            .get_token(&["https://www.googleapis.com/auth/spreadsheets"])
            .await?;
        Ok(SheetsApi {
            client: reqwest::Client::builder()
                .default_headers(default_headers(token.as_str())?)
                .build()?,
        })
    }

    pub async fn get_spread_sheet<'a>(&'a self, sheet: impl AsRef<str>) -> Result<SpreadSheet<'a>> {
        Ok(SpreadSheet {
            data: serde_json::from_str(
                self.client
                    .get(format!(
                        "https://sheets.googleapis.com/v4/spreadsheets/{}",
                        sheet.as_ref()
                    ))
                    .send()
                    .await?
                    .text()
                    .await?
                    .as_str(),
            )?,
            id: sheet.as_ref().into(),
            api: self,
        })
    }

    async fn clear(&self, document_id: impl AsRef<str>, range: impl AsRef<str>) -> Result<()> {
        self.client
            .post(
                format!(
                    "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}:clear",
                    document_id.as_ref(),
                    range.as_ref()
                )
                .as_str(),
            )
            .header("Content-length", 0)
            .send()
            .await?;
        Ok(())
    }

    async fn append(
        &self,
        document_id: &str,
        range: impl AsRef<str>,
        rows: Vec<Vec<&str>>,
    ) -> Result<()> {
        self.client
            .post(format!(
                "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}:append",
                document_id,
                range.as_ref(),
            ))
            .json(&AppendBody { values: rows })
            .query(&[("valueInputOption", "USER_ENTERED")])
            .send()
            .await?;
        Ok(())
    }
}

impl<'a> SpreadSheet<'a> {
    async fn clear(&self, range: impl AsRef<str>) -> Result<()> {
        self.api.clear(self.id.as_str(), range).await?;
        Ok(())
    }

    pub fn sheet_at<'b: 'a>(&'b self, index: usize) -> Sheet<'a, 'b> {
        Sheet {
            data: &self.data.sheets[index],
            spread_sheet: self,
        }
    }

    pub fn sheet_by_title<'b: 'a>(&'b self, name: impl AsRef<str>) -> Option<Sheet<'a, 'b>> {
        self.data
            .sheets
            .iter()
            .find(|sheet| sheet.properties.title == name.as_ref())
            .map(|data| Sheet {
                data,
                spread_sheet: self,
            })
    }

    async fn append(&self, range: impl AsRef<str>, rows: Vec<Vec<&str>>) -> Result<()> {
        self.api.append(self.id.as_str(), range, rows).await?;
        Ok(())
    }
}

impl<'a, 'b> Sheet<'a, 'b> {
    pub fn title(&'a self) -> &'a str {
        &self.data.properties.title
    }
    pub async fn clear(&self, range: impl AsRef<str>) -> Result<()> {
        self.spread_sheet.clear(self.range(range)).await?;
        Ok(())
    }

    fn range(&self, range: impl AsRef<str>) -> String {
        format!("{}!{}", self.title(), range.as_ref())
    }

    pub async fn append(&self, range: impl AsRef<str>, rows: Vec<Vec<&str>>) -> Result<()> {
        self.spread_sheet.append(self.range(range), rows).await?;
        Ok(())
    }
}
