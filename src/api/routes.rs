use crate::{
    api::forms::RepositoryResponse,
    client::HttpClient,
    Result,
};
use actix_web::{
    get,
    web::{self, Json},
};
use tracing::instrument;

#[instrument(skip(client))]
#[get("/{owner}/{repo}")]
pub async fn get_repo_info(
    path: web::Path<(String, String)>,
    client: web::Data<HttpClient>,
) -> Result<Json<RepositoryResponse>> {
    let (org, repo) = path.into_inner();
    let response = client.fetch(&org, &repo).await?;

    Ok(Json(response))
}
