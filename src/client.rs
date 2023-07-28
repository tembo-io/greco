use std::borrow::Cow;
use crate::{Result, Error, TOKEN};
use std::path::Path;

use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use tokio::try_join;

use crate::api::forms::RepositoryResponse;
use crate::pandoc::save_to_file_and_convert;

#[derive(Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
}

fn extract_extension(file_name: &str) -> Option<Cow<'_, str>> {
    Path::new(file_name)
        .extension()
        .map(|os_str| os_str.to_string_lossy())
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
        }
    }

    fn build_url(organization: &str, repository: &str, get_readme: bool) -> Result<Url> {
        let url = format!(
            "https://api.github.com/repos/{organization}/{repository}{}",
            if get_readme { "/readme" } else { "" }
        );

        Url::parse(&url).map_err(Into::into)
    }

    async fn fetch_description(&self, organization: &str, repository: &str) -> Result<String> {
        let url = Self::build_url(organization, repository, false)?;

        #[derive(Deserialize)]
        struct Response {
            description: Option<String>,
        }

        let resp: Response = self.get(url).await?;

        Ok(resp.description.unwrap_or_default())
    }

    async fn fetch_readme(&self, organization: &str, repository: &str) -> Result<String> {
        let url = Self::build_url(organization, repository, true)?;

        #[derive(Deserialize)]
        struct Response {
            name: String,
            download_url: String,
        }

        let Response { name, download_url } = self.get(url).await?;

        let readme = if let Some(extension) = extract_extension(&name) {
            match &*extension {
                "md" => {
                    // Markdown, do no extra work
                    self.get_text(&download_url).await?
                }
                other_extension => {
                    tracing::info!(
                        "Got another extension: {}",
                        other_extension
                    );

                    save_to_file_and_convert(&download_url, other_extension, &self.inner).await?
                }
            }
        } else {
            return Err(Error::NoExtensionFound);
        };

        Ok(readme)
    }

    async fn get_text(&self, url: &str) -> Result<String> {
        self.inner
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "request")
            .bearer_auth(&*TOKEN)
            .send()
            .await?
            .text()
            .await
            .map_err(Into::into)
    }

    async fn get<T: DeserializeOwned>(&self, url: Url) -> Result<T> {
        tracing::info!("Hitting {url}");
        let response = self
            .inner
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "request")
            .bearer_auth(&*TOKEN)
            .send()
            .await?;

        tracing::info!(
            "Request got a response with status code {}",
            response.status()
        );

        response.json().await.map_err(Into::into)
    }

    pub async fn fetch(&self, organization: &str, repository: &str) -> Result<RepositoryResponse> {
        let (description, readme) = try_join!(
            self.fetch_description(organization, repository),
            self.fetch_readme(organization, repository)
        )?;
        
        let resp = RepositoryResponse {
            description,
            readme,
        };

        Ok(resp)
    }
}
