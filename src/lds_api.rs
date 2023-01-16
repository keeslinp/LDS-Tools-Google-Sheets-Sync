use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::Deserialize;

use crate::urls::{AUTH_COOKIE, MEMBER_LIST_URL};
fn default_headers(token: impl AsRef<str>) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        HeaderValue::from_str(format!("{}={}", AUTH_COOKIE, token.as_ref()).as_str())?,
    );
    Ok(headers)
}

#[derive(Deserialize)]
struct NameFormats {
    #[serde(rename = "givenPreferredLocal")]
    given_preferred_local: String,
    #[serde(rename = "familyPreferredLocal")]
    family_preferred_local: String,
}

#[derive(Deserialize)]
pub struct Member {
    #[serde(rename = "nameFormats")]
    name_formats: NameFormats,

    uuid: String,
}

impl Member {
    pub fn given_name<'a>(&'a self) -> &'a str {
        self.name_formats.given_preferred_local.as_str()
    }
    pub fn family_name<'a>(&'a self) -> &'a str {
        self.name_formats.family_preferred_local.as_str()
    }
    pub fn uuid<'a>(&'a self) -> &'a str {
        &self.uuid
    }
}

pub struct LDSApi {
    client: reqwest::Client,
}

impl LDSApi {
    pub fn new(auth_token: impl AsRef<str>) -> Result<LDSApi> {
        Ok(LDSApi {
            client: reqwest::Client::builder()
                .default_headers(default_headers(&auth_token)?)
                .build()?,
        })
    }

    pub async fn get_member_list(&self) -> Result<Vec<Member>> {
        Ok(serde_json::from_str::<Vec<Member>>(
            self.client
                .get(MEMBER_LIST_URL)
                .send()
                .await?
                .text()
                .await?
                .as_str(),
        )?)
    }
}
