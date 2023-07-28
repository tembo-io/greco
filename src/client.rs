use crate::{Error, Result, TOKEN, b64};
use std::borrow::Cow;
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

    fn build_description_url(organization: &str, repository: &str) -> Result<Url> {
        let url = format!("https://api.github.com/repos/{organization}/{repository}",);

        Url::parse(&url).map_err(Into::into)
    }

    fn build_readme_url(
        organization: &str,
        repository: &str,
        maybe_subdir: Option<&str>,
    ) -> Result<Url> {
        let url = if let Some(subdir) = maybe_subdir {
            format!("https://api.github.com/repos/{organization}/{repository}/readme/{subdir}",)
        } else {
            format!("https://api.github.com/repos/{organization}/{repository}/readme")
        };

        Url::parse(&url).map_err(Into::into)
    }

    async fn fetch_description(&self, organization: &str, repository: &str) -> Result<String> {
        let url = Self::build_description_url(organization, repository)?;

        #[derive(Deserialize)]
        struct Response {
            description: Option<String>,
        }

        let resp: Response = self.get(url).await?;

        Ok(resp.description.unwrap_or_default())
    }

    async fn fetch_readme(
        &self,
        organization: &str,
        repository: &str,
        subdirectory: Option<&str>,
    ) -> Result<String> {
        let url = Self::build_readme_url(organization, repository, subdirectory)?;

        #[derive(Deserialize)]
        struct Response {
            name: String,
            content: String,
        }

        let Response { name, content } = self.get(url).await?;

        let readme = if let Some(extension) = extract_extension(&name) {
            match &*extension {
                "md" => {
                    // Markdown, do no extra work
                    String::from_utf8(b64::decode(&content)?)?
                }
                other_extension => {
                    tracing::info!("Got another extension: {}", other_extension);

                    save_to_file_and_convert(&name, other_extension, &content).await?
                }
            }
        } else {
            return Err(Error::NoExtensionFound);
        };

        Ok(readme)
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

    pub async fn fetch(
        &self,
        organization: &str,
        repository: &str,
        subdirectory: Option<&str>,
    ) -> Result<RepositoryResponse> {
        let (description, readme) = try_join!(
            self.fetch_description(organization, repository),
            self.fetch_readme(organization, repository, subdirectory)
        )?;

        Ok(RepositoryResponse {
            description,
            readme,
        })
    }
}
