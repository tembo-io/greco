use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct RepositoryResponse {
    pub description: String,
    pub readme: String,
}
