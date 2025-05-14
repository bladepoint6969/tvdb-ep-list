use std::{error::Error, fmt::Display};

use async_recursion::async_recursion;
use reqwest::{
    Client, ClientBuilder, Response, StatusCode, Url,
    header::{self, HeaderMap},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

const BASE_PATH: &str = "https://api.thetvdb.com";
const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug, Deserialize)]
struct LoginResponse {
    token: String,
}

#[derive(Debug, Serialize)]
struct LoginRequest {
    apikey: String,
}

#[derive(Debug)]
pub enum ClientError {
    InvalidAPIKey,
    HTTPError(StatusCode),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAPIKey => write!(f, "Invalid API Key"),
            Self::HTTPError(status) => write!(f, "Response Code {status}"),
        }
    }
}

impl Error for ClientError {}

#[derive(Debug, Deserialize)]
struct SeriesSearch {
    data: Vec<Series>,
}

#[derive(Debug, Deserialize)]
pub struct Series {
    pub id: u64,
    #[serde(rename = "seriesName")]
    pub series_name: String,
}

#[derive(Debug, Deserialize)]
struct SeriesDetailResponse {
    data: SeriesDetail,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesDetail {
    pub id: u64,
    pub series_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpisodeResponse {
    data: Vec<Episode>,
    links: Links,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Links {
    next: Option<u64>,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    pub aired_season: i64,
    pub aired_episode_number: i64,
    pub dvd_season: Option<i64>,
    pub dvd_episode_number: Option<i64>,
    pub episode_name: Option<String>,
}

pub struct Api {
    client: Client,
    default_headers: HeaderMap,
}

impl Api {
    fn url<T: Display>(path: T) -> Result<Url, url::ParseError> {
        Url::parse(&format!("{BASE_PATH}{path}"))
    }

    pub async fn new(api_key: &str) -> Result<Self, Box<dyn Error>> {
        let login_body = LoginRequest {
            apikey: api_key.into(),
        };

        let client = ClientBuilder::new().user_agent(USER_AGENT).build()?;

        let login_resp = client
            .post(Self::url("/login")?)
            .json(&login_body)
            .send()
            .await?;

        if login_resp.status() == StatusCode::OK {
            let token_holder: LoginResponse = login_resp.json().await?;
            let mut default_headers = HeaderMap::new();
            default_headers.insert(
                header::AUTHORIZATION,
                format!("Bearer {}", token_holder.token).parse()?,
            );

            Ok(Self {
                client,
                default_headers,
            })
        } else {
            Err(ClientError::InvalidAPIKey.into())
        }
    }

    pub async fn search_series(
        &self,
        name: Option<&str>,
        imdb_id: Option<&str>,
        zap2it_id: Option<&str>,
        slug: Option<&str>,
        accept_language: Option<&str>,
    ) -> Result<Vec<Series>, Box<dyn Error>> {
        let mut headers = self.default_headers.clone();
        let mut queries: Vec<(&str, &str)> = vec![];

        if let Some(name) = name {
            queries.push(("name", name));
        }
        if let Some(imdb_id) = imdb_id {
            queries.push(("imdbId", imdb_id));
        }
        if let Some(zap2it_id) = zap2it_id {
            queries.push(("zap2itId", zap2it_id));
        }
        if let Some(slug) = slug {
            queries.push(("slug", slug));
        }

        if let Some(accept_language) = accept_language {
            headers.insert(header::ACCEPT_LANGUAGE, accept_language.parse()?);
        }

        let response = self
            .client
            .get(Api::url("/search/series")?)
            .headers(headers)
            .query(&queries)
            .send()
            .await?;

        Ok(self.handle_response::<SeriesSearch>(response).await?.data)
    }

    pub async fn get_series(
        &self,
        series_id: u64,
        accept_language: Option<&str>,
    ) -> Result<SeriesDetail, Box<dyn Error>> {
        let mut headers = self.default_headers.clone();

        if let Some(accept_language) = accept_language {
            headers.insert(header::ACCEPT_LANGUAGE, accept_language.parse()?);
        }

        let response = self
            .client
            .get(Api::url(format!("/series/{series_id}"))?)
            .headers(headers)
            .send()
            .await?;

        Ok(self
            .handle_response::<SeriesDetailResponse>(response)
            .await?
            .data)
    }

    pub async fn get_series_episodes(
        &self,
        series_id: u64,
    ) -> Result<Vec<Episode>, Box<dyn Error>> {
        self.get_eps_internal(series_id, 1).await
    }

    #[async_recursion]
    async fn get_eps_internal(
        &self,
        series_id: u64,
        page: u64,
    ) -> Result<Vec<Episode>, Box<dyn Error>> {
        let headers = self.default_headers.clone();

        let response = self
            .client
            .get(Api::url(format!("/series/{series_id}/episodes"))?)
            .headers(headers)
            .query(&[("page", page)])
            .send()
            .await?;

        let resp_body = self.handle_response::<EpisodeResponse>(response).await?;
        let mut episode_list = resp_body.data;

        if let Some(next) = resp_body.links.next {
            episode_list.append(&mut self.get_eps_internal(series_id, next).await?);
        }
        Ok(episode_list)
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        resp: Response,
    ) -> Result<T, Box<dyn Error>> {
        if resp.status() == StatusCode::OK {
            Ok(resp.json().await?)
        } else {
            Err(ClientError::HTTPError(resp.status()).into())
        }
    }
}
