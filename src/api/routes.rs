use crate::{api::forms::RepositoryResponse, client::HttpClient, Result};
use actix_web::{
    get,
    web::{self, Json},
};

#[get("/{owner}/{repo}")]
pub async fn get_repo_info(
    path: web::Path<(String, String)>,
    client: web::Data<HttpClient>,
) -> Result<Json<RepositoryResponse>> {
    let (org, repo) = path.into_inner();
    let response = client.fetch(&org, &repo, None).await?;

    Ok(Json(response))
}

#[get("/{owner}/{repo}/{subdir}")]
pub async fn get_repo_info_with_subfolder(
    path: web::Path<(String, String, String)>,
    client: web::Data<HttpClient>,
) -> Result<Json<RepositoryResponse>> {
    let (org, repo, subdir) = path.into_inner();
    let response = client.fetch(&org, &repo, Some(&subdir)).await?;

    Ok(Json(response))
}
